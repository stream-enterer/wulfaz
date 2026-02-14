use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

mod components;
mod events;
mod font;
mod loading;
mod render;
mod rng;
mod systems;
mod tile_map;
mod world;

use components::Tick;
use systems::combat::run_combat;
use systems::death::run_death;
use systems::eating::run_eating;
use systems::hunger::run_hunger;
use systems::temperature::run_temperature;
use systems::wander::run_wander;
use world::World;

/// Convert sRGB component (0-1) to linear for use as wgpu clear color.
fn srgb_to_linear(s: f64) -> f64 {
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

const BG_SRGB: [f32; 3] = [40.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0]; // #282828
const FG_SRGB: [f32; 3] = [235.0 / 255.0, 219.0 / 255.0, 178.0 / 255.0]; // #ebdbb2

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
}

impl GpuState {
    fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("failed to find a suitable GPU adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("wulfaz_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            },
            None,
        ))
        .expect("failed to create GPU device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            window,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    fn render(&self, font: &font::FontRenderer, vertex_count: u32) {
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("out of GPU memory");
                return;
            }
            Err(e) => {
                log::warn!("surface error: {e:?}");
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: srgb_to_linear(BG_SRGB[0] as f64),
                            g: srgb_to_linear(BG_SRGB[1] as f64),
                            b: srgb_to_linear(BG_SRGB[2] as f64),
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            font.render(&mut render_pass, vertex_count);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

/// Simulation tick rate — matches Dwarf Fortress default FPS_CAP:100.
const SIM_TICKS_PER_SEC: f64 = 100.0;
const SIM_TICK_INTERVAL: f64 = 1.0 / SIM_TICKS_PER_SEC;
/// Cap simulation catch-up to avoid spiral of death after long pauses.
const MAX_TICKS_PER_FRAME: u32 = 5;

struct Camera {
    x: i32,
    y: i32,
}

struct App {
    gpu: Option<GpuState>,
    font: Option<font::FontRenderer>,
    world: World,
    camera: Camera,
    last_frame_time: Instant,
    tick_accumulator: f64,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Wulfaz")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let gpu = GpuState::new(window.clone());

        let font_renderer = font::FontRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.surface_format(),
            9.0,
            window.scale_factor(),
        );

        self.gpu = Some(gpu);
        self.font = Some(font_renderer);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != winit::event::ElementState::Pressed {
                    return;
                }
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                    // Camera movement: WASD + arrow keys
                    PhysicalKey::Code(KeyCode::KeyW | KeyCode::ArrowUp) => {
                        self.camera.y -= 1;
                    }
                    PhysicalKey::Code(KeyCode::KeyS | KeyCode::ArrowDown) => {
                        self.camera.y += 1;
                    }
                    PhysicalKey::Code(KeyCode::KeyA | KeyCode::ArrowLeft) => {
                        self.camera.x -= 1;
                    }
                    PhysicalKey::Code(KeyCode::KeyD | KeyCode::ArrowRight) => {
                        self.camera.x += 1;
                    }
                    _ => {}
                }
            }
            WindowEvent::Resized(new_size) => {
                gpu.resize(new_size);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.font = Some(font::FontRenderer::new(
                    &gpu.device,
                    &gpu.queue,
                    gpu.surface_format(),
                    9.0,
                    scale_factor,
                ));
            }
            WindowEvent::RedrawRequested => {
                // Fixed-timestep simulation: accumulate real time, run ticks at SIM_TICKS_PER_SEC
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame_time).as_secs_f64();
                self.last_frame_time = now;
                self.tick_accumulator += dt;

                let mut ticks_this_frame = 0u32;
                while self.tick_accumulator >= SIM_TICK_INTERVAL
                    && ticks_this_frame < MAX_TICKS_PER_FRAME
                {
                    let tick = self.world.tick;

                    // === Phase 1: Environment ===
                    run_temperature(&mut self.world, tick);

                    // === Phase 2: Needs ===
                    run_hunger(&mut self.world, tick);

                    // === Phase 3: Decisions ===
                    // (no decision systems yet)

                    // === Phase 4: Actions ===
                    run_wander(&mut self.world, tick);
                    run_eating(&mut self.world, tick);
                    run_combat(&mut self.world, tick);

                    // === Phase 5: Consequences ===
                    // run_death() is ALWAYS last in Phase 5
                    run_death(&mut self.world, tick);

                    // === Debug Validation ===
                    #[cfg(debug_assertions)]
                    world::validate_world(&self.world);

                    self.world.tick = Tick(tick.0 + 1);
                    self.tick_accumulator -= SIM_TICK_INTERVAL;
                    ticks_this_frame += 1;
                }

                // === Render ===
                if let Some(font) = self.font.as_mut() {
                    let m = font.metrics();
                    let screen_w = gpu.config.width;
                    let screen_h = gpu.config.height;
                    let padding = 4.0_f32;

                    // Layout: status bar (top), map (center, square cells), event log (bottom)
                    let status_lines = 1_usize;
                    let event_lines = 5_usize;
                    let status_h = status_lines as f32 * m.line_height;
                    let event_h = event_lines as f32 * m.line_height;
                    let map_cell = m.line_height; // square cells
                    let map_pixel_h = screen_h as f32 - status_h - event_h - padding * 4.0;
                    let map_pixel_w = screen_w as f32 - padding * 2.0;

                    let viewport_cols = (map_pixel_w / map_cell).floor().max(1.0) as usize;
                    let viewport_rows = (map_pixel_h / map_cell).floor().max(1.0) as usize;

                    // Clamp camera so viewport overlaps the tilemap
                    let map_w = self.world.tiles.width() as i32;
                    let map_h = self.world.tiles.height() as i32;
                    let max_cam_x = (map_w - 1).max(0);
                    let max_cam_y = (map_h - 1).max(0);
                    self.camera.x = self.camera.x.clamp(0, max_cam_x);
                    self.camera.y = self.camera.y.clamp(0, max_cam_y);

                    let status = render::render_status(&self.world);
                    let map_text = render::render_world_to_string(
                        &self.world,
                        self.camera.x,
                        self.camera.y,
                        viewport_cols,
                        viewport_rows,
                    );
                    let events = render::render_recent_events(&self.world, event_lines);

                    font.begin_frame(&gpu.queue, screen_w, screen_h, FG_SRGB, BG_SRGB);

                    // Status bar at top
                    font.prepare_text(&status, padding, padding);

                    // Map in center with square cells
                    let map_y = padding * 2.0 + status_h;
                    font.prepare_map(&map_text, padding, map_y);

                    // Event log at bottom
                    let event_y = screen_h as f32 - event_h - padding;
                    font.prepare_text(&events, padding, event_y);

                    let vertex_count = font.flush(&gpu.queue, &gpu.device);
                    gpu.render(font, vertex_count);
                }
                gpu.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let mut world = World::new_with_seed(42);

    // Load data from KDL files
    loading::load_terrain(&mut world, "data/terrain.kdl");
    loading::load_creatures(&mut world, "data/creatures.kdl");
    loading::load_items(&mut world, "data/items.kdl");

    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        gpu: None,
        font: None,
        world,
        camera: Camera { x: 0, y: 0 },
        last_frame_time: Instant::now(),
        tick_accumulator: 0.0,
    };
    event_loop.run_app(&mut app).unwrap();
}
