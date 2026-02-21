#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PanelVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub size_px: [f32; 2],
    pub bg_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub shadow_width: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PanelUniforms {
    pub projection: [[f32; 4]; 4],
}

pub struct PanelRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    vertex_capacity: usize,
    frame_vertices: Vec<PanelVertex>,
}

impl PanelRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("panel_uniforms"),
            size: std::mem::size_of::<PanelUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let initial_capacity = 600; // ~100 panels
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("panel_vertices"),
            size: (initial_capacity * std::mem::size_of::<PanelVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("panel_bind_group_layout"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("panel_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("panel_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("panel.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("panel_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("panel_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<PanelVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        // position
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // uv
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // size_px
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        // bg_color
                        wgpu::VertexAttribute {
                            offset: 24,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        // border_color
                        wgpu::VertexAttribute {
                            offset: 40,
                            shader_location: 4,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        // border_width
                        wgpu::VertexAttribute {
                            offset: 56,
                            shader_location: 5,
                            format: wgpu::VertexFormat::Float32,
                        },
                        // shadow_width
                        wgpu::VertexAttribute {
                            offset: 60,
                            shader_location: 6,
                            format: wgpu::VertexFormat::Float32,
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
            pipeline,
            bind_group,
            uniform_buffer,
            vertex_buffer,
            vertex_capacity: initial_capacity,
            frame_vertices: Vec::new(),
        }
    }

    /// Clear vertices and write the ortho projection uniform.
    pub fn begin_frame(&mut self, queue: &wgpu::Queue, screen_w: u32, screen_h: u32) {
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

        let uniforms = PanelUniforms { projection };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Push a panel quad (6 vertices) with the given style.
    #[allow(clippy::too_many_arguments)]
    pub fn add_panel(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        bg_color: [f32; 4],
        border_color: [f32; 4],
        border_width: f32,
        shadow_width: f32,
    ) {
        let x0 = x;
        let y0 = y;
        let x1 = x + w;
        let y1 = y + h;
        let size_px = [w, h];

        let make = |px: f32, py: f32, u: f32, v: f32| PanelVertex {
            position: [px, py],
            uv: [u, v],
            size_px,
            bg_color,
            border_color,
            border_width,
            shadow_width,
        };

        self.frame_vertices.push(make(x0, y0, 0.0, 0.0));
        self.frame_vertices.push(make(x1, y0, 1.0, 0.0));
        self.frame_vertices.push(make(x0, y1, 0.0, 1.0));

        self.frame_vertices.push(make(x1, y0, 1.0, 0.0));
        self.frame_vertices.push(make(x1, y1, 1.0, 1.0));
        self.frame_vertices.push(make(x0, y1, 0.0, 1.0));
    }

    /// Upload vertices to GPU. Returns vertex count for render().
    pub fn flush(&mut self, queue: &wgpu::Queue, device: &wgpu::Device) -> u32 {
        let vertex_count = self.frame_vertices.len() as u32;
        if self.frame_vertices.is_empty() {
            return 0;
        }

        if self.frame_vertices.len() > self.vertex_capacity {
            self.vertex_capacity = self.frame_vertices.len().next_power_of_two();
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("panel_vertices"),
                size: (self.vertex_capacity * std::mem::size_of::<PanelVertex>()) as u64,
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
