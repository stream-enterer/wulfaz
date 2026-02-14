#![allow(dead_code)]

use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::c_int;

use freetype::face::LoadFlag;
use freetype::Library;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
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
    pub advance_x: f32,
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

#[derive(Clone, Copy)]
pub struct FontMetrics {
    pub ascender: f32,
    pub descender: f32,
    pub line_height: f32,
    pub cell_width: f32,
}

pub struct FontRenderer {
    glyphs: HashMap<u32, GlyphInfo>,
    metrics: FontMetrics,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
}

impl FontRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        font_size_pts: f32,
        scale_factor: f64,
    ) -> Self {
        let (glyphs, metrics, atlas_data, atlas_width, atlas_height) =
            rasterize_glyphs(font_size_pts, scale_factor);

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
        }
    }

    pub fn metrics(&self) -> FontMetrics {
        self.metrics
    }

    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        text: &str,
        x: f32,
        y: f32,
        screen_w: u32,
        screen_h: u32,
        fg: [f32; 3],
        bg: [f32; 3],
    ) -> u32 {
        let sw = screen_w as f32;
        let sh = screen_h as f32;

        // Orthographic projection: pixel coords → clip space
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

        // Build vertex data — monospace grid with integer-pixel positions (matching kitty)
        let mut vertices: Vec<TextVertex> = Vec::new();
        let cell_w = self.metrics.cell_width;
        let mut pen_x = x.floor();
        let pen_y = (y + self.metrics.ascender).floor();
        let question = self.glyphs.get(&(b'?' as u32)).copied();

        for ch in text.chars() {
            let cp = ch as u32;
            let glyph = match self.glyphs.get(&cp) {
                Some(g) => *g,
                None => match question {
                    Some(g) => g,
                    None => {
                        pen_x += cell_w;
                        continue;
                    }
                },
            };

            if glyph.width == 0 || glyph.height == 0 {
                pen_x += cell_w;
                continue;
            }

            // Integer-snap glyph position within cell
            let x0 = (pen_x + glyph.bearing_x as f32).floor();
            let y0 = (pen_y - glyph.bearing_y as f32).floor();
            let x1 = x0 + glyph.width as f32;
            let y1 = y0 + glyph.height as f32;

            // Two triangles per glyph quad
            vertices.push(TextVertex { position: [x0, y0], uv: [glyph.u0, glyph.v0] });
            vertices.push(TextVertex { position: [x1, y0], uv: [glyph.u1, glyph.v0] });
            vertices.push(TextVertex { position: [x0, y1], uv: [glyph.u0, glyph.v1] });

            vertices.push(TextVertex { position: [x1, y0], uv: [glyph.u1, glyph.v0] });
            vertices.push(TextVertex { position: [x1, y1], uv: [glyph.u1, glyph.v1] });
            vertices.push(TextVertex { position: [x0, y1], uv: [glyph.u0, glyph.v1] });

            pen_x += cell_w;
        }

        let vertex_count = vertices.len() as u32;
        if vertices.is_empty() {
            return 0;
        }

        // Grow vertex buffer if needed
        if vertices.len() > self.vertex_capacity {
            self.vertex_capacity = vertices.len().next_power_of_two();
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
            bytemuck::cast_slice(&vertices),
        );

        vertex_count
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
    font_path: String,
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
        fn FcPatternGetString(
            p: *const std::ffi::c_void,
            object: *const std::ffi::c_char,
            n: c_int,
            s: *mut *const u8,
        ) -> c_int;
        fn FcPatternDestroy(p: *mut std::ffi::c_void);
    }

    let fc_family = CString::new("family").unwrap();
    let fc_file = CString::new("file").unwrap();
    let fc_hinting = CString::new("hinting").unwrap();
    let fc_hintstyle = CString::new("hintstyle").unwrap();

    for family in families {
        let family_c = CString::new(*family).unwrap();
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

            // Read font path
            let mut path_ptr: *const u8 = std::ptr::null();
            if FcPatternGetString(matched, fc_file.as_ptr(), 0, &mut path_ptr) != 0 {
                FcPatternDestroy(matched);
                continue;
            }
            let path = std::ffi::CStr::from_ptr(path_ptr as *const std::ffi::c_char)
                .to_string_lossy()
                .into_owned();

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
                font_path: path,
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

/// Find system monospace font path and hinting config via fontconfig, with fallbacks.
fn find_font_with_hinting() -> (String, LoadFlag) {
    let families = &[
        "Noto Sans Mono",
        "monospace",
        "DejaVu Sans Mono",
        "Liberation Mono",
    ];

    if let Some(config) = query_fontconfig(families) {
        let flags = hinting_load_flags(&config);
        return (config.font_path, flags);
    }

    // Fallback: no fontconfig, hardcoded paths, default hinting
    for path in &[
        "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    ] {
        if std::path::Path::new(path).exists() {
            return (path.to_string(), LoadFlag::TARGET_LIGHT);
        }
    }
    panic!("no monospace font found");
}

/// Rasterize ASCII 32-126 and pack into a glyph atlas.
/// Returns (glyphs, metrics, atlas_pixels, atlas_width, atlas_height).
fn rasterize_glyphs(
    font_size_pts: f32,
    scale_factor: f64,
) -> (HashMap<u32, GlyphInfo>, FontMetrics, Vec<u8>, u32, u32) {
    let (font_path, hinting_flags) = find_font_with_hinting();
    let dpi = (scale_factor * 96.0) as u32;

    let lib = Library::init().expect("freetype init failed");
    let face = lib
        .new_face(&font_path, 0)
        .expect("failed to load font face");

    // Set char size in 1/64th of a point
    let size_26_6 = (font_size_pts * 64.0).ceil() as isize;
    face.set_char_size(0, size_26_6, dpi, dpi)
        .expect("failed to set char size");

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
        descender: (size_metrics.descender as f32 / 64.0).ceil(),
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
        advance_x: f32,
        pixels: Vec<u8>,
    }

    let mut raw_glyphs: Vec<RawGlyph> = Vec::new();

    for cp in 32u32..=126 {
        if face.load_char(cp as usize, load_flags).is_err() {
            continue;
        }
        let glyph = face.glyph();
        let bitmap = glyph.bitmap();
        let w = bitmap.width() as u32;
        let h = bitmap.rows() as u32;
        let pitch = bitmap.pitch();
        let advance_x = glyph.advance().x as f32 / 64.0;

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
            advance_x,
            pixels,
        });
    }

    // Shelf-pack atlas: sort by height descending for better packing
    raw_glyphs.sort_by(|a, b| b.height.cmp(&a.height));

    let atlas_width: u32 = 512;
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

    let content_height = shelf_y + shelf_height;
    let atlas_height = content_height.next_power_of_two().max(1);

    // Blit glyphs into atlas
    let mut atlas_data = vec![0u8; (atlas_width * atlas_height) as usize];
    let mut glyphs_map = HashMap::new();

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
            advance_x: g.advance_x,
            u0: pos.x as f32 / aw,
            v0: pos.y as f32 / ah,
            u1: (pos.x + g.width) as f32 / aw,
            v1: (pos.y + g.height) as f32 / ah,
        };
        glyphs_map.insert(g.cp, info);
    }

    (glyphs_map, metrics, atlas_data, atlas_width, atlas_height)
}
