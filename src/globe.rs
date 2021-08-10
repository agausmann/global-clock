use crate::GraphicsContext;
use bytemuck::{Pod, Zeroable};
use once_cell::sync::Lazy;
use std::convert::TryInto;
use std::num::NonZeroU32;
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

pub struct Globe {
    gfx: GraphicsContext,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Globe {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Globe.bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStage::FRAGMENT,
                            ty: wgpu::BindingType::Sampler {
                                comparison: false,
                                filtering: true,
                            },
                            count: None,
                        },
                    ],
                });
        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Globe.pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader_module = gfx
            .device
            .create_shader_module(&wgpu::include_wgsl!("globe.wgsl"));

        let render_pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Globe.render_pipeline"),
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
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrite::ALL,
                    }],
                }),
            });

        let vertex_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Globe.vertex_buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsage::VERTEX,
            });
        let index_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Globe.index_buffer"),
                contents: bytemuck::cast_slice(&INDICES),
                usage: wgpu::BufferUsage::INDEX,
            });

        let globe_image = image::load_from_memory(include_bytes!("textures/globe.jpg"))
            .expect("failed to load texture")
            .into_rgba8();
        let size = wgpu::Extent3d {
            width: globe_image.width(),
            height: globe_image.height(),
            ..Default::default()
        };
        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Globe.texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        gfx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &globe_image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(size.width * 4),
                rows_per_image: NonZeroU32::new(size.height),
            },
            size,
        );
        let texture_view = texture.create_view(&Default::default());
        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Globe.sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Globe.bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            gfx: gfx.clone(),
            render_pipeline,
            vertex_buffer,
            index_buffer,
            bind_group,
        }
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, frame_view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Globe.render_pass"),
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
        render_pass.draw_indexed(0..INDICES.len().try_into().unwrap(), 0, 0..1);
    }
}
