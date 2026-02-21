use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_int;
use std::rc::Rc;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics as CosmicMetrics, Shaping};
use freetype::Library;
use freetype::face::LoadFlag;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextUniforms {
    pub projection: [[f32; 4]; 4],
    pub fg_color: [f32; 4],
    pub bg_color: [f32; 4],
    pub gamma_adj: f32,
    pub contrast: f32,
    pub _pad: [f32; 2],
}

#[derive(Clone, Copy)]
pub struct GlyphInfo {
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

#[derive(Clone, Copy)]
pub struct FontMetrics {
    pub ascender: f32,
    pub line_height: f32,
    pub cell_width: f32,
}

/// Grid cell layout for map text rendering.
struct GridLayout {
    h_advance: f32,
    h_offset: f32,
    v_advance: f32,
}

struct PendingGlyphUpload {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

pub struct FontRenderer {
    // Codepoint-keyed glyph cache (used by prepare_map / build_vertices)
    glyphs: HashMap<u32, GlyphInfo>,
    metrics: FontMetrics,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    frame_vertices: Vec<TextVertex>,

    // cosmic-text layout engine
    font_system: FontSystem,
    font_size_px: f32,

    // Glyph cache keyed by glyph_id (for cosmic-text shaped output)
    shaped_glyphs: HashMap<u32, GlyphInfo>,

    // FreeType context (kept alive for on-demand rasterization)
    _ft_lib: Library,
    ft_face: freetype::Face,
    ft_load_flags: LoadFlag,

    // Atlas management for dynamic glyph addition
    atlas_texture: wgpu::Texture,
    atlas_data: Vec<u8>,
    atlas_width: u32,
    atlas_height: u32,
    atlas_shelf_x: u32,
    atlas_shelf_y: u32,
    atlas_shelf_height: u32,
    pending_atlas_uploads: Vec<PendingGlyphUpload>,
}

impl FontRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        font_size_pts: f32,
        scale_factor: f64,
    ) -> Self {
        let (font_path, hinting_flags) = find_font_with_hinting();
        let dpi = (scale_factor * 96.0) as u32;

        // Load font bytes into memory
        let font_bytes = std::fs::read(&font_path).expect("failed to read font file");

        // Create persistent FreeType state
        let ft_lib = Library::init().expect("freetype init failed");
        let ft_face = ft_lib
            .new_memory_face(Rc::new(font_bytes.clone()), 0)
            .expect("failed to load font face");

        let size_26_6 = (font_size_pts * 64.0).ceil() as isize;
        ft_face
            .set_char_size(0, size_26_6, dpi, dpi)
            .expect("failed to set char size");
        let ft_load_flags = LoadFlag::RENDER | hinting_flags;

        // Rasterize initial glyphs into atlas
        let (
            glyphs,
            shaped_glyphs,
            metrics,
            atlas_data,
            atlas_width,
            atlas_height,
            shelf_x,
            shelf_y,
            shelf_height,
        ) = rasterize_glyphs(&ft_face, hinting_flags);

        // Compute font_size_px from FreeType's y_ppem
        let size_metrics = ft_face.size_metrics().expect("no size metrics");
        let font_size_px = size_metrics.y_ppem as f32;

        // Build cosmic-text FontSystem with our single font
        let mut db = cosmic_text::fontdb::Database::new();
        db.load_font_data(font_bytes);
        let font_system = FontSystem::new_with_locale_and_db("en-US".to_string(), db);

        // Create atlas texture (fixed 512x4096)
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph_atlas"),
            size: wgpu::Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_width),
                rows_per_image: Some(atlas_height),
            },
            wgpu::Extent3d {
                width: atlas_width,
                height: atlas_height,
                depth_or_array_layers: 1,
            },
        );

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("glyph_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text_uniforms"),
            size: std::mem::size_of::<TextUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let initial_capacity = 6000;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text_vertices"),
            size: (initial_capacity * std::mem::size_of::<TextVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&atlas_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            glyphs,
            metrics,
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            vertex_capacity: initial_capacity,
            frame_vertices: Vec::new(),
            font_system,
            font_size_px,
            shaped_glyphs,
            _ft_lib: ft_lib,
            ft_face,
            ft_load_flags,
            atlas_texture,
            atlas_data,
            atlas_width,
            atlas_height,
            atlas_shelf_x: shelf_x,
            atlas_shelf_y: shelf_y,
            atlas_shelf_height: shelf_height,
            pending_atlas_uploads: Vec::new(),
        }
    }

    /// Return font metrics for layout calculations.
    pub fn metrics(&self) -> FontMetrics {
        self.metrics
    }

    /// Start a new frame. Clears accumulated vertices and writes uniforms.
    pub fn begin_frame(
        &mut self,
        queue: &wgpu::Queue,
        screen_w: u32,
        screen_h: u32,
        fg: [f32; 3],
        bg: [f32; 3],
    ) {
        self.frame_vertices.clear();

        let sw = screen_w as f32;
        let sh = screen_h as f32;

        #[rustfmt::skip]
        let projection: [[f32; 4]; 4] = [
            [2.0 / sw,  0.0,        0.0, 0.0],
            [0.0,      -2.0 / sh,   0.0, 0.0],
            [0.0,       0.0,        1.0, 0.0],
            [-1.0,      1.0,        0.0, 1.0],
        ];

        let uniforms = TextUniforms {
            projection,
            fg_color: [fg[0], fg[1], fg[2], 1.0],
            bg_color: [bg[0], bg[1], bg[2], 1.0],
            gamma_adj: 1.0,
            contrast: 1.0,
            _pad: [0.0; 2],
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Append text vertices using cosmic-text shaping (for status/event log).
    pub fn prepare_text(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) {
        self.prepare_text_shaped(text, x, y, color);
    }

    /// Map cell dimensions (width, height). Square cells for uniform grid.
    pub fn map_cell(&self) -> (f32, f32) {
        let s = self.metrics.line_height;
        (s, s)
    }

    /// Append text vertices for the map grid.
    pub fn prepare_map(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) {
        let (w, h) = self.map_cell();
        let grid = GridLayout {
            h_advance: w,
            h_offset: ((w - self.metrics.cell_width) / 2.0).floor(),
            v_advance: h,
        };
        self.build_vertices(text, x, y, &grid, color);
    }

    /// Upload accumulated vertices to the GPU. Returns vertex count for render().
    pub fn flush(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) -> u32 {
        // Drain pending atlas uploads (sub-region writes for on-demand glyphs)
        for upload in self.pending_atlas_uploads.drain(..) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: upload.x,
                        y: upload.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &upload.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.width),
                    rows_per_image: Some(upload.height),
                },
                wgpu::Extent3d {
                    width: upload.width,
                    height: upload.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let vertex_count = self.frame_vertices.len() as u32;
        if self.frame_vertices.is_empty() {
            return 0;
        }

        if self.frame_vertices.len() > self.vertex_capacity {
            self.vertex_capacity = self.frame_vertices.len().next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("text_vertices"),
                size: (self.vertex_capacity * std::mem::size_of::<TextVertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.frame_vertices),
        );

        vertex_count
    }

    /// Rasterize a single glyph by glyph_id into the atlas on demand.
    fn rasterize_glyph_on_demand(&mut self, glyph_id: u32) -> Option<GlyphInfo> {
        if let Some(info) = self.shaped_glyphs.get(&glyph_id) {
            return Some(*info);
        }

        if self
            .ft_face
            .load_glyph(glyph_id, self.ft_load_flags)
            .is_err()
        {
            return None;
        }

        let glyph_slot = self.ft_face.glyph();
        let bitmap = glyph_slot.bitmap();
        let w = bitmap.width() as u32;
        let h = bitmap.rows() as u32;

        if w == 0 || h == 0 {
            let info = GlyphInfo {
                width: 0,
                height: 0,
                bearing_x: 0,
                bearing_y: 0,
                u0: 0.0,
                v0: 0.0,
                u1: 0.0,
                v1: 0.0,
            };
            self.shaped_glyphs.insert(glyph_id, info);
            return Some(info);
        }

        let padding: u32 = 1;

        // Check if glyph fits on current shelf
        if self.atlas_shelf_x + w + padding > self.atlas_width {
            self.atlas_shelf_y += self.atlas_shelf_height + padding;
            self.atlas_shelf_x = 0;
            self.atlas_shelf_height = 0;
        }

        if self.atlas_shelf_y + h > self.atlas_height {
            log::warn!("glyph atlas full, cannot add glyph_id {}", glyph_id);
            return None;
        }

        let pos_x = self.atlas_shelf_x;
        let pos_y = self.atlas_shelf_y;
        self.atlas_shelf_height = self.atlas_shelf_height.max(h);
        self.atlas_shelf_x += w + padding;

        // Extract bitmap pixels
        let pitch = bitmap.pitch();
        let buf = bitmap.buffer();
        let abs_pitch = pitch.unsigned_abs() as usize;
        let mut pixels = Vec::with_capacity((w * h) as usize);
        for row in 0..h {
            let src_row = if pitch >= 0 {
                row as usize
            } else {
                (h - 1 - row) as usize
            };
            let start = src_row * abs_pitch;
            let end = start + w as usize;
            pixels.extend_from_slice(&buf[start..end]);
        }

        // Blit into CPU-side atlas
        for row in 0..h {
            let src_start = (row * w) as usize;
            let dst_start = ((pos_y + row) * self.atlas_width + pos_x) as usize;
            self.atlas_data[dst_start..dst_start + w as usize]
                .copy_from_slice(&pixels[src_start..src_start + w as usize]);
        }

        // Queue deferred GPU upload
        self.pending_atlas_uploads.push(PendingGlyphUpload {
            x: pos_x,
            y: pos_y,
            width: w,
            height: h,
            pixels,
        });

        let aw = self.atlas_width as f32;
        let ah = self.atlas_height as f32;
        let info = GlyphInfo {
            width: w,
            height: h,
            bearing_x: glyph_slot.bitmap_left(),
            bearing_y: glyph_slot.bitmap_top(),
            u0: pos_x as f32 / aw,
            v0: pos_y as f32 / ah,
            u1: (pos_x + w) as f32 / aw,
            v1: (pos_y + h) as f32 / ah,
        };
        self.shaped_glyphs.insert(glyph_id, info);
        Some(info)
    }

    /// Layout and shape text using cosmic-text, then build vertices.
    fn prepare_text_shaped(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) {
        let cosmic_metrics = CosmicMetrics::new(self.font_size_px, self.metrics.line_height);
        let mut buffer = Buffer::new(&mut self.font_system, cosmic_metrics);
        buffer.set_size(&mut self.font_system, None, None);
        let attrs = Attrs::new().family(Family::Name("Libertinus Mono"));
        buffer.set_text(&mut self.font_system, text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.font_system, false);

        // Collect glyph positions to avoid borrow conflicts with self
        let mut glyph_positions: Vec<(u16, i32, i32)> = Vec::new();
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((x, y + run.line_y), 1.0);
                glyph_positions.push((physical.cache_key.glyph_id, physical.x, physical.y));
            }
        }
        drop(buffer);

        // Fallback glyph for .notdef (glyph_id 0)
        let question = self.glyphs.get(&(b'?' as u32)).copied();

        for &(glyph_id_u16, px, py) in &glyph_positions {
            let gid = glyph_id_u16 as u32;

            // Ensure glyph exists in shaped cache
            if !self.shaped_glyphs.contains_key(&gid) && gid != 0 {
                self.rasterize_glyph_on_demand(gid);
            }

            let info = if gid == 0 {
                // .notdef — fall back to '?'
                match question {
                    Some(g) => g,
                    None => continue,
                }
            } else {
                match self.shaped_glyphs.get(&gid) {
                    Some(g) => *g,
                    None => match question {
                        Some(g) => g,
                        None => continue,
                    },
                }
            };

            if info.width == 0 || info.height == 0 {
                continue;
            }

            let x0 = (px as f32 + info.bearing_x as f32).floor();
            let y0 = (py as f32 - info.bearing_y as f32).floor();
            let x1 = x0 + info.width as f32;
            let y1 = y0 + info.height as f32;

            self.frame_vertices.push(TextVertex {
                position: [x0, y0],
                uv: [info.u0, info.v0],
                color,
            });
            self.frame_vertices.push(TextVertex {
                position: [x1, y0],
                uv: [info.u1, info.v0],
                color,
            });
            self.frame_vertices.push(TextVertex {
                position: [x0, y1],
                uv: [info.u0, info.v1],
                color,
            });

            self.frame_vertices.push(TextVertex {
                position: [x1, y0],
                uv: [info.u1, info.v0],
                color,
            });
            self.frame_vertices.push(TextVertex {
                position: [x1, y1],
                uv: [info.u1, info.v1],
                color,
            });
            self.frame_vertices.push(TextVertex {
                position: [x0, y1],
                uv: [info.u0, info.v1],
                color,
            });
        }
    }

    /// Build vertex quads for text and append to frame_vertices.
    fn build_vertices(&mut self, text: &str, x: f32, y: f32, grid: &GridLayout, color: [f32; 4]) {
        let mut pen_x = x.floor();
        let mut pen_y = (y + self.metrics.ascender).floor();
        let question = self.glyphs.get(&(b'?' as u32)).copied();

        for ch in text.chars() {
            if ch == '\n' {
                pen_x = x.floor();
                pen_y += grid.v_advance;
                continue;
            }
            let cp = ch as u32;
            let glyph = match self.glyphs.get(&cp) {
                Some(g) => *g,
                None => match question {
                    Some(g) => g,
                    None => {
                        pen_x += grid.h_advance;
                        continue;
                    }
                },
            };

            if glyph.width == 0 || glyph.height == 0 {
                pen_x += grid.h_advance;
                continue;
            }

            let x0 = (pen_x + grid.h_offset + glyph.bearing_x as f32).floor();
            let y0 = (pen_y - glyph.bearing_y as f32).floor();
            let x1 = x0 + glyph.width as f32;
            let y1 = y0 + glyph.height as f32;

            self.frame_vertices.push(TextVertex {
                position: [x0, y0],
                uv: [glyph.u0, glyph.v0],
                color,
            });
            self.frame_vertices.push(TextVertex {
                position: [x1, y0],
                uv: [glyph.u1, glyph.v0],
                color,
            });
            self.frame_vertices.push(TextVertex {
                position: [x0, y1],
                uv: [glyph.u0, glyph.v1],
                color,
            });

            self.frame_vertices.push(TextVertex {
                position: [x1, y0],
                uv: [glyph.u1, glyph.v0],
                color,
            });
            self.frame_vertices.push(TextVertex {
                position: [x1, y1],
                uv: [glyph.u1, glyph.v1],
                color,
            });
            self.frame_vertices.push(TextVertex {
                position: [x0, y1],
                uv: [glyph.u0, glyph.v1],
                color,
            });

            pen_x += grid.h_advance;
        }
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, vertex_count: u32) {
        if vertex_count == 0 {
            return;
        }
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..vertex_count, 0..1);
    }
}

/// Fontconfig hinting configuration.
struct HintingConfig {
    hinting: bool,
    hintstyle: i32, // 0=none, 1=slight, 2=medium, 3=full
}

/// Query fontconfig for font path and hinting settings — matches kitty's fontconfig.c approach.
fn query_fontconfig(families: &[&str]) -> Option<HintingConfig> {
    // fontconfig FFI
    #[link(name = "fontconfig")]
    unsafe extern "C" {
        fn FcPatternCreate() -> *mut std::ffi::c_void;
        fn FcPatternAddString(
            p: *mut std::ffi::c_void,
            object: *const std::ffi::c_char,
            s: *const u8,
        ) -> c_int;
        fn FcConfigSubstitute(
            config: *mut std::ffi::c_void,
            p: *mut std::ffi::c_void,
            kind: c_int,
        ) -> c_int;
        fn FcDefaultSubstitute(pattern: *mut std::ffi::c_void);
        fn FcFontMatch(
            config: *mut std::ffi::c_void,
            p: *mut std::ffi::c_void,
            result: *mut c_int,
        ) -> *mut std::ffi::c_void;
        fn FcPatternGetInteger(
            p: *const std::ffi::c_void,
            object: *const std::ffi::c_char,
            n: c_int,
            i: *mut c_int,
        ) -> c_int;
        fn FcPatternGetBool(
            p: *const std::ffi::c_void,
            object: *const std::ffi::c_char,
            n: c_int,
            b: *mut c_int,
        ) -> c_int;
        fn FcPatternDestroy(p: *mut std::ffi::c_void);
    }

    let fc_family = CString::new("family").expect("CString family");
    let fc_hinting = CString::new("hinting").expect("CString hinting");
    let fc_hintstyle = CString::new("hintstyle").expect("CString hintstyle");

    for family in families {
        let family_c = CString::new(*family).expect("CString family name");
        unsafe {
            let pat = FcPatternCreate();
            if pat.is_null() {
                continue;
            }
            FcPatternAddString(pat, fc_family.as_ptr(), family_c.as_ptr() as *const u8);
            FcConfigSubstitute(std::ptr::null_mut(), pat, 0 /* FcMatchPattern */);
            FcDefaultSubstitute(pat);

            let mut result: c_int = 0;
            let matched = FcFontMatch(std::ptr::null_mut(), pat, &mut result);
            FcPatternDestroy(pat);

            if matched.is_null() || result != 0 {
                if !matched.is_null() {
                    FcPatternDestroy(matched);
                }
                continue;
            }

            // Read hinting (bool), default true
            let mut hinting_val: c_int = 1;
            FcPatternGetBool(matched, fc_hinting.as_ptr(), 0, &mut hinting_val);

            // Read hintstyle (int), default 1 (slight)
            let mut hintstyle_val: c_int = 1;
            FcPatternGetInteger(matched, fc_hintstyle.as_ptr(), 0, &mut hintstyle_val);

            FcPatternDestroy(matched);

            return Some(HintingConfig {
                hinting: hinting_val != 0,
                hintstyle: hintstyle_val,
            });
        }
    }
    None
}

/// Map fontconfig hinting config to FreeType load flags — matches kitty's get_load_flags().
fn hinting_load_flags(config: &HintingConfig) -> LoadFlag {
    if !config.hinting {
        LoadFlag::NO_HINTING
    } else if config.hintstyle >= 3 {
        LoadFlag::TARGET_NORMAL
    } else if config.hintstyle > 0 {
        LoadFlag::TARGET_LIGHT
    } else {
        // hintstyle=0 with hinting=true: no target flag (defaults to normal)
        LoadFlag::DEFAULT
    }
}

/// Bundled font path relative to the project root.
const BUNDLED_FONT: &str = "fonts/libertinus/LibertinusMono-Regular.otf";

/// Find font path and hinting config. Uses the bundled font with system hinting preferences.
fn find_font_with_hinting() -> (String, LoadFlag) {
    // Query fontconfig for system hinting preferences (font path is ignored)
    let hinting_flags = query_fontconfig(&["monospace"])
        .map(|config| hinting_load_flags(&config))
        .unwrap_or(LoadFlag::TARGET_LIGHT);

    // Use bundled font relative to the executable's directory first, then working directory
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let path = dir.join(BUNDLED_FONT);
        if path.exists() {
            return (path.to_string_lossy().into_owned(), hinting_flags);
        }
    }
    if std::path::Path::new(BUNDLED_FONT).exists() {
        return (BUNDLED_FONT.to_string(), hinting_flags);
    }
    panic!("bundled font not found: {}", BUNDLED_FONT);
}

/// Rasterize glyphs for ASCII + Latin-1 Supplement + extras and pack into a glyph atlas.
/// Returns (codepoint_glyphs, glyph_id_glyphs, metrics, atlas_pixels, atlas_w, atlas_h, shelf_x, shelf_y, shelf_h).
#[allow(clippy::type_complexity)]
fn rasterize_glyphs(
    face: &freetype::Face,
    hinting_flags: LoadFlag,
) -> (
    HashMap<u32, GlyphInfo>,
    HashMap<u32, GlyphInfo>,
    FontMetrics,
    Vec<u8>,
    u32,
    u32,
    u32,
    u32,
    u32,
) {
    let load_flags = LoadFlag::RENDER | hinting_flags;

    // Compute cell_width like kitty: ceil(max(horiAdvance/64)) across ASCII 32-127
    let mut cell_width: f32 = 0.0;
    for cp in 32u32..128 {
        if face.load_char(cp as usize, hinting_flags).is_ok() {
            let advance = (face.glyph().advance().x as f32 / 64.0).ceil();
            cell_width = cell_width.max(advance);
        }
    }
    if cell_width < 1.0 {
        let size_metrics = face.size_metrics().expect("no size metrics");
        cell_width = (size_metrics.max_advance as f32 / 64.0).ceil();
    }

    // Extract font metrics — ceil to integer pixels like kitty
    let size_metrics = face.size_metrics().expect("no size metrics");
    let metrics = FontMetrics {
        ascender: (size_metrics.ascender as f32 / 64.0).ceil(),
        line_height: (size_metrics.height as f32 / 64.0).ceil(),
        cell_width,
    };

    // Rasterize each glyph
    struct RawGlyph {
        cp: u32,
        width: u32,
        height: u32,
        bearing_x: i32,
        bearing_y: i32,
        pixels: Vec<u8>,
    }

    let mut raw_glyphs: Vec<RawGlyph> = Vec::new();

    // Codepoint ranges to rasterize:
    // - ASCII printable: U+0020..U+007E
    // - Latin-1 Supplement: U+00A0..U+00FF (e, e, e, e, a, a, c, o, u, u, u, ae, AE, etc.)
    // - Latin Extended-A subset: U+0152..U+0153 (OE, oe)
    // - General Punctuation: U+2010..U+2027 (dashes, smart quotes, daggers, bullet, ellipsis)
    let codepoint_ranges: &[(u32, u32)] = &[
        (0x0020, 0x007E),
        (0x00A0, 0x00FF),
        (0x0152, 0x0153),
        (0x2010, 0x2027),
    ];

    for &(range_start, range_end) in codepoint_ranges {
        for cp in range_start..=range_end {
            if face.load_char(cp as usize, load_flags).is_err() {
                continue;
            }
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            let w = bitmap.width() as u32;
            let h = bitmap.rows() as u32;
            let pitch = bitmap.pitch();

            let mut pixels = Vec::new();
            if w > 0 && h > 0 {
                let buf = bitmap.buffer();
                let abs_pitch = pitch.unsigned_abs() as usize;
                // Copy row by row, handling pitch != width and negative pitch (bottom-up)
                for row in 0..h {
                    let src_row = if pitch >= 0 {
                        row as usize
                    } else {
                        (h - 1 - row) as usize
                    };
                    let start = src_row * abs_pitch;
                    let end = start + w as usize;
                    pixels.extend_from_slice(&buf[start..end]);
                }
            }

            raw_glyphs.push(RawGlyph {
                cp,
                width: w,
                height: h,
                bearing_x: glyph.bitmap_left(),
                bearing_y: glyph.bitmap_top(),
                pixels,
            });
        }
    }

    // Shelf-pack atlas: sort by height descending for better packing
    raw_glyphs.sort_by(|a, b| b.height.cmp(&a.height));

    let atlas_width: u32 = 512;
    let atlas_height: u32 = 4096;
    let padding: u32 = 1;
    let mut shelf_x: u32 = 0;
    let mut shelf_y: u32 = 0;
    let mut shelf_height: u32 = 0;

    // First pass: compute positions
    struct PackedPos {
        x: u32,
        y: u32,
    }
    let mut positions: Vec<PackedPos> = Vec::new();

    for g in &raw_glyphs {
        if g.width == 0 || g.height == 0 {
            positions.push(PackedPos { x: 0, y: 0 });
            continue;
        }
        if shelf_x + g.width + padding > atlas_width {
            // New shelf row
            shelf_y += shelf_height + padding;
            shelf_x = 0;
            shelf_height = 0;
        }
        positions.push(PackedPos {
            x: shelf_x,
            y: shelf_y,
        });
        shelf_height = shelf_height.max(g.height);
        shelf_x += g.width + padding;
    }

    // Blit glyphs into atlas
    let mut atlas_data = vec![0u8; (atlas_width * atlas_height) as usize];
    let mut glyphs_map = HashMap::new();
    let mut shaped_map = HashMap::new();

    for (i, g) in raw_glyphs.iter().enumerate() {
        let pos = &positions[i];
        let aw = atlas_width as f32;
        let ah = atlas_height as f32;

        if g.width > 0 && g.height > 0 {
            // Copy glyph pixels into atlas
            for row in 0..g.height {
                let src_start = (row * g.width) as usize;
                let dst_start = ((pos.y + row) * atlas_width + pos.x) as usize;
                atlas_data[dst_start..dst_start + g.width as usize]
                    .copy_from_slice(&g.pixels[src_start..src_start + g.width as usize]);
            }
        }

        let info = GlyphInfo {
            width: g.width,
            height: g.height,
            bearing_x: g.bearing_x,
            bearing_y: g.bearing_y,
            u0: pos.x as f32 / aw,
            v0: pos.y as f32 / ah,
            u1: (pos.x + g.width) as f32 / aw,
            v1: (pos.y + g.height) as f32 / ah,
        };
        glyphs_map.insert(g.cp, info);

        // Also insert by glyph_id for shaped text lookup
        if let Some(glyph_id) = face.get_char_index(g.cp as usize) {
            shaped_map.insert(glyph_id, info);
        }
    }

    (
        glyphs_map,
        shaped_map,
        metrics,
        atlas_data,
        atlas_width,
        atlas_height,
        shelf_x,
        shelf_y,
        shelf_height,
    )
}
