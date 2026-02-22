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
