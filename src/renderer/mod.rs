mod text_renderer;
mod gpu_state;
mod glyph_atlas;
pub mod vertex;

pub use text_renderer::TextRenderer;
pub use gpu_state::GpuState;
pub use glyph_atlas::GlyphAtlas;

use crate::terminal::Grid;
use crate::ui::PaneManager;
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

    pub fn render_panes(&mut self, pane_manager: &PaneManager) -> anyhow::Result<()> {
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

        // Clear the screen
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
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

        // Calculate pane bounds
        let window_width = self.gpu_state.size.width as f32;
        let window_height = self.gpu_state.size.height as f32;
        let pane_bounds = pane_manager.layout().calculate_bounds(window_width, window_height);

        // Render each pane with borders
        let active_pane = pane_manager.active_pane();

        for (pane_id, (x, y, width, height)) in pane_bounds.iter() {
            if let Some(pane) = pane_manager.get_pane(*pane_id) {
                let grid = pane.terminal.grid();
                let is_active = active_pane == Some(*pane_id);

                // Render pane with viewport
                self.text_renderer.render_with_viewport(
                    &self.gpu_state,
                    &mut encoder,
                    &view,
                    &grid,
                    (*x as u32, *y as u32, *width as u32, *height as u32),
                )?;

                // Render pane border
                self.text_renderer.render_pane_border(
                    &self.gpu_state,
                    &mut encoder,
                    &view,
                    (*x as u32, *y as u32, *width as u32, *height as u32),
                    is_active,
                )?;
            }
        }

        self.gpu_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
