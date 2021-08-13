use crate::viewport::Viewport;
use crate::{asset_str, GraphicsContext};
use bytemuck::{Pod, Zeroable};
use once_cell::sync::Lazy;
use std::convert::TryInto;
use std::f32::consts::TAU;
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

static VERTEX_ATTRIBUTES: Lazy<[wgpu::VertexAttribute; 2]> = Lazy::new(|| {
    wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
    ]
});

impl Vertex {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>().try_into().unwrap(),
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &VERTEX_ATTRIBUTES[..],
        }
    }
}

const VERTICES: [Vertex; 4] = [
    Vertex {
        position: [1.0, 1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0],
        uv: [1.0, 1.0],
    },
];

const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

const TEXTURE_WIDTH: u32 = 1024;

fn render_clock_face() -> tiny_skia::Pixmap {
    use tiny_skia::*;

    let mut paint = Paint::default();
    paint.set_color(Color::from_rgba(1.0, 1.0, 1.0, 0.5).unwrap());
    paint.anti_alias = true;
    paint.blend_mode = BlendMode::Source;

    let mut major_stroke = Stroke::default();
    major_stroke.width = 0.02;
    major_stroke.line_cap = LineCap::Round;

    let mut minor_stroke = Stroke::default();
    minor_stroke.width = 0.015;
    minor_stroke.line_cap = LineCap::Round;

    let mut pixmap = Pixmap::new(TEXTURE_WIDTH, TEXTURE_WIDTH).unwrap();
    // Transform from normalized coordinates (-1.0..1.0) to pixels
    let transform = Transform::identity()
        .post_translate(1.0, 1.0)
        .post_scale(TEXTURE_WIDTH as f32 / 2.0, TEXTURE_WIDTH as f32 / 2.0);

    let major_ticks = 4;
    let major_inner_radius = 0.85;
    let major_outer_radius = 0.95;

    let minor_ticks = 5;
    let minor_inner_radius = 0.9;
    let minor_outer_radius = 0.95;

    let major_path = {
        let mut pb = PathBuilder::new();

        for tick in 0..major_ticks {
            let angle = (tick as f32) / (major_ticks as f32) * TAU;
            pb.move_to(
                major_inner_radius * angle.cos(),
                major_inner_radius * angle.sin(),
            );
            pb.line_to(
                major_outer_radius * angle.cos(),
                major_outer_radius * angle.sin(),
            );
        }
        pb.finish().unwrap()
    };

    let minor_path = {
        let mut pb = PathBuilder::new();

        for tick in 0..major_ticks {
            let start_angle = (tick as f32) / (major_ticks as f32) * TAU;
            for minor_tick in 1..=minor_ticks {
                let angle = start_angle
                    + (minor_tick as f32) / (minor_ticks as f32 + 1.0) / (major_ticks as f32) * TAU;

                pb.move_to(
                    minor_inner_radius * angle.cos(),
                    minor_inner_radius * angle.sin(),
                );
                pb.line_to(
                    minor_outer_radius * angle.cos(),
                    minor_outer_radius * angle.sin(),
                );
            }
        }
        pb.finish().unwrap()
    };

    pixmap.stroke_path(&major_path, &paint, &major_stroke, transform, None);
    pixmap.stroke_path(&minor_path, &paint, &minor_stroke, transform, None);
    pixmap
}

pub struct ClockFace {
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl ClockFace {
    pub fn new(gfx: &GraphicsContext, viewport: &Viewport) -> anyhow::Result<Self> {
        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ClockFace.bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                    ],
                });
        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ClockFace.pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout, viewport.bind_group_layout()],
                push_constant_ranges: &[],
            });

        let shader_module = gfx
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("ClockFace.shader_module"),
                source: wgpu::ShaderSource::Wgsl(asset_str!("shaders/clock_face.wgsl")),
                flags: wgpu::ShaderFlags::VALIDATION,
            });

        let render_pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ClockFace.render_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "main",
                    buffers: &[Vertex::buffer_layout()],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    clamp_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: gfx.render_format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
            });

        let vertex_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ClockFace.vertex_buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            });
        let index_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ClockFace.index_buffer"),
                contents: bytemuck::cast_slice(&INDICES),
                usage: wgpu::BufferUsage::INDEX,
            });

        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ClockFace.sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let pixmap = render_clock_face();
        let texture = gfx.device.create_texture_with_data(
            &gfx.queue,
            &wgpu::TextureDescriptor {
                label: Some("ClockFace.texture"),
                size: wgpu::Extent3d {
                    width: TEXTURE_WIDTH,
                    height: TEXTURE_WIDTH,
                    ..Default::default()
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::SAMPLED,
            },
            bytemuck::cast_slice(pixmap.pixels()),
        );
        let texture_view = texture.create_view(&Default::default());

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ClockFace.bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        Ok(Self {
            render_pipeline,
            vertex_buffer,
            index_buffer,
            bind_group,
        })
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        viewport: &Viewport,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ClockFace.render_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, viewport.bind_group(), &[]);
        render_pass.draw_indexed(0..INDICES.len().try_into().unwrap(), 0, 0..1);
    }
}
