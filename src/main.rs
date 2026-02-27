use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Window, WindowId};

use wulfaz::components;
use wulfaz::components::Tick;
use wulfaz::font;
use wulfaz::loading;
use wulfaz::loading_gis;
use wulfaz::panel;
use wulfaz::render;
use wulfaz::systems::combat::run_combat;
use wulfaz::systems::death::run_death;
use wulfaz::systems::decisions::run_decisions;
use wulfaz::systems::eating::run_eating;
use wulfaz::systems::fatigue::run_fatigue;
use wulfaz::systems::hunger::run_hunger;
use wulfaz::systems::temperature::run_temperature;
use wulfaz::systems::wander::run_wander;
use wulfaz::world::World;

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

/// Map overlay: light parchment, 15% alpha — hover highlight
const OVERLAY_HOVER: [f32; 4] = [0.941, 0.902, 0.824, 0.15];
/// Map overlay: gold, 35% alpha — selected entity
const OVERLAY_SELECTION: [f32; 4] = [0.784, 0.659, 0.314, 0.35];
/// Map overlay: muted green, 25% alpha — wander target
const OVERLAY_PATH: [f32; 4] = [0.376, 0.627, 0.376, 0.25];

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

    /// Render a frame: clear → map text → map overlay panels.
    fn render(
        &self,
        font: &font::FontRenderer,
        panel: &panel::PanelRenderer,
        map_text_vertices: u32,
        map_overlay_panel_vertices: u32,
    ) {
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
                label: Some("main_pass"),
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

            // Layer 1: map ASCII text
            font.render_range(&mut render_pass, 0, map_text_vertices);
            // Layer 2: map overlay panels (tile highlights)
            panel.render_range(&mut render_pass, 0, map_overlay_panel_vertices);
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
    /// Smooth float target for lerp interpolation.
    target_x: f32,
    target_y: f32,
    /// Zoom level: 1.0 = default (1 tile per character cell). >1 = zoomed in.
    zoom: f32,
    /// Target zoom for smooth interpolation.
    target_zoom: f32,
}

enum PlayerAction {
    Move(i32, i32),
    Wait,
}

fn run_one_tick(world: &mut World) {
    macro_rules! timed {
        ($label:expr, $body:expr) => {{
            let _t = std::time::Instant::now();
            $body;
            let _us = _t.elapsed().as_micros();
            if _us > 500 {
                log::warn!("  tick sys {}: {}us", $label, _us);
            }
        }};
    }
    // Spatial contract: Phase 2-3 (needs, decisions) see pre-movement positions.
    // Phase 4 (eating, combat) sees post-movement positions.
    // Adding a new position-mutating system requires placing a rebuild after it.
    timed!("spatial1", world.rebuild_spatial_index());
    let tick = world.tick;
    timed!("temperature", run_temperature(world, tick));
    timed!("hunger", run_hunger(world, tick));
    timed!("fatigue", run_fatigue(world, tick));
    timed!("decisions", run_decisions(world, tick));
    timed!("wander", run_wander(world, tick));
    // Spatial contract (rebuild 2 of 2): after wander mutates positions,
    // eating/combat need post-movement positions for same-tile checks.
    timed!("spatial2", world.rebuild_spatial_index());
    timed!("eating", run_eating(world, tick));
    timed!("combat", run_combat(world, tick));
    timed!("death", run_death(world, tick));
    #[cfg(debug_assertions)]
    timed!("validate", wulfaz::world::validate_world(world));
    world.tick = Tick(tick.0 + 1);
}

struct App {
    gpu: Option<GpuState>,
    font: Option<font::FontRenderer>,
    panel: Option<panel::PanelRenderer>,
    world: World,
    camera: Camera,
    last_frame_time: Instant,
    tick_accumulator: f64,
    cursor_pos: winit::dpi::PhysicalPosition<f64>,
    modifiers: ModifiersState,
    pending_player_action: Option<PlayerAction>,
    // Map layout for click hit-testing (set during render)
    map_origin: (f32, f32),
    map_cell_w: f32,
    map_cell_h: f32,
    paused: bool,
    sim_speed: u32, // 1 = normal, 2-5 = faster
    selected_entity: Option<components::Entity>,
    viewport_cols: usize,
    viewport_rows: usize,
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
            10.0,
            window.scale_factor(),
        );

        let panel_renderer = panel::PanelRenderer::new(&gpu.device, gpu.surface_format());

        self.gpu = Some(gpu);
        self.font = Some(font_renderer);
        self.panel = Some(panel_renderer);
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
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 20.0,
                };
                // Zoom-to-cursor.
                let zoom_factor = 1.1_f32;
                let old_zoom = self.camera.target_zoom;
                let new_zoom = if dy > 0.0 {
                    (old_zoom * zoom_factor).min(4.0)
                } else if dy < 0.0 {
                    (old_zoom / zoom_factor).max(0.25)
                } else {
                    old_zoom
                };
                if (new_zoom - old_zoom).abs() > f32::EPSILON && self.map_cell_w > 0.0 {
                    let cx = self.cursor_pos.x as f32 - self.map_origin.0;
                    let cy = self.cursor_pos.y as f32 - self.map_origin.1;
                    let base_w = self.map_cell_w / old_zoom;
                    let base_h = self.map_cell_h / old_zoom;
                    let dx = (cx / base_w) * (1.0 / old_zoom - 1.0 / new_zoom);
                    let dy_adj = (cy / base_h) * (1.0 / old_zoom - 1.0 / new_zoom);
                    self.camera.target_x += dx;
                    self.camera.target_y += dy_adj;
                }
                self.camera.target_zoom = new_zoom;
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
                        let PhysicalKey::Code(kc) = event.physical_key else {
                            return;
                        };

                        match kc {
                            KeyCode::Space => {
                                self.paused = !self.paused;
                                if !self.paused {
                                    self.last_frame_time = Instant::now();
                                    self.tick_accumulator = 0.0;
                                }
                            }
                            KeyCode::Digit1 => self.sim_speed = 1,
                            KeyCode::Digit2 => self.sim_speed = 2,
                            KeyCode::Digit3 => self.sim_speed = 3,
                            KeyCode::Digit4 => self.sim_speed = 4,
                            KeyCode::Digit5 => self.sim_speed = 5,
                            KeyCode::Escape => {
                                if self.selected_entity.is_some() {
                                    self.selected_entity = None;
                                } else {
                                    event_loop.exit();
                                }
                            }
                            // Camera: WASD + arrows (realtime mode only).
                            // Pan speed scales inversely with zoom.
                            KeyCode::KeyW | KeyCode::ArrowUp if self.world.player.is_none() => {
                                let speed = (3.0 / self.camera.zoom).max(1.0);
                                self.camera.target_y -= speed;
                            }
                            KeyCode::KeyS | KeyCode::ArrowDown if self.world.player.is_none() => {
                                let speed = (3.0 / self.camera.zoom).max(1.0);
                                self.camera.target_y += speed;
                            }
                            KeyCode::KeyA | KeyCode::ArrowLeft if self.world.player.is_none() => {
                                let speed = (3.0 / self.camera.zoom).max(1.0);
                                self.camera.target_x -= speed;
                            }
                            KeyCode::KeyD | KeyCode::ArrowRight if self.world.player.is_none() => {
                                let speed = (3.0 / self.camera.zoom).max(1.0);
                                self.camera.target_x += speed;
                            }
                            // Numpad movement (roguelike mode)
                            kc if self.world.player.is_some() => {
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
                            10.0,
                            scale_factor,
                        ));
                        self.panel =
                            Some(panel::PanelRenderer::new(&gpu.device, gpu.surface_format()));
                    }
                    WindowEvent::MouseInput {
                        state: btn_state,
                        button,
                        ..
                    } => {
                        let px = self.cursor_pos.x as f32;
                        let py = self.cursor_pos.y as f32;
                        let pressed = btn_state == ElementState::Pressed;

                        // Shift+left-click toggles player control.
                        if pressed
                            && button == MouseButton::Left
                            && self.modifiers.shift_key()
                            && self.map_cell_w > 0.0
                        {
                            let vx = (px - self.map_origin.0) / self.map_cell_w;
                            let vy = (py - self.map_origin.1) / self.map_cell_h;
                            if vx >= 0.0 && vy >= 0.0 {
                                let tile_x = self.camera.x + vx as i32;
                                let tile_y = self.camera.y + vy as i32;
                                let mut candidates: Vec<components::Entity> = self
                                    .world
                                    .body
                                    .positions
                                    .iter()
                                    .filter(|(e, p)| {
                                        p.x == tile_x
                                            && p.y == tile_y
                                            && self.world.body.combat_stats.contains_key(e)
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
                                        self.world.body.move_cooldowns.remove(&e);
                                        self.world.mind.wander_targets.remove(&e);
                                    }
                                }
                            }
                        }

                        // Plain left-click selects entity.
                        if pressed
                            && button == MouseButton::Left
                            && !self.modifiers.shift_key()
                            && self.map_cell_w > 0.0
                        {
                            let vx = (px - self.map_origin.0) / self.map_cell_w;
                            let vy = (py - self.map_origin.1) / self.map_cell_h;
                            if vx >= 0.0 && vy >= 0.0 {
                                let tile_x = self.camera.x + vx as i32;
                                let tile_y = self.camera.y + vy as i32;
                                let mut candidates: Vec<components::Entity> = self
                                    .world
                                    .body
                                    .positions
                                    .iter()
                                    .filter(|(e, p)| {
                                        p.x == tile_x
                                            && p.y == tile_y
                                            && self.world.alive.contains(e)
                                    })
                                    .map(|(e, _)| *e)
                                    .collect();
                                candidates.sort_by_key(|e| e.0);
                                self.selected_entity = candidates.first().copied();
                            } else {
                                self.selected_entity = None;
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
                                        let can_move = if let Some(pos) =
                                            self.world.body.positions.get(&player)
                                        {
                                            let mw = self.world.tiles.width() as i32;
                                            let mh = self.world.tiles.height() as i32;
                                            let tx = (pos.x + dx).clamp(0, (mw - 1).max(0));
                                            let ty = (pos.y + dy).clamp(0, (mh - 1).max(0));
                                            if self
                                                .world
                                                .tiles
                                                .is_walkable(tx as usize, ty as usize)
                                            {
                                                self.world.mind.wander_targets.insert(
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
                                                .body
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
                                            .body
                                            .current_gaits
                                            .get(&player)
                                            .copied()
                                            .unwrap_or(components::Gait::Walk);
                                        let base = self
                                            .world
                                            .body
                                            .gait_profiles
                                            .get(&player)
                                            .map(|p| p.cooldown(gait))
                                            .unwrap_or(9);
                                        self.world.body.move_cooldowns.insert(
                                            player,
                                            components::MoveCooldown { remaining: base },
                                        );
                                        for _ in 0..base {
                                            run_one_tick(&mut self.world);
                                        }
                                    }
                                }
                            }
                        } else if !self.paused {
                            // Realtime: fixed-timestep simulation
                            let now = Instant::now();
                            let dt = now.duration_since(self.last_frame_time).as_secs_f64();
                            self.last_frame_time = now;
                            self.tick_accumulator += dt * self.sim_speed as f64;

                            let mut ticks_this_frame = 0u32;
                            while self.tick_accumulator >= SIM_TICK_INTERVAL
                                && ticks_this_frame < MAX_TICKS_PER_FRAME
                            {
                                run_one_tick(&mut self.world);
                                self.tick_accumulator -= SIM_TICK_INTERVAL;
                                ticks_this_frame += 1;
                            }
                        } else {
                            // Paused: keep frame time current to avoid tick burst on unpause.
                            self.last_frame_time = Instant::now();
                        }

                        // === Render ===
                        if let (Some(font), Some(panel)) = (self.font.as_mut(), self.panel.as_mut())
                        {
                            let screen_w = gpu.config.width;
                            let screen_h = gpu.config.height;
                            let padding = 4.0_f32;

                            let (mcw, mch) = font.map_cell();
                            let map_y = padding;
                            let map_pixel_h = screen_h as f32 - padding * 2.0;
                            let map_pixel_w = screen_w as f32 - padding * 2.0;

                            let viewport_cols = (map_pixel_w / mcw).floor().max(1.0) as usize;
                            let viewport_rows = (map_pixel_h / mch).floor().max(1.0) as usize;

                            // Store layout for click hit-testing
                            self.map_origin = (padding, map_y);
                            self.map_cell_w = mcw;
                            self.map_cell_h = mch;
                            self.viewport_cols = viewport_cols;
                            self.viewport_rows = viewport_rows;

                            // Camera: follow player in roguelike mode
                            if let Some(player) = self.world.player
                                && let Some(pos) = self.world.body.positions.get(&player)
                            {
                                self.camera.x = pos.x - viewport_cols as i32 / 2;
                                self.camera.y = pos.y - viewport_rows as i32 / 2;
                                self.camera.target_x = self.camera.x as f32;
                                self.camera.target_y = self.camera.y as f32;
                            } else {
                                // Smooth lerp: camera position interpolates toward target.
                                let lerp_factor = 0.15_f32;
                                self.camera.zoom +=
                                    (self.camera.target_zoom - self.camera.zoom) * lerp_factor;
                                let tx = self.camera.target_x;
                                let ty = self.camera.target_y;
                                let curr_x = self.camera.x as f32;
                                let curr_y = self.camera.y as f32;
                                let new_x = curr_x + (tx - curr_x) * lerp_factor;
                                let new_y = curr_y + (ty - curr_y) * lerp_factor;
                                self.camera.x = new_x.round() as i32;
                                self.camera.y = new_y.round() as i32;
                            }

                            let map_w = self.world.tiles.width() as i32;
                            let map_h = self.world.tiles.height() as i32;
                            let max_cam_x = (map_w - viewport_cols as i32).max(0);
                            let max_cam_y = (map_h - viewport_rows as i32).max(0);
                            self.camera.target_x =
                                self.camera.target_x.clamp(0.0, max_cam_x as f32);
                            self.camera.target_y =
                                self.camera.target_y.clamp(0.0, max_cam_y as f32);
                            self.camera.x = self.camera.x.clamp(0, max_cam_x);
                            self.camera.y = self.camera.y.clamp(0, max_cam_y);

                            let map_text = render::render_world_to_string(
                                &self.world,
                                self.camera.x,
                                self.camera.y,
                                viewport_cols,
                                viewport_rows,
                            );

                            // Panels: map overlays.
                            let sw = screen_w as f32;
                            let sh = screen_h as f32;
                            let no_border = [0.0_f32; 4];
                            let no_clip_min = [0.0_f32, 0.0];
                            let no_clip_max = [sw, sh];

                            panel.begin_frame(&gpu.queue, screen_w, screen_h);

                            // Map overlay: hover tile highlight.
                            {
                                let px = self.cursor_pos.x as f32;
                                let py = self.cursor_pos.y as f32;
                                let vx = (px - self.map_origin.0) / mcw;
                                let vy = (py - self.map_origin.1) / mch;
                                if vx >= 0.0
                                    && vy >= 0.0
                                    && (vx as usize) < viewport_cols
                                    && (vy as usize) < viewport_rows
                                {
                                    let ox = self.map_origin.0 + (vx.floor()) * mcw;
                                    let oy = self.map_origin.1 + (vy.floor()) * mch;
                                    panel.add_panel(
                                        ox,
                                        oy,
                                        mcw,
                                        mch,
                                        OVERLAY_HOVER,
                                        no_border,
                                        0.0,
                                        0.0,
                                        no_clip_min,
                                        no_clip_max,
                                    );
                                }
                            }

                            // Map overlay: selected entity highlight.
                            if let Some(entity) = self.selected_entity
                                && let Some(pos) = self.world.body.positions.get(&entity)
                            {
                                let vx = pos.x - self.camera.x;
                                let vy = pos.y - self.camera.y;
                                if vx >= 0
                                    && vy >= 0
                                    && (vx as usize) < viewport_cols
                                    && (vy as usize) < viewport_rows
                                {
                                    let ox = self.map_origin.0 + vx as f32 * mcw;
                                    let oy = self.map_origin.1 + vy as f32 * mch;
                                    panel.add_panel(
                                        ox,
                                        oy,
                                        mcw,
                                        mch,
                                        OVERLAY_SELECTION,
                                        no_border,
                                        0.0,
                                        0.0,
                                        no_clip_min,
                                        no_clip_max,
                                    );
                                }

                                // Wander target highlight.
                                if let Some(target) = self.world.mind.wander_targets.get(&entity) {
                                    let tx = target.goal_x - self.camera.x;
                                    let ty = target.goal_y - self.camera.y;
                                    if tx >= 0
                                        && ty >= 0
                                        && (tx as usize) < viewport_cols
                                        && (ty as usize) < viewport_rows
                                    {
                                        let ox = self.map_origin.0 + tx as f32 * mcw;
                                        let oy = self.map_origin.1 + ty as f32 * mch;
                                        panel.add_panel(
                                            ox,
                                            oy,
                                            mcw,
                                            mch,
                                            OVERLAY_PATH,
                                            no_border,
                                            0.0,
                                            0.0,
                                            no_clip_min,
                                            no_clip_max,
                                        );
                                    }
                                }
                            }

                            let map_overlay_panel_vertices = panel.pending_vertex_count();

                            // Text: map ASCII.
                            font.begin_frame(&gpu.queue, screen_w, screen_h, FG_SRGB, BG_SRGB);
                            let fg4 = [FG_SRGB[0], FG_SRGB[1], FG_SRGB[2], 1.0];
                            font.prepare_map(&map_text, padding, map_y, fg4);
                            let map_text_vertices = font.pending_vertex_count();

                            panel.flush(&gpu.queue, &gpu.device);
                            font.flush(&gpu.queue, &gpu.device);

                            gpu.render(font, panel, map_text_vertices, map_overlay_panel_vertices);
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

    // Load map: prefer binary tiles+bincode meta → fallback RON → fallback default terrain.
    let paris_tiles = std::path::Path::new("data/paris.tiles");
    let paris_meta = std::path::Path::new("data/paris.meta.bin");
    let paris_ron = std::path::Path::new("data/paris.ron.zst");
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

    let archetypes = loading::load_archetypes("data/archetypes.kdl");
    let person = archetypes
        .get("person")
        .expect("data/archetypes.kdl must define a 'person' archetype");
    loading_gis::spawn_gis_entities(&mut world, "Arcis", person);

    // Center camera on the Arcis quartier (centroid of spawned entity positions).
    let start_camera = {
        let (mut sum_x, mut sum_y, mut count) = (0i64, 0i64, 0u32);
        for &e in &world.alive {
            if let Some(pos) = world.body.positions.get(&e) {
                sum_x += pos.x as i64;
                sum_y += pos.y as i64;
                count += 1;
            }
        }
        let (cx, cy) = if count > 0 {
            ((sum_x / count as i64) as i32, (sum_y / count as i64) as i32)
        } else {
            (3750, 3450) // fallback: Seine near Île de la Cité
        };
        Camera {
            x: cx,
            y: cy,
            target_x: cx as f32,
            target_y: cy as f32,
            zoom: 1.0,
            target_zoom: 1.0,
        }
    };

    let event_loop = EventLoop::new().expect("create event loop");
    let mut app = App {
        gpu: None,
        font: None,
        panel: None,
        world,
        camera: start_camera,
        last_frame_time: Instant::now(),
        tick_accumulator: 0.0,
        cursor_pos: winit::dpi::PhysicalPosition::new(0.0, 0.0),
        modifiers: ModifiersState::empty(),
        pending_player_action: None,
        map_origin: (0.0, 0.0),
        map_cell_w: 0.0,
        map_cell_h: 0.0,
        paused: false,
        sim_speed: 1,
        selected_entity: None,
        viewport_cols: 0,
        viewport_rows: 0,
    };
    event_loop.run_app(&mut app).expect("run event loop");
}
