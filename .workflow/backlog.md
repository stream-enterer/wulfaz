# Backlog

Incomplete work only. Delete entries when done.
See `architecture.md` for technical spec on all SCALE tasks.
GIS data reference: `~/Development/paris/PROJECT.md`

## Phase A — Chunked Map + GIS Loading

Goal: See Paris on screen. No entities.

Map dimensions: 6,309 x 4,753 tiles at 1m/tile (vertex-crop of all buildings + 30m padding).
That is ~99 x 75 chunks at 64×64 = ~7,400 chunks, ~30M tiles.

- **SCALE-A09** — Water/bridge polish. Needs: A08 (done). **Remaining known limitations:**
  - **Eastern coverage gap**: ~150-tile-wide hole in ALPAGE data (tiles ~4950-5100 X, ~3500-3900 Y). Road patch in the Seine near Pont d'Austerlitz. Components #12 (2777 tiles) and #13 (424 tiles) are data-gap artifacts, not real bridges. Fix: obtain APUR PLAN D'EAU shapefile, reproject from Lambert-93 (EPSG:2154) to WGS84 via ogr2ogr, feed through `rasterize_water_polygons()`.
  - **Western bridge coverage**: ALPAGE water polygons don't extend west of ~lon 2.336 (5 bridges: Invalides, Concorde, Royal, Carrousel, Arts). Same fix — supplemental data needed.
  - **North arm bridge gap**: No detected bridge components in the north arm between Pont Neuf and Ile Saint-Louis (Pont au Change, Notre-Dame, d'Arcole). Either ALPAGE data doesn't fully cover this arm or bridges merged with island road network. Needs investigation.
  - **Canal Saint-Martin**: not in the ALPAGE Vasserot Hydrography layer. Separate historical data source needed.
  - **Diagnostic match rate**: 7/15 in-coverage bridges match (47%). 7 confident matches (dist 2-6 tiles). 8 misses are north-arm bridges or small bridges without separate components.
  - **Bridge names**: Should name bridges as landmarks consistent with current naming system

## Phase B — Entities in One Neighborhood

Goal: ~200 entities with full AI on the real map.

- **SCALE-B06** — Building interior generation. Needs: B05, A07 (done). **BLOCKED: design review required.**
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

- **SCALE-B03** — GIS-aware entity spawning. Needs: A07 (done), B05. **BLOCKED: design review required.**
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
  - Must accept a `force_statistical: bool` override (from speed settings). When true, all zones become Statistical regardless of camera position. Used by speeds 3-5 in the non-linear speed model (see `SURVIVAL_MIGRATION.md` §1.1b).
  - Zone transitions triggered by both camera movement AND speed changes. Speed increase (1→3) = dehydrate all. Speed decrease (3→1) = hydrate around camera.

- **SCALE-C03** — Zone-aware system filtering. Needs: C02.
  - Active only: combat, wander/pathfinding, eating (Phase 4 action systems).
  - Active+Nearby: survival (digestion/thirst/vitamins/stim), fatigue, decisions.
  - Statistical: no per-entity iteration. All simulation via `run_district_stats()`.
  - Temperature (Phase 1) runs per-chunk with dirty flags — zone filtering is chunk-level, not entity-level.
  - Note: "Hunger" system is deleted by survival migration. This item refers to the replacement `run_survival()` system.

- **SCALE-C04** — District aggregate model + `run_district_stats`. Population, avg needs, death rates, resource flows as equations. Needs: C01, A07 (done). **BLOCKED: design review required.**
  - Seed `population_by_type` from NAICS distribution per quartier. 22 industry categories. Aggregate from building registry occupant data (baked in by A07 preprocessor), not from raw GeoPackage.
  - City-wide distribution (1845): Manufacturing 18%, Food stores 13.5%, Clothing 11.7%, Furniture 8.2%, Legal 5.9%, Health 5.5%, Rentiers 4.5%, Arts 3.9%, Construction 3.6%. Use these as priors, adjust per quartier from actual registry data.
  - **Survival aggregates (required for speeds 3-5):** Each district must track aggregate survival state sufficient to model calorie burn, food consumption, and death without per-entity simulation. Minimum fields per district:
    - `population: u32` — current living population
    - `avg_stored_kcal: f32` — mean calorie reserves (tracks weight/starvation)
    - `avg_thirst: f32` — mean thirst level
    - `food_supply: f32` — abstract food availability (produced/consumed per tick)
    - `water_supply: f32` — abstract water availability
    - `death_rate: f32` — deaths per survival tick from starvation/dehydration
  - District survival equation (per survival tick): `avg_stored_kcal += food_intake - bmr_burn`. When `avg_stored_kcal` drops below starvation thresholds, `death_rate` increases and `population` decreases. Exact equations TBD during design review — must produce results consistent with per-entity simulation over the same time period.
  - These aggregates are seeded during dehydration (D02) from collapsing entity state, and sampled during hydration (D01) to rebuild entity survival snapshots.
  - **Statistical death picks real entity IDs.** When the district model determines N deaths this tick, it selects N `SleepingEntity` records from the district roster (weighted by vulnerability — lowest `stored_kcal`, highest `thirst`) and marks them `dead: true`. This ensures identity consistency: a specific named person dies, not an abstract population decrement. Dead sleepers are never hydrated.

- **SCALE-C05** — Statistical population seeding. Every district outside active zone gets aggregate population. Needs: C02, C04, A07 (done). **BLOCKED: design review required.**
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

- **SCALE-D01** — Hydration. Sleeping → Active: reconstitute entities at building positions. Batch ~100/tick. Needs: C05, B03.
  - Triggered by: camera entering district range OR speed decrease (3→1, 3→2).
  - Look up `SleepingEntity` records for the district. Skip any with `dead: true`. For each living sleeper:
    - Re-insert into `alive` set using the **same Entity ID** (identity preserved).
    - Rebuild full component state from compact snapshot: `stored_kcal` → `BodyComposition`, `thirst` → `Thirst`, `health` → `Health`. Plus identity fields from the record: `name`, `home_building`, `occupation`.
    - Stomach/guts start empty (mid-digestion state not preserved in snapshot).
    - Vitamins at midpoint (insufficient per-entity data in snapshot).
    - Position: entity's `home_building` tile from building registry.
  - Remove reconstituted entities from the sleeping roster.
  - Update district aggregate (C04): subtract reconstituted population from district stats.
- **SCALE-D02** — Dehydration. Active → Sleeping: snapshot and suspend entities. Nearby zone buffers for ~200 ticks. Needs: C02.
  - Triggered by: camera leaving district range OR speed increase (1→3, 2→3+).
  - For each active entity in the dehydrating district:
    - Write a `SleepingEntity` record preserving: Entity ID, name, home_building, occupation, stored_kcal, thirst, health. Dead flag = false.
    - Remove from `alive` and all property tables (same as `despawn()` but entity is NOT dead — it's sleeping).
    - Do NOT reuse the Entity ID. `next_entity_id` is not affected.
  - Compute district aggregates from the snapshot data (mean stored_kcal, mean thirst, population count) and feed into C04.
  - **Sleeping entity storage:** `SleepingEntity` records live in a new `HashMap<Entity, SleepingEntity>` on World (or on `GisTables`, grouped by district). ~100 bytes per entity × 1M population ≈ 100MB. Bounded.
- **Sleeping entity model** (cross-cutting D01/D02/C04):
  - Three-tier entity lifecycle: **Active** (in `alive`, full component state) ←→ **Sleeping** (compact record, no components) → **Dead** (removed entirely).
  - Entity IDs are permanent. `Entity(47392)` is always the same person regardless of lifecycle tier.
  - `despawn()` is for Active entity death only. Sleeping entities bypass it.
  - Statistical death (C04): when the district model rolls deaths, it picks sleeping entity IDs from the district roster, marks them `dead: true`, and decrements district population. Dead sleepers are never hydrated. They can be pruned periodically or retained for historical records.
  - The player entity is never dehydrated. Optionally, a small set of "important" entities (quest targets, historical figures the player has interacted with) can be exempt from dehydration.
- **SCALE-D03** — HPA* pathfinding. Chunk-level nav graph, border nodes, precomputed intra-chunk paths. Replaces B04. Needs: A02 (done).
- **SCALE-D04** — Profile and tune. Zone radii, hydration batch size, tick budget per zone, entity count limits.

## Architecture — DA-1/DA-2 Dialectic Results

Source: `.workflow/dialectic-da1-da2/99-results-summary.md`

All ARCH items implemented. ARCH-001 through ARCH-007 completed and deleted from backlog.

## Simulation Features (parallel or post-Phase B)

Developable on test map or integrated after Phase B.

- **SIM-001** — Plant growth (Phase 1). Food regeneration. Garden tiles only (24 in dataset). Needs: B05 (garden placement). **BLOCKED: design review required — using CDDA implementation as reference.**
- **SIM-002** — Thirst (Phase 2). Requires Water tiles (Seine) and fountains (3 named "Fontaine" buildings + "Pompe de la Samaritaine" in data). **BLOCKED: design review required — using CDDA implementation as reference.**
- **SIM-003** — Decay (Phase 1). Corpse decomposition.
- **SIM-004** — Tiredness/sleep (Phase 2). Rest cycles. Entities return to their home building. **BLOCKED: design review required — using CDDA implementation as reference.**
- **SIM-005** — Injury (Phase 5). Non-binary damage states.
- **SIM-006** — Weather (Phase 1). Rain/drought/cold.
- **SIM-007** — Emotions/mood (Phase 2). Aggregate need state.
- **SIM-008** — Relationships (Phase 5). Bonds from interaction.
- **SIM-009** — Reputation (Phase 5). Observed behavior.
- **SIM-010** — Building (Phase 4). Requires inventory.
- **SIM-011** — Crafting (Phase 4). Requires recipes.
- **SIM-012** — Fluid flow (Phase 1). Cellular automaton. Needs: A08 (done, Seine placement).

## Phase UI-5 — Polish & Architecture (remaining)

- **UI-500** — Retained tree optimization (incremental rebuild). Needs: UI-505 (done). Blocks: none.
  - Phase 1: `generation: u64` counter + `WidgetTree::gc()`.
  - Phase 2: builders skip rebuild if data unchanged.
  - Phase 3: incremental layout via dirty flags.

## Deferred

### Panels & Screens

- **UI-D01** — egui dev tools overlay. Add `egui-wgpu` + `egui-winit`. Second render pass after game UI. Entity inspector, world state browser, system performance view. Toggle with a key (F12). Debug-only layer, not player-facing. Independent of the custom widget pipeline — can be added at any point.
  - Test: toggle egui overlay with F12, assert the second render pass executes and displays at least one egui window.

- **UI-D02** — Family tree graph view. Scrollable/zoomable graph of entity portraits connected by lines. Requires line-drawing primitives in DrawList (`LineCommand`). Needs: UI-D06, SIM-008.
  - Test: build a family tree with 5 entities and 4 connections, assert correct number of LineCommands emitted.

- **UI-D03** — District/holding hierarchy panel. Tree view of quartiers/blocks/buildings with holders and stats. Needs: SCALE-C01.
  - Test: build hierarchy panel for a test quartier with 3 blocks, assert 3 collapsible sections rendered.

- **UI-D04** — Economy/resources panel. Income/expense ledger with per-district breakdown. Needs: SCALE-C04.
  - Test: build economy panel with mock district data, assert income and expense rows sum correctly.

- **UI-D05** — Battle/combat viewer panel. Army list, combat progress bars, outcome display. Needs: SIM-005.
  - Test: build combat viewer with 2 armies, assert progress bars reflect HP ratios.

- **UI-404** — Decision panel. Premature until SIM-008/SIM-009 provide real decision trees. Widget dependencies (Collapsible, TabContainer) done.
  - Create `src/ui/decision_panel.rs`. Lists available player actions grouped by category.
  - Each decision: label, requirement tooltip (conditions not yet met shown in red), execute button.
  - Decisions grouped into Collapsible sections: Diplomacy, Intrigue, Stewardship, etc.
  - Enabled/disabled state based on world conditions. Disabled decisions shown with `theme.disabled` color.
  - Clicking "Execute" dispatches a callback key and triggers the action in the simulation.
  - Test: build decision panel with 2 categories and 3 decisions, assert collapsible sections contain the expected decision buttons.

### Rendering & Text

- **UI-D06** — DrawList line primitives. Add `lines: Vec<LineCommand>` to DrawList and a line-rendering shader. Needed for family trees, tech trees, graph connections. Blocks: UI-D02.
  - Test: emit 3 LineCommands, assert all 3 appear in the DrawList with correct start/end coordinates.

- **UI-D11** — Text formatting DSL. Inline markup for styled text spans: `#high`, `#low`, `#P` (positive), `#N` (negative), `#bold`, `#size:18`. Parse markup into `Vec<TextSpan>` for `Widget::RichText`. CK3 uses this extensively for tooltip and event text. Enables data-driven text styling without code changes per string. Ready — semantic colors (UI-700) done.
  - Test: parse `"#P;+5 #N;-3 normal"` into 3 spans with correct colors.

- **UI-D12** — Glow/shadow text effects. Add optional `glow_color: Option<[f32; 4]>` to `TextCommand` and `TextSpan`. Render as a second text pass with offset and blur (or pre-multiplied alpha halo in the fragment shader). CK3 uses 4 glow tiers (none/weak/medium/strong) for emphasis hierarchy on dark backgrounds.
  - Test: emit a TextCommand with `glow_color`, assert it produces extra vertices in the draw pass.

- **UI-D13** — Fourth font size tier (subheader). Add `font_subheader_size: f32` (14px) to Theme, between body (12) and header (16). Useful for section headings that don't need full header treatment. CK3 has Small(15) filling this role.
  - Test: assert `font_subheader_size` is between `font_body_size` and `font_header_size`.

- **UI-D14** — Light-background text overrides. CK3 systematically remaps all semantic text colors to dark-on-light variants when rendering on parchment/letter backgrounds. Add `TextOverrides` struct with color remapping table. Apply via a `text_overrides: Option<TextOverrides>` field on Panel or a context parameter. Needs: a use case (letter event UI, parchment dialogs).
  - Test: create a `TextOverrides` that maps `text_light` to `text_low`, assert Label inside overridden Panel uses the remapped color.

- **UI-D20** — Status-colored panel backgrounds. Apply `Theme::bg_status_good/bad/mixed` as panel background tints for at-a-glance status in dense data views. Theme colors added (UI-701), but no screens use them yet. When a screen needs colored row/cell backgrounds (e.g., character list health column, combat outcome panels), use these tints as `bg_color` on inner panel widgets. Defer until a concrete screen needs visual status scanning beyond text color.
  - Test: build a panel with `bg_status_bad`, draw, assert panel bg_color matches theme value.

### Tooltips

- **UI-D07** — Tooltip shortcut display. Show keyboard shortcut text at tooltip bottom-right (CK3 pattern). Add optional `shortcut: Option<String>` to `TooltipContent`. When present, render right-aligned label below content. Wire to `KeyBindings::format_binding()` at tooltip creation sites. Needs: more keybindings to be worth discovering.
  - Test: create tooltip with shortcut "Ctrl+C", assert tooltip tree contains a right-aligned label with that text.

- **UI-D08** — Nested tooltip edge-relative positioning. Position nested tooltips relative to parent tooltip rect edge instead of cursor. Use `tooltip_stack.last()` to get parent rect, place nested tooltip at `parent_rect.right + offset_x`. Guarantees no overlap between tooltip levels regardless of cursor position within parent.
  - Test: show nested tooltip, assert nested tooltip rect does not overlap parent tooltip rect.

### Layout & Widgets

- **UI-D17** — Grid layout widget. Add `Widget::Grid { col_width, row_height, columns, gap }` variant. Children placed left-to-right, wrapping to next row every `columns` items. CK3's `fixedgridbox` with `addcolumn`/`addrow`/`datamodel_wrap`. Needed for: trait displays on character panels, skill grids, inventory views, any tiled/icon layout. Defer until a concrete screen requires it.
  - Test: insert 7 children into a 3-column grid, assert items wrap to 3 rows (3+3+1), assert child rects have correct x/y positions.

- **UI-D18** — Standardized sort/filter list header. Reusable `FilterableList` builder pattern with integrated sort toggles and filter dropdown in the list header. CK3's `hbox_list_sort_buttons` + `window_character_filter` pattern. Needed when entity counts exceed ~200 and search alone is insufficient. Defer until scale demands it.
  - Test: build a FilterableList with 3 sort columns, click a sort header, assert sort callback dispatched with correct column index.

- **UI-502** — Drag-and-drop support. CK3 barely uses this; low priority.
  - Current drag infrastructure in `src/ui/input.rs` tracks `captured`, `press_origin`, `dragging` — but only for scrollbar thumb dragging.
  - Generalize: add `pub draggable: bool` and `pub drop_target: bool` fields to `WidgetNode`.
  - When a draggable widget is dragged past the threshold: create a ghost overlay (semi-transparent copy of the widget) that follows the cursor. Store `drag_payload: Option<String>` on `UiState`.
  - On mouse release: hit-test for the drop target under the cursor. If a `drop_target` widget is found, dispatch `UiEvent::Drop { payload: String }`.
  - Use case: reordering pinned characters in the outliner, moving items between inventory slots (future SIM-011).
  - Test: start drag on draggable widget, move to drop target, release, assert Drop event is dispatched with correct payload.

### Input & Interaction

- **UI-D10** — Per-widget focus policy. Add `focusable: bool` field to `WidgetNode` (default false). Set true for Button, ScrollList, and future EditBox. Update `collect_focusable` to check the flag instead of matching widget type. Needed when text inputs or custom focusable widgets are added.
  - Test: insert a Label with `focusable = true`, assert it appears in `focusable_widgets()`.

- **UI-D21** — Window dragging. Allow floating panels to be repositioned by dragging their header. Detect drag on panel header → update `Position::Fixed` with delta → re-layout. Drag infrastructure and window frame builders both exist. Implement when floating dialog UX is needed.
  - Test: simulate drag on a movable panel header, assert panel position updated by drag delta.

- **UI-D23** — Widget-contextual keyboard shortcuts. Allow focused widget type to intercept keys before global dispatch. When a text input is focused, ESC cancels editing instead of closing the panel. When a settings panel with unsaved changes is focused, ESC prompts "discard changes?" instead of closing. Needs: text input widgets with mutable state.
  - Test: focus a text input, press ESC, assert ESC consumed by text input and not dispatched globally.

### Animation

- **UI-D15** — Inspector panel hide animation. The inspector currently vanishes instantly on close (ESC or deselect). Add a reverse slide-out animation (EaseIn, 150ms) before removal. Requires either: (a) caching the last `InspectorInfo` so the inspector can keep rendering during the hide animation after `selected_entity` is set to None, or (b) introducing an `inspector_closing` state that preserves the entity selection until the animation completes. Approach (a) is simpler but stales data; approach (b) complicates the ESC chain priority logic. Needs: retained tree (UI-500) would make this trivial since the widget survives without rebuilding.
  - Test: deselect entity, assert inspector slide animation starts with target 1.0 (off-screen right), assert widget removed after animation duration.

- **UI-D16** — Animation state machine (multi-step chaining). CK3 uses `next = "state_name"` to create multi-step animation sequences (e.g., bounce: scale up → overshoot → settle; attention flash: bright → dim → bright → fade). Our `start_looping()` covers the main use case (two-state ping-pong for glow/pulse), but doesn't support: asymmetric timing per leg, 3+ state sequences, or one-shot chains (A→B→C→done). Add when effects like notification bounce (1.35s 3-stage size 72→88→72) or staggered multi-step fades are needed.
  - Test: define a 3-state chain A→B→C, assert values traverse all three segments in order, assert animation completes after total duration.

- **UI-D24** — Cubic bezier easing curves. Add `Easing::CubicBezier(f32, f32, f32, f32)` that evaluates an arbitrary cubic bezier curve (same parameterization as CSS `cubic-bezier()`). CK3's default curve is `{0.25, 0.1, 0.25, 1}` (CSS `ease`); also uses custom curves like `{0, 0.9, 1, 0.4}` and overshoot curves `{0.43, 0, 0.2, 2.2}`. Our existing EaseIn/EaseOut/EaseInOut are fixed cubics that cover common cases. Add when a specific animation needs a curve that the fixed variants can't match.
  - Test: evaluate CubicBezier(0.25, 0.1, 0.25, 1) at t=0.5, assert result matches CSS ease reference value.

- **UI-D27** — Modal show/hide fade animation. Modals appear/disappear instantly. CK3 fades in over 0.25s. Use Animator to tween dim layer alpha (0→0.5) and content alpha (0→1.0) on push, reverse on pop. Requires deferring widget removal until fade-out completes (same pattern as PanelManager::close_animated). Needs: retained tree (UI-500) would simplify but not required.
  - Test: push modal, assert dim layer alpha starts at 0.0 and reaches target after animation duration.

### System Integration

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
- **GROW-004** — Revisit D14 (modals as panels merge). Trigger: ModalStack and PanelManager start diverging in behavior (different close semantics, different animation patterns, or duplicate logic appearing in both). Current ModalStack is clean at ~80 lines; merge is principled but margin is narrow (72% confidence). Requires P1S6 (PanelKind enum) from `ui-architecture-migration.md` Phase 1 to be complete first. See `.workflow/ui-architecture-patterns.md` Pass 5.
- **GROW-005** — Revisit D1 (UiContext sub-struct boundaries). Trigger: UiContext sub-fields feel unnatural during implementation, or a new piece of UI state doesn't fit cleanly into any existing sub-struct. Evaluate after `ui-architecture-migration.md` Phase 2 (P2S1–P2S5) is complete and has been used for at least one feature cycle. The sim layer's World uses `BodyTables`/`MindTables`/`GisTables` groupings; if UI state proves more heterogeneous than sim state, the sub-struct approach may need adjustment. 75% confidence. See `.workflow/ui-architecture-patterns.md` Pass 5.
- **GROW-006** — Revisit D15 (scroll key type). Trigger: `ui-architecture-migration.md` Phase 1 (P1S6, PanelKind enum) is complete. Scroll keys should match panel keys for consistency — migrate from `HashMap<String, f32>` to `HashMap<ScrollKey, f32>` where `ScrollKey` is either `PanelKind` or a new enum that covers both panel and non-panel scrollables. Depends on Phase 2 (P2S2, scroll decoupling) being done. 75% confidence. See `.workflow/ui-architecture-patterns.md` Pass 5.
