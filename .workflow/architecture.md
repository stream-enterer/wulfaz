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

Tile accessors keep same signatures, route through flat Vec indexing internally (no hashing). Binary serialization: WULF header (32 bytes) + chunks in row-major order. Temperature not serialized (defaults to 16.0 on load). Cold chunks fast-forward on reload: `elapsed × drift_rate`, clamp to equilibrium. See "Chunk (full definition)" section for per-tile ID layers.

## Building Registry

```rust
struct BuildingId(u32);  // 1-based sequential index into Vec (0 = no building in tile arrays)

struct BuildingData {
    id: BuildingId,
    identif: u32,            // original Identif from BATI.shp (NOT unique per record)
    quartier: String,        // neighborhood name (36 values)
    superficie: f32,         // footprint area in m²
    bati: u8,                // 1=main, 2=annex, 3=market stall
    nom_bati: Option<String>,// name if notable (146 buildings)
    num_ilot: String,        // block number
    floor_count: u8,         // estimated: <50m²=2, 50-150=3-4, 150-400=4-5, >400=5-6
    tiles: Vec<(i32, i32)>,  // all tiles belonging to this building
    addresses: Vec<Address>, // joined from address shapefile
}

struct Address {
    street: String,       // NOM_ENTIER from address shapefile
    number: String,       // NUM_VOIES
    occupants: Vec<Occupant>, // joined from SoDUCo for chosen year
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

Note: BATI.shp has 40,350 records but only 17,155 unique Identif values. Multiple records per Identif represent separate structures on the same parcel (main + annex + stall). Vec storage ensures no data loss.

## Block Registry

```rust
struct BlockId(u16);   // sequential index assigned during loading

struct BlockData {
    id_ilots: String,         // original ID from Vasserot_Ilots.shp, e.g. "860IL74"
    quartier: String,         // neighborhood name
    aire: f32,                // block area in m²
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
}
// Binary size per chunk: 4096 + 4096×4 + 4096×2 + 4096 = 32 KB
```

All per-tile ID layers populated during GIS loading (SCALE-A03) and are write-once (static city). Lookup chains:
- Tile → `BuildingId` → `buildings[id-1]` → quartier, occupants, NAICS, floor count, addresses. O(1) Vec index.
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
- `world.buildings: BuildingRegistry` — populated by A03
- `world.blocks: BlockRegistry` — populated by A03
- `world.streets: StreetRegistry` — populated by A07

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

## What Does NOT Change

- Phase-ordered sequential main loop (add zone filtering, don't restructure)
- Collect-then-apply mutation pattern
- Deterministic RNG (`world.rng` only)
- `pending_deaths` + `run_death` always last
- EventLog ring buffer
- KDL data files (add district defs)
- One system per file
- `HashMap<Entity, T>` for active zone (profile before considering dense storage)
