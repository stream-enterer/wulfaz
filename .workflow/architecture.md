# Scale-Up Architecture

Reference for 19th-century Paris simulation. Read this before implementing any SCALE task.

## Target Numbers

- Map: ~17M tiles. Chunk: 64×64 (4,096 tiles). ~4,150 chunks total.
- Population: ~1M modeled. ~4K active (full sim), ~50K nearby (simplified), ~950K statistical (aggregate).
- Tick budget: 10ms (100 ticks/sec).

## Simulation LOD

Three concentric zones around camera. This is the core scaling strategy — you simulate 4K entities, not 1M.

**Active** (~150 tile radius, ~70K tiles, ~4K entities)
All current systems run as-is. Full AI, A* pathfinding, combat, events.

**Nearby** (~500 tile radius, ~800K tiles, ~50K entities)
Real Entity values but simplified processing. Needs tick (hunger/fatigue), direct vector movement (no pathfinding), no combat. Skip `run_decisions`, `run_combat`.

**Statistical** (rest of city, ~16M tiles, ~950K modeled)
No individual entities. District-level aggregates ticked with equations. Population count, avg needs, death/birth rates, resource flows.

Zone derived from entity position vs camera position. Recomputed each tick or on camera move.

## Chunked TileMap

```rust
struct ChunkCoord { cx: i32, cy: i32 }  // cx = x / 64, cy = y / 64

struct Chunk {
    terrain: [Terrain; 4096],
    temperature: [f32; 4096],
    dirty: bool,
    last_tick: Tick,
}

struct TileMap {
    chunks: HashMap<ChunkCoord, Chunk>,
    width_chunks: i32,
    height_chunks: i32,
}
```

Tile accessors keep same signatures, route through chunk lookup internally. Cold chunks fast-forward on reload: `elapsed × drift_rate`, clamp to equilibrium.

## Terrain Types

```rust
enum Terrain {
    Road,       // walkable — streets, alleys
    Building,   // walkable interior — entities spawn/live here
    Courtyard,  // walkable — enclosed open space within blocks
    Garden,     // walkable — parks, green space
    Water,      // blocked — Seine
    Bridge,     // walkable — over water
    Wall,       // blocked — fortifications
}
```

Temperature targets: Water 10°, Bridge 12°, Garden 15°, Wall 15°, Road 16°, Courtyard 16°, Building 18°.

## Spatial Index

```rust
struct SpatialGrid {
    cells: HashMap<(i32, i32), SmallVec<[Entity; 4]>>,
}
```

Rebuilt from `world.positions` at tick start (before Phase 3). O(1) same-tile lookup. Area queries iterate cell range. Used by combat, eating, decision target selection.

## Hierarchical Pathfinding (HPA*)

Chunk borders → entry/exit nodes. Precompute intra-chunk shortest paths between border nodes. Long-range: A* on chunk graph (~100 nodes cross-city). Short-range: regular A* within current + adjacent chunks (8K limit fine). Rebuild only on terrain change (never for static city).

## District Aggregates

```rust
struct District {
    id: u32,
    bounds: Rect,
    population: u32,
    population_by_type: HashMap<String, u32>,
    avg_hunger: f32,
    avg_health: f32,
    death_rate: f32,
    resource_stockpile: f32,
}
```

`run_district_stats` (Phase 5): ticks all statistical districts with equations. Population flows between adjacent districts based on resource gradients.

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
