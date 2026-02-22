# Scale-Up Architecture

Reference for 19th-century Paris simulation. Read this before implementing any SCALE task.

## Target Numbers

- Map: 6,309 x 4,753 tiles (~30M tiles). Chunk: 64×64 (4,096 tiles). ~99 x 75 = ~7,400 chunks total.
- Population: ~1M modeled. ~4K active (full sim), ~50K nearby (simplified), ~950K statistical (aggregate).
- Tick budget: 10ms (100 ticks/sec).

## Simulation LOD

Three concentric zones around camera. This is the core scaling strategy — you simulate 4K entities, not 1M.

**Active** (~150 tile radius, ~70K tiles, ~4K entities)
All current systems run as-is. Full AI, A* pathfinding, combat, events.

**Nearby** (~500 tile radius, ~800K tiles, ~50K entities)
Real Entity values but simplified processing. Needs tick (hunger/fatigue), direct vector movement (no pathfinding), no combat. Skip `run_decisions`, `run_combat`.

**Statistical** (rest of city, ~29M tiles, ~950K modeled)
No individual entities. District-level aggregates ticked with equations. Population count, avg needs, death/birth rates, resource flows.

Zone derived from entity position vs camera position. Recomputed each tick or on camera move.

## Chunked TileMap

```rust
struct ChunkCoord { cx: i32, cy: i32 }  // cx = x / 64, cy = y / 64

struct TileMap {
    chunks: Vec<Chunk>,      // flat Vec, indexed by cy * chunks_x + cx
    chunks_x: usize,         // number of chunks in X direction
    width: usize,            // total tiles
    height: usize,           // total tiles
}
```

Tile accessors keep same signatures, route through flat Vec indexing internally (no hashing). Binary serialization: WULF v2 header (40 bytes: magic, version, dimensions, generation UUID) + zstd-compressed chunks in row-major order. Temperature not serialized — `initialize_temperatures()` sets each tile to its `terrain.target_temperature()` after load and marks all chunks `at_equilibrium`. See "Chunk (full definition)" section for per-tile ID layers.

Chunk access methods: `chunk_at(cx, cy)` / `chunk_at_mut(cx, cy)` for O(1) indexed access, `visible_chunk_range(cam_x, cam_y, vp_w, vp_h) -> ChunkRange` for viewport culling.

## Building Registry

```rust
struct BuildingId(u32);  // 1-based sequential index into Vec (0 = no building in tile arrays)

struct BuildingData {
    id: BuildingId,
    identif: u32,            // original Identif from BATI.shp (NOT unique per record)
    quartier: String,        // neighborhood name (36 values)
    superficie: f32,         // footprint area in m²
    bati: u8,                // 1=built area, 2=non-built open space, 3=minor feature
    nom_bati: Option<String>,// name if notable (146 buildings)
    num_ilot: String,        // block number
    perimetre: f32,          // perimeter in meters (PERIMETRE field)
    geox: f64,               // centroid X, Lambert projection (GEOX field)
    geoy: f64,               // centroid Y, Lambert projection (GEOY field)
    date_coyec: Option<String>, // survey date (DATE_COYEC field)
    floor_count: u8,         // estimated: <50m²=2, 50-150=3-4, 150-400=4-5, >400=5-6
    tiles: Vec<(i32, i32)>,  // all tiles belonging to this building
    addresses: Vec<Address>, // joined from address shapefile (time-invariant)
    occupants_by_year: HashMap<u16, Vec<Occupant>>, // keyed by SoDUCo snapshot year
}

struct Address {
    street: String,       // NOM_ENTIER from address shapefile
    number: String,       // NUM_VOIES
}

struct Occupant {
    name: String,         // persons field
    activity: String,     // activities field (French occupation)
    naics: String,        // NAICS industry category
}

struct BuildingRegistry {
    buildings: Vec<BuildingData>,            // indexed by BuildingId.0 - 1
    identif_index: HashMap<u32, Vec<BuildingId>>, // cadastral parcel → buildings
}
```

Note: BATI.shp has 40,350 records but only 17,155 unique Identif values. Multiple BATI.shp records share the same Identif (cadastral parcel ID). BATI=1 records are built structures; BATI=2 are adjacent open spaces (courtyards, gardens). Only BATI=1 entries enter the BuildingRegistry. After filtering, ~21,040 buildings from ~17,155 unique Identif values.

## Block Registry

```rust
struct BlockId(u16);   // sequential index assigned during loading

struct BlockData {
    id_ilots: String,         // original ID from Vasserot_Ilots.shp, e.g. "860IL74"
    quartier: String,         // neighborhood name
    aire: f32,                // block area in m²
    ilots_vass: String,       // Vasserot's original block numbering (ILOTS_VASS field)
    buildings: Vec<BuildingId>, // buildings within this block
}

struct BlockRegistry {
    blocks: HashMap<BlockId, BlockData>,
}
```

## Chunk (full definition)

```rust
struct Chunk {
    terrain: [Terrain; 4096],     // #[repr(u8)], serialized
    temperature: [f32; 4096],     // runtime-only, NOT serialized (defaults to 16.0)
    building_id: [u32; 4096],     // 0 = no building, else BuildingId value. Serialized as LE u32.
    block_id: [u16; 4096],        // 0 = no block, else BlockId value. Serialized as LE u16.
    quartier_id: [u8; 4096],      // 0 = unassigned, 1-36 = quartier index. Serialized.
    dirty: bool,
    last_tick: Tick,
    at_equilibrium: bool,   // runtime-only, NOT serialized. Cleared by set_terrain().
}
// Binary size per chunk: 4096 + 4096×4 + 4096×2 + 4096 = 32 KB
```

All per-tile ID layers populated during GIS loading (SCALE-A03) and are write-once (static city). Lookup chains:
- Tile → `BuildingId` → `buildings[id-1]` → quartier, floor count, addresses, `occupants_by_year[active_year]` → NAICS. O(1) Vec index.
- Tile → `BlockId` → `BlockData` → block ID string, area, member buildings.
- Tile → `quartier_id` → district name. Covers all tiles, not just buildings (road tiles get quartier from nearest block polygon or Voronoi fill).

## Street Registry

```rust
struct StreetId(u16);

struct StreetData {
    name: String,                    // NOM_ENTIER from address shapefile
    buildings: Vec<BuildingId>,      // buildings addressed on this street
}

struct StreetRegistry {
    streets: HashMap<StreetId, StreetData>,
    name_to_id: HashMap<String, StreetId>,
}
```

Built during address loading (SCALE-A07). Streets are not tile-mapped (no explicit street geometry in data — streets are negative space between blocks). The registry provides name lookups: given a building, find its street name(s); given a street name, find all buildings on it.

## Terrain Types

```rust
enum Terrain {
    Road,       // walkable — streets, alleys, open ground outside block polygons
    Wall,       // blocked — building perimeter (building tile with a non-building cardinal neighbor)
    Floor,      // walkable — building interior (building tile surrounded by building tiles)
    Door,       // walkable — building entrance (wall tile adjacent to both floor and road)
    Courtyard,  // walkable — enclosed open space within blocks (inside block polygon, outside buildings)
    Garden,     // walkable — parks, green space (24 buildings named "parc ou jardin")
    Water,      // blocked — Seine (hardcoded band, not in GIS data)
    Bridge,     // walkable — over water (hardcoded at known historical locations)
}
```

Temperature targets: Water 10°, Bridge 12°, Garden 15°, Wall 15°, Road 16°, Courtyard 16°, Door 17°, Floor 18°.

## Spatial Index

```rust
struct SpatialGrid {
    cells: HashMap<(i32, i32), SmallVec<[Entity; 4]>>,
}
```

Rebuilt from `world.positions` at tick start. O(1) same-tile lookup. Area queries iterate cell range. Used by combat, eating, decision target selection.

## Hierarchical Pathfinding (HPA*)

Chunk borders → entry/exit nodes. Precompute intra-chunk shortest paths between border nodes. Long-range: A* on chunk graph (~100 nodes cross-city). Short-range: regular A* within current + adjacent chunks (8K limit fine). Rebuild only on terrain change (never for static city).

## Registry Ownership

All registries live on `World` alongside `tiles`:
- `world.buildings: BuildingRegistry` — populated by A03, occupants added by A07
- `world.blocks: BlockRegistry` — populated by A03
- `world.streets: StreetRegistry` — populated by A07
- `world.active_year: u16` — selects which SoDUCo snapshot to use (default 1845). Indexes into `occupants_by_year` on BuildingData. 16 available years: 1829, 1833, 1839, 1842, 1845, 1850, 1855, 1860, 1864, 1871, 1875, 1880, 1885, 1896, 1901, 1907.

## District Aggregates

```rust
struct District {
    id: u32,
    quartier: String,       // neighborhood name, matches QUARTIER field
    bounds: Rect,
    population: u32,
    population_by_type: HashMap<String, u32>,
    avg_hunger: f32,
    avg_health: f32,
    death_rate: f32,
    resource_stockpile: f32,
}
```

`run_district_stats` (SCALE-C04): ticks all statistical districts with equations. Population flows between adjacent districts based on resource gradients.

## Hydration / Dehydration

**Hydrate** (statistical → active): Spawn entities from district distribution. Position from building footprints. Stats sampled from district averages ± noise. Batch ~100/tick to avoid pop-in.

**Dehydrate** (active → statistical): Collapse entity stats back into district averages. Remove from property tables. Nearby zone buffers the transition — entities simplify for ~200 ticks before collapsing.

## UI Layer (CK3-Style Widget System)

Retained-mode widget layer on wgpu. cosmic-text for shaping/layout, FreeType for glyph rasterization, custom widget tree for CK3 parchment+gold aesthetic. Lives on `App` in main.rs, not on `World` — UI is not simulation state.

### Architecture

- **WidgetTree** — slotmap arena of `WidgetNode`. Multiple roots (panels, tooltips). Rebuilt every frame (DD-5: full rebuild, ~50-100 widgets, trivial cost).
- **Widget** — flat enum: `Panel`, `Label`, `Button`, `RichText`, `ScrollList`. No trait objects (DD-1).
- **DrawList** — intermediate commands (`PanelCommand`, `TextCommand`, `RichTextCommand`). Decouples widget tree from GPU renderers. Tree emits commands; `PanelRenderer` and `FontRenderer` consume independently.
- **Theme** — single global `Theme` struct with DD-2 CK3 palette (parchment, gold, white, dark, red, grey), font sizes (9/12/16pt), spacing constants, animation durations.
- **UiState** — interaction state: hover, focus, pressed, captured, drag, tooltip stack. Lives on App.
- **Animator** — wall-clock f32 interpolation keyed by string. Easing: Linear, EaseOut, EaseInOut. Used for tooltip fade, inspector slide, button hover highlight, demo slide.
- **KeyBindings** — `HashMap<KeyCombo, Action>`. Defaults: Space=pause, Esc=close, F11=demo, 1-5=speed. Reverse map for display labels.

### Rendering Pipeline

1. `WidgetTree::draw()` emits `DrawList` (panels + texts + rich_texts)
2. Map overlays (hover/selection/path tile highlights) added directly to `PanelRenderer`
3. `PanelRenderer` consumes `PanelCommand`s → panel.wgsl (SDF borders + inner shadow)
4. `FontRenderer` consumes `TextCommand`s → `prepare_text_with_font()` (single style per call)
5. `FontRenderer` consumes `RichTextCommand`s → `prepare_rich_text()` (per-span color/font via cosmic-text)
6. Map text rendered via `prepare_map()` (mono font, codepoint-based cache)
7. Render order: map glyphs → map overlays → UI panels → UI text → tooltips (on top)

### Multi-Font Atlas

Single shared R8Unorm atlas (512×4096). Composite key: `(fontdb::ID, font_size_bits, glyph_id)`. Two fonts: Libertinus Mono (data/terminal) + Libertinus Serif (body/headers). On-demand FreeType rasterization with shelf-packing. Per-vertex color in `TextVertex` → sRGB-to-linear conversion in text.wgsl fragment shader.

### Input Dispatch Order

1. Global keybindings (UI-I03) — before widget focus
2. Widget focus dispatch (Tab, ScrollList nav, click)
3. Game keys (WASD camera, numpad movement, roguelike controls)

### Game Panels

| Panel | Builder | Lifecycle | Location |
|-------|---------|-----------|----------|
| Status bar | `build_status_bar()` | Permanent, rebuilt every frame | Top of screen |
| Hover tooltip | `build_hover_tooltip()` | Created on map hover, destroyed on leave | Cursor-anchored |
| Event log | `build_event_log()` | Permanent, rebuilt every frame | Bottom of screen |
| Entity inspector | `build_entity_inspector()` | Created on entity click, Esc closes | Right side |
| Widget showcase | `demo::build_demo()` | Toggled via F11 or `--ui-demo` | Left side |

### Data Extraction Pattern

Plain structs (`StatusBarInfo`, `HoverInfo`, `EventLogEntry`, `EntityInspectorInfo`, `DemoLiveData`) are collected from `World` in main.rs, then passed to builder functions. Zero references to `World` inside widget builders.

## Project Structure

```
CLAUDE.md
Cargo.toml
.workflow/
  backlog.md             # incomplete tasks only — delete when done
  checkpoint.md          # rolling state snapshot — overwritten, never appended
  architecture.md        # this file — rottable specs, data structures, file listing
data/
  creatures.kdl          # creature definitions (KDL)
  items.kdl              # item definitions (KDL)
  terrain.kdl            # terrain definitions (KDL)
  paris.tiles            # binary tile data (WULF v2, zstd-compressed chunks, generation UUID)
  paris.meta.bin         # building/block registries (bincode+zstd, WULM v1 header, generation UUID)
  paris.meta.ron         # same as above in human-readable RON (debug artifact, not loaded at runtime)
  paris.ron.zst          # intermediate GIS polygon data (zstd-compressed RON, fallback rasterization)
  utility.ron            # preprocessor utility data
src/
  main.rs                # phased main loop + wgpu renderer + UI integration
  lib.rs                 # crate root (sim-only: world, components, events, systems)
  world.rs               # World struct, spawn, despawn, validate
  events.rs              # Event enum + EventLog ring buffer
  components.rs          # property structs (Position, Hunger, etc.)
  tile_map.rs            # TileMap — chunked storage, accessors, WULF v2 binary ser/de (zstd+UUID)
  registry.rs            # BuildingRegistry, BlockRegistry, BuildingData, Address, Occupant
  loading.rs             # KDL parsing, entity spawning (small test map)
  loading_gis.rs         # GIS shapefile parsing, rasterization, binary load
  render.rs              # string-based debug rendering (map, status) + render_world_to_string
  font.rs                # FontRenderer — cosmic-text shaping, FreeType rasterization, multi-font wgpu atlas
  panel.rs               # PanelRenderer — colored quads with SDF borders + inner shadow
  text.wgsl              # text glyph shader — per-vertex color, sRGB-to-linear, alpha compositing
  panel.wgsl             # panel quad shader — SDF border + inner shadow from uniforms
  linebreak_table.rs     # Unicode line-break property table (generated, used by cosmic-text)
  rng.rs                 # deterministic seeded RNG wrapper
  ui/
    mod.rs               # WidgetTree, layout, draw, game panels (status bar, hover, event log, inspector)
    widget.rs            # Widget enum (Panel, Label, Button, RichText, ScrollList) + TooltipContent
    draw.rs              # DrawList, PanelCommand, TextCommand, RichTextCommand, FontFamily, TextSpan
    theme.rs             # Theme struct — DD-2 CK3 palette, font sizes, spacing, animation durations
    animation.rs         # Animator — wall-clock f32 interpolation with easing
    input.rs             # UiState — hit testing, focus, drag, mouse capture, tooltip system
    keybindings.rs       # KeyBindings — configurable shortcut map, KeyCombo, Action enum
    demo.rs              # Widget showcase panel (F11 toggle, --ui-demo flag)
  bin/
    preprocess.rs        # offline GIS → binary pipeline (generates tiles + bincode meta + RON debug)
    water_diag.rs        # diagnostic tool for water rasterization + bridge detection
  systems/
    mod.rs
    hunger.rs            # Phase 2: hunger increase
    fatigue.rs           # Phase 2: fatigue/tiredness
    temperature.rs       # Phase 1: tile heat diffusion
    decisions.rs         # Phase 3: AI target selection
    wander.rs            # Phase 4: movement
    eating.rs            # Phase 4: food consumption
    combat.rs            # Phase 4: fighting
    death.rs             # Phase 5: ALWAYS last
tests/
  invariants.rs          # property-based cross-system tests
  determinism.rs         # replay/seed tests
```

## What Does NOT Change

- Phase-ordered sequential main loop (add zone filtering, don't restructure)
- Collect-then-apply mutation pattern
- Deterministic RNG (`world.rng` only)
- `pending_deaths` + `run_death` always last
- EventLog ring buffer
- KDL data files (add district defs)
- One system per file
- `HashMap<Entity, T>` for active zone (profile before considering dense storage)
