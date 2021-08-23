use crate::viewport::Viewport;
use crate::{asset_str, GraphicsContext};
use bytemuck::{Pod, Zeroable};
use chrono::{NaiveTime, Timelike};
use once_cell::sync::Lazy;
use std::convert::TryInto;
use std::f32::consts::TAU;
use std::num::NonZeroU32;
use tiny_skia::{BlendMode, Color, LineCap, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};
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
            step_mode: wgpu::VertexStepMode::Vertex,
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

struct Config {
    width: u32,
    major_ticks: u32,
    minor_ticks: u32,
    major_inner_radius: f32,
    major_outer_radius: f32,
    minor_inner_radius: f32,
    minor_outer_radius: f32,
    hour_hand_length: f32,
    minute_hand_length: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 1024,
            major_ticks: 4,
            minor_ticks: 5,
            major_inner_radius: 0.85,
            major_outer_radius: 0.95,
            minor_inner_radius: 0.9,
            minor_outer_radius: 0.95,
            hour_hand_length: 0.4,
            minute_hand_length: 0.6,
        }
    }
}

struct Renderer {
    pixmap: Pixmap,
    paint: Paint<'static>,
    major_stroke: Stroke,
    minor_stroke: Stroke,
    transform: Transform,
    major_tick_path: Path,
    minor_tick_path: Path,
    hour_hand_path: Path,
    minute_hand_path: Path,
    hour_angle: f32,
    minute_angle: f32,
}

impl Renderer {
    fn new(config: &Config) -> Self {
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

        let pixmap = Pixmap::new(config.width, config.width).unwrap();
        // Transform from normalized coordinates (-1.0..1.0) to pixels
        // Also flip Y axis so +1.0 is up => row 0
        let transform = Transform::identity()
            .post_translate(1.0, -1.0)
            .post_scale(config.width as f32 / 2.0, config.width as f32 / -2.0);

        let major_tick_path = {
            let mut pb = PathBuilder::new();

            for tick in 0..config.major_ticks {
                let angle = (tick as f32) / (config.major_ticks as f32) * TAU;
                pb.move_to(
                    config.major_inner_radius * angle.cos(),
                    config.major_inner_radius * angle.sin(),
                );
                pb.line_to(
                    config.major_outer_radius * angle.cos(),
                    config.major_outer_radius * angle.sin(),
                );
            }
            pb.finish().unwrap()
        };

        let minor_tick_path = {
            let mut pb = PathBuilder::new();

            for tick in 0..config.major_ticks {
                let start_angle = (tick as f32) / (config.major_ticks as f32) * TAU;
                for minor_tick in 1..=config.minor_ticks {
                    let angle = start_angle
                        + (minor_tick as f32)
                            / (config.minor_ticks as f32 + 1.0)
                            / (config.major_ticks as f32)
                            * TAU;

                    pb.move_to(
                        config.minor_inner_radius * angle.cos(),
                        config.minor_inner_radius * angle.sin(),
                    );
                    pb.line_to(
                        config.minor_outer_radius * angle.cos(),
                        config.minor_outer_radius * angle.sin(),
                    );
                }
            }
            pb.finish().unwrap()
        };

        let hour_hand_path = {
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, 0.0);
            pb.line_to(0.0, config.hour_hand_length);
            pb.finish().unwrap()
        };

        let minute_hand_path = {
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, 0.0);
            pb.line_to(0.0, config.minute_hand_length);
            pb.finish().unwrap()
        };

        Self {
            pixmap,
            paint,
            major_stroke,
            minor_stroke,
            transform,
            major_tick_path,
            minor_tick_path,
            hour_hand_path,
            minute_hand_path,
            hour_angle: 0.0,
            minute_angle: 0.0,
        }
    }

    fn set_time(&mut self, time: &NaiveTime) {
        self.hour_angle = time.num_seconds_from_midnight() as f32 / 86400.0 * TAU;
        self.minute_angle = time.num_seconds_from_midnight() as f32 / 3600.0 * TAU;
    }

    fn redraw(&mut self) {
        self.pixmap.fill(Color::TRANSPARENT);
        self.pixmap.stroke_path(
            &self.major_tick_path,
            &self.paint,
            &self.major_stroke,
            self.transform,
            None,
        );
        self.pixmap.stroke_path(
            &self.minor_tick_path,
            &self.paint,
            &self.minor_stroke,
            self.transform,
            None,
        );
        self.pixmap.stroke_path(
            &self.hour_hand_path,
            &self.paint,
            &self.major_stroke,
            self.transform
                .pre_concat(Transform::from_rotate(-self.hour_angle.to_degrees())),
            None,
        );
        self.pixmap.stroke_path(
            &self.minute_hand_path,
            &self.paint,
            &self.minor_stroke,
            self.transform
                .pre_concat(Transform::from_rotate(-self.minute_angle.to_degrees())),
            None,
        );
    }
}

pub struct ClockFace {
    gfx: GraphicsContext,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
    renderer: Renderer,
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
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
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
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
            });

        let vertex_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ClockFace.vertex_buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ClockFace.index_buffer"),
                contents: bytemuck::cast_slice(&INDICES),
                usage: wgpu::BufferUsages::INDEX,
            });

        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ClockFace.sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let config = Config::default();
        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ClockFace.texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.width,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let texture_view = texture.create_view(&Default::default());
        let renderer = Renderer::new(&config);

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
            gfx: gfx.clone(),
            render_pipeline,
            vertex_buffer,
            index_buffer,
            bind_group,
            texture,
            renderer,
        })
    }

    pub fn set_time(&mut self, time: &NaiveTime) {
        self.renderer.set_time(time)
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        viewport: &Viewport,
    ) {
        self.renderer.redraw();
        let pixmap = &self.renderer.pixmap;
        self.gfx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(pixmap.pixels()),
            wgpu::ImageDataLayout {
                bytes_per_row: Some(NonZeroU32::new(pixmap.width() * 4).unwrap()),
                ..Default::default()
            },
            wgpu::Extent3d {
                width: pixmap.width(),
                height: pixmap.height(),
                ..Default::default()
            },
        );

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
