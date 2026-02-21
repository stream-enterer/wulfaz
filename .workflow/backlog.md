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

- **SCALE-B01** — Spatial index. `HashMap<(i32,i32), SmallVec<[Entity; 4]>>` on World, rebuilt from positions each tick. Blocks: B02.

- **SCALE-B02** — Convert spatial queries. `run_combat`, `run_eating`, `run_decisions` target selection use spatial index, not full position scan. Needs: B01.

- **SCALE-B03** — GIS-aware entity spawning. Needs: A07, B05. **BLOCKED: design review required.**
  - The building registry (populated by A03 + A07) already knows each building's occupants, addresses, and NAICS categories. This task spawns actual entities from that data.
  - For known occupants (3.7% of population): spawn entity with real name, real occupation, at their building's floor tiles. Position from building's tile list in the registry.
  - For generated occupants (96.3%): see C05 for the procedural generation rules.
  - Single neighborhood first: filter to one QUARTIER (recommend "Arcis" — 825 buildings, dense, central, ~150m×300m).
  - The full data pipeline reference (address → building → people) is documented in SCALE-A07 and `~/Development/paris/PROJECT.md`.

- **SCALE-B04** — Increase A* node limit to 32K. Stopgap for larger-map pathing. Replaced by SCALE-D03.

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

## Phase UI — CK3-Style Game Interface

Goal: Retained-mode widget layer on wgpu for player-facing UI. cosmic-text for shaping/layout, FreeType for rasterization, custom widget tree for CK3 parchment+gold aesthetic.

Foundation (cosmic-text migration) is complete. Remaining steps below.

Dependency graph:
```
Parallel starts: UI-P01, UI-P03, UI-W01

UI-P01 → UI-P02 → UI-R01 ──────────────────────┐
                └──→ UI-R02 ────────────────────┤
UI-P03 ──────────→ UI-R02                       ↓
                                             UI-I01
UI-W01 → UI-W02 → UI-W03 ─────────────────────┘
  │          ├──→ UI-W04 ──────────────────────┘
  │          └──→ UI-I03
  └──→ UI-W05

UI-P01 + UI-P03 → UI-I02
```

### Rendering foundation

Items here modify the GPU pipeline. No widget logic.

- **UI-P01** — Per-vertex color attribute. Needs: nothing. Blocks: UI-P02, UI-R01, UI-I02.
  - Current `text.wgsl` uses a single `fg_color` uniform — every glyph in the frame is the same color. CK3-style UI needs gold headers, white body, red warnings, grey disabled text in one draw call.
  - Add `color: [f32; 4]` to `TextVertex` (16 bytes → 32 bytes per vertex).
  - Modify `text.wgsl`: read per-vertex color in `vs_main`, pass to `fs_main`, replace uniform `fg_color` with interpolated vertex color in the alpha compositing math.
  - Modify `build_vertices()` and `prepare_text_shaped()` to accept and propagate a color parameter.
  - Existing `begin_frame()` fg_color becomes the default; per-vertex color overrides when set.
  - Bind group layout, sampler, atlas — unchanged. Pipeline vertex layout changes (new attribute at location 2).

- **UI-P02** — Multi-font atlas support. Needs: UI-P01. Blocks: UI-R01, UI-R02. **BLOCKED: design review required.**
  - Current atlas and `FontRenderer` assume one font (Libertinus Mono) at one size. CK3-style needs at minimum: a serif for flavor/narrative text, monospace for data, possibly a display face for panel headers.
  - Decisions needed: (a) one shared atlas with mixed fonts keyed by `(font_id, glyph_id)`, or separate atlas textures per font (separate bind groups, multiple draw calls)? (b) Which fonts? Libertinus Serif is bundled alongside Libertinus Mono — use that, or source something else? (c) How many font sizes to support — fixed set (9pt data, 12pt body, 16pt header) or arbitrary?
  - cosmic-text already supports multiple fonts via fontdb — add font files to the Database, use `Attrs::new().family(Family::Name("Libertinus Serif"))` per span. The shaping side is free.
  - FreeType rasterization side: `rasterize_glyph_on_demand()` needs to handle multiple `ft_face` instances. Store `HashMap<fontdb::ID, freetype::Face>` instead of a single `ft_face`.
  - Atlas key changes from `u32` (glyph_id) to `(fontdb::ID, u16, u32)` (font_id, glyph_id, size_bits).

- **UI-P03** — 9-slice panel renderer. Needs: nothing (separate quad pipeline, not text pipeline). Blocks: UI-R02, UI-I02.
  - WGSL shader taking a panel texture + source rect + 9-slice margins as uniforms. Renders textured panels with fixed-size ornate corners and stretchable center/edges.
  - Rust-side: `PanelRenderer` struct with its own pipeline, vertex buffer, bind group. Separate from `FontRenderer` — panels are textured quads, not glyph quads.
  - Input: panel rect (screen position + size), texture region, margin insets. Output: 9 quads (4 corners, 4 edges, 1 center) with correct UVs.
  - Render order: panels draw BEFORE text (text renders on top of panel backgrounds).
  - Panel textures loaded from PNG/image files into a separate `Rgba8Unorm` texture (not the R8Unorm glyph atlas). Sampler can use linear filtering for panel textures.

### Widget system

Items here are pure Rust data structures and algorithms. No GPU changes. Widget `draw()` emits commands into a `DrawList` that renderers (from P01/P03) consume — the widget tree does not call wgpu directly.

- **UI-W01** — Widget tree core + layout model. Needs: nothing. Blocks: UI-W02, UI-W03, UI-W04, UI-W05, UI-I01. **BLOCKED: design review required.**
  - Retained-mode widget hierarchy. Base types: `Panel`, `Label`, `Button`.
  - Decisions needed:
    - **Layout model**: flexbox-like (main axis + cross axis + wrap) vs constraint-based (anchor edges to parent/sibling) vs simple box model (fixed/percentage + padding/margin). Flexbox is the most expressive but most complex. Simple box model may suffice for CK3-style fixed-chrome panels.
    - **Widget identity**: `Box<dyn Widget>` trait objects (flexible, dynamic dispatch) vs flat enum (`Widget::Panel { .. } | Widget::Label { .. }`) (faster, no vtable, but closed set) vs ECS-style where widgets are entities with component tables (aligns with project architecture but may be overkill for UI).
    - **Tree storage**: arena-allocated (indextree/slotmap) vs recursive `Vec<Box<dyn Widget>>` children. Arena is better for cache locality and avoids recursive borrow issues.
  - Core trait/interface:
    - `measure(constraints) -> Size` — compute intrinsic size given min/max constraints
    - `layout(allocated_rect)` — assign final position to self and children
    - `draw(draw_list: &mut DrawList)` — emit panel quads and text runs into a draw list
  - `DrawList`: intermediate representation. Contains `Vec<PanelCommand>` (rect + texture + 9-slice margins) and `Vec<TextCommand>` (string + position + color + font attrs). Consumed by `PanelRenderer` and `FontRenderer` during the render pass. Decouples widget logic from GPU.
  - Dirty-flagging: each widget has a `dirty: bool`. `layout()` only recurses into dirty subtrees. Text content changes and window resize set dirty on affected widgets.
  - Note: visually complete panels require UI-P03 (backgrounds) and UI-P01 (colored text), but the widget tree itself is renderer-agnostic and can be developed and tested with text-only output first.

- **UI-W02** — Input routing + hit testing. Needs: UI-W01. Blocks: UI-W03, UI-W04, UI-I03.
  - Mouse position → walk widget tree back-to-front → first widget whose rect contains cursor gets hover/click.
  - Event types: `Hover`, `Click(button)`, `DragStart`, `DragMove`, `DragEnd`, `Scroll(delta)`.
  - Focus management: `focused_widget: Option<WidgetId>`. Tab advances focus. Focused widget receives keyboard events.
  - Mouse capture: dragging a scrollbar or slider holds capture even when cursor leaves the widget rect. Release on mouse-up.
  - All UI input handling runs BEFORE the simulation tick in the main loop (reads `winit` events, updates `UiState`, consumes events so they don't reach the sim).

- **UI-W03** — ScrollList widget. Needs: UI-W01, UI-W02.
  - Scrollable content area: `content_height` measured from children, `scroll_offset: f32` tracks position.
  - Scrollbar: thin vertical bar rendered as a Panel quad. Draggable (uses mouse capture from UI-W02). Auto-hides when content fits.
  - Keyboard navigation: arrow keys move selection, Page Up/Down jump by visible height, Home/End go to extremes.
  - Virtual scrolling: only `measure()` + `draw()` children whose Y range intersects the visible viewport. Essential for entity lists (hundreds of items).
  - Scroll-to-item: `scroll_list.ensure_visible(child_index)` — smooth or instant scroll to bring a specific child into view.
  - Momentum/overscroll: optional, can defer. Functional without it.

- **UI-W04** — Tooltip system. Needs: UI-W01, UI-W02.
  - `TooltipStack: Vec<TooltipEntry>` in `UiState`. Each entry: content widget tree, anchor position, hover source widget ID.
  - Nested tooltips: when hovering a clickable/hoverable element inside tooltip N, push tooltip N+1. When cursor leaves tooltip N's rect AND is not inside tooltip N+1, pop N+1. Recursive dismissal — popping N also pops N+1..N+k.
  - Positioning: prefer below-right of cursor. If tooltip would clip screen edge, flip to above/left. Each nesting level offsets slightly to avoid total overlap.
  - Hover delay: ~300ms before showing (configurable). Instant show if another tooltip was recently visible (CK3 behavior — fast tooltip switching).
  - Each tooltip renders as a Panel (9-slice background from UI-P03) containing Label/RichText children. Z-order: tooltips always render above all other panels. Tooltip N+1 renders above tooltip N.

- **UI-W05** — Animation system. Needs: UI-W01. Enhancement, not on critical path.
  - Time-driven interpolation for widget properties: position (slide in/out), opacity (fade), color (hover highlight), size (expand/collapse).
  - Core: `AnimationState` stored per-widget. `animate(property, from, to, duration, easing)`. Ticks on wall-clock delta (from `winit` `Instant`, not sim tick — UI animation is always real-time).
  - Easing functions: linear, ease-in-out (cubic), ease-out (decel). Small enum, not a plugin system.
  - Hover highlight: `Button` lerps background color/opacity on hover enter/leave (~200ms). Panel transitions: slide from screen edge on open/close. Tooltip fade-in ~150ms.
  - Minimal scope: animate `f32` values only. No keyframe chains or timeline editor.
  - Hover-driven animations need UI-W02 (input events trigger them), but the animation tick itself is independent.

### Rich content

- **UI-R01** — Rich text rendering. Needs: UI-P01, UI-P02. **BLOCKED: design review required.**
  - Leverage cosmic-text `set_rich_text()` for mixed-style text: `&[(&str, Attrs)]` spans with different families, weights, colors per span.
  - Decisions needed:
    - **Markup format**: how does game data specify styled text? Options: (a) simple inline markup like `{bold}text{/bold}` or `[color=gold]text[/color]`, parsed into spans at render time; (b) structured data — `Vec<TextSpan>` with `enum Style { Bold, Italic, Color(Color), Icon(IconId) }` built programmatically; (c) no markup — all styling applied by the widget type (Labels are always white, Headers are always gold, etc.). Option (c) is simplest and may be sufficient initially.
    - **Inline icons**: CK3 puts trait/resource icons inline with wrapped text ("Your [gold_icon] 420 gold"). This requires either: synthetic glyph entries in the atlas for each icon (cosmic-text shapes them as normal glyphs via PUA codepoints U+E000+), or a post-layout pass that inserts icon quads at measured positions between text runs. The synthetic glyph approach is cleaner but requires registering icons as font glyphs. The post-layout approach is more flexible but breaks line wrapping around icons.
  - Per-vertex color from UI-P01 carries the color for each glyph. `prepare_text_shaped()` reads `glyph.color_opt` from cosmic-text's layout output and writes it into the vertex color attribute.

- **UI-R02** — Theme and visual style. Needs: UI-P02, UI-P03. **BLOCKED: design review required.**
  - Decisions needed:
    - **Art pipeline**: hand-drawn panel textures (PNGs, requires artist or asset source), procedural shader generation (panel borders as shader math — no external assets but limited aesthetic), or creative-commons asset pack (e.g., Kenney UI, OpenGameArt medieval sets). For a historical Paris sim, hand-drawn parchment/wood textures are most appropriate but highest effort.
    - **Color palette**: define the exact colors. CK3 reference: parchment bg `#D4B896`, gold accent `#C8A850`, text white `#F0E6D2`, text dark `#3C2A1A`, danger red `#C04040`, disabled grey `#808080`. Adapt or define custom.
    - **Font roster**: Libertinus Mono for data/terminal text, Libertinus Serif for narrative/flavor text, and what for headers? Libertinus display weights, or a separate face?
    - **9-slice vs shader borders**: texture-based 9-slice (more authentic, requires border textures) vs shader-generated borders (gold stroke + inner shadow computed in fragment shader — no texture dependency, but flat/geometric look).
  - Implementation: `Theme` struct holding color constants, font family names, panel texture IDs, margin/padding defaults. Passed to `draw()` so widgets don't hardcode colors. Single global theme initially (no runtime theme switching).

### Integration

- **UI-I01** — Game UI panels + data binding. Needs: UI-W01, UI-W02, UI-W03, UI-W04, UI-R01, UI-R02. **BLOCKED: design review required.**
  - This is a meta-task. Will be broken into per-panel sub-tasks during design review.
  - Decisions needed:
    - **Panel inventory**: which panels exist? Minimum viable set: (a) status bar (top — tick count, population, paused state), (b) entity inspector (side panel on entity click — name, occupation, needs, inventory), (c) event log (bottom — scrollable recent events), (d) hover tooltip (cursor over map tile — terrain, entities present, building info). What else? Building inspector? District overview? Mini-map?
    - **Data binding model**: how does the widget tree react to World changes? Options: (a) full rebuild every frame — simple, no caching, fine for small UI; (b) poll + diff — each panel's `update(world)` method checks relevant World fields and sets dirty if changed; (c) event-driven — World pushes change notifications that panels subscribe to. Option (b) is the pragmatic middle ground. Option (a) is acceptable initially.
    - **Panel lifecycle**: are panels always present (hidden/shown) or created/destroyed on demand? CK3 uses both — chrome panels are permanent, inspector panels are created on click and destroyed on close.
  - Replaces the current `render::render_status()` / `render::render_hover_info()` / `render::render_recent_events()` string-based rendering in `main.rs`. The `prepare_text()` calls are replaced by `ui.layout()` → `ui.draw(&mut draw_list)` → draw_list fed to renderers.
  - `UiState` struct on `App` (not on `World` — UI is not simulation state). Holds the widget tree root, tooltip stack, focused widget, panel visibility flags.

- **UI-I02** — Map overlay integration. Needs: UI-P01, UI-P03.
  - UI elements that composite with the world-space map: tile selection highlight, movement path lines, area-of-effect indicators, entity health bars above map glyphs.
  - Render order: (1) map glyphs (`prepare_map`), (2) map overlays (highlights, paths — same coordinate space as map), (3) UI panels (screen-space, on top of everything), (4) tooltips (above panels).
  - Tile highlight: colored semi-transparent quad drawn at the hovered tile's screen position. Uses the panel renderer (a single untextured colored quad) or a dedicated overlay pass.
  - Scissor rects: UI panels that overlap the map viewport clip the map render beneath them. Requires `set_scissor_rect()` on the render pass, or stencil buffer, or simply rendering panels as opaque backgrounds that occlude the map.
  - Path visualization: line segments between tile centers for A* paths. Could be a simple line renderer (2 vertices per segment) or a series of highlight quads on each tile in the path.

- **UI-I03** — Keyboard shortcut system. Needs: UI-W02. Enhancement, not on critical path.
  - Global keybindings processed before widget focus dispatch. Configurable map: `HashMap<KeyCombo, Action>` where `KeyCombo` is modifier flags + keycode, `Action` is an enum (PauseSim, TogglePanel(PanelId), SpeedUp, SpeedDown, etc.).
  - Default bindings: Space = pause, Escape = close topmost panel/tooltip, 1-5 = sim speed, Tab = cycle panels.
  - Displayed in tooltips: "Pause (Space)" — keybinding text sourced from the map, not hardcoded in UI strings.

## Deferred

- **UI-D01** — egui dev tools overlay. Add `egui-wgpu` + `egui-winit`. Second render pass after game UI. Entity inspector, world state browser, system performance view. Toggle with a key (F12). Debug-only layer, not player-facing. Independent of the custom widget pipeline — can be added at any point.

## Pending (threshold not yet met)

- **GROW-002** — Phase function grouping. Trigger: >30 system calls.
- **GROW-003** — System dependency analyzer. Trigger: >15 system files.
