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

Foundation (cosmic-text migration) is complete. Remaining work organized into 5 tiers for incremental construction. Each tier produces visible, testable output. A growing **UI-DEMO** showcase verifies everything works together.

### How to read this section

**Tiers** are milestone groups, not hard gates. A task can start as soon as its per-task `Needs:` deps are met — you don't have to wait for an entire tier to finish. Tiers define checkpoints: when all tasks in a tier are done, the UI-DEMO milestone for that tier should work. Tasks marked **||** have zero mutual dependencies and can be built simultaneously.

**Design decisions** are listed at the top with recommended defaults (marked with >>). Review and override before building the tier that needs them. Defaults are chosen to unblock progress — all can be revised later without architectural rework.

**"Available after this tier"** lists what's newly usable in code once the tier's tasks are complete.

### Overview

| Tier | Tasks | Milestone |
|------|-------|-----------|
| 1 — Foundation | COMPLETE | Colored text + panel backgrounds + widget tree |
| 2 — Styled Panels | COMPLETE | Theme + mouse input |
| 3 — Full Widget Set | COMPLETE | Rich text + scroll list + tooltips |
| 4 — Game Integration | 2 (I01d + I02) | Real game UI replaces string rendering |
| 5 — Polish | 2 | Animation + keyboard shortcuts |
| DEMO | 1 | Growing showcase, verifies each tier |

5 tasks remaining. Ordering governed by per-task `Needs:` lines, not tier boundaries.

### Design Decisions

Resolve before the tier that needs them. Recommended defaults marked with >>.

**DD-1 — Widget architecture** (resolve before Tier 1)
- **Layout model**: >> Simple box model (fixed position + percentage + padding/margin). CK3 chrome is fixed-position panels; flexbox adds complexity not needed here. Upgrade path: add flex properties later if needed. Alternatives: flexbox-like (most expressive, most complex), constraint-based (anchor edges to parent/sibling).
- **Widget identity**: >> Flat enum (`Widget::Panel { .. } | Widget::Label { .. } | Widget::Button { .. }`). Aligns with project's no-trait-objects style. Closed set is fine — we know the widget types. Alternatives: `Box<dyn Widget>` trait objects (open set, dynamic dispatch), ECS-style component tables (aligns with sim architecture but overkill for UI).
- **Tree storage**: >> Arena (slotmap). Cache-friendly, O(1) lookup by WidgetId, avoids recursive borrow issues. Alternative: recursive `Vec<Box<dyn Widget>>` children.

**DD-2 — Visual style** (resolve before Tier 2)
- **Color palette**: >> CK3-derived: parchment bg `#D4B896`, gold accent `#C8A850`, text white `#F0E6D2`, text dark `#3C2A1A`, danger red `#C04040`, disabled grey `#808080`.
- **Font roster**: >> Libertinus Mono (data/terminal), Libertinus Serif (body/headers). Two fonts. Add a display face for headers later if needed.
- **Font sizes**: >> Fixed set: 9pt (data), 12pt (body), 16pt (headers).
- **Borders**: >> Shader-generated first (gold stroke + inner shadow in fragment shader). No external asset dependency. Upgrade to textured 9-slice when art assets exist. Alternative: texture-based 9-slice (more authentic, requires border PNGs).
- **Art pipeline**: >> Procedural shader generation initially. No external assets needed. When art is available, swap to hand-drawn parchment textures (most appropriate for historical Paris) or CC asset packs (Kenney UI, OpenGameArt medieval sets).

**DD-3 — Multi-font atlas** (resolve before Tier 2)
- **Atlas strategy**: >> Single shared atlas, key = `(fontdb::ID, u16, u32)` (font_id, size_bits, glyph_id). One texture, one bind group, one draw call. Alternative: separate atlas textures per font (separate bind groups, multiple draw calls — simpler but more draw calls).
- **cosmic-text integration**: Add font files to `fontdb::Database`, select via `Attrs::new().family(Family::Name("Libertinus Serif"))` per span. Shaping side is free.

**DD-4 — Rich text** (resolve before Tier 3)
- **Markup format**: >> By-widget-type initially (Labels = white, Headers = gold, Warnings = red). No inline markup parser needed. Upgrade to structured spans (`Vec<TextSpan>`) when mixed styles needed within a single widget. Alternative: inline markup like `{bold}text{/bold}` (parsed at render time).
- **Inline icons**: >> Defer. Not needed for MVP. When added, use PUA codepoints (U+E000+) registered as synthetic glyphs in the atlas — cleaner than post-layout insertion and preserves line-wrapping. Alternative: post-layout pass inserting icon quads between text runs (more flexible, breaks wrapping around icons).

**DD-5 — Game panels** (resolve before Tier 4)
- **Panel inventory (MVP)**: >> (a) status bar (top — tick count, population, paused state), (b) hover tooltip (cursor over map tile — terrain, entities, building info), (c) event log (bottom — scrollable recent events), (d) entity inspector (side panel on entity click — name, occupation, needs, inventory). Additional candidates for later: building inspector, district overview, mini-map.
- **Data binding**: >> Full rebuild every frame. ~10 panels x ~50 widgets = trivial cost. Upgrade to poll+diff when profiling says so. Alternative: event-driven (World pushes change notifications — most complex).
- **Panel lifecycle**: >> Chrome panels permanent (status bar, event log — hidden/shown). Inspector/tooltip created on demand, destroyed on close/defocus.

---

### Tier 1 — Foundation — COMPLETE

All three tasks done: UI-P01 (per-vertex color), UI-P03 (panel renderer), UI-W01 (widget tree).

**Available:**
- Per-glyph color in the text pipeline (gold, white, red, grey in one draw call)
- Panel quad renderer (rectangles with shader-generated borders)
- Widget tree (slotmap arena, flat enum, box model layout, DrawList output)
- Tier 1 demo: parchment panel + 3 colored labels rendered via widget tree

---

### Tier 2 — Styled Panels — COMPLETE

All tasks done: UI-P02 (multi-font atlas), UI-R02 (theme), UI-W02 (input routing).

**Available:**
- Multiple fonts (serif + mono) rendered from a shared atlas
- `Theme` struct centralizing all colors, fonts, and spacing constants
- Mouse hover and click dispatched to widgets (buttons respond to clicks)
- Focus management (Tab to cycle, keyboard events to focused widget)
- Mouse capture for drag operations, 4px threshold
- UI events consumed before game input

---

### Tier 3 — Full Widget Set — COMPLETE

All tasks done: UI-R01 (rich text), UI-W03 (scroll list), UI-W04 (tooltip system).

**Available:**
- Rich text with mixed styles (bold, italic, color) in one text block via cosmic-text spans
- Scrollable lists with virtual scrolling (hundreds of items, only visible ones measured/drawn)
- Nested CK3-style tooltips with hover delay and recursive dismissal
- Every widget type needed for game UI panels
- Tier 3 demo: ScrollList with 100 items, rich text block, 3-level nested tooltip chain

---

### Tier 4 — Game Integration

Sub-panels ordered simplest to most complex — each proves more of the pipeline. I01a only needs W01+R02 and can start as early as mid-Tier 2. I02 only needs P01+P03 and can start right after Tier 1. Per-task `Needs:` lines are the real gates.

**Available after this tier:**
- Real game panels replacing current string-based `render_status()`/`render_hover_info()`/`render_recent_events()`
- Map overlays: tile highlight, movement path visualization
- Complete player-facing UI
- `UiState` struct on `App` (not on `World` — UI is not simulation state)

**UI-DEMO after Tier 4:** Retired — the game itself is the demo. The showcase code remains as a developer reference panel (F11), but the live game UI with status bar, inspector, event log, tooltip, and map overlays is the real verification.

- **UI-I01d** — Entity inspector. Needs: UI-W01, UI-W02, UI-W03, UI-R01.
  - Side panel on entity click: name, occupation, needs bars, inventory list.
  - Most complex panel — tests creation/destruction lifecycle, rich text (R01), multiple widget types composed together.
  - Created on entity click, destroyed on close button or Escape (per DD-5).
  - Rich text for entity description (name in gold, occupation in serif, stats in mono).

- **UI-I02 ||** — Map overlay integration. Needs: UI-P01, UI-P03. (Can build in parallel with I01 sub-panels.)
  - UI elements that composite with the world-space map: tile selection highlight, movement path lines, area-of-effect indicators, entity health bars above map glyphs.
  - Render order: (1) map glyphs (`prepare_map`), (2) map overlays (highlights, paths — same coordinate space as map), (3) UI panels (screen-space, on top of everything), (4) tooltips (above panels).
  - Tile highlight: colored semi-transparent quad drawn at the hovered tile's screen position. Uses the panel renderer (a single untextured colored quad) or a dedicated overlay pass.
  - Scissor rects: UI panels that overlap the map viewport clip the map render beneath them. Requires `set_scissor_rect()` on the render pass, or stencil buffer, or simply rendering panels as opaque backgrounds that occlude the map.
  - Path visualization: line segments between tile centers for A* paths. Could be a simple line renderer (2 vertices per segment) or a series of highlight quads on each tile in the path.

---

### Tier 5 — Polish (not on critical path)

Enhancements. Buildable any time after their dependencies are met.

- **UI-W05** — Animation system. Needs: UI-W01. Enhancement.
  - Time-driven interpolation for widget properties: position (slide in/out), opacity (fade), color (hover highlight), size (expand/collapse).
  - Core: `AnimationState` stored per-widget. `animate(property, from, to, duration, easing)`. Ticks on wall-clock delta (from `winit` `Instant`, not sim tick — UI animation is always real-time).
  - Easing functions: linear, ease-in-out (cubic), ease-out (decel). Small enum, not a plugin system.
  - Hover highlight: `Button` lerps background color/opacity on hover enter/leave (~200ms). Panel transitions: slide from screen edge on open/close. Tooltip fade-in ~150ms.
  - Minimal scope: animate `f32` values only. No keyframe chains or timeline editor.
  - Hover-driven animations need UI-W02 (input events trigger them), but the animation tick itself is independent.

- **UI-I03** — Keyboard shortcut system. Needs: UI-W02. Enhancement.
  - Global keybindings processed before widget focus dispatch. Configurable map: `HashMap<KeyCombo, Action>` where `KeyCombo` is modifier flags + keycode, `Action` is an enum (PauseSim, TogglePanel(PanelId), SpeedUp, SpeedDown, etc.).
  - Default bindings: Space = pause, Escape = close topmost panel/tooltip, 1-5 = sim speed, Tab = cycle panels.
  - Displayed in tooltips: "Pause (Space)" — keybinding text sourced from the map, not hardcoded in UI strings.

---

### UI-DEMO — Widget Showcase

A persistent developer reference panel (toggled with F11 or `--ui-demo` flag) that renders every available widget and style. Grows with each tier. Serves as:

- **Visual regression test** — run after any rendering change to verify nothing broke
- **Developer widget reference** — see every available widget type and its API in action
- **Style guide** — all theme colors, fonts, sizes, and border styles in one view

Implemented as a function in `src/ui/demo.rs` that builds a widget tree demonstrating all available elements. Called in the main loop when demo mode is active, rendered alongside (not replacing) the normal game view.

Scope by tier:
- **After Tier 1**: 3 colored text labels (gold, white, red) inside a bordered panel. Verifies P01 color, P03 panel, W01 layout.
- **After Tier 2**: Serif header (16pt) + mono data (9pt) + themed button with hover highlight. Verifies P02 multi-font, R02 theme, W02 input.
- **After Tier 3**: ScrollList with 100 items + rich text block with mixed fonts/colors + nested tooltip chain (3 levels). Verifies R01, W03, W04.
- **After Tier 4**: Live data panel showing a real entity's stats pulled from World. Verifies I01 data binding pipeline.
- **After Tier 5**: Animated panel slide-in + button hover fade + keybinding labels on buttons. Verifies W05, I03.

## Deferred

- **UI-D01** — egui dev tools overlay. Add `egui-wgpu` + `egui-winit`. Second render pass after game UI. Entity inspector, world state browser, system performance view. Toggle with a key (F12). Debug-only layer, not player-facing. Independent of the custom widget pipeline — can be added at any point.

## Pending (threshold not yet met)

- **GROW-002** — Phase function grouping. Trigger: >30 system calls.
- **GROW-003** — System dependency analyzer. Trigger: >15 system files.
