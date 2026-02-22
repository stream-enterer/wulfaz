# Backlog

Incomplete work only. Delete entries when done.
See `architecture.md` for technical spec on all SCALE tasks.
GIS data reference: `~/Development/paris/PROJECT.md`

## Phase A — Chunked Map + GIS Loading

Goal: See Paris on screen. No entities.

Map dimensions: 6,309 x 4,753 tiles at 1m/tile (vertex-crop of all buildings + 30m padding).
That is ~99 x 75 chunks at 64×64 = ~7,400 chunks, ~30M tiles.

- **SCALE-A09** — Water/bridge polish. Needs: A08. **Remaining known limitations:**
  - **Eastern coverage gap**: ~150-tile-wide hole in ALPAGE data (tiles ~4950-5100 X, ~3500-3900 Y). Road patch in the Seine near Pont d'Austerlitz. Components #12 (2777 tiles) and #13 (424 tiles) are data-gap artifacts, not real bridges. Fix: obtain APUR PLAN D'EAU shapefile, reproject from Lambert-93 (EPSG:2154) to WGS84 via ogr2ogr, feed through `rasterize_water_polygons()`.
  - **Western bridge coverage**: ALPAGE water polygons don't extend west of ~lon 2.336 (5 bridges: Invalides, Concorde, Royal, Carrousel, Arts). Same fix — supplemental data needed.
  - **North arm bridge gap**: No detected bridge components in the north arm between Pont Neuf and Ile Saint-Louis (Pont au Change, Notre-Dame, d'Arcole). Either ALPAGE data doesn't fully cover this arm or bridges merged with island road network. Needs investigation.
  - **Canal Saint-Martin**: not in the ALPAGE Vasserot Hydrography layer. Separate historical data source needed.
  - **Diagnostic match rate**: 7/15 in-coverage bridges match (47%). 7 confident matches (dist 2-6 tiles). 8 misses are north-arm bridges or small bridges without separate components.

## Phase B — Entities in One Neighborhood

Goal: ~200 entities with full AI on the real map.

- **SCALE-B05** — Door placement + passage carving. Needs: A03. Blocks: B06, B03. **BLOCKED: design review required.**
  - **Preprocessor** (extend `preprocess.rs`): runs after wall/floor classification, same pattern as classify_walls_floors. Static tile modification baked into `paris.tiles`.
  - Place Door tiles: for each building, find a wall tile adjacent to both a floor tile and a Road or Courtyard tile. That tile becomes a Door.
  - Landlocked buildings (no wall tile adjacent to Road or Courtyard): carve a 1-tile passage through intervening buildings to the nearest Road or Courtyard. This models the narrow covered passages (allées) that provided access to interior buildings in dense Parisian blocks.
  - Garden buildings (24 "parc ou jardin"): convert their interior Floor tiles to Garden instead of Floor.
  - Game loads Door/Garden terrain from binary, no runtime classification needed.

- **SCALE-B06** — Building interior generation. Needs: B05, A07. **BLOCKED: design review required.**
  - **Preprocessor** (extend `preprocess.rs`): runs after door placement + address loading. Static tile modifications baked into `paris.tiles`.
  - Furnish building interiors based on occupant type. NAICS category from building registry (populated by A07 in preprocessor). Place furniture tiles:
    - Food stores → counters, barrels, shelves
    - Restaurants → tables, chairs, hearth
    - Clothing → looms, counters, fabric
    - Manufacturing → workbenches, anvils, forges
    - Residential/unknown → beds, table, chairs, hearth, chest
  - Buildings with no known occupant get default residential furnishing.
  - Small buildings (<15 floor tiles) get minimal furnishing (bed, table).
  - Requires new Terrain variants for furniture types (or a separate furniture tile layer in Chunk).

- **SCALE-B03** — GIS-aware entity spawning. Needs: A07, B05. **BLOCKED: design review required.**
  - The building registry (populated by A03 + A07) already knows each building's occupants, addresses, and NAICS categories. This task spawns actual entities from that data.
  - For known occupants (3.7% of population): spawn entity with real name, real occupation, at their building's floor tiles. Position from building's tile list in the registry.
  - For generated occupants (96.3%): see C05 for the procedural generation rules.
  - Single neighborhood first: filter to one QUARTIER (recommend "Arcis" — 825 buildings, dense, central, ~150m×300m).
  - The full data pipeline reference (address → building → people) is documented in SCALE-A07 and `~/Development/paris/PROJECT.md`.

## Phase C — Simulation LOD (1M Population)

Goal: Full city population. ~4K active, rest statistical.
Census population 1846: 1,034,196. Directory-listed people: 38,188 (3.7%).

- **SCALE-C01** — District definitions from GIS. Blocks: C02, C04.
  - 36 quartiers defined by the `QUARTIER` field on every building and block polygon. No separate quartier boundary geometry needed — derive bounds from the bounding box of all buildings with that QUARTIER value.
  - Quartier sizes range from 265 buildings (Palais de Justice) to 2,391 buildings (Temple).
  - Sub-district: blocks (`NUM_ILOT` field on buildings, `ID_ILOTS` on plot polygons) group ~30-100 buildings each. Use as LOD sub-units if quartier granularity is too coarse.
  - Per-district density derivable from: building count, total building area (SUPERFICIE sum), and occupant count from building registry (baked in by A07).

- **SCALE-C02** — LOD zone framework. Active/Nearby/Statistical derived from camera + district bounds. Needs: C01. Blocks: C03, C05.

- **SCALE-C03** — Zone-aware system filtering. Combat: Active only. Hunger: Active+Nearby. Statistical: no entity iteration. Needs: C02.

- **SCALE-C04** — District aggregate model + `run_district_stats`. Population, avg needs, death rates, resource flows as equations. Needs: C01, A07. **BLOCKED: design review required.**
  - Seed `population_by_type` from NAICS distribution per quartier. 22 industry categories. Aggregate from building registry occupant data (baked in by A07 preprocessor), not from raw GeoPackage.
  - City-wide distribution (1845): Manufacturing 18%, Food stores 13.5%, Clothing 11.7%, Furniture 8.2%, Legal 5.9%, Health 5.5%, Rentiers 4.5%, Arts 3.9%, Construction 3.6%. Use these as priors, adjust per quartier from actual registry data.

- **SCALE-C05** — Statistical population seeding. Every district outside active zone gets aggregate population. Needs: C02, C04, A07. **BLOCKED: design review required.**
  - Procedural population generation rules (for the 96% not in directories):
    - **Concierge**: every building with >4 floor tiles gets one. Ground floor.
    - **Shopkeeper household**: for each directory-listed person, generate spouse + 1-4 children + 0-1 apprentice. Place on ground floor and first upper floor.
    - **Bourgeois tenants**: buildings >100m² get 1-2 wealthy households on lower floors (rentiers, professionals). 3-5 people each.
    - **Working tenants**: remaining floor capacity filled with laborer households. Common unlisted occupations: blanchisseuse (laundress), couturière (seamstress), journalier (day laborer), domestique (servant), porteur d'eau (water carrier), chiffonnier (ragpicker), marchand ambulant (street vendor).
    - **Vertical stratification**: wealthiest on floor 1 (étage noble), progressively poorer upward, servants in garret.
    - **Floor estimate**: building height not in data. Estimate from SUPERFICIE: <50m² = 2 floors, 50-150m² = 3-4 floors, 150-400m² = 4-5 floors, >400m² = 5-6 floors. Multiply footprint area by floor count for total interior space.
    - **Density target**: ~116 people per 1,000m² of footprint (from census population / total building area). Adjust per quartier.
  - 3 active time snapshots from SoDUCo (filtered to best Vasserot overlap): 1845, 1850, 1855. Match rates: 40.1%, 37.1%, 38.0% (52,909 total matched occupants). Active year selected at runtime via `world.active_year` (default 1845). Building geometry is fixed 1810-1836.

## Phase D — Seamless Transitions

Goal: Camera movement smoothly activates/deactivates zones.

- **SCALE-D01** — Hydration. Statistical → active: spawn entities from distribution at building positions. Batch ~100/tick. Needs: C05, B03.
- **SCALE-D02** — Dehydration. Active → statistical: collapse to district averages. Nearby zone buffers for ~200 ticks. Needs: C02.
- **SCALE-D03** — HPA* pathfinding. Chunk-level nav graph, border nodes, precomputed intra-chunk paths. Replaces B04. Needs: A02.
- **SCALE-D04** — Profile and tune. Zone radii, hydration batch size, tick budget per zone, entity count limits.

## Simulation Features (parallel or post-Phase B)

Developable on test map or integrated after Phase B.

- **SIM-001** — Plant growth (Phase 1). Food regeneration. Garden tiles only (24 in dataset). Needs: B05 (garden placement).
- **SIM-002** — Thirst (Phase 2). Requires Water tiles (Seine) and fountains (3 named "Fontaine" buildings + "Pompe de la Samaritaine" in data).
- **SIM-003** — Decay (Phase 1). Corpse decomposition.
- **SIM-004** — Tiredness/sleep (Phase 2). Rest cycles. Entities return to their home building.
- **SIM-005** — Injury (Phase 5). Non-binary damage states.
- **SIM-006** — Weather (Phase 1). Rain/drought/cold.
- **SIM-007** — Emotions/mood (Phase 2). Aggregate need state.
- **SIM-008** — Relationships (Phase 5). Bonds from interaction.
- **SIM-009** — Reputation (Phase 5). Observed behavior.
- **SIM-010** — Building (Phase 4). Requires inventory.
- **SIM-011** — Crafting (Phase 4). Requires recipes.
- **SIM-012** — Fluid flow (Phase 1). Cellular automaton. Needs: A08 (Seine placement).

## Phase UI-1 — Layout Primitives

Goal: Dense multi-column panel layouts. CK3 panels stack labels, stats, icons, and bars in tight rows and columns — current Fixed/Percent/Fit positioning cannot express this.

- **UI-100** — Row (HBox) auto-layout container. Blocks: UI-101, UI-200, UI-204, UI-205, UI-300, UI-301, UI-302.
  - Add `Widget::Row { gap: f32, align: CrossAlign }` to `src/ui/widget.rs`. `CrossAlign` enum: `Start`, `Center`, `End`, `Stretch`.
  - In `measure_node()` (`src/ui/mod.rs`): intrinsic width = sum of child widths + gap * (n-1). Height = max child height.
  - In `layout_node()`: allocate children left-to-right. Each child gets `x = prev_x + prev_w + gap`. Cross-axis position derived from `align` and row height.
  - `Sizing::Fit` children use measured width. `Sizing::Percent` children split remaining space proportionally after fixed children are placed. This is the "flex" behavior CK3 uses for stat rows.
  - In `draw_node()`: Row itself emits no draw commands (transparent container). Recurse into children.
  - Test: build a Row with 3 labels of known widths, assert rects are contiguous with correct gap spacing.

- **UI-101** — Column (VBox) auto-layout container. Needs: UI-100 (shares CrossAlign enum). Blocks: UI-201, UI-204, UI-300, UI-301, UI-303, UI-304.
  - Add `Widget::Column { gap: f32, align: CrossAlign }` to `src/ui/widget.rs`.
  - In `measure_node()`: intrinsic height = sum of child heights + gap * (n-1). Width = max child width.
  - In `layout_node()`: allocate children top-to-bottom. Each child gets `y = prev_y + prev_h + gap`. Cross-axis position from `align`.
  - Same flex behavior as Row but on the vertical axis.
  - Column replaces most manual `Position::Fixed { y: N }` stacking currently done in `build_entity_inspector()` and `build_hover_tooltip()`.
  - Test: build a Column with 5 labels, assert vertical stacking matches expected positions.

- **UI-102** — Text wrapping / line-breaking for Labels. Blocks: UI-301, UI-401.
  - Currently `Widget::Label` is measured as single-line in `measure_node()` (`src/ui/mod.rs`, line ~560): `text.len() * char_w`. No wrapping.
  - Add `wrap: bool` field to `Widget::Label`. Default `false` for backward compatibility.
  - When `wrap: true` and parent provides a max width constraint: break text at word boundaries to fit within `max_width`. Measure height as `line_count * line_height`.
  - Wrapping uses the same approximate `char_w` metric already in `measure_node()`. Exact glyph widths would require `FontRenderer` access during measure — defer that to UI-500.
  - In `draw_node()`: emit one `TextCommand` per wrapped line with incrementing `y` offset.
  - Test: wrap a 200-char string into a 100px-wide area, assert measured height > single line height.

- **UI-103** — Min/Max size constraints on widgets. Blocks: UI-206.
  - `Constraints` struct already exists in `src/ui/mod.rs` (line ~53) with `min_width`, `min_height`, `max_width`, `max_height` and a `clamp()` method — but it is never stored on `WidgetNode` or applied during layout.
  - Add `pub constraints: Option<Constraints>` field to `WidgetNode`. Default `None`.
  - Add `WidgetTree::set_constraints(id, constraints)` setter.
  - In `layout_node()`: after resolving width/height from `Sizing`, apply `constraints.clamp()` if present. This happens before setting `node.rect`.
  - Test: set min_width 200 on a Fit-sized label with 3 chars. Assert rect.width >= 200.

- **UI-104** — Scissor-rect clipping for nested panels. Blocks: UI-204, UI-300, UI-303, UI-401.
  - Currently `DrawList` has no clipping support. `ScrollList` relies on virtual scrolling to avoid drawing out-of-bounds items, but child widget text can still overdraw.
  - Add `pub clip: Option<Rect>` to `DrawList`'s command types (`PanelCommand`, `TextCommand`, `RichTextCommand`). Default `None` = no clipping.
  - Add `pub clip_rect: Option<Rect>` to `WidgetNode`. Set automatically by parent containers that clip (ScrollList, future Modal).
  - In `draw_node()`: propagate `clip_rect` from parent to children. When emitting commands, copy the clip rect onto the command.
  - In `src/main.rs` render loop (line ~1089): before drawing each command, call `render_pass.set_scissor_rect()` if `clip` is `Some`. Reset to full viewport after.
  - `wgpu::RenderPass::set_scissor_rect()` is already available in the pipeline — no shader changes needed.
  - Test: place a Label partially outside a Panel's rect, set clip, assert the command carries the clip rect.

- **UI-105** — Pause overlay. No dependencies.
  - When `paused == true`, emit a fullscreen semi-transparent `PanelCommand` with `[0.0, 0.0, 0.0, 0.15]` after the tile map render, before UI panels. Trivial — one PanelCommand in the main loop's render section.
  - Test: pause simulation, assert a fullscreen panel command is emitted.

- **UI-106** — Map click dispatch. Blocks: UI-400, UI-403.
  - When a mouse click falls through `UiState::hit_test()` (returns None), translate screen coords to tile coords via camera transform and emit a `MapClick { tile_x, tile_y, button }` event on `UiState`.
  - Left-click selects entity at tile (or selects the tile if no entity). Right-click opens context menu for the tile/entity.
  - This is the foundation for all map-level interactions. Currently `handle_mouse_input` in `src/ui/input.rs` only routes to the widget tree.
  - Test: click on an empty map area, assert MapClick event is emitted with correct tile coordinates.

- **UI-107** — Camera controls (pan, zoom, edge-scroll). Blocks: UI-407.
  - Formalize the camera system: WASD/arrow keys for pan, mouse wheel for zoom, edge-of-screen scrolling (move camera when cursor within 20px of screen edge).
  - Zoom-to-cursor: zoom centered on mouse position, not screen center.
  - Smooth interpolation: camera position lerps toward target each frame (existing animation system not needed — just a lerp in the main loop).
  - Zoom levels: 5 discrete levels or continuous. At max zoom-out, each character-cell covers 1 pixel. At max zoom-in, 1 tile = ~32px.
  - Camera bounds: clamp to tile map dimensions. No scrolling past map edge.
  - Test: press W key, assert camera.y decreases. Scroll wheel up, assert camera zoom increases.

- **UI-108** — Time/date display. No dependencies. Integrate into existing status bar.
  - Convert simulation ticks to in-game date. Define a tick-to-date mapping: e.g., 1 tick = 1 minute, 100 ticks = 1 hour, 2400 ticks = 1 day. Starting date: configurable via `world.start_date` (default: January 1, 1845).
  - Display format in status bar: "15 March 1845, 14:30" (or similar period-appropriate format).
  - Add `GameDate` struct to `src/components.rs` (or `world.rs`): year, month, day, hour, minute. Computed from `tick.0` and `world.start_date`.
  - The status bar builder (`build_status_bar` in `src/ui/mod.rs`) replaces "Tick: N" with the formatted date.
  - Test: at tick 0 with start_date Jan 1 1845, assert displayed date is "1 January 1845, 00:00". At tick 2400, assert date is "2 January 1845, 00:00".

## Phase UI-2 — Essential Widgets

Goal: The widget types CK3 uses pervasively in every panel. These are the building blocks game screens are composed from.

The Widget enum will grow from 5 to 13+ variants. When `measure_node`/`draw_node` match arms exceed readability, extract to methods on Widget. Keep `mod.rs` focused on WidgetTree and geometry.

- **UI-200** — Progress bar widget. Needs: UI-100 (Row, for inline placement). Blocks: UI-400, UI-406, UI-414.
  - Add `Widget::ProgressBar { fraction: f32, fg_color: [f32;4], bg_color: [f32;4], border_color: [f32;4], height: f32 }` to `src/ui/widget.rs`.
  - `fraction` is 0.0..=1.0. Clamped in draw.
  - In `measure_node()`: height from field, width = parent-provided (progress bars are always stretch-width in CK3).
  - In `draw_node()`: emit two `PanelCommand`s — background rect at full width, foreground rect at `width * fraction`. Border on the background rect.
  - Add `Theme` fields: `progress_bar_height`, `progress_bar_border_width`, default colors for health (green), stress (red), opinion (blue-to-red gradient later).
  - Test: fraction 0.5 at 200px width produces a foreground panel of width 100.

- **UI-201** — Separator/divider widget. Needs: UI-101 (Column, to stack between sections).
  - Add `Widget::Separator { color: [f32;4], thickness: f32, horizontal: bool }` to `src/ui/widget.rs`.
  - Horizontal separator: width = parent, height = thickness. Vertical: width = thickness, height = parent.
  - In `draw_node()`: emit a single thin `PanelCommand` with no border and no shadow.
  - Add `Theme` field: `separator_color` (default: gold at 30% alpha), `separator_thickness` (default: 1.0).
  - Test: horizontal separator in a 300px-wide column has rect.width = 300, rect.height = 1.

- **UI-202a** — SpriteAtlas data structure. Blocks: UI-202b, UI-202c.
  - **This begins the sprite pipeline — the most substantial UI-2 work.** The current renderer only handles text glyphs (`src/font.rs`, R8Unorm atlas) and panel quads (`src/panel.rs`). Icons require a color texture pipeline.
  - Create `src/ui/sprite.rs`. `SpriteAtlas` struct: loads RGBA PNG at startup, shelf-packs regions by name, stores UV rects in `HashMap<String, SpriteRect>`. Atlas size: 512x512 minimum (holds 1024 16x16 icons), support runtime growth to a second atlas texture if icons exceed capacity. Pure data, no GPU.
  - Test: load a test PNG, register 3 regions, assert UV rects are correct.

- **UI-202b** — Sprite rendering pipeline. Needs: UI-202a. Blocks: UI-202c.
  - Create `src/sprite.wgsl` (textured-quad shader sampling RGBA atlas with optional tint multiply, similar to `text.wgsl` but RGBA).
  - Create `SpriteRenderer` in `src/sprite_renderer.rs` (bind group, vertex buffer, pipeline). Render after panels, before text in main.rs.
  - Test: render a single sprite quad, assert GPU pipeline executes without errors.

- **UI-202c** — Icon widget integration. Needs: UI-202b. Blocks: UI-203, UI-205, UI-400, UI-406, UI-407.
  - Add `Widget::Icon { sprite: String, size: f32, tint: Option<[f32;4]> }` to `src/ui/widget.rs`. `sprite` is the atlas region name.
  - Add `SpriteCommand { sprite: String, x, y, width, height, tint }` to `DrawList` in `src/ui/draw.rs`. New `sprites: Vec<SpriteCommand>` field.
  - Wire `measure_node`, `draw_node`, `apply_opacity` for the Icon variant.
  - Test: insert Icon widget, assert SpriteCommand emitted with UV coords matching the atlas region.

- **UI-202d** — Placeholder sprite sheet. Parallelizable with UI-202a-c.
  - Create a 512x512 RGBA PNG with ~30 icons at 16x16: heart, sword, shield, skull, coin, food, water, star, arrow, cross, house, person, crown, scroll, hammer, gem, leaf, fire, moon, sun, etc. Register all regions in a manifest.
  - Test: all 30 regions resolve to valid non-overlapping UV rects.

- **UI-203** — Checkbox/toggle widget. Needs: UI-202c (Icon, for checkmark sprite). Blocks: UI-400, UI-413.
  - Add `Widget::Checkbox { checked: bool, label: String, color: [f32;4], font_size: f32 }`.
  - In `draw_node()`: emit a small bordered `PanelCommand` for the box (16x16 default), an `Icon` sprite command for the checkmark when checked, and a `TextCommand` for the label offset to the right.
  - In `measure_node()`: width = box_size + gap + label_width. Height = max(box_size, label_height).
  - Checkbox is focusable. Enter/Space toggles. Click toggles. State is read/written by the builder each frame (immediate-mode pattern over the retained tree).
  - Add `UiEvent::Toggle` variant to `src/ui/input.rs` for checkbox state changes.
  - Test: build checkbox, simulate click, assert the builder receives the toggle event.

- **UI-204** — Dropdown/select widget. Needs: UI-100 (Row), UI-101 (Column), UI-104 (clipping the overlay). Blocks: UI-400, UI-403, UI-413.
  - Add `Widget::Dropdown { selected: usize, options: Vec<String>, open: bool, color: [f32;4], bg_color: [f32;4], font_size: f32 }`.
  - Closed state: renders like a Button showing `options[selected]` with a down-arrow indicator.
  - Open state: spawns a Column of clickable option labels below the dropdown rect, overlaid on top of other content. Uses `WidgetTree::insert_root()` for the overlay (drawn last = on top).
  - Clicking an option sets `selected`, closes the dropdown, removes the overlay root.
  - Clicking outside the dropdown while open closes it. Escape closes it.
  - If options count exceeds viewport capacity (overlay height > remaining screen height), wrap the options Column in a ScrollList. Max visible items: `(screen_height - dropdown_rect.bottom) / item_height`, minimum 5.
  - In `measure_node()`: width = widest option text + arrow indicator width + padding. Height = single row.
  - Test: open dropdown, click option 2, assert selected changes and overlay is removed.

- **UI-205** — Slider widget. Needs: UI-100 (Row), UI-202c (Icon, for thumb sprite). Blocks: UI-403, UI-413.
  - Add `Widget::Slider { value: f32, min: f32, max: f32, track_color: [f32;4], thumb_color: [f32;4], width: f32 }`.
  - In `draw_node()`: emit a thin `PanelCommand` for the track (full width, centered vertically), a small `PanelCommand` or `SpriteCommand` for the thumb at `(value - min) / (max - min) * width`.
  - Drag interaction: clicking on the track or thumb captures the widget (reuse existing drag infrastructure in `UiState`). `DragMove` updates `value` proportionally to cursor X.
  - In `measure_node()`: width from field, height = thumb_size (default 16px).
  - Test: set value to 0.5 in a 200px slider, assert thumb center is at x=100.

- **UI-206** — Text input widget. Needs: UI-103 (min/max constraints for field width). Blocks: UI-402, UI-412.
  - Add `Widget::TextInput { text: String, cursor_pos: usize, color: [f32;4], bg_color: [f32;4], font_size: f32, placeholder: String }`.
  - Focusable. When focused: draws a blinking cursor line at `cursor_pos`, accepts character input events.
  - Wire `winit::event::WindowEvent::Ime` / `ReceivedCharacter` (or `KeyEvent` with text) through `UiState` to the focused TextInput.
  - Support: character insertion, Backspace/Delete, Left/Right arrow, Home/End, Ctrl+A select all.
  - Clipboard: Ctrl+C copies selection, Ctrl+V pastes. Use `arboard` crate or winit clipboard access.
  - Word deletion: Ctrl+Backspace deletes previous word.
  - Selection: Shift+Left/Right extends selection. Shift+Home/End selects to start/end. Selected text highlighted with `theme.gold` at 30% alpha.
  - IME composition string support deferred to UI-5xx.
  - In `draw_node()`: emit background `PanelCommand`, `TextCommand` for content (or placeholder in disabled color if empty), thin `PanelCommand` for cursor line.
  - Add `UiEvent::TextChanged(String)` variant for builders to detect input changes.
  - Test: focus input, simulate typing "hello", assert text field contains "hello" and cursor_pos = 5.

## Phase UI-3 — Interaction Patterns

Goal: CK3's core UX patterns — modals, tabs, notifications, context menus, and panel management. These compose Phase UI-2 widgets into reusable interaction flows.

Manager structs (ModalStack, NotificationManager, PanelManager, ContextMenu) each get their own file under `src/ui/`, re-exported from `mod.rs`.

- **UI-300** — Modal/dialog system. Needs: UI-100, UI-101 (Row/Column layout), UI-104 (clipping), UI-305, UI-307. Blocks: UI-401.
  - **The defining CK3 interaction.** Event popups, confirmation dialogs, character interaction prompts.
  - Add a `ModalStack` struct to `src/ui/modal.rs`: `Vec<WidgetId>` of modal root panels. Each modal is a root widget that dims everything behind it.
  - Modal roots are inserted via `WidgetTree::insert_root()` and always drawn last (highest Z). Current system has no Z-ordering beyond implicit draw order — modal roots are appended after all other roots.
  - Dim layer: a fullscreen `PanelCommand` with `[0.0, 0.0, 0.0, 0.4]` background emitted before each modal's subtree.
  - Input blocking: `UiState::hit_test()` already walks roots back-to-front. When a modal is active, hit testing the dim panel returns the modal root, blocking clicks to widgets behind it.
  - Escape dismisses the topmost modal (integrate with existing `Action::CloseTopmost` in `src/ui/keybindings.rs`).
  - `ModalStack::push(tree, widget_id)` / `pop(tree)` API. `pop` calls `tree.remove()` on the modal root.
  - Test: push two modals, press Escape, assert only the top modal is removed, bottom modal persists.

- **UI-301** — Tab container widget. Needs: UI-100 (Row for tab bar), UI-101 (Column for content area), UI-102 (text wrapping in tab labels), UI-305. Blocks: UI-400, UI-412, UI-413, UI-415.
  - Add `Widget::TabContainer { tabs: Vec<String>, active: usize, tab_color: [f32;4], active_color: [f32;4], font_size: f32 }`.
  - Layout: Column with a Row of tab buttons at the top, and a content Panel below. Only the `active` tab's content children are laid out and drawn.
  - Tab buttons are clickable. Clicking a tab sets `active` and triggers a rebuild of the content area.
  - Content area children are managed by the builder: each frame, the builder inserts only the children for the active tab. The TabContainer handles drawing the tab bar; content is standard child layout.
  - In `measure_node()`: width = max(tab_bar_width, widest_content). Height = tab_bar_height + active_content_height.
  - Add `Theme` fields: `tab_active_color`, `tab_inactive_color`, `tab_bar_height`.
  - Test: build 3-tab container, set active=1, assert only tab 1's content children are visible.

- **UI-302** — Notification/toast system. Needs: UI-100 (Row for icon+text). Blocks: UI-405.
  - Top-right notification stack, exactly like CK3's alert system: war declared, heir born, scheme discovered.
  - Add `NotificationManager` struct to `src/ui/notification.rs`. Holds `Vec<Notification>` where each has: `message: String`, `icon: Option<String>`, `priority: NotificationPriority`, `created: Instant`, `duration: Duration`, `dismissed: bool`.
  - `NotificationPriority` enum: `Info`, `Important`, `Critical`. Critical: `theme.danger` border + Animator-driven pulsing alpha. Important: `theme.gold` border. Info: standard. Sort: Critical on top, then Important, then Info.
  - Each tick, `NotificationManager::build()` constructs a Column of notification panels anchored to top-right of screen (using `Position::Percent { x: 1.0, y: 0.0 }` with negative x offset for width).
  - Notifications auto-dismiss after `duration` (default 8 seconds). Click to dismiss immediately. Stacking: newest on top, older slide down.
  - Animate fade-in/fade-out using the existing `Animator` in `src/ui/animation.rs`.
  - Max visible: 5 notifications. Excess queued until space opens.
  - Test: push 3 notifications, advance time past duration of first, assert first is dismissed and remaining 2 shift up.

- **UI-303** — Context menu (right-click). Needs: UI-101 (Column for menu items), UI-104 (clipping for screen-edge adjustment), UI-305. Blocks: UI-400, UI-406.
  - Add `ContextMenu` struct in `src/ui/context_menu.rs`: spawns a Column of clickable labels at cursor position on right-click.
  - Menu is a root widget (like dropdown overlay) inserted via `insert_root()`. Positioned at click coords, clamped to screen bounds.
  - Each menu item: `{ label: String, action: String, enabled: bool }`. Disabled items shown in `theme.disabled` color, not clickable.
  - Clicking an item fires a callback (store action string, builder checks it next frame), then removes the menu.
  - Clicking outside or pressing Escape dismisses the menu.
  - Supports nested submenus: hovering an item with children spawns a child menu to the right. Max nesting depth: 2.
  - Test: spawn context menu at (100, 100), click item 1, assert menu is removed and action string matches.

- **UI-304** — Collapsible section widget. Needs: UI-101 (Column). Blocks: UI-400, UI-405, UI-406.
  - Add `Widget::Collapsible { header: String, expanded: bool, color: [f32;4], font_size: f32 }`.
  - Header row: clickable label with a triangle indicator (right-pointing when collapsed, down-pointing when expanded). Uses Unicode triangles (already in the glyph atlas: U+25B6, U+25BC).
  - When collapsed: children are not laid out or drawn. `measure_node()` returns header height only.
  - When expanded: children are laid out in a Column below the header. `measure_node()` returns header + children height.
  - Click on header toggles `expanded`. Animate expand/collapse with existing `Animator` (slide content height from 0 to measured).
  - Test: build collapsible with 3 children, toggle collapsed, assert measured height decreases to header-only.

- **UI-305** — Event/callback dispatch system. Needs: none (refactoring of existing code). Blocks: UI-300, UI-301, UI-303, UI-400.
  - Currently click handling in `src/main.rs` (line ~528) polls `UiState` and checks widget IDs manually: `if clicked_id == some_button_id { ... }`. This does not scale to many interactive widgets.
  - Add `pub on_click: Option<String>` field to `WidgetNode`. Builder sets a callback key (e.g. `"close_inspector"`, `"tab_switch:2"`, `"context:declare_war"`).
  - Add `UiState::poll_click() -> Option<(WidgetId, String)>` that returns the most recent click's widget and callback key. Cleared each frame.
  - Main loop checks `poll_click()` once and dispatches by string prefix. No function pointers, no closures, no trait objects — just string keys matched in a `match` block.
  - String keys use hierarchical namespace: `panel_name::action::param`. Consider migrating to enum-based callback IDs if match block exceeds 50 arms.
  - Extend to `on_toggle`, `on_text_changed`, `on_select` for Checkbox, TextInput, Dropdown.
  - Migrate existing button-click checks in main.rs to use callback keys.
  - Test: set on_click = "test_action" on a button, simulate click, assert poll_click returns "test_action".

- **UI-306** — Panel stack / window management. Needs: UI-305 (callback dispatch for close buttons), UI-307 (z-order tiers). Blocks: UI-400, UI-402, UI-405, UI-412, UI-413, UI-415.
  - Multiple panels open simultaneously (character panel, outliner, event log — all visible in CK3).
  - Add `PanelManager` struct to `src/ui/panel_manager.rs`: tracks open panels by name (`HashMap<String, WidgetId>`), draw order (`Vec<String>`), and closeable flag.
  - Opening a panel: builder creates widget subtree, registers with `PanelManager`. Panel gets a close button (top-right X) with callback key `"panel_close:<name>"`.
  - Clicking a panel brings it to front: `PanelManager::raise(name)` reorders the draw list so it is the last root (topmost).
  - Closing a panel: removes its subtree via `WidgetTree::remove()` and deregisters from PanelManager.
  - Z-order: panels draw in `draw_order` sequence. Modals (UI-300) always draw above all panels.
  - Escape closes the topmost non-modal panel (extend `Action::CloseTopmost` logic).
  - Test: open 3 panels, raise panel 1, assert it is last in draw order. Close panel 2, assert its subtree is removed.

- **UI-307** — Z-order tier system for root widgets. Blocks: UI-300, UI-306.
  - Split `WidgetTree::roots` into tiered draw order: `panel_roots`, `overlay_roots` (dropdowns, context menus), `modal_roots`, `tooltip_roots`. Draw in that order. Alternatively, add `z_tier: u8` to root entries and sort.
  - This prevents `PanelManager::raise()` from accidentally placing a panel above a modal.
  - Test: raise a panel while a modal is open, assert the modal still draws last.

## Phase UI-4 — Game Screens

Goal: CK3-equivalent screens composed from Phase UI-1 through UI-3 primitives. Each screen is a builder function in its own file under `src/ui/`.

- **UI-400** — Full character panel. Needs: UI-106 (map click to select entity), UI-200 (ProgressBar), UI-202c (Icon), UI-203 (Checkbox), UI-301 (TabContainer), UI-303 (ContextMenu), UI-304 (Collapsible), UI-305 (callbacks), UI-306 (PanelManager). Blocks: UI-402, UI-406.
  - Create `src/ui/character_panel.rs`. Builder function: `pub fn build_character_panel(tree: &mut WidgetTree, theme: &Theme, entity: Entity, world: &World) -> WidgetId`.
  - **Overview tab**: entity name (header font), icon (creature type), health bar (ProgressBar), hunger bar, age, current gait. Stat rows in a Column, each a Row of label + value. Trait icon ring around creature icon in the header — small icons (12x12) from sprite atlas arranged in a row below the entity name. Needs UI-202c.
  - **Family tab**: parent/children/spouse labels. Placeholder until SIM-008 (relationships) is implemented. Show entity IDs for now.
  - **Relations tab**: opinion modifiers list using Collapsible sections (one per relationship type). Placeholder until SIM-008.
  - **Traits tab**: list of entity properties as icons with labels. Trait tooltips showing values.
  - Right-click on the character panel header spawns a ContextMenu with entity interactions (attack, follow, inspect).
  - Panel registered with PanelManager as `"character:<entity_id>"`. Opening a second character panel for a different entity creates a second instance.
  - Test: build character panel for a test entity, assert TabContainer has 4 tabs, Overview tab contains a ProgressBar for health.

- **UI-401** — Event popup screen. Needs: UI-102 (text wrapping), UI-104 (clipping), UI-300 (Modal). Blocks: UI-405.
  - Create `src/ui/event_popup.rs`. Builder function: `pub fn build_event_popup(tree: &mut WidgetTree, theme: &Theme, event: &NarrativeEvent) -> WidgetId`.
  - `NarrativeEvent` struct (new, in `src/events.rs`): `title: String`, `body: String`, `choices: Vec<EventChoice>`. `EventChoice`: `label: String`, `tooltip: Option<String>`, `callback: String`.
  - Layout: modal overlay. Top half: title (header font) + wrapped body text (body font). Bottom: Row of choice buttons.
  - Choice button tooltips show consequence preview (e.g., "Opinion -10 with Vassals"). Uses existing tooltip system.
  - Clicking a choice dispatches its callback key via UI-305 and dismisses the modal via UI-300.
  - CK3 style: parchment background, gold border, slightly larger than standard panels (60% of screen width, capped at 600px).
  - Test: build event popup with 3 choices, click choice 2, assert modal is dismissed and callback "choice:2" is dispatched.

- **UI-402** — Character finder. Needs: UI-206 (TextInput), UI-306 (PanelManager).
  - Create `src/ui/character_finder.rs`. Builder function: `pub fn build_character_finder(tree: &mut WidgetTree, theme: &Theme, world: &World) -> WidgetId`.
  - TextInput at the top for search. Below: ScrollList of matching entities, each row showing icon + name + occupation + location.
  - Filter as you type: each frame, the builder filters `world.names` (or equivalent property table) by substring match against the TextInput value.
  - Sort options: by name (alpha), by distance to camera, by health. Add sort Dropdown (UI-204) in the header row.
  - Clicking an entity row opens its character panel (UI-400) and centers the camera on it.
  - Panel registered with PanelManager as `"finder"`. Only one finder open at a time.
  - Keybinding: Ctrl+F opens/focuses the finder (add `Action::ToggleFinder` to `src/ui/keybindings.rs`).
  - Test: build finder with 10 entities, type "gob" in search, assert filtered list contains only entities whose names contain "gob".

- **UI-403** — Map mode selector. Needs: UI-106 (map click dispatch), UI-204 (Dropdown), UI-205 (Slider for speed visual).
  - Create `src/ui/map_mode.rs`. Builder function integrates into the status bar area.
  - Map modes: Terrain (default), Political (quartier coloring from SCALE-C01), Population density (heat map from SCALE-C04). More modes added as simulation features land.
  - Map mode enum should be open-ended (not hardcoded to 3 variants). The Dropdown should support adding modes as simulation features land (de jure overlays, economic heat maps, etc.).
  - UI: Dropdown or row of toggle buttons in the status bar. Active mode stored on App state, read by the tile renderer each frame.
  - Game speed slider: visual indicator next to the map mode selector. Reads `sim_speed` from App. Dragging the slider changes speed (1x-5x), same as pressing 1-5 keys.
  - Purely a UI control — the actual map rendering changes are in the tile shader, not in this task.
  - Test: select "Political" mode from dropdown, assert App state reflects the change.

- **UI-405** — Outliner panel. Needs: UI-302 (Notifications for alerts), UI-304 (Collapsible), UI-306 (PanelManager), UI-401 (event popups linked from outliner).
  - Create `src/ui/outliner.rs`. Persistent side panel (right edge, CK3 style) showing pinned items.
  - Collapsible sections: "Pinned Characters" (click to open character panel), "Active Events" (click to open event popup), "Alerts" (summary of pending notifications).
  - Each section is a Collapsible with a ScrollList of items inside. Items are clickable.
  - "Pin" button on character panels (UI-400) adds entities to the outliner's pinned list. Stored on App state, not World.
  - Outliner auto-opens on game start. Toggle with a keybinding (add `Action::ToggleOutliner`).
  - Test: pin 3 entities, build outliner, assert "Pinned Characters" section contains 3 items.

- **UI-406** — Relationship/opinion view. Needs: UI-200 (ProgressBar for opinion bar), UI-202c (Icon for sentiment indicators), UI-303 (ContextMenu), UI-304 (Collapsible), UI-400 (parent character panel). **BLOCKED: requires SIM-008 (relationships).** Can be stubbed with mock data for UI testing.
  - Create `src/ui/opinion_view.rs`. Sub-panel within the character panel's Relations tab (UI-400).
  - Opinion bar: colored ProgressBar from -100 (red) to +100 (green), centered at 0. Needs a centered-origin progress bar variant or two stacked bars.
  - Opinion breakdown: Collapsible section listing each modifier (e.g., "Same culture +15", "Rival -50", "Recent gift +10") with icon, label, and value.
  - Each modifier row optionally displays a duration/expiry field (e.g., "3y remaining") if the modifier is temporary.
  - Sentiment indicators: ally/rival/friend/lover icons from the sprite atlas (UI-202c).
  - Right-click on the opinion header opens a ContextMenu with interaction options.
  - Test: build opinion view with mock modifiers summing to +25, assert opinion bar fraction = 0.625 (mapping -100..+100 to 0..1).

- **UI-407** — Mini-map. Needs: UI-107 (camera controls for click-to-navigate), UI-202c (sprite/texture pipeline for the minimap texture).
  - Create `src/ui/minimap.rs`. Small panel in the bottom-right showing a downscaled overview of the tile map.
  - Render approach: CPU-side downscale of the tilemap to a small RGBA buffer (e.g., 128x96 for the 6309x4753 map). Upload as a wgpu texture. Display via the sprite rendering pipeline (UI-202c).
  - Viewport indicator: a white/gold rectangle overlay on the minimap showing the current camera bounds. Updated each frame from camera position + zoom.
  - Click-to-navigate: clicking on the minimap teleports the camera to that world position. Map cursor coords to world coords via the downscale ratio.
  - Rebuild the minimap texture only when the camera moves more than N tiles or every 60 frames — not every frame.
  - Test: click at minimap center, assert camera position moves to world center.

- **UI-412** — Save/Load screen. Needs: UI-206 (TextInput for save name), UI-301 (TabContainer), UI-306 (PanelManager). Blocks: UI-415.
  - Create `src/ui/save_load.rs`. Two tabs: Save and Load.
  - Save tab: TextInput for save name + Save button.
  - Load tab: ScrollList of save files (read from disk) with timestamp, click to select, Load button. Delete button with confirmation.
  - Panel registered with PanelManager. Keybinding: Ctrl+S for quick save, F5/F9 for save/load.
  - Test: build save screen, assert TabContainer has 2 tabs and Save tab contains a TextInput.

- **UI-413** — Settings screen. Needs: UI-203 (Checkbox), UI-204 (Dropdown), UI-205 (Slider), UI-301 (TabContainer), UI-306 (PanelManager).
  - Create `src/ui/settings.rs`. Tabs: Display (ui_scale slider from UI-504, window mode dropdown), Audio (volume sliders — placeholder until audio system), Controls (read-only keybinding list).
  - Panel registered with PanelManager. Keybinding: Escape from main menu (future), or dedicated key.
  - Test: build settings screen, change ui_scale slider, assert Theme reflects new scale value.

- **UI-414** — Loading screen. Needs: UI-200 (ProgressBar).
  - Displayed during startup while GIS data loads (paris.tiles binary, KDL creature definitions, sprite atlas).
  - Full-screen panel with centered title ("Wulfaz"), a ProgressBar showing load progress (0.0 to 1.0), and a status label ("Loading terrain..." / "Spawning entities..." / etc.).
  - Loading stages: define an enum of load stages. Each stage advances the progress bar. The main loop renders the loading screen between stages (requires splitting initialization into yield points).
  - After loading completes, transition to the game view (remove loading screen root, start the simulation).
  - Test: set progress to 0.5 with status "Loading terrain...", assert ProgressBar fraction is 0.5 and label text matches.

- **UI-415** — Main menu. Needs: UI-301 (TabContainer or just buttons), UI-306 (PanelManager), UI-412 (Save/Load for the Load button).
  - Displayed on application start before the game world loads.
  - Centered panel with game title, buttons: "New Game", "Continue" (loads most recent save), "Load Game" (opens UI-412), "Settings" (opens UI-413), "Quit".
  - No background map rendering (just the parchment background or a static splash).
  - "New Game" triggers world generation / GIS loading (UI-414 loading screen).
  - State machine in App: `AppState::MainMenu | AppState::Loading | AppState::InGame`. Only InGame runs the simulation tick loop.
  - Test: build main menu, click "Quit", assert App state transitions to exit. Click "New Game", assert App state transitions to Loading.

## Phase UI-5 — Polish & Architecture

Goal: Performance, accessibility, and deferred architectural improvements. These tasks are parallelizable and can land in any order.

- **UI-500** — Retained tree optimization (incremental rebuild). Needs: UI-505 (baseline measurements before optimizing). Blocks: none (performance improvement).
  - Currently the UI tree is fully rebuilt every frame (DD-5 decision in `src/main.rs`, line ~735 onward). Every `build_*` function constructs the entire subtree from scratch.
  - Phase 1: add a `generation: u64` counter to `WidgetTree`. Each `build_*` call stamps its widgets. After all builders run, `WidgetTree::gc()` removes widgets from previous generations — avoids rebuilding unchanged panels.
  - Phase 2: builders check if their data has changed (hash of StatusBarInfo, event log length, etc.) and skip rebuild if unchanged. Return the existing root WidgetId instead of constructing new widgets.
  - Phase 3: incremental layout — only re-layout subtrees whose root is dirty. The `dirty` flag on `WidgetNode` already exists but is not used to skip layout.
  - Measure impact: add `std::time::Instant` timing around `build_*` + `layout()` + `draw()` in the main loop. Log per-frame UI cost. Target: <1ms total UI time at 60fps.
  - Test: build tree, assert generation increments. Build again without changes, assert no new widget allocations.

- **UI-501** — Variable-height ScrollList items. Blocks: none (enhancement to existing widget).
  - Current `ScrollList` in `src/ui/widget.rs` has a fixed `item_height: f32`. All items are the same height. Event log entries with different line counts waste space or truncate.
  - Replace `item_height` with `item_heights: Vec<f32>` (or measure each child individually).
  - In `layout_node()` for ScrollList: compute cumulative Y offsets from variable heights. Virtual scrolling: binary search for the first visible item instead of `index * item_height`.
  - Scrollbar thumb size: `viewport_height / total_content_height * viewport_height`. Scrollbar position: `scroll_offset / total_content_height * viewport_height`.
  - Update `handle_key_input` in `src/ui/input.rs` to scroll by the actual height of the next/previous item rather than a fixed `item_height`. Requires knowing which item is currently at the top of the viewport.
  - Backward-compatible: if all items have the same measured height, behaves identically to current fixed-height mode.
  - Test: build ScrollList with items of heights [20, 40, 20, 60], assert total content height = 140 and third item starts at y=60.

- **UI-504** — UI scaling / accessibility. Blocks: none (can land independently).
  - Add `ui_scale: f32` to `Theme` (default 1.0). All pixel values in Theme are multiplied by `ui_scale`.
  - Add `Action::ScaleUp` and `Action::ScaleDown` keybindings (Ctrl+= and Ctrl+-). Adjust `ui_scale` by 0.1 increments, clamped to 0.5..=2.0.
  - Font sizes: `theme.font_body_size * ui_scale` passed to all `build_*` functions. This scales text uniformly.
  - Panel dimensions, padding, margins, gap values: all multiplied by `ui_scale` at builder time.
  - High-contrast mode: add `high_contrast: bool` to Theme. When enabled, increase border widths by 1px, boost text alpha to 1.0, use higher-contrast color pairs (pure white on dark brown instead of parchment on parchment).
  - The `line_height` parameter in `WidgetTree::layout()` (called from `src/main.rs` in the render section) must also scale with `ui_scale`.
  - Test: set ui_scale 2.0, build status bar, assert all rects are 2x the default size.

- **UI-505** — Performance profiling of UI tree rebuild + draw. Blocks: UI-500. No other dependencies (diagnostic).
  - Add per-phase timing to the main loop's UI section in `src/main.rs`: `build_time`, `layout_time`, `draw_emit_time`, `render_time`.
  - Display in the status bar (or a toggleable debug overlay) as: "UI: build 0.3ms | layout 0.1ms | draw 0.2ms | render 0.4ms".
  - Track widget count, panel command count, text command count, sprite command count per frame.
  - Log warnings when any phase exceeds 2ms (60fps budget = 16ms total, UI should be <25% = 4ms).
  - This is a prerequisite measurement for UI-500 (retained tree optimization) — provides the baseline numbers to prove optimization impact.
  - Test: run a frame with the demo showcase active, assert all four timing values are populated and positive.

## Deferred

- **UI-D01** — egui dev tools overlay. Add `egui-wgpu` + `egui-winit`. Second render pass after game UI. Entity inspector, world state browser, system performance view. Toggle with a key (F12). Debug-only layer, not player-facing. Independent of the custom widget pipeline — can be added at any point.
  - Test: toggle egui overlay with F12, assert the second render pass executes and displays at least one egui window.

- **UI-D06** — DrawList line primitives. Add `lines: Vec<LineCommand>` to DrawList and a line-rendering shader. Needed for family trees, tech trees, graph connections. Blocks: UI-D02.
  - Test: emit 3 LineCommands, assert all 3 appear in the DrawList with correct start/end coordinates.

- **UI-D02** — Family tree graph view. Scrollable/zoomable graph of entity portraits connected by lines. Requires line-drawing primitives in DrawList (`LineCommand`). Needs: UI-400, UI-104, UI-D06, SIM-008.
  - Test: build a family tree with 5 entities and 4 connections, assert correct number of LineCommands emitted.

- **UI-D03** — District/holding hierarchy panel. Tree view of quartiers/blocks/buildings with holders and stats. Needs: SCALE-C01.
  - Test: build hierarchy panel for a test quartier with 3 blocks, assert 3 collapsible sections rendered.

- **UI-D04** — Economy/resources panel. Income/expense ledger with per-district breakdown. Needs: SCALE-C04.
  - Test: build economy panel with mock district data, assert income and expense rows sum correctly.

- **UI-D05** — Battle/combat viewer panel. Army list, combat progress bars, outcome display. Needs: SIM-005.
  - Test: build combat viewer with 2 armies, assert progress bars reflect HP ratios.

- **UI-404** — Decision panel. Needs: UI-304 (Collapsible), UI-301 (TabContainer). Premature until SIM-008/SIM-009 provide real decision trees.
  - Create `src/ui/decision_panel.rs`. Lists available player actions grouped by category.
  - Each decision: label, requirement tooltip (conditions not yet met shown in red), execute button.
  - Decisions grouped into Collapsible sections: Diplomacy, Intrigue, Stewardship, etc.
  - Enabled/disabled state based on world conditions. Disabled decisions shown with `theme.disabled` color.
  - Clicking "Execute" dispatches a callback key and triggers the action in the simulation.
  - Placeholder content until simulation systems (SIM-008, SIM-009) provide real decision trees.
  - Test: build decision panel with 2 categories and 3 decisions, assert collapsible sections contain the expected decision buttons.

- **UI-502** — Drag-and-drop support. CK3 barely uses this; low priority.
  - Current drag infrastructure in `src/ui/input.rs` tracks `captured`, `press_origin`, `dragging` — but only for scrollbar thumb dragging.
  - Generalize: add `pub draggable: bool` and `pub drop_target: bool` fields to `WidgetNode`.
  - When a draggable widget is dragged past the threshold: create a ghost overlay (semi-transparent copy of the widget) that follows the cursor. Store `drag_payload: Option<String>` on `UiState`.
  - On mouse release: hit-test for the drop target under the cursor. If a `drop_target` widget is found, dispatch `UiEvent::Drop { payload: String }`.
  - Use case: reordering pinned characters in the outliner, moving items between inventory slots (future SIM-011).
  - Test: start drag on draggable widget, move to drop target, release, assert Drop event is dispatched with correct payload.

- **UI-503** — Sound effect hooks. No audio backend exists; deferred until one does.
  - Add `SoundEvent` enum: `Click`, `Hover`, `Open`, `Close`, `Error`, `Notification`.
  - Add `pub sound_events: Vec<SoundEvent>` to `UiState`. Populated during input processing (click on button = `Click`, modal open = `Open`, etc.).
  - Main loop drains `sound_events` each frame and plays corresponding audio. Audio backend is out of scope for this task — just emit the events.
  - Hook points: `UiState::handle_mouse_input()` emits `Click` on button clicks, `input.rs` tooltip show emits a subtle sound, `ModalStack::push` emits `Open`.
  - No audio crate dependency in this task. The sound playback system is a separate integration.
  - Test: simulate button click, assert `sound_events` contains `SoundEvent::Click`.

## Pending (threshold not yet met)

- **GROW-002** — Phase function grouping. Trigger: >30 system calls.
- **GROW-003** — System dependency analyzer. Trigger: >15 system files.
