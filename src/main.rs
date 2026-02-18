use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Window, WindowId};

mod components;
mod events;
mod font;
mod loading;
mod loading_gis;
mod registry;
mod render;
mod rng;
mod systems;
mod tile_map;
mod world;

use components::Tick;
use systems::combat::run_combat;
use systems::death::run_death;
use systems::decisions::run_decisions;
use systems::eating::run_eating;
use systems::fatigue::run_fatigue;
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

        let surface = instance
            .create_surface(window.clone())
            .expect("create surface");

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

enum PlayerAction {
    Move(i32, i32),
    Wait,
}

fn run_one_tick(world: &mut World) {
    let tick = world.tick;
    run_temperature(world, tick);
    run_hunger(world, tick);
    run_fatigue(world, tick);
    run_decisions(world, tick);
    run_wander(world, tick);
    run_eating(world, tick);
    run_combat(world, tick);
    run_death(world, tick);
    #[cfg(debug_assertions)]
    world::validate_world(world);
    world.tick = Tick(tick.0 + 1);
}

struct App {
    gpu: Option<GpuState>,
    font: Option<font::FontRenderer>,
    world: World,
    camera: Camera,
    last_frame_time: Instant,
    tick_accumulator: f64,
    // Roguelike mode input state
    cursor_pos: winit::dpi::PhysicalPosition<f64>,
    modifiers: ModifiersState,
    pending_player_action: Option<PlayerAction>,
    // Map layout for click hit-testing (set during render)
    map_origin: (f32, f32),
    map_cell_size: f32,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Wulfaz")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));
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
        match event {
            // Input tracking (no GPU needed)
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            // Everything below requires GPU
            other => {
                let Some(gpu) = self.gpu.as_mut() else {
                    return;
                };
                match other {
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state != ElementState::Pressed {
                            return;
                        }
                        match event.physical_key {
                            PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                            // Camera: WASD + arrows (realtime mode only)
                            PhysicalKey::Code(KeyCode::KeyW | KeyCode::ArrowUp)
                                if self.world.player.is_none() =>
                            {
                                self.camera.y -= 1;
                            }
                            PhysicalKey::Code(KeyCode::KeyS | KeyCode::ArrowDown)
                                if self.world.player.is_none() =>
                            {
                                self.camera.y += 1;
                            }
                            PhysicalKey::Code(KeyCode::KeyA | KeyCode::ArrowLeft)
                                if self.world.player.is_none() =>
                            {
                                self.camera.x -= 1;
                            }
                            PhysicalKey::Code(KeyCode::KeyD | KeyCode::ArrowRight)
                                if self.world.player.is_none() =>
                            {
                                self.camera.x += 1;
                            }
                            // Numpad movement (roguelike mode)
                            PhysicalKey::Code(kc) if self.world.player.is_some() => {
                                let dir = match kc {
                                    KeyCode::Numpad7 => Some((-1, -1)),
                                    KeyCode::Numpad8 => Some((0, -1)),
                                    KeyCode::Numpad9 => Some((1, -1)),
                                    KeyCode::Numpad4 => Some((-1, 0)),
                                    KeyCode::Numpad5 => Some((0, 0)),
                                    KeyCode::Numpad6 => Some((1, 0)),
                                    KeyCode::Numpad1 => Some((-1, 1)),
                                    KeyCode::Numpad2 => Some((0, 1)),
                                    KeyCode::Numpad3 => Some((1, 1)),
                                    _ => None,
                                };
                                if let Some((dx, dy)) = dir {
                                    if dx == 0 && dy == 0 {
                                        self.pending_player_action = Some(PlayerAction::Wait);
                                    } else {
                                        self.pending_player_action =
                                            Some(PlayerAction::Move(dx, dy));
                                    }
                                }
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
                    // Shift+left-click: toggle player control on a creature
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    } if self.modifiers.shift_key() => {
                        if self.map_cell_size > 0.0 {
                            let px = self.cursor_pos.x as f32;
                            let py = self.cursor_pos.y as f32;
                            let vx = (px - self.map_origin.0) / self.map_cell_size;
                            let vy = (py - self.map_origin.1) / self.map_cell_size;
                            if vx >= 0.0 && vy >= 0.0 {
                                let tile_x = self.camera.x + vx as i32;
                                let tile_y = self.camera.y + vy as i32;
                                let mut candidates: Vec<components::Entity> = self
                                    .world
                                    .positions
                                    .iter()
                                    .filter(|(e, p)| {
                                        p.x == tile_x
                                            && p.y == tile_y
                                            && self.world.combat_stats.contains_key(e)
                                            && self.world.alive.contains(e)
                                    })
                                    .map(|(e, _)| *e)
                                    .collect();
                                candidates.sort_by_key(|e| e.0);
                                if let Some(e) = candidates.first().copied() {
                                    if self.world.player == Some(e) {
                                        self.world.player = None;
                                        self.last_frame_time = Instant::now();
                                        self.tick_accumulator = 0.0;
                                    } else {
                                        self.world.player = Some(e);
                                        self.world.move_cooldowns.remove(&e);
                                        self.world.wander_targets.remove(&e);
                                    }
                                }
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        // === Tick processing ===
                        if self.world.player.is_some() {
                            // Roguelike: advance on player action only
                            if let Some(action) = self.pending_player_action.take() {
                                let player = self.world.player.expect("player entity");
                                match action {
                                    PlayerAction::Move(dx, dy) => {
                                        let can_move =
                                            if let Some(pos) = self.world.positions.get(&player) {
                                                let mw = self.world.tiles.width() as i32;
                                                let mh = self.world.tiles.height() as i32;
                                                let tx = (pos.x + dx).clamp(0, (mw - 1).max(0));
                                                let ty = (pos.y + dy).clamp(0, (mh - 1).max(0));
                                                if self
                                                    .world
                                                    .tiles
                                                    .is_walkable(tx as usize, ty as usize)
                                                {
                                                    self.world.wander_targets.insert(
                                                        player,
                                                        components::WanderTarget {
                                                            goal_x: tx,
                                                            goal_y: ty,
                                                        },
                                                    );
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            };
                                        if can_move {
                                            run_one_tick(&mut self.world);
                                            let cooldown = self
                                                .world
                                                .move_cooldowns
                                                .get(&player)
                                                .map(|cd| cd.remaining)
                                                .unwrap_or(0);
                                            for _ in 0..cooldown {
                                                run_one_tick(&mut self.world);
                                            }
                                        }
                                    }
                                    PlayerAction::Wait => {
                                        let gait = self
                                            .world
                                            .current_gaits
                                            .get(&player)
                                            .copied()
                                            .unwrap_or(components::Gait::Walk);
                                        let base = self
                                            .world
                                            .gait_profiles
                                            .get(&player)
                                            .map(|p| p.cooldown(gait))
                                            .unwrap_or(9);
                                        self.world.move_cooldowns.insert(
                                            player,
                                            components::MoveCooldown { remaining: base },
                                        );
                                        for _ in 0..base {
                                            run_one_tick(&mut self.world);
                                        }
                                    }
                                }
                            }
                        } else {
                            // Realtime: fixed-timestep simulation
                            let now = Instant::now();
                            let dt = now.duration_since(self.last_frame_time).as_secs_f64();
                            self.last_frame_time = now;
                            self.tick_accumulator += dt;

                            let mut ticks_this_frame = 0u32;
                            while self.tick_accumulator >= SIM_TICK_INTERVAL
                                && ticks_this_frame < MAX_TICKS_PER_FRAME
                            {
                                run_one_tick(&mut self.world);
                                self.tick_accumulator -= SIM_TICK_INTERVAL;
                                ticks_this_frame += 1;
                            }
                        }

                        // === Render ===
                        if let Some(font) = self.font.as_mut() {
                            let m = font.metrics();
                            let screen_w = gpu.config.width;
                            let screen_h = gpu.config.height;
                            let padding = 4.0_f32;

                            let status_lines = 1_usize;
                            let event_lines = 5_usize;
                            let status_h = status_lines as f32 * m.line_height;
                            let event_h = event_lines as f32 * m.line_height;
                            let map_cell = m.line_height;
                            let map_pixel_h = screen_h as f32 - status_h - event_h - padding * 4.0;
                            let map_pixel_w = screen_w as f32 - padding * 2.0;

                            let viewport_cols = (map_pixel_w / map_cell).floor().max(1.0) as usize;
                            let viewport_rows = (map_pixel_h / map_cell).floor().max(1.0) as usize;

                            // Store layout for click hit-testing
                            let map_y = padding * 2.0 + status_h;
                            self.map_origin = (padding, map_y);
                            self.map_cell_size = map_cell;

                            // Camera: follow player in roguelike mode
                            if let Some(player) = self.world.player
                                && let Some(pos) = self.world.positions.get(&player)
                            {
                                self.camera.x = pos.x - viewport_cols as i32 / 2;
                                self.camera.y = pos.y - viewport_rows as i32 / 2;
                            }

                            let map_w = self.world.tiles.width() as i32;
                            let map_h = self.world.tiles.height() as i32;
                            let max_cam_x = (map_w - viewport_cols as i32).max(0);
                            let max_cam_y = (map_h - viewport_rows as i32).max(0);
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
                            font.prepare_text(&status, padding, padding);
                            font.prepare_map(&map_text, padding, map_y);
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
    }
}

fn main() {
    env_logger::init();

    let mut world = World::new_with_seed(42);

    // Load map: prefer binary tiles → fallback RON → fallback default terrain.
    let paris_tiles = std::path::Path::new("data/paris.tiles");
    let paris_meta = std::path::Path::new("data/paris.meta.ron");
    let paris_ron = std::path::Path::new("data/paris.ron");
    if paris_tiles.exists() && paris_meta.exists() {
        loading_gis::load_paris_binary(
            &mut world,
            paris_tiles.to_str().expect("tiles path UTF-8"),
            paris_meta.to_str().expect("meta path UTF-8"),
        );
    } else if paris_ron.exists() {
        let data = loading_gis::load_paris_ron(paris_ron.to_str().expect("ron path UTF-8"));
        loading_gis::apply_paris_ron(&mut world, data);
    } else {
        loading::load_terrain(&mut world, "data/terrain.kdl");
    }

    world.tiles.initialize_temperatures();

    loading::load_utility_config(&mut world, "data/utility.ron");

    let start_camera = Camera {
        x: world.tiles.width() as i32 / 2,
        y: world.tiles.height() as i32 / 2,
    };

    let event_loop = EventLoop::new().expect("create event loop");
    let mut app = App {
        gpu: None,
        font: None,
        world,
        camera: start_camera,
        last_frame_time: Instant::now(),
        tick_accumulator: 0.0,
        cursor_pos: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        modifiers: ModifiersState::empty(),
        pending_player_action: None,
        map_origin: (0.0, 0.0),
        map_cell_size: 0.0,
    };
    event_loop.run_app(&mut app).expect("run event loop");
}
