use std::sync::Arc;

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

struct App {
    gpu: Option<GpuState>,
    font: Option<font::FontRenderer>,
    world: World,
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
            WindowEvent::KeyboardInput { event, .. }
                if event.physical_key == PhysicalKey::Code(KeyCode::Escape) =>
            {
                event_loop.exit();
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
                // === Simulation Tick ===
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

                // === Render ===
                if let Some(font) = self.font.as_mut() {
                    let display_text = render::render_world_to_string(&self.world);
                    let status = render::render_status(&self.world);
                    let events = render::render_recent_events(&self.world, 5);
                    let full_text = if events.is_empty() {
                        format!("{}\n{}", status, display_text)
                    } else {
                        format!("{}\n{}\n{}", status, events, display_text)
                    };
                    let vertex_count = font.prepare(
                        &gpu.queue,
                        &gpu.device,
                        &full_text,
                        10.0,
                        10.0,
                        gpu.config.width,
                        gpu.config.height,
                        FG_SRGB,
                        BG_SRGB,
                    );
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
    };
    event_loop.run_app(&mut app).unwrap();
}
