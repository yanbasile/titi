use super::GpuState;
use crate::terminal::{Color, Grid};
use crate::Config;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use std::sync::{Arc, Mutex};

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    cell_width: f32,
    cell_height: f32,
    font_size: f32,
}

impl TextRenderer {
    pub fn new(_gpu_state: &GpuState, config: &Config) -> anyhow::Result<Self> {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let font_size = config.font.size;

        // Measure cell dimensions using a monospace character
        let metrics = Metrics::new(font_size, font_size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_size(&mut font_system, Some(100.0), Some(100.0));
        buffer.set_text(&mut font_system, "M", Attrs::new(), cosmic_text::Shaping::Advanced);

        // Calculate cell dimensions
        let cell_width = font_size * 0.6; // Approximation for monospace
        let cell_height = font_size * 1.2;

        Ok(Self {
            font_system,
            swash_cache,
            cell_width,
            cell_height,
            font_size,
        })
    }

    pub fn render(
        &mut self,
        _gpu_state: &GpuState,
        _encoder: &mut wgpu::CommandEncoder,
        _view: &wgpu::TextureView,
        _grid: &Arc<Mutex<Grid>>,
    ) -> anyhow::Result<()> {
        // For now, this is a placeholder
        // In a full implementation, we would:
        // 1. Create vertex buffers for each character
        // 2. Rasterize glyphs using swash_cache
        // 3. Upload glyph textures to GPU
        // 4. Render quads with appropriate textures and colors

        // This would require implementing shaders and a proper rendering pipeline
        // For the MVP, we're focusing on architecture

        Ok(())
    }

    pub fn cell_dimensions(&self) -> (f32, f32) {
        (self.cell_width, self.cell_height)
    }

    fn color_to_rgba(&self, color: &Color, config: &Config) -> [f32; 4] {
        match color {
            Color::Black => config.colors.black,
            Color::Red => config.colors.red,
            Color::Green => config.colors.green,
            Color::Yellow => config.colors.yellow,
            Color::Blue => config.colors.blue,
            Color::Magenta => config.colors.magenta,
            Color::Cyan => config.colors.cyan,
            Color::White => config.colors.white,
            Color::BrightBlack => config.colors.bright_black,
            Color::BrightRed => config.colors.bright_red,
            Color::BrightGreen => config.colors.bright_green,
            Color::BrightYellow => config.colors.bright_yellow,
            Color::BrightBlue => config.colors.bright_blue,
            Color::BrightMagenta => config.colors.bright_magenta,
            Color::BrightCyan => config.colors.bright_cyan,
            Color::BrightWhite => config.colors.bright_white,
            Color::Default => config.colors.foreground,
            Color::Rgb(r, g, b) => [
                *r as f32 / 255.0,
                *g as f32 / 255.0,
                *b as f32 / 255.0,
                1.0,
            ],
        }
    }
}
