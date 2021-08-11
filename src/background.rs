use crate::GraphicsContext;

pub struct Background {}

impl Background {
    pub fn new(_gfx: &GraphicsContext) -> Self {
        Self {}
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, frame_view: &wgpu::TextureView) {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Background.render_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
    }
}
