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
mod sprite_renderer;
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

/// Vertex counts per render layer.
/// The render pass draws back-to-front:
///   map text -> map overlay panels -> for each root { panels -> [sprite?] -> text }
struct FrameLayers {
    map_text_vertices: u32,
    map_overlay_panel_vertices: u32,
    /// Per-root (panel_range, text_range) in back-to-front draw order.
    /// panel_range and text_range are each (start, count) in their vertex buffers.
    root_layers: Vec<((u32, u32), (u32, u32))>,
    /// If set, draw sprites after this root's panels (before its text).
    /// Used for the minimap: sprite renders inside the minimap panel but
    /// under later roots (tooltips).
    sprite_after_root_panels: Option<usize>,
}

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

    /// Render a frame with proper Z-layering:
    /// 1. Clear background
    /// 2. Map text (ASCII grid) — under panels
    /// 3. UI panels (overlays + widgets)
    /// 4. UI text (labels, buttons, etc.) — over panels
    /// 5. Sprite overlay (minimap) — topmost
    fn render(
        &self,
        font: &font::FontRenderer,
        panel: &panel::PanelRenderer,
        layers: &FrameLayers,
        sprites: Option<&sprite_renderer::SpriteRenderer>,
        sprite_vertices: u32,
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

            // Layer 1: map ASCII text (under all UI)
            font.render_range(&mut render_pass, 0, layers.map_text_vertices);
            // Layer 2: map overlay panels (tile highlights)
            panel.render_range(&mut render_pass, 0, layers.map_overlay_panel_vertices);
            // Layers 3+: per-root panels then text (fixes cross-root text bleed-through).
            // Minimap sprite is inserted after the minimap root's panels so that
            // later roots (tooltips) properly occlude it.
            for (i, &(panel_range, text_range)) in layers.root_layers.iter().enumerate() {
                let (ps, pc) = panel_range;
                panel.render_range(&mut render_pass, ps, pc);
                if layers.sprite_after_root_panels == Some(i)
                    && let Some(sr) = sprites
                {
                    sr.render(&mut render_pass, sprite_vertices);
                }
                let (ts, tc) = text_range;
                font.render_range(&mut render_pass, ts, tc);
            }
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
    /// Smooth float target for lerp interpolation (UI-107).
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
    timed!("validate", world::validate_world(world));
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
    // UI persistent state (UI-P2: all mutable state in one struct).
    ui: ui::UiContext,
    // Ephemeral widget tree — rebuilt every frame.
    ui_tree: ui::WidgetTree,
    // Immutable theme configuration.
    ui_theme: ui::Theme,
    last_hover_tile: Option<(i32, i32)>,
    last_selected_entity: Option<components::Entity>,
    // Keyboard shortcut system (UI-I03)
    keybindings: ui::KeyBindings,
    paused: bool,
    sim_speed: u32, // 1 = normal, 2-5 = faster
    // Entity inspector (UI-I01d)
    selected_entity: Option<components::Entity>,
    inspector_close_id: Option<ui::WidgetId>,
    // Performance metrics (UI-505) — stores previous frame's metrics.
    ui_perf: ui::UiPerfMetrics,
    // Minimap (UI-407)
    minimap_sprites: Option<sprite_renderer::SpriteRenderer>,
    minimap_texture: Option<ui::MinimapTexture>,
    minimap_panel_id: Option<ui::WidgetId>,
    minimap_area_id: Option<ui::WidgetId>,
    minimap_dragging: bool,
    // Cached viewport dimensions for minimap click centering.
    viewport_cols: usize,
    viewport_rows: usize,
}

impl App {
    /// Handle a sidebar tab click. Manages open/close/switch transitions.
    fn handle_tab_click(&mut self, tab_idx: usize) {
        let now = Instant::now();
        let is_closing = self.ui.animator.target("sidebar_slide") == Some(1.0);
        let current_slide = self.ui.animator.get("sidebar_slide", now).unwrap_or(
            if self.ui.sidebar.active_tab.is_some() {
                0.0
            } else {
                1.0
            },
        );

        if self.ui.sidebar.active_tab == Some(tab_idx) && !is_closing {
            // Clicking active tab: start close animation.
            self.ui.animator.start(
                "sidebar_slide",
                ui::Anim {
                    from: current_slide,
                    to: 1.0,
                    duration: std::time::Duration::from_millis(self.ui_theme.anim_panel_hide_ms),
                    easing: ui::Easing::EaseIn,
                    ..ui::Anim::DEFAULT
                },
                now,
            );
        } else {
            // Opening new tab or switching while open.
            let need_slide = self.ui.sidebar.active_tab.is_none() || is_closing;
            self.ui.sidebar.active_tab = Some(tab_idx);
            if need_slide {
                self.ui.animator.start(
                    "sidebar_slide",
                    ui::Anim {
                        from: current_slide,
                        to: 0.0,
                        duration: std::time::Duration::from_millis(
                            self.ui_theme.anim_inspector_slide_ms,
                        ),
                        easing: ui::Easing::EaseOut,
                        ..ui::Anim::DEFAULT
                    },
                    now,
                );
            }
        }
    }

    /// Convert a screen-space cursor position to world coords via the minimap
    /// and set the camera target. Used for both click-to-jump and drag-to-pan.
    fn update_minimap_drag(&mut self, px: f32, py: f32) {
        if let Some(area_id) = self.minimap_area_id
            && let Some(rect) = self.ui_tree.node_rect(area_id)
        {
            let (wx, wy) = ui::minimap_click_to_world(
                px,
                py,
                rect.x,
                rect.y,
                self.world.tiles.width() as u32,
                self.world.tiles.height() as u32,
            );
            // Center viewport on the world position.
            self.camera.target_x = wx - self.viewport_cols as f32 / 2.0;
            self.camera.target_y = wy - self.viewport_rows as f32 / 2.0;
        }
    }

    /// Clean up focus state after popping a modal.
    fn cleanup_after_modal_pop(&mut self) {
        // Clear focus if the focused widget was removed with the modal.
        if let Some(f) = self.ui.input.focused
            && self.ui_tree.get(f).is_none()
        {
            self.ui.input.focused = None;
        }
        // Reset focus tier to match remaining stack.
        self.ui.input.focus_min_tier = if self.ui.modals.is_empty() {
            ui::ZTier::Panel
        } else {
            ui::ZTier::Modal
        };
    }

    /// Pop the topmost modal, clean up focus, and dispatch its action.
    fn pop_modal_with(&mut self, use_confirm: bool) {
        if self.ui.modals.is_empty() {
            return;
        }
        let pop = self.ui.modals.pop(&mut self.ui_tree);
        self.cleanup_after_modal_pop();
        if let Some(p) = pop {
            let action = if use_confirm {
                p.on_confirm
            } else {
                p.on_dismiss
            };
            if let Some(action) = action {
                self.dispatch_click(action);
            }
        }
    }

    /// Dispatch a UI click action. Centralizes all on_click handling.
    fn dispatch_click(&mut self, action: ui::UiAction) {
        match action {
            ui::UiAction::InspectorClose => {
                self.selected_entity = None;
            }
            ui::UiAction::ModalDismiss | ui::UiAction::DialogCancel => {
                self.pop_modal_with(false);
            }
            ui::UiAction::DialogAccept => {
                self.pop_modal_with(true);
            }
            ui::UiAction::SelectTab(idx) => {
                self.handle_tab_click(idx);
            }
            ui::UiAction::MenuNewGame => {}
            ui::UiAction::MenuContinue => {}
            ui::UiAction::MenuLoad => {}
            ui::UiAction::MenuSettings => {}
            ui::UiAction::MenuQuit => {}
            ui::UiAction::OutlinerSelectCharacter(_entity) => {}
            ui::UiAction::OutlinerSelectEvent(_cb) => {}
            ui::UiAction::FinderSort => {}
            ui::UiAction::FinderSelect(_entity) => {}
            ui::UiAction::SettingsUiScale => {}
            ui::UiAction::SettingsWindowMode => {}
            ui::UiAction::SaveLoadSave => {}
            ui::UiAction::SaveLoadLoad => {}
            ui::UiAction::SaveLoadSelect(_name) => {}
            ui::UiAction::MapModeChange => {}
            ui::UiAction::MapModeSpeed => {}
            ui::UiAction::EventChoice(_cb) => {}
            ui::UiAction::ContextAction(_action) => {}
        }
    }
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

        // Minimap sprite renderer (UI-407): 128×96 RGBA atlas.
        let zeroed_pixels = vec![0u8; 128 * 96 * 4];
        let minimap_sprites = sprite_renderer::SpriteRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.surface_format(),
            128,
            96,
            &zeroed_pixels,
        );

        self.gpu = Some(gpu);
        self.font = Some(font_renderer);
        self.panel = Some(panel_renderer);
        self.minimap_sprites = Some(minimap_sprites);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            // Input tracking (no GPU needed)
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;
                // Minimap drag-to-pan (UI-407).
                if self.minimap_dragging {
                    self.update_minimap_drag(position.x as f32, position.y as f32);
                }
                // Route to UI input system.
                self.ui.input.handle_cursor_moved(
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
                if !self.ui.input.handle_scroll(&mut self.ui_tree, dy) {
                    // Scroll not consumed by UI — zoom-to-cursor (UI-107).
                    // Adjust camera so the tile under cursor stays under cursor after zoom.
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
                        // Cursor position relative to map origin in screen pixels.
                        let cx = self.cursor_pos.x as f32 - self.map_origin.0;
                        let cy = self.cursor_pos.y as f32 - self.map_origin.1;
                        // base_cell = map_cell / zoom (font char size without zoom).
                        let base_w = self.map_cell_w / old_zoom;
                        let base_h = self.map_cell_h / old_zoom;
                        // Offset in tiles: how far the cursor is from camera origin.
                        // delta = cursor_px * (1/old_zoom - 1/new_zoom) / base_cell
                        //       = cursor_px / base * (1/old - 1/new)
                        let dx = (cx / base_w) * (1.0 / old_zoom - 1.0 / new_zoom);
                        let dy_adj = (cy / base_h) * (1.0 / old_zoom - 1.0 / new_zoom);
                        self.camera.target_x += dx;
                        self.camera.target_y += dy_adj;
                    }
                    self.camera.target_zoom = new_zoom;
                }
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
                                ui::Action::Pause => {
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
                                ui::Action::ToggleSidebar => {
                                    let tab = self.ui.sidebar.active_tab.unwrap_or(0);
                                    self.handle_tab_click(tab);
                                }
                                ui::Action::ConfirmModal => {
                                    // Enter confirms the topmost modal dialog.
                                    if !self.ui.modals.is_empty()
                                        && let Some(action) =
                                            self.ui.modals.confirm_action().cloned()
                                    {
                                        let pop = self.ui.modals.pop(&mut self.ui_tree);
                                        self.cleanup_after_modal_pop();
                                        self.dispatch_click(action);
                                        drop(pop);
                                    }
                                }
                                ui::Action::CloseTopmost => {
                                    // Priority: tooltips → modals → panels → inspector → sidebar → exit.
                                    if self.ui.input.tooltip_count() > 0 {
                                        self.ui.input.dismiss_all_tooltips(
                                            &mut self.ui_tree,
                                            Instant::now(),
                                        );
                                    } else if !self.ui.modals.is_empty() {
                                        self.pop_modal_with(false);
                                    } else if self
                                        .ui
                                        .panels
                                        .close_topmost(&mut self.ui_tree)
                                        .is_some()
                                    {
                                        // Closed a panel — done.
                                    } else if self.selected_entity.is_some() {
                                        self.selected_entity = None;
                                    } else if self.ui.sidebar.active_tab.is_some()
                                        && self.ui.animator.target("sidebar_slide") != Some(1.0)
                                    {
                                        // Close active sidebar tab.
                                        let tab = self.ui.sidebar.active_tab.unwrap_or(0);
                                        self.handle_tab_click(tab);
                                    } else {
                                        event_loop.exit();
                                    }
                                }
                                // Phase UI-4: new keybinding actions (stubs for now).
                                ui::Action::ToggleFinder => {
                                    // TODO: toggle character finder panel (UI-402)
                                }
                                ui::Action::ToggleOutliner => {
                                    // TODO: toggle outliner panel (UI-405)
                                }
                                ui::Action::QuickSave => {
                                    // TODO: quick save (UI-412)
                                }
                                ui::Action::QuickLoad => {
                                    // TODO: quick load (UI-412)
                                }
                                ui::Action::ScaleUp => {
                                    self.ui_theme.ui_scale =
                                        (self.ui_theme.ui_scale + 0.1).min(2.0);
                                }
                                ui::Action::ScaleDown => {
                                    self.ui_theme.ui_scale =
                                        (self.ui_theme.ui_scale - 0.1).max(0.5);
                                }
                            }
                            return;
                        }

                        // 2. Modal focus scoping — restrict Tab to active tier.
                        self.ui.input.focus_min_tier = if self.ui.modals.is_empty() {
                            ui::ZTier::Panel
                        } else {
                            ui::ZTier::Modal
                        };
                        // Clear focus if it belongs to a root below the active tier.
                        if let Some(focused) = self.ui.input.focused
                            && let Some(tier) = self.ui_tree.z_tier_of_widget(focused)
                            && tier < self.ui.input.focus_min_tier
                        {
                            self.ui.input.focused = None;
                        }

                        // 3. UI widget focus dispatch (Tab, ScrollList nav).
                        if self.ui.input.handle_key_input(&mut self.ui_tree, kc, true) {
                            return;
                        }

                        // 4. Game keys.
                        match kc {
                            // Camera: WASD + arrows (realtime mode only) (UI-107).
                            // Pan speed scales inversely with zoom (faster when zoomed out).
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

                        // Minimap drag-to-pan (UI-407): handle before UI system.
                        if button == MouseButton::Left {
                            if pressed {
                                if let Some(area_id) = self.minimap_area_id
                                    && let Some(rect) = self.ui_tree.node_rect(area_id)
                                    && px >= rect.x
                                    && px < rect.x + rect.width
                                    && py >= rect.y
                                    && py < rect.y + rect.height
                                {
                                    self.minimap_dragging = true;
                                    self.update_minimap_drag(px, py);
                                    return;
                                }
                            } else if self.minimap_dragging {
                                self.minimap_dragging = false;
                                return;
                            }
                        }

                        // UI consumes click — don't pass to game.
                        if self.ui.input.handle_mouse_input(
                            &mut self.ui_tree,
                            ui_btn,
                            pressed,
                            px,
                            py,
                        ) {
                            // Dispatch widget click actions (UI-305).
                            if let Some((_widget_id, action)) = self.ui.input.poll_click() {
                                self.dispatch_click(action);
                            }
                            return;
                        }

                        // Map click dispatch (UI-106): translate screen coords to tile coords.
                        if pressed && self.map_cell_w > 0.0 {
                            let vx = (px - self.map_origin.0) / self.map_cell_w;
                            let vy = (py - self.map_origin.1) / self.map_cell_h;
                            if vx >= 0.0 && vy >= 0.0 {
                                let tile_x = self.camera.x + vx as i32;
                                let tile_y = self.camera.y + vy as i32;
                                self.ui.input.submit_map_click(tile_x, tile_y, ui_btn);
                            }
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
                        let sim_start = Instant::now();
                        let mut sim_ticks_this_frame = 0u32;
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
                                            sim_ticks_this_frame += 1;
                                            let cooldown = self
                                                .world
                                                .body
                                                .move_cooldowns
                                                .get(&player)
                                                .map(|cd| cd.remaining)
                                                .unwrap_or(0);
                                            for _ in 0..cooldown {
                                                run_one_tick(&mut self.world);
                                                sim_ticks_this_frame += 1;
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
                                            sim_ticks_this_frame += 1;
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

                            while self.tick_accumulator >= SIM_TICK_INTERVAL
                                && sim_ticks_this_frame < MAX_TICKS_PER_FRAME
                            {
                                run_one_tick(&mut self.world);
                                self.tick_accumulator -= SIM_TICK_INTERVAL;
                                sim_ticks_this_frame += 1;
                            }
                        } else {
                            // Paused: keep frame time current to avoid tick burst on unpause.
                            self.last_frame_time = Instant::now();
                        }
                        let sim_us = sim_start.elapsed().as_micros() as u64;

                        // === Render ===
                        if let (Some(font), Some(panel)) = (self.font.as_mut(), self.panel.as_mut())
                        {
                            let m = font.metrics();
                            let screen_w = gpu.config.width;
                            let screen_h = gpu.config.height;
                            let padding = 4.0_f32;

                            // Rebuild game UI tree every frame (DD-5: full rebuild).
                            let build_start = Instant::now();
                            let player_name = self
                                .world
                                .player
                                .and_then(|p| self.world.body.names.get(&p))
                                .map(|n| n.value.as_str());
                            // Persist sidebar scroll offset before destroying the tree,
                            // since handle_scroll may have modified it during this frame.
                            if let Some(sv_id) = self.ui.sidebar.scroll_view_id {
                                self.ui.sidebar.scroll_offset = self.ui_tree.scroll_offset(sv_id);
                            }
                            self.ui_tree = ui::WidgetTree::new();
                            self.ui_tree
                                .set_scroll_row_alt_alpha(self.ui_theme.scroll_row_alt_alpha);
                            self.ui_tree
                                .set_control_border_width(self.ui_theme.control_border());
                            let game_date = components::GameDate::from_tick(
                                self.world.tick,
                                &self.world.start_date,
                            );
                            let status_info = ui::StatusBarInfo {
                                tick: self.world.tick.0,
                                date: game_date.format(),
                                population: self.world.alive.len(),
                                is_turn_based: self.world.player.is_some(),
                                player_name,
                                paused: self.paused,
                                sim_speed: self.sim_speed,
                                keybindings: &self.keybindings,
                                screen_width: screen_w as f32,
                                perf: Some(self.ui_perf),
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
                            self.ui_tree.layout(screen_size, font);
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
                                (screen_h as f32 - status_bar_h - event_log_h - padding * 3.0)
                                    .max(0.0);
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
                                // Smooth lerp: camera position interpolates toward target (UI-107).
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
                            // Clamp both target and actual camera position.
                            self.camera.target_x =
                                self.camera.target_x.clamp(0.0, max_cam_x as f32);
                            self.camera.target_y =
                                self.camera.target_y.clamp(0.0, max_cam_y as f32);
                            self.camera.x = self.camera.x.clamp(0, max_cam_x);
                            self.camera.y = self.camera.y.clamp(0, max_cam_y);

                            let now = Instant::now();
                            let mut hover_tooltip_id = None;

                            // Build entity inspector if selected (UI-I01d).
                            self.inspector_close_id = None;
                            if let Some(entity) = self.selected_entity {
                                if let Some(info) = ui::collect_inspector_info(entity, &self.world)
                                {
                                    // Start slide-in when entity first selected (UI-W05).
                                    if self.last_selected_entity != Some(entity) {
                                        self.last_selected_entity = Some(entity);
                                        self.ui.animator.start(
                                            "inspector_slide",
                                            ui::Anim {
                                                from: 1.0,
                                                to: 0.0,
                                                duration: std::time::Duration::from_millis(
                                                    self.ui_theme.anim_inspector_slide_ms,
                                                ),
                                                easing: ui::Easing::EaseOut,
                                                ..ui::Anim::DEFAULT
                                            },
                                            now,
                                        );
                                    }

                                    let (inspector_id, close_id) = ui::build_entity_inspector(
                                        &mut self.ui_tree,
                                        &self.ui_theme,
                                        &info,
                                    );

                                    // Apply slide-in offset (UI-W05).
                                    let slide =
                                        self.ui.animator.get("inspector_slide", now).unwrap_or(0.0);
                                    let target_x = screen_w as f32 - 220.0 - padding;
                                    let slide_offset = slide * (220.0 + padding);
                                    self.ui_tree.set_position(
                                        inspector_id,
                                        ui::Position::Fixed {
                                            x: target_x + slide_offset,
                                            y: status_bar_h + padding,
                                        },
                                    );
                                    self.ui_tree
                                        .set_on_click(close_id, ui::UiAction::InspectorClose);
                                    self.inspector_close_id = Some(close_id);
                                } else {
                                    // Entity died or lost position — auto-close.
                                    self.selected_entity = None;
                                    self.last_selected_entity = None;
                                    self.ui.animator.remove("inspector_slide");
                                }
                            } else {
                                self.last_selected_entity = None;
                                self.ui.animator.remove("inspector_slide");
                            }

                            // Build sidebar tab strip (always visible).
                            let _tab_ids = ui::sidebar::build_tab_strip(
                                &mut self.ui_tree,
                                &self.ui_theme,
                                screen_size,
                                self.ui.sidebar.active_tab,
                            );

                            // Build active sidebar main-tab view.
                            let mut sidebar_panel_id = None;
                            if let Some(tab_idx) = self.ui.sidebar.active_tab {
                                match tab_idx {
                                    0 => {
                                        // Widget showcase view.
                                        let first_entity =
                                            self.world.alive.iter().copied().min_by_key(|e| e.0);
                                        let entity_info = first_entity.and_then(|e| {
                                            ui::collect_inspector_info(e, &self.world)
                                        });
                                        let live = ui::sidebar::SidebarInfo {
                                            entity_info: entity_info.as_ref(),
                                            tick: self.world.tick.0,
                                            population: self.world.alive.len(),
                                        };
                                        let (view_id, view_sv) = ui::sidebar::build_showcase_view(
                                            &mut self.ui_tree,
                                            &self.ui_theme,
                                            &self.keybindings,
                                            &live,
                                            screen_size,
                                            self.ui.sidebar.scroll_offset,
                                        );
                                        sidebar_panel_id = Some(view_id);
                                        self.ui.sidebar.scroll_view_id = Some(view_sv);
                                    }
                                    n => {
                                        // Placeholder views for tabs 1+.
                                        let pid = ui::sidebar::build_placeholder_view(
                                            &mut self.ui_tree,
                                            &self.ui_theme,
                                            screen_size,
                                            n,
                                        );
                                        sidebar_panel_id = Some(pid);
                                        self.ui.sidebar.scroll_view_id = None;
                                    }
                                }
                            }

                            // Minimap (UI-407).
                            self.minimap_panel_id = None;
                            self.minimap_area_id = None;
                            if self.minimap_texture.is_some() {
                                let minimap_info = ui::MinimapInfo {
                                    map_width: self.world.tiles.width() as u32,
                                    map_height: self.world.tiles.height() as u32,
                                    camera_x: self.camera.x as f32,
                                    camera_y: self.camera.y as f32,
                                    viewport_w: viewport_cols as f32,
                                    viewport_h: viewport_rows as f32,
                                    screen_width: screen_w as f32,
                                    screen_height: screen_h as f32,
                                };
                                let (panel_id, area_id) = ui::build_minimap(
                                    &mut self.ui_tree,
                                    &self.ui_theme,
                                    &minimap_info,
                                );
                                self.minimap_panel_id = Some(panel_id);
                                self.minimap_area_id = Some(area_id);
                            }

                            // Pause overlay (UI-105): dim layer when paused.
                            if self.paused {
                                ui::build_pause_overlay(
                                    &mut self.ui_tree,
                                    screen_w as f32,
                                    screen_h as f32,
                                );
                            }

                            let build_us = build_start.elapsed().as_micros() as u64;

                            // Re-layout tree with all widgets included.
                            let layout_start = Instant::now();
                            self.ui_tree.layout(screen_size, font);

                            // Apply sidebar slide animation.
                            if let Some(panel_id) = sidebar_panel_id {
                                let slide =
                                    self.ui.animator.get("sidebar_slide", now).unwrap_or(0.0);
                                let base_x = screen_w as f32
                                    - ui::sidebar::MAIN_TAB_WIDTH
                                    - ui::sidebar::SIDEBAR_MARGIN;
                                if slide > 0.0 {
                                    // Slide offset: panel must travel full width + margin
                                    // to clear the screen edge.
                                    let offset = slide
                                        * (ui::sidebar::MAIN_TAB_WIDTH
                                            + ui::sidebar::SIDEBAR_MARGIN);
                                    self.ui_tree.set_position(
                                        panel_id,
                                        ui::Position::Fixed {
                                            x: base_x + offset,
                                            y: 4.0,
                                        },
                                    );
                                    // Need re-layout after position change.
                                    self.ui_tree.layout(screen_size, font);
                                }
                                // Hide animation complete: panel fully off-screen.
                                if self.ui.animator.target("sidebar_slide") == Some(1.0)
                                    && !self.ui.animator.is_active("sidebar_slide", now)
                                {
                                    self.ui.sidebar.active_tab = None;
                                    self.ui.animator.remove("sidebar_slide");
                                    self.ui_tree.remove(panel_id);
                                }
                            }

                            // Build hover tooltip after all UI is laid out, gated
                            // on cursor not being over any UI widget (UI-I01b).
                            let cursor_over_ui = self
                                .ui_tree
                                .hit_test(self.cursor_pos.x as f32, self.cursor_pos.y as f32)
                                .is_some();
                            if !cursor_over_ui {
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
                                            font,
                                        );
                                        hover_tooltip_id = Some(tooltip_id);

                                        // Start fade-in only on first appearance (no previous
                                        // tile), not when sliding between adjacent tiles.
                                        let current_tile = (tile_x, tile_y);
                                        let first_hover = self.last_hover_tile.is_none();
                                        if self.last_hover_tile != Some(current_tile) {
                                            self.last_hover_tile = Some(current_tile);
                                            if first_hover {
                                                self.ui.animator.start(
                                                    "hover_tooltip",
                                                    ui::Anim {
                                                        from: 0.0,
                                                        to: 1.0,
                                                        duration: std::time::Duration::from_millis(
                                                            self.ui_theme.anim_tooltip_fade_ms,
                                                        ),
                                                        easing: ui::Easing::EaseOut,
                                                        ..ui::Anim::DEFAULT
                                                    },
                                                    now,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            if hover_tooltip_id.is_none() {
                                self.last_hover_tile = None;
                                self.ui.animator.remove("hover_tooltip");
                            }
                            if hover_tooltip_id.is_some() {
                                self.ui_tree.layout(screen_size, font);
                            }
                            if let Some(tooltip_id) = hover_tooltip_id {
                                let opacity =
                                    self.ui.animator.get("hover_tooltip", now).unwrap_or(1.0);
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
                                if self.ui.animator.target("btn_hover_close") != Some(target) {
                                    let current =
                                        self.ui.animator.get("btn_hover_close", now).unwrap_or(0.0);
                                    self.ui.animator.start(
                                        "btn_hover_close",
                                        ui::Anim {
                                            from: current,
                                            to: target,
                                            duration: std::time::Duration::from_millis(
                                                self.ui_theme.anim_hover_highlight_ms,
                                            ),
                                            easing: ui::Easing::EaseOut,
                                            ..ui::Anim::DEFAULT
                                        },
                                        now,
                                    );
                                }
                                let hover_alpha =
                                    self.ui.animator.get("btn_hover_close", now).unwrap_or(0.0);
                                let alpha = self.ui_theme.anim_hover_highlight_alpha * hover_alpha;
                                self.ui_tree.set_widget_bg_alpha(close_id, alpha);
                            } else {
                                self.ui.animator.remove("btn_hover_close");
                            }

                            // Clean up completed animations (UI-W05).
                            self.ui.animator.gc(now);
                            self.ui.panels.flush_closed(&mut self.ui_tree, now);

                            let map_text = render::render_world_to_string(
                                &self.world,
                                self.camera.x,
                                self.camera.y,
                                viewport_cols,
                                viewport_rows,
                            );

                            let layout_us = layout_start.elapsed().as_micros() as u64;

                            // Emit draw commands from UI tree.
                            let draw_start = Instant::now();
                            let mut draw_list = ui::DrawList::new();
                            self.ui_tree.draw(&mut draw_list, font);
                            let draw_us = draw_start.elapsed().as_micros() as u64;

                            // Panels: map overlays first, then UI panels on top.
                            let render_start = Instant::now();
                            panel.begin_frame(&gpu.queue, screen_w, screen_h);
                            let no_border = [0.0_f32; 4];
                            let sw = screen_w as f32;
                            let sh = screen_h as f32;
                            let no_clip_min = [0.0_f32, 0.0];
                            let no_clip_max = [sw, sh];

                            // Map overlay: hover tile highlight (UI-I02).
                            // Suppressed when cursor is over a UI widget.
                            if !cursor_over_ui {
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
                                        no_clip_min,
                                        no_clip_max,
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
                                            self.ui_theme.overlay_path,
                                            no_border,
                                            0.0,
                                            0.0,
                                            no_clip_min,
                                            no_clip_max,
                                        );
                                    }
                                }
                            }

                            // Map overlay panels are already in the buffer.
                            let map_overlay_panel_vertices = panel.pending_vertex_count();

                            // UI: add per-root panels and text in back-to-front order.
                            // Text: map text first, then UI text per-root.
                            font.begin_frame(&gpu.queue, screen_w, screen_h, FG_SRGB, BG_SRGB);

                            // Map ASCII text (renders under all UI).
                            let fg4 = [FG_SRGB[0], FG_SRGB[1], FG_SRGB[2], 1.0];
                            font.prepare_map(&map_text, padding, map_y, fg4);
                            let map_text_vertices = font.pending_vertex_count();

                            let root_ids = self.ui_tree.roots();
                            let mut sprite_after_root_panels = None;
                            let mut root_layers = Vec::with_capacity(draw_list.root_slices.len());
                            for (root_idx, slice) in draw_list.root_slices.iter().enumerate() {
                                if root_ids.get(root_idx) == self.minimap_panel_id.as_ref() {
                                    sprite_after_root_panels = Some(root_idx);
                                }
                                // Panels for this root
                                let ps = panel.pending_vertex_count();
                                for cmd in &draw_list.panels[slice.panels.clone()] {
                                    let (cmin, cmax) = match &cmd.clip {
                                        Some(r) => ([r.x, r.y], [r.x + r.width, r.y + r.height]),
                                        None => (no_clip_min, no_clip_max),
                                    };
                                    panel.add_panel(
                                        cmd.x,
                                        cmd.y,
                                        cmd.width,
                                        cmd.height,
                                        cmd.bg_color,
                                        cmd.border_color,
                                        cmd.border_width,
                                        cmd.shadow_width,
                                        cmin,
                                        cmax,
                                    );
                                }
                                let pe = panel.pending_vertex_count();

                                // Text for this root
                                let ts = font.pending_vertex_count();
                                for cmd in &draw_list.texts[slice.texts.clone()] {
                                    let (cmin, cmax) = match &cmd.clip {
                                        Some(r) => ([r.x, r.y], [r.x + r.width, r.y + r.height]),
                                        None => (no_clip_min, no_clip_max),
                                    };
                                    font.prepare_text_with_font(
                                        &cmd.text,
                                        [cmd.x, cmd.y],
                                        cmd.color,
                                        cmd.font_family.family_name(),
                                        cmd.font_size,
                                        [cmin, cmax],
                                    );
                                }
                                for cmd in &draw_list.rich_texts[slice.rich_texts.clone()] {
                                    let (cmin, cmax) = match &cmd.clip {
                                        Some(r) => ([r.x, r.y], [r.x + r.width, r.y + r.height]),
                                        None => (no_clip_min, no_clip_max),
                                    };
                                    let spans: Vec<(String, [f32; 4], &str)> = cmd
                                        .spans
                                        .iter()
                                        .map(|s| {
                                            (s.text.clone(), s.color, s.font_family.family_name())
                                        })
                                        .collect();
                                    font.prepare_rich_text(
                                        &spans,
                                        [cmd.x, cmd.y],
                                        cmd.font_size,
                                        [cmin, cmax],
                                    );
                                }
                                let te = font.pending_vertex_count();

                                root_layers.push(((ps, pe - ps), (ts, te - ts)));
                            }
                            panel.flush(&gpu.queue, &gpu.device);
                            font.flush(&gpu.queue, &gpu.device);

                            // Minimap sprite pass (UI-407).
                            let sprite_vertex_count = if let Some(sprites) =
                                self.minimap_sprites.as_mut()
                            {
                                if let Some(tex) = self.minimap_texture.as_mut() {
                                    let cam_cx = self.camera.x as f32 + viewport_cols as f32 / 2.0;
                                    let cam_cy = self.camera.y as f32 + viewport_rows as f32 / 2.0;
                                    tex.render_frame(
                                        cam_cx,
                                        cam_cy,
                                        viewport_cols as f32,
                                        viewport_rows as f32,
                                        map_w as u32,
                                        map_h as u32,
                                    );
                                    sprites.upload_atlas(&gpu.queue, tex.pixels());
                                }
                                sprites.begin_frame(&gpu.queue, screen_w, screen_h);
                                if let Some(area_id) = self.minimap_area_id
                                    && let Some(rect) = self.ui_tree.node_rect(area_id)
                                {
                                    sprites.add_sprite(
                                        rect.x,
                                        rect.y,
                                        rect.width,
                                        rect.height,
                                        0.0,
                                        0.0,
                                        1.0,
                                        1.0,
                                        [1.0, 1.0, 1.0, 1.0],
                                    );
                                }
                                sprites.flush(&gpu.queue, &gpu.device)
                            } else {
                                0
                            };

                            let layers = FrameLayers {
                                map_text_vertices,
                                map_overlay_panel_vertices,
                                root_layers,
                                sprite_after_root_panels,
                            };
                            gpu.render(
                                font,
                                panel,
                                &layers,
                                self.minimap_sprites.as_ref(),
                                sprite_vertex_count,
                            );
                            let render_us = render_start.elapsed().as_micros() as u64;

                            // Capture perf metrics (UI-505) — one-frame lag.
                            self.ui_perf = ui::UiPerfMetrics {
                                sim_us,
                                sim_ticks: sim_ticks_this_frame,
                                build_us,
                                layout_us,
                                draw_us,
                                render_us,
                                widget_count: self.ui_tree.widget_count(),
                                panel_cmds: draw_list.panels.len(),
                                text_cmds: draw_list.texts.len() + draw_list.rich_texts.len(),
                                sprite_cmds: draw_list.sprites.len(),
                            };

                            // Warn when any phase exceeds 2ms (UI-505).
                            if sim_us > 2000 {
                                log::warn!(
                                    "Sim phase slow: {}us ({} ticks)",
                                    sim_us,
                                    sim_ticks_this_frame,
                                );
                            }
                            if build_us > 2000 {
                                log::warn!("UI build phase slow: {}us", build_us);
                            }
                            if layout_us > 2000 {
                                log::warn!("UI layout phase slow: {}us", layout_us);
                            }
                            if draw_us > 2000 {
                                log::warn!("UI draw phase slow: {}us", draw_us);
                            }
                            if render_us > 2000 {
                                log::warn!("UI render phase slow: {}us", render_us);
                            }
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

    loading_gis::spawn_gis_entities(&mut world, "Arcis");

    // Start camera overlooking the Seine near Ile de la Cité / Notre-Dame
    let start_camera = Camera {
        x: 3750,
        y: 3450,
        target_x: 3750.0,
        target_y: 3450.0,
        zoom: 1.0,
        target_zoom: 1.0,
    };

    // Minimap texture (UI-407): blank base, viewport indicator stamped per-frame.
    let minimap_texture = ui::MinimapTexture::new();

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
        ui: ui::UiContext {
            input: ui::UiState::new(),
            animator: ui::Animator::new(),
            modals: ui::ModalStack::new(),
            panels: ui::PanelManager::new(),
            scroll: std::collections::HashMap::new(),
            sidebar: ui::SidebarState {
                active_tab: if std::env::args().any(|a| a == "--sidebar") {
                    Some(0)
                } else {
                    None
                },
                scroll_offset: 0.0,
                scroll_view_id: None,
            },
        },
        ui_tree: {
            let mut t = ui::WidgetTree::new();
            t.set_control_border_width(ui_theme.control_border());
            t
        },
        ui_theme,
        last_hover_tile: None,
        last_selected_entity: None,
        keybindings: ui::KeyBindings::defaults(),
        paused: false,
        sim_speed: 1,
        selected_entity: None,
        inspector_close_id: None,
        ui_perf: ui::UiPerfMetrics::default(),
        minimap_sprites: None, // created in resumed() when GPU is available
        minimap_texture: Some(minimap_texture),
        minimap_panel_id: None,
        minimap_area_id: None,
        minimap_dragging: false,
        viewport_cols: 0,
        viewport_rows: 0,
    };
    event_loop.run_app(&mut app).expect("run event loop");
}
