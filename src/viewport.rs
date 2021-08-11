use crate::GraphicsContext;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec4};
use wgpu::util::DeviceExt;

pub struct Viewport {
    gfx: GraphicsContext,
    uniform_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Viewport {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let uniform_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Viewport.uniform_buffer"),
                contents: bytemuck::bytes_of(&Uniforms::default()),
                usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            });
        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Viewport.bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Viewport.bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            gfx: gfx.clone(),
            uniform_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn window_resized(&self) {
        let window_size = self.gfx.window.inner_size();

        self.gfx.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&Uniforms::new(Vec2::new(
                window_size.width as _,
                window_size.height as _,
            ))),
        );
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn default() -> Self {
        Self {
            proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    fn new(size: Vec2) -> Self {
        // Preserve the -1..1 XY square, correcting for the aspect ratio of the window.
        let proj = Mat4::from_cols(
            size.min_element() / size.x * Vec4::X,
            size.min_element() / size.y * Vec4::Y,
            Vec4::Z,
            Vec4::W,
        );
        Self {
            proj: proj.to_cols_array_2d(),
        }
    }
}
