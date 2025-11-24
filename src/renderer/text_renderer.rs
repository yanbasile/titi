use super::{GpuState, glyph_atlas::GlyphAtlas};
use crate::terminal::{Color, Grid};
use crate::renderer::vertex::{Vertex, Uniforms};
use crate::Config;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, SwashCache};
use std::sync::{Arc, Mutex};
use wgpu::util::DeviceExt;

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    cell_width: f32,
    cell_height: f32,
    font_size: f32,
    glyph_atlas: GlyphAtlas,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl TextRenderer {
    pub fn new(gpu_state: &GpuState, config: &Config) -> anyhow::Result<Self> {
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

        // Create glyph atlas
        let glyph_atlas = GlyphAtlas::new(&gpu_state.device, font_size);

        // Create shader module
        let shader = gpu_state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Text Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/text.wgsl").into()),
            });

        // Create uniform buffer for transform matrix
        let uniforms = Uniforms::ortho(gpu_state.size.width as f32, gpu_state.size.height as f32);
        let uniform_buffer = gpu_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // Create bind group layout for uniforms
        let uniform_bind_group_layout =
            gpu_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Uniform Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // Create uniform bind group
        let uniform_bind_group = gpu_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform Bind Group"),
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });

        // Create texture bind group layout
        let texture_bind_group_layout =
            gpu_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        // Create sampler for atlas texture
        let sampler = gpu_state.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create texture view for glyph atlas
        let atlas_view = glyph_atlas
            .texture()
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create texture bind group
        let texture_bind_group = gpu_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Texture Bind Group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&atlas_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        // Create render pipeline layout
        let render_pipeline_layout =
            gpu_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        // Create render pipeline
        let render_pipeline =
            gpu_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Text Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        buffers: &[Vertex::desc()],
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gpu_state.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                });

        // Create initial vertex and index buffers (will be resized as needed)
        let max_vertices = 10000; // Initial capacity
        let max_indices = 15000;

        let vertex_buffer = gpu_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (std::mem::size_of::<Vertex>() * max_vertices) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = gpu_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (std::mem::size_of::<u32>() * max_indices) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            font_system,
            swash_cache,
            cell_width,
            cell_height,
            font_size,
            glyph_atlas,
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group,
            vertex_buffer,
            index_buffer,
            num_indices: 0,
        })
    }

    pub fn render(
        &mut self,
        gpu_state: &GpuState,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        grid: &Arc<Mutex<Grid>>,
    ) -> anyhow::Result<()> {
        let grid = grid.lock().unwrap();
        let (cols, rows) = grid.size();

        // Generate vertices and indices for all visible characters
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for row in 0..rows {
            for col in 0..cols {
                if let Some(cell) = grid.get_cell(col, row) {
                    // Skip empty cells
                    if cell.c == ' ' || cell.c == '\0' {
                        continue;
                    }

                    // Get or cache the glyph
                    let glyph_info = self.glyph_atlas.get_or_cache_glyph(
                        &gpu_state.queue,
                        cell.c,
                        cell.style.bold,
                        cell.style.italic,
                    );

                    if let Some(glyph) = glyph_info {
                        // Calculate screen position
                        let x = col as f32 * self.cell_width;
                        let y = row as f32 * self.cell_height;

                        // Convert colors to RGBA
                        let fg_color = self.color_to_rgba_array(&cell.style.fg);
                        let bg_color = self.color_to_rgba_array(&cell.style.bg);

                        // Render background if not default
                        if !matches!(cell.style.bg, Color::Default) {
                            let base_vertex = vertices.len() as u32;

                            // Background quad (no texture, just solid color)
                            vertices.extend_from_slice(&[
                                Vertex {
                                    position: [x, y],
                                    tex_coords: [0.0, 0.0],
                                    color: bg_color,
                                },
                                Vertex {
                                    position: [x + self.cell_width, y],
                                    tex_coords: [0.0, 0.0],
                                    color: bg_color,
                                },
                                Vertex {
                                    position: [x + self.cell_width, y + self.cell_height],
                                    tex_coords: [0.0, 0.0],
                                    color: bg_color,
                                },
                                Vertex {
                                    position: [x, y + self.cell_height],
                                    tex_coords: [0.0, 0.0],
                                    color: bg_color,
                                },
                            ]);

                            indices.extend_from_slice(&[
                                base_vertex, base_vertex + 1, base_vertex + 2,
                                base_vertex, base_vertex + 2, base_vertex + 3,
                            ]);
                        }

                        // Render foreground character
                        let base_vertex = vertices.len() as u32;

                        // Calculate glyph size in pixels
                        let (atlas_width, atlas_height) = self.glyph_atlas.atlas_size();
                        let glyph_width = glyph.width * atlas_width as f32;
                        let glyph_height = glyph.height * atlas_height as f32;

                        // Character quad with glyph texture
                        vertices.extend_from_slice(&[
                            Vertex {
                                position: [x, y],
                                tex_coords: [glyph.atlas_x, glyph.atlas_y],
                                color: fg_color,
                            },
                            Vertex {
                                position: [x + glyph_width, y],
                                tex_coords: [glyph.atlas_x + glyph.width, glyph.atlas_y],
                                color: fg_color,
                            },
                            Vertex {
                                position: [x + glyph_width, y + glyph_height],
                                tex_coords: [glyph.atlas_x + glyph.width, glyph.atlas_y + glyph.height],
                                color: fg_color,
                            },
                            Vertex {
                                position: [x, y + glyph_height],
                                tex_coords: [glyph.atlas_x, glyph.atlas_y + glyph.height],
                                color: fg_color,
                            },
                        ]);

                        indices.extend_from_slice(&[
                            base_vertex, base_vertex + 1, base_vertex + 2,
                            base_vertex, base_vertex + 2, base_vertex + 3,
                        ]);
                    }
                }
            }
        }

        drop(grid);

        // If no vertices to render, early return
        if vertices.is_empty() {
            return Ok(());
        }

        // Update vertex and index buffers
        gpu_state.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
        gpu_state.queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&indices),
        );

        self.num_indices = indices.len() as u32;

        // Begin render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Text Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Load existing content (background cleared by main pass)
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        Ok(())
    }

    pub fn cell_dimensions(&self) -> (f32, f32) {
        (self.cell_width, self.cell_height)
    }

    pub fn render_with_viewport(
        &mut self,
        gpu_state: &GpuState,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        grid: &Arc<Mutex<Grid>>,
        viewport: (u32, u32, u32, u32), // (x, y, width, height)
    ) -> anyhow::Result<()> {
        let (viewport_x, viewport_y, viewport_width, viewport_height) = viewport;

        let grid = grid.lock().unwrap();
        let (cols, rows) = grid.size();

        // Generate vertices and indices for all visible characters
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for row in 0..rows {
            for col in 0..cols {
                if let Some(cell) = grid.get_cell(col, row) {
                    // Skip empty cells
                    if cell.c == ' ' || cell.c == '\0' {
                        continue;
                    }

                    // Get or cache the glyph
                    let glyph_info = self.glyph_atlas.get_or_cache_glyph(
                        &gpu_state.queue,
                        cell.c,
                        cell.style.bold,
                        cell.style.italic,
                    );

                    if let Some(glyph) = glyph_info {
                        // Calculate position relative to viewport
                        let x = viewport_x as f32 + col as f32 * self.cell_width;
                        let y = viewport_y as f32 + row as f32 * self.cell_height;

                        // Convert colors to RGBA
                        let fg_color = self.color_to_rgba_array(&cell.style.fg);
                        let bg_color = self.color_to_rgba_array(&cell.style.bg);

                        // Render background if not default
                        if !matches!(cell.style.bg, Color::Default) {
                            let base_vertex = vertices.len() as u32;

                            vertices.extend_from_slice(&[
                                Vertex {
                                    position: [x, y],
                                    tex_coords: [0.0, 0.0],
                                    color: bg_color,
                                },
                                Vertex {
                                    position: [x + self.cell_width, y],
                                    tex_coords: [0.0, 0.0],
                                    color: bg_color,
                                },
                                Vertex {
                                    position: [x + self.cell_width, y + self.cell_height],
                                    tex_coords: [0.0, 0.0],
                                    color: bg_color,
                                },
                                Vertex {
                                    position: [x, y + self.cell_height],
                                    tex_coords: [0.0, 0.0],
                                    color: bg_color,
                                },
                            ]);

                            indices.extend_from_slice(&[
                                base_vertex, base_vertex + 1, base_vertex + 2,
                                base_vertex, base_vertex + 2, base_vertex + 3,
                            ]);
                        }

                        // Render foreground character
                        let base_vertex = vertices.len() as u32;

                        // Calculate glyph size in pixels
                        let (atlas_width, atlas_height) = self.glyph_atlas.atlas_size();
                        let glyph_width = glyph.width * atlas_width as f32;
                        let glyph_height = glyph.height * atlas_height as f32;

                        vertices.extend_from_slice(&[
                            Vertex {
                                position: [x, y],
                                tex_coords: [glyph.atlas_x, glyph.atlas_y],
                                color: fg_color,
                            },
                            Vertex {
                                position: [x + glyph_width, y],
                                tex_coords: [glyph.atlas_x + glyph.width, glyph.atlas_y],
                                color: fg_color,
                            },
                            Vertex {
                                position: [x + glyph_width, y + glyph_height],
                                tex_coords: [glyph.atlas_x + glyph.width, glyph.atlas_y + glyph.height],
                                color: fg_color,
                            },
                            Vertex {
                                position: [x, y + glyph_height],
                                tex_coords: [glyph.atlas_x, glyph.atlas_y + glyph.height],
                                color: fg_color,
                            },
                        ]);

                        indices.extend_from_slice(&[
                            base_vertex, base_vertex + 1, base_vertex + 2,
                            base_vertex, base_vertex + 2, base_vertex + 3,
                        ]);
                    }
                }
            }
        }

        drop(grid);

        // If no vertices to render, early return
        if vertices.is_empty() {
            return Ok(());
        }

        // Update vertex and index buffers
        gpu_state.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
        gpu_state.queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&indices),
        );

        self.num_indices = indices.len() as u32;

        // Begin render pass with viewport and scissor rectangle
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Pane Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // Load existing content (don't clear)
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Set viewport and scissor rectangle for this pane
        render_pass.set_viewport(
            viewport_x as f32,
            viewport_y as f32,
            viewport_width as f32,
            viewport_height as f32,
            0.0,
            1.0,
        );
        render_pass.set_scissor_rect(viewport_x, viewport_y, viewport_width, viewport_height);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        Ok(())
    }

    pub fn render_pane_border(
        &mut self,
        gpu_state: &GpuState,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        viewport: (u32, u32, u32, u32),
        is_active: bool,
    ) -> anyhow::Result<()> {
        let (x, y, width, height) = viewport;

        // Border color: bright for active pane, dim for inactive
        let border_color = if is_active {
            [0.0, 0.6, 0.8, 1.0] // Cyan blue for active
        } else {
            [0.2, 0.2, 0.2, 1.0] // Dark gray for inactive
        };

        // Border width in pixels
        let border_width = if is_active { 2.0 } else { 1.0 };

        let x = x as f32;
        let y = y as f32;
        let width = width as f32;
        let height = height as f32;

        // Create 4 rectangles for the border (top, right, bottom, left)
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Top border
        let base_vertex = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex {
                position: [x, y],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + width, y],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + width, y + border_width],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x, y + border_width],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
        ]);
        indices.extend_from_slice(&[
            base_vertex, base_vertex + 1, base_vertex + 2,
            base_vertex, base_vertex + 2, base_vertex + 3,
        ]);

        // Right border
        let base_vertex = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex {
                position: [x + width - border_width, y],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + width, y],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + width, y + height],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + width - border_width, y + height],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
        ]);
        indices.extend_from_slice(&[
            base_vertex, base_vertex + 1, base_vertex + 2,
            base_vertex, base_vertex + 2, base_vertex + 3,
        ]);

        // Bottom border
        let base_vertex = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex {
                position: [x, y + height - border_width],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + width, y + height - border_width],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + width, y + height],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x, y + height],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
        ]);
        indices.extend_from_slice(&[
            base_vertex, base_vertex + 1, base_vertex + 2,
            base_vertex, base_vertex + 2, base_vertex + 3,
        ]);

        // Left border
        let base_vertex = vertices.len() as u32;
        vertices.extend_from_slice(&[
            Vertex {
                position: [x, y],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + border_width, y],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x + border_width, y + height],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
            Vertex {
                position: [x, y + height],
                tex_coords: [0.0, 0.0],
                color: border_color,
            },
        ]);
        indices.extend_from_slice(&[
            base_vertex, base_vertex + 1, base_vertex + 2,
            base_vertex, base_vertex + 2, base_vertex + 3,
        ]);

        // Update buffers
        gpu_state.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
        gpu_state.queue.write_buffer(
            &self.index_buffer,
            0,
            bytemuck::cast_slice(&indices),
        );

        self.num_indices = indices.len() as u32;

        // Render border
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Border Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        Ok(())
    }

    fn color_to_rgba_array(&self, color: &Color) -> [f32; 4] {
        match color {
            Color::Black => [0.0, 0.0, 0.0, 1.0],
            Color::Red => [0.8, 0.0, 0.0, 1.0],
            Color::Green => [0.0, 0.8, 0.0, 1.0],
            Color::Yellow => [0.8, 0.8, 0.0, 1.0],
            Color::Blue => [0.0, 0.0, 0.8, 1.0],
            Color::Magenta => [0.8, 0.0, 0.8, 1.0],
            Color::Cyan => [0.0, 0.8, 0.8, 1.0],
            Color::White => [0.8, 0.8, 0.8, 1.0],
            Color::BrightBlack => [0.5, 0.5, 0.5, 1.0],
            Color::BrightRed => [1.0, 0.0, 0.0, 1.0],
            Color::BrightGreen => [0.0, 1.0, 0.0, 1.0],
            Color::BrightYellow => [1.0, 1.0, 0.0, 1.0],
            Color::BrightBlue => [0.0, 0.0, 1.0, 1.0],
            Color::BrightMagenta => [1.0, 0.0, 1.0, 1.0],
            Color::BrightCyan => [0.0, 1.0, 1.0, 1.0],
            Color::BrightWhite => [1.0, 1.0, 1.0, 1.0],
            Color::Default => [0.9, 0.9, 0.9, 1.0],
            Color::Rgb(r, g, b) => [
                *r as f32 / 255.0,
                *g as f32 / 255.0,
                *b as f32 / 255.0,
                1.0,
            ],
        }
    }
}
