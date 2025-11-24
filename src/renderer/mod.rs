mod text_renderer;
mod gpu_state;

pub use text_renderer::TextRenderer;
pub use gpu_state::GpuState;

use crate::terminal::Grid;
use crate::Config;
use std::sync::{Arc, Mutex};

pub struct Renderer {
    gpu_state: GpuState,
    text_renderer: TextRenderer,
}

impl Renderer {
    pub async fn new(
        window: Arc<winit::window::Window>,
        config: &Config,
    ) -> anyhow::Result<Self> {
        let gpu_state = GpuState::new(window).await?;
        let text_renderer = TextRenderer::new(&gpu_state, config)?;

        Ok(Self {
            gpu_state,
            text_renderer,
        })
    }

    pub fn render(&mut self, grid: &Arc<Mutex<Grid>>) -> anyhow::Result<()> {
        let output = self.gpu_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.169,
                            b: 0.212,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Render text
        self.text_renderer.render(&self.gpu_state, &mut encoder, &view, grid)?;

        self.gpu_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.gpu_state.resize(new_size);
        }
    }

    pub fn cell_dimensions(&self) -> (f32, f32) {
        self.text_renderer.cell_dimensions()
    }
}
