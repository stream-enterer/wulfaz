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
mod panel;
mod registry;
mod render;
mod rng;
mod systems;
mod tile_map;
#[allow(dead_code)] // Public API for Tier 2+ UI tasks
mod ui;
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

    fn render(
        &self,
        panel: &panel::PanelRenderer,
        panel_vertex_count: u32,
        font: &font::FontRenderer,
        text_vertex_count: u32,
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

            panel.render(&mut render_pass, panel_vertex_count);
            font.render(&mut render_pass, text_vertex_count);
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

/// Extract structured hover data from a map tile (UI-I01b).
/// Returns None if coords are out of bounds or no terrain.
fn collect_hover_info(world: &World, tile_x: i32, tile_y: i32) -> Option<ui::HoverInfo> {
    if tile_x < 0 || tile_y < 0 {
        return None;
    }
    let ux = tile_x as usize;
    let uy = tile_y as usize;
    let terrain = world.tiles.get_terrain(ux, uy)?;

    let mut info = ui::HoverInfo {
        tile_x,
        tile_y,
        terrain: format!("{:?}", terrain),
        quartier: None,
        address: None,
        building_name: None,
        occupants: Vec::new(),
        occupant_year_suffix: None,
        entities: Vec::new(),
    };

    // Quartier
    if let Some(qid) = world.tiles.get_quartier_id(ux, uy)
        && qid > 0
        && let Some(name) = world.gis.quartier_names.get((qid - 1) as usize)
    {
        info.quartier = Some(name.clone());
    }

    // Building
    if let Some(bid) = world.tiles.get_building_id(ux, uy)
        && let Some(building) = world.gis.buildings.get(bid)
    {
        if let Some(a) = building.addresses.first() {
            info.address = Some(format!("{} {}", a.house_number, a.street_name));
        }
        info.building_name = building.nom_bati.clone();

        if let Some((year, occupants)) = building.occupants_nearest(world.gis.active_year, 20) {
            info.occupants = occupants
                .iter()
                .map(|o| (o.name.clone(), o.activity.clone()))
                .collect();
            if year != world.gis.active_year {
                info.occupant_year_suffix = Some(format!("[{}]", year));
            }
        }
    }

    // Entities on this tile
    for (&entity, pos) in &world.body.positions {
        if pos.x == tile_x && pos.y == tile_y && world.alive.contains(&entity) {
            let name = world
                .body
                .names
                .get(&entity)
                .map(|n| n.value.clone())
                .unwrap_or_else(|| format!("E{}", entity.0));
            let icon = world.body.icons.get(&entity).map(|i| i.ch).unwrap_or('?');
            info.entities.push((icon, name));
        }
    }
    info.entities.sort_by(|a, b| a.1.cmp(&b.1));

    Some(info)
}

struct App {
    gpu: Option<GpuState>,
    font: Option<font::FontRenderer>,
    panel: Option<panel::PanelRenderer>,
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
    map_cell_w: f32,
    map_cell_h: f32,
    // UI widget system (UI-W02)
    ui_state: ui::UiState,
    ui_tree: ui::WidgetTree,
    ui_theme: ui::Theme,
    // Animation system (UI-W05)
    animator: ui::Animator,
    last_hover_tile: Option<(i32, i32)>,
    last_selected_entity: Option<components::Entity>,
    // Keyboard shortcut system (UI-I03)
    keybindings: ui::KeyBindings,
    paused: bool,
    sim_speed: u32, // 1 = normal, 2-5 = faster
    // Entity inspector (UI-I01d)
    selected_entity: Option<components::Entity>,
    inspector_close_id: Option<ui::WidgetId>,
    // Widget showcase (UI-DEMO)
    show_demo: bool,
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
                // Route to UI input system.
                self.ui_state.handle_cursor_moved(
                    &mut self.ui_tree,
                    position.x as f32,
                    position.y as f32,
                );
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 20.0,
                };
                self.ui_state.handle_scroll(&mut self.ui_tree, dy);
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

                        // 1. Global keybindings (UI-I03) — processed before widget focus.
                        let combo = ui::KeyCombo {
                            modifiers: ui::ModifierFlags {
                                shift: self.modifiers.shift_key(),
                                ctrl: self.modifiers.control_key(),
                                alt: self.modifiers.alt_key(),
                            },
                            key: kc,
                        };
                        if let Some(action) = self.keybindings.lookup(combo) {
                            match action {
                                ui::Action::PauseSim => {
                                    self.paused = !self.paused;
                                    if !self.paused {
                                        // Reset accumulator to avoid tick burst on unpause.
                                        self.last_frame_time = Instant::now();
                                        self.tick_accumulator = 0.0;
                                    }
                                }
                                ui::Action::SpeedSet(speed) => {
                                    self.sim_speed = speed;
                                }
                                ui::Action::ToggleDemo => {
                                    self.show_demo = !self.show_demo;
                                    if self.show_demo {
                                        self.animator.start(
                                            "demo_slide",
                                            -1.0,
                                            0.0,
                                            std::time::Duration::from_millis(
                                                self.ui_theme.anim_inspector_slide_ms,
                                            ),
                                            ui::Easing::EaseOut,
                                            Instant::now(),
                                        );
                                    } else {
                                        self.animator.remove("demo_slide");
                                    }
                                }
                                ui::Action::CloseTopmost => {
                                    // Priority: tooltips → inspector → demo → exit.
                                    if self.ui_state.tooltip_count() > 0 {
                                        self.ui_state.dismiss_all_tooltips(
                                            &mut self.ui_tree,
                                            Instant::now(),
                                        );
                                    } else if self.selected_entity.is_some() {
                                        self.selected_entity = None;
                                    } else if self.show_demo {
                                        self.show_demo = false;
                                        self.animator.remove("demo_slide");
                                    } else {
                                        event_loop.exit();
                                    }
                                }
                            }
                            return;
                        }

                        // 2. UI widget focus dispatch (Tab, ScrollList nav).
                        if self.ui_state.handle_key_input(&mut self.ui_tree, kc, true) {
                            return;
                        }

                        // 3. Game keys.
                        match kc {
                            // Camera: WASD + arrows (realtime mode only)
                            KeyCode::KeyW | KeyCode::ArrowUp if self.world.player.is_none() => {
                                self.camera.y -= 1;
                            }
                            KeyCode::KeyS | KeyCode::ArrowDown if self.world.player.is_none() => {
                                self.camera.y += 1;
                            }
                            KeyCode::KeyA | KeyCode::ArrowLeft if self.world.player.is_none() => {
                                self.camera.x -= 1;
                            }
                            KeyCode::KeyD | KeyCode::ArrowRight if self.world.player.is_none() => {
                                self.camera.x += 1;
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
                            9.0,
                            scale_factor,
                        ));
                        self.panel =
                            Some(panel::PanelRenderer::new(&gpu.device, gpu.surface_format()));
                    }
                    // Mouse click: route to UI first, then game
                    WindowEvent::MouseInput {
                        state: btn_state,
                        button,
                        ..
                    } => {
                        let px = self.cursor_pos.x as f32;
                        let py = self.cursor_pos.y as f32;
                        let ui_btn = match button {
                            MouseButton::Left => ui::MouseButton::Left,
                            MouseButton::Right => ui::MouseButton::Right,
                            MouseButton::Middle => ui::MouseButton::Middle,
                            _ => ui::MouseButton::Left,
                        };
                        let pressed = btn_state == ElementState::Pressed;

                        // UI consumes click — don't pass to game.
                        if self.ui_state.handle_mouse_input(
                            &mut self.ui_tree,
                            ui_btn,
                            pressed,
                            px,
                            py,
                        ) {
                            // Check if the close button was clicked (UI-I01d).
                            if pressed
                                && button == MouseButton::Left
                                && self.inspector_close_id.is_some()
                            {
                                let hit = self.ui_tree.hit_test(px, py);
                                if hit == self.inspector_close_id {
                                    self.selected_entity = None;
                                }
                            }
                            return;
                        }

                        // Game: Shift+left-click toggles player control.
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

                        // Game: plain left-click selects entity for inspector (UI-I01d).
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
                            // Realtime: fixed-timestep simulation (UI-I03: pause + speed)
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
                            let m = font.metrics();
                            let screen_w = gpu.config.width;
                            let screen_h = gpu.config.height;
                            let padding = 4.0_f32;

                            // Rebuild game UI tree every frame (DD-5: full rebuild).
                            let player_name = self
                                .world
                                .player
                                .and_then(|p| self.world.body.names.get(&p))
                                .map(|n| n.value.as_str());
                            self.ui_tree = ui::WidgetTree::new();
                            let status_info = ui::StatusBarInfo {
                                tick: self.world.tick.0,
                                population: self.world.alive.len(),
                                is_turn_based: self.world.player.is_some(),
                                player_name,
                                paused: self.paused,
                                sim_speed: self.sim_speed,
                                keybindings: &self.keybindings,
                                screen_width: screen_w as f32,
                            };
                            let status_bar_id = ui::build_status_bar(
                                &mut self.ui_tree,
                                &self.ui_theme,
                                &status_info,
                            );

                            // Layout UI tree to compute status bar height.
                            let screen_size = ui::Size {
                                width: screen_w as f32,
                                height: screen_h as f32,
                            };
                            self.ui_tree.layout(screen_size, m.line_height);
                            let status_bar_h = self
                                .ui_tree
                                .node_rect(status_bar_id)
                                .map(|r| r.height)
                                .unwrap_or(m.line_height);

                            // Screen layout: status bar | gap | map | gap | event log | gap
                            // Hover tooltip is an overlay (positioned at cursor), not a fixed row.
                            let event_log_visible = 5_usize;
                            let event_log_h = event_log_visible as f32
                                * self.ui_theme.scroll_item_height
                                + self.ui_theme.status_bar_padding_v * 2.0;

                            // Build event log panel (UI-I01c).
                            let event_entries = ui::collect_event_entries(
                                &self.world.events,
                                &self.world.body.names,
                            );
                            let event_log_id = ui::build_event_log(
                                &mut self.ui_tree,
                                &self.ui_theme,
                                &event_entries,
                                screen_w as f32,
                                event_log_h,
                            );
                            let event_log_y = screen_h as f32 - event_log_h - padding;
                            self.ui_tree.set_position(
                                event_log_id,
                                ui::Position::Fixed {
                                    x: 0.0,
                                    y: event_log_y,
                                },
                            );

                            let (mcw, mch) = font.map_cell();
                            let map_y = status_bar_h + padding;
                            let map_pixel_h =
                                screen_h as f32 - status_bar_h - event_log_h - padding * 3.0;
                            let map_pixel_w = screen_w as f32 - padding * 2.0;

                            let viewport_cols = (map_pixel_w / mcw).floor().max(1.0) as usize;
                            let viewport_rows = (map_pixel_h / mch).floor().max(1.0) as usize;

                            // Store layout for click hit-testing
                            self.map_origin = (padding, map_y);
                            self.map_cell_w = mcw;
                            self.map_cell_h = mch;

                            // Camera: follow player in roguelike mode
                            if let Some(player) = self.world.player
                                && let Some(pos) = self.world.body.positions.get(&player)
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

                            // Build hover tooltip if cursor is over a map tile (UI-I01b).
                            let now = Instant::now();
                            let mut hover_tooltip_id = None;
                            {
                                let px = self.cursor_pos.x as f32;
                                let py = self.cursor_pos.y as f32;
                                let vx = (px - self.map_origin.0) / self.map_cell_w;
                                let vy = (py - self.map_origin.1) / self.map_cell_h;
                                if vx >= 0.0
                                    && vy >= 0.0
                                    && (vx as usize) < viewport_cols
                                    && (vy as usize) < viewport_rows
                                {
                                    let tile_x = self.camera.x + vx as i32;
                                    let tile_y = self.camera.y + vy as i32;
                                    if let Some(info) =
                                        collect_hover_info(&self.world, tile_x, tile_y)
                                    {
                                        let tooltip_id = ui::build_hover_tooltip(
                                            &mut self.ui_tree,
                                            &self.ui_theme,
                                            &info,
                                            (px, py),
                                            screen_size,
                                            m.line_height,
                                        );
                                        hover_tooltip_id = Some(tooltip_id);

                                        // Start fade-in when hovering a new tile (UI-W05).
                                        let current_tile = (tile_x, tile_y);
                                        if self.last_hover_tile != Some(current_tile) {
                                            self.last_hover_tile = Some(current_tile);
                                            self.animator.start(
                                                "hover_tooltip",
                                                0.0,
                                                1.0,
                                                std::time::Duration::from_millis(
                                                    self.ui_theme.anim_tooltip_fade_ms,
                                                ),
                                                ui::Easing::EaseOut,
                                                now,
                                            );
                                        }
                                    }
                                }
                                if hover_tooltip_id.is_none() {
                                    self.last_hover_tile = None;
                                    self.animator.remove("hover_tooltip");
                                }
                            }

                            // Build entity inspector if selected (UI-I01d).
                            self.inspector_close_id = None;
                            let mut _inspector_panel_id = None;
                            if let Some(entity) = self.selected_entity {
                                if let Some(info) = ui::collect_inspector_info(entity, &self.world)
                                {
                                    // Start slide-in when entity first selected (UI-W05).
                                    if self.last_selected_entity != Some(entity) {
                                        self.last_selected_entity = Some(entity);
                                        self.animator.start(
                                            "inspector_slide",
                                            1.0,
                                            0.0,
                                            std::time::Duration::from_millis(
                                                self.ui_theme.anim_inspector_slide_ms,
                                            ),
                                            ui::Easing::EaseOut,
                                            now,
                                        );
                                    }

                                    let (inspector_id, close_id) = ui::build_entity_inspector(
                                        &mut self.ui_tree,
                                        &self.ui_theme,
                                        &info,
                                        screen_h as f32,
                                    );

                                    // Apply slide-in offset (UI-W05).
                                    let slide =
                                        self.animator.get("inspector_slide", now).unwrap_or(0.0);
                                    let target_x = screen_w as f32 - 220.0 - padding;
                                    let slide_offset = slide * (220.0 + padding);
                                    self.ui_tree.set_position(
                                        inspector_id,
                                        ui::Position::Fixed {
                                            x: target_x + slide_offset,
                                            y: status_bar_h + padding,
                                        },
                                    );
                                    self.inspector_close_id = Some(close_id);
                                    _inspector_panel_id = Some(inspector_id);
                                } else {
                                    // Entity died or lost position — auto-close.
                                    self.selected_entity = None;
                                    self.last_selected_entity = None;
                                    self.animator.remove("inspector_slide");
                                }
                            } else {
                                self.last_selected_entity = None;
                                self.animator.remove("inspector_slide");
                            }

                            // Build widget showcase (UI-DEMO) when active.
                            let mut demo_root_id = None;
                            if self.show_demo {
                                // Pick first alive entity for live data section.
                                let first_entity =
                                    self.world.alive.iter().copied().min_by_key(|e| e.0);
                                let entity_info = first_entity
                                    .and_then(|e| ui::collect_inspector_info(e, &self.world));
                                let live = ui::demo::DemoLiveData {
                                    entity_info: entity_info.as_ref(),
                                    tick: self.world.tick.0,
                                    population: self.world.alive.len(),
                                };
                                let demo_id = ui::demo::build_demo(
                                    &mut self.ui_tree,
                                    &self.ui_theme,
                                    &self.keybindings,
                                    &live,
                                    screen_size,
                                );
                                demo_root_id = Some(demo_id);
                            }

                            // Re-layout tree with all widgets included.
                            self.ui_tree.layout(screen_size, m.line_height);

                            // Apply demo slide-in animation (UI-DEMO + UI-W05).
                            if let Some(demo_id) = demo_root_id {
                                let slide = self.animator.get("demo_slide", now).unwrap_or(0.0);
                                if slide < 0.0 {
                                    // Slide from off-screen left: offset = slide * panel_width.
                                    let offset = slide * 404.0; // 400 + 4 margin
                                    self.ui_tree.set_position(
                                        demo_id,
                                        ui::Position::Fixed {
                                            x: 4.0 + offset,
                                            y: 4.0,
                                        },
                                    );
                                    // Need re-layout after position change.
                                    self.ui_tree.layout(screen_size, m.line_height);
                                }
                            }

                            // Apply hover tooltip fade-in (UI-W05).
                            if let Some(tooltip_id) = hover_tooltip_id {
                                let opacity =
                                    self.animator.get("hover_tooltip", now).unwrap_or(1.0);
                                if opacity < 1.0 {
                                    self.ui_tree.set_subtree_opacity(tooltip_id, opacity);
                                }
                            }

                            // Apply button hover highlight on inspector close button (UI-W05).
                            if let Some(close_id) = self.inspector_close_id {
                                let hit = self
                                    .ui_tree
                                    .hit_test(self.cursor_pos.x as f32, self.cursor_pos.y as f32);
                                let is_hovered = hit == Some(close_id);
                                let target = if is_hovered { 1.0 } else { 0.0 };
                                if self.animator.target("btn_hover_close") != Some(target) {
                                    let current =
                                        self.animator.get("btn_hover_close", now).unwrap_or(0.0);
                                    self.animator.start(
                                        "btn_hover_close",
                                        current,
                                        target,
                                        std::time::Duration::from_millis(
                                            self.ui_theme.anim_hover_highlight_ms,
                                        ),
                                        ui::Easing::EaseOut,
                                        now,
                                    );
                                }
                                let hover_alpha =
                                    self.animator.get("btn_hover_close", now).unwrap_or(0.0);
                                let alpha = self.ui_theme.anim_hover_highlight_alpha * hover_alpha;
                                self.ui_tree.set_widget_bg_alpha(close_id, alpha);
                            } else {
                                self.animator.remove("btn_hover_close");
                            }

                            // Clean up completed animations (UI-W05).
                            self.animator.gc(now);

                            let map_text = render::render_world_to_string(
                                &self.world,
                                self.camera.x,
                                self.camera.y,
                                viewport_cols,
                                viewport_rows,
                            );

                            // Emit draw commands from UI tree.
                            let mut draw_list = ui::DrawList::new();
                            self.ui_tree.draw(&mut draw_list);

                            // Panels: map overlays first, then UI panels on top.
                            panel.begin_frame(&gpu.queue, screen_w, screen_h);
                            let no_border = [0.0_f32; 4];

                            // Map overlay: hover tile highlight (UI-I02).
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
                                        self.ui_theme.overlay_hover,
                                        no_border,
                                        0.0,
                                        0.0,
                                    );
                                }
                            }

                            // Map overlay: selected entity highlight (UI-I02).
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
                                        self.ui_theme.overlay_selection,
                                        no_border,
                                        0.0,
                                        0.0,
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
                                            self.ui_theme.overlay_path,
                                            no_border,
                                            0.0,
                                            0.0,
                                        );
                                    }
                                }
                            }

                            for cmd in &draw_list.panels {
                                panel.add_panel(
                                    cmd.x,
                                    cmd.y,
                                    cmd.width,
                                    cmd.height,
                                    cmd.bg_color,
                                    cmd.border_color,
                                    cmd.border_width,
                                    cmd.shadow_width,
                                );
                            }
                            let panel_vertex_count = panel.flush(&gpu.queue, &gpu.device);

                            // Text (on top)
                            font.begin_frame(&gpu.queue, screen_w, screen_h, FG_SRGB, BG_SRGB);
                            // UI widget text commands (multi-font via font_family + font_size)
                            for cmd in &draw_list.texts {
                                font.prepare_text_with_font(
                                    &cmd.text,
                                    cmd.x,
                                    cmd.y,
                                    cmd.color,
                                    cmd.font_family.family_name(),
                                    cmd.font_size,
                                );
                            }
                            // Rich text commands (per-span color + font via cosmic-text)
                            for cmd in &draw_list.rich_texts {
                                let spans: Vec<(String, [f32; 4], &str)> = cmd
                                    .spans
                                    .iter()
                                    .map(|s| (s.text.clone(), s.color, s.font_family.family_name()))
                                    .collect();
                                font.prepare_rich_text(&spans, cmd.x, cmd.y, cmd.font_size);
                            }
                            // Map still uses string-based rendering.
                            // Status bar, hover tooltip, and event log are widgets.
                            let fg4 = [FG_SRGB[0], FG_SRGB[1], FG_SRGB[2], 1.0];
                            font.prepare_map(&map_text, padding, map_y, fg4);

                            let text_vertex_count = font.flush(&gpu.queue, &gpu.device);
                            gpu.render(panel, panel_vertex_count, font, text_vertex_count);
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

    // Start camera overlooking the Seine near Ile de la Cité / Notre-Dame
    let start_camera = Camera { x: 3750, y: 3450 };

    let event_loop = EventLoop::new().expect("create event loop");
    let ui_theme = ui::Theme::default();
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
        ui_state: ui::UiState::new(),
        ui_tree: ui::WidgetTree::new(),
        ui_theme,
        animator: ui::Animator::new(),
        last_hover_tile: None,
        last_selected_entity: None,
        keybindings: ui::KeyBindings::defaults(),
        paused: false,
        sim_speed: 1,
        selected_entity: None,
        inspector_close_id: None,
        show_demo: std::env::args().any(|a| a == "--ui-demo"),
    };
    event_loop.run_app(&mut app).expect("run event loop");
}
