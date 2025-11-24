use cosmic_text::{Attrs, Buffer, Color, FontSystem, Metrics, SwashCache};
use std::collections::HashMap;
use wgpu::{Device, Extent3d, Queue, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub ch: char,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    pub atlas_x: f32,
    pub atlas_y: f32,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
}

pub struct GlyphAtlas {
    texture: Texture,
    atlas_width: u32,
    atlas_height: u32,
    current_x: u32,
    current_y: u32,
    row_height: u32,
    glyph_cache: HashMap<GlyphKey, GlyphInfo>,
    font_system: FontSystem,
    swash_cache: SwashCache,
    font_size: f32,
}

impl GlyphAtlas {
    pub fn new(device: &Device, font_size: f32) -> Self {
        let atlas_width = 2048;
        let atlas_height = 2048;

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Self {
            texture,
            atlas_width,
            atlas_height,
            current_x: 0,
            current_y: 0,
            row_height: 0,
            glyph_cache: HashMap::new(),
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            font_size,
        }
    }

    pub fn get_or_cache_glyph(
        &mut self,
        queue: &Queue,
        ch: char,
        bold: bool,
        italic: bool,
    ) -> Option<GlyphInfo> {
        let key = GlyphKey { ch, bold, italic };

        if let Some(info) = self.glyph_cache.get(&key) {
            return Some(*info);
        }

        // Rasterize glyph
        let (bitmap, metrics) = self.rasterize_glyph(ch, bold, italic)?;

        // Find space in atlas
        let glyph_width = metrics.0 as u32;
        let glyph_height = metrics.1 as u32;

        if self.current_x + glyph_width > self.atlas_width {
            // Move to next row
            self.current_x = 0;
            self.current_y += self.row_height;
            self.row_height = 0;
        }

        if self.current_y + glyph_height > self.atlas_height {
            // Atlas full - would need to implement atlas resizing or recycling
            return None;
        }

        // Upload to atlas
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: self.current_x,
                    y: self.current_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &bitmap,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(glyph_width),
                rows_per_image: Some(glyph_height),
            },
            Extent3d {
                width: glyph_width,
                height: glyph_height,
                depth_or_array_layers: 1,
            },
        );

        let info = GlyphInfo {
            atlas_x: self.current_x as f32 / self.atlas_width as f32,
            atlas_y: self.current_y as f32 / self.atlas_height as f32,
            width: glyph_width as f32 / self.atlas_width as f32,
            height: glyph_height as f32 / self.atlas_height as f32,
            advance: metrics.2,
        };

        self.glyph_cache.insert(key, info);

        self.current_x += glyph_width;
        self.row_height = self.row_height.max(glyph_height);

        Some(info)
    }

    fn rasterize_glyph(&mut self, ch: char, _bold: bool, _italic: bool) -> Option<(Vec<u8>, (usize, usize, f32))> {
        // Create buffer with single character
        let metrics = Metrics::new(self.font_size, self.font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        let text = ch.to_string();
        buffer.set_text(&mut self.font_system, &text, Attrs::new(), cosmic_text::Shaping::Advanced);

        buffer.shape_until_scroll(&mut self.font_system, false);

        // For MVP, return a simple rasterized rectangle
        // In production, you would use swash to properly rasterize glyphs
        let width = (self.font_size * 0.6) as usize;
        let height = (self.font_size * 1.2) as usize;
        let advance = self.font_size * 0.6;

        // Create simple bitmap (white rectangle for now - production would use actual glyph)
        let mut bitmap = vec![0u8; width * height];

        // Fill with a simple pattern to make characters visible
        for y in 0..height {
            for x in 0..width {
                // Create a simple pattern based on character
                let intensity = if ch.is_whitespace() {
                    0
                } else {
                    255
                };
                bitmap[y * width + x] = intensity;
            }
        }

        Some((bitmap, (width, height, advance)))
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn atlas_size(&self) -> (u32, u32) {
        (self.atlas_width, self.atlas_height)
    }
}
