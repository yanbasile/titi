use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
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

    fn rasterize_glyph(&mut self, ch: char, bold: bool, italic: bool) -> Option<(Vec<u8>, (usize, usize, f32))> {
        // Create buffer with single character
        let metrics = Metrics::new(self.font_size, self.font_size * 1.2);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(self.font_size * 2.0), Some(self.font_size * 2.0));

        // Set text attributes for bold/italic
        let mut attrs = Attrs::new();
        if bold {
            attrs = attrs.weight(cosmic_text::Weight::BOLD);
        }
        if italic {
            attrs = attrs.style(cosmic_text::Style::Italic);
        }

        let text = ch.to_string();
        buffer.set_text(&mut self.font_system, &text, attrs, cosmic_text::Shaping::Advanced);

        buffer.shape_until_scroll(&mut self.font_system, false);

        // Try to get the actual glyph from layout runs
        let mut found_glyph = false;
        let mut max_width = 0;
        let mut max_height = 0;
        let mut glyph_advance = self.font_size * 0.6;
        let mut rasterized_data: Vec<u8> = Vec::new();

        for run in buffer.layout_runs() {
            for layout_glyph in run.glyphs.iter() {
                found_glyph = true;
                glyph_advance = layout_glyph.w;

                // Get physical glyph for rasterization
                let physical = layout_glyph.physical((0.0, 0.0), 1.0);

                // Rasterize using swash cache
                let image = self.swash_cache.get_image(&mut self.font_system, physical.cache_key);

                if let Some(img) = image {
                    max_width = img.placement.width as usize;
                    max_height = img.placement.height as usize;

                    // Convert cosmic_text image to our format (single-channel alpha)
                    match img.content {
                        cosmic_text::SwashContent::Mask => {
                            // Already single-channel, copy directly
                            rasterized_data = img.data.to_vec();
                        }
                        cosmic_text::SwashContent::Color => {
                            // RGBA, extract alpha channel
                            rasterized_data = img.data.chunks(4)
                                .map(|chunk| chunk.get(3).copied().unwrap_or(0))
                                .collect();
                        }
                        cosmic_text::SwashContent::SubpixelMask => {
                            // RGB subpixel, average to single channel
                            rasterized_data = img.data.chunks(3)
                                .map(|chunk| {
                                    if chunk.len() == 3 {
                                        ((chunk[0] as u16 + chunk[1] as u16 + chunk[2] as u16) / 3) as u8
                                    } else {
                                        0
                                    }
                                })
                                .collect();
                        }
                    }
                }

                // Only process first glyph
                break;
            }
            if found_glyph {
                break;
            }
        }

        // If no glyph found or empty, use fallback dimensions
        if !found_glyph || max_width == 0 || max_height == 0 {
            let width = (self.font_size * 0.6) as usize;
            let height = (self.font_size * 1.2) as usize;

            let mut bitmap = vec![0u8; width * height];

            // For visible characters without glyphs, show placeholder
            if !ch.is_whitespace() {
                let margin_x = width / 8;
                let margin_y = height / 8;
                for y in margin_y..(height - margin_y) {
                    for x in margin_x..(width - margin_x) {
                        bitmap[y * width + x] = 200;
                    }
                }
            }

            return Some((bitmap, (width, height, glyph_advance)));
        }

        Some((rasterized_data, (max_width, max_height, glyph_advance)))
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn atlas_size(&self) -> (u32, u32) {
        (self.atlas_width, self.atlas_height)
    }
}
