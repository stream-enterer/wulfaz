# Spatial Query Infrastructure for City-Scale Agent Simulation

## The Problem in Numbers

Wulfaz models 1840s Paris on a 6,309 x 4,753 tile map (30M tiles) at 1 tile = 1 meter. The simulation must support 4,000 active entities with full AI (utility scoring, A* pathfinding, combat, hunger) within a 10ms total tick budget at 100 ticks/second. Current performance at 360 entities: `run_decisions` alone costs ~40ms, driven by `entities_in_range(pos, 30)` performing 3,721 HashMap lookups per call, called 2-4 times per entity per tick.

The existing spatial index is `HashMap<(i32, i32), SmallVec<[Entity; 4]>>` -- a tile-granularity hash map rebuilt from scratch each tick. This document argues for the specific spatial infrastructure that sustains the simulation from current scale through Phase C (4K active) and into Phase D (seamless zone transitions with hydration/dehydration across the full 30M-tile map).

---

## 1. Query Pattern Taxonomy

Every spatial query the simulation performs falls into one of six categories. The frequency, latency, and result-size characteristics at each scale determine which data structure serves each query.

### 1.1 Range Sensing (Chebyshev Distance R)

**Callers**: `FoodNearby` axis, `EnemyNearby` axis in `decisions.rs`; `select_eat_target`, `select_attack_target` in `decisions.rs`.

**Semantics**: Return all entities within Chebyshev distance R of a center point. Current R=30 (SENSE_RANGE constant, 30 meters).

**Frequency per tick**:
- At 360 entities: ~720 calls (2 axes per entity, plus ~0.5 target selection calls per entity on average). After Layer 1 pruning (from `perf-decisions-design.md`): ~30-50 calls.
- At 4K entities: ~8,000 calls unpruned. After pruning: ~200-400 calls. Most GIS-spawned entities (historical directory people) lack `combat_stats`, so the Aggression axis returns 0.0, pruning the Attack spatial query before it fires. Similarly, low-hunger entities prune the FoodNearby query.
- At 40K entities (hypothetical Nearby zone with simplified AI): This query is not needed in the Nearby zone per the architecture -- Nearby entities skip `run_decisions` entirely.

**Latency requirement**: Must complete all range sensing within ~3ms of the 10ms budget (leaving 7ms for pathfinding, other systems, and overhead).

**Result size**: At R=30, the query covers 61x61 = 3,721 tiles. With entity density of ~0.027 entities/tile in active building areas, the expected result is ~100 entities in dense quartiers, ~10-30 in sparse areas. The caller then filters by type (has `nutritions`? has `combat_stats`?) and takes a count or min-by-distance.

**Current cost**: 3,721 HashMap probes at ~15ns each = ~56us per call. At 720 calls = ~40ms.

**Target cost**: <1us per call after coarse grid + pruning. At 200-400 calls = ~0.2-0.4ms total.

### 1.2 Exact Tile Lookup

**Callers**: `entities_at(x, y)` in `combat.rs` (line 122), `eating.rs` (line 48).

**Semantics**: Return all entities occupying a single tile coordinate.

**Frequency per tick**:
- At 360 entities: ~100-200 calls (only entities with Attack/Eat intention that are at the same tile as their target).
- At 4K entities: ~500-1,000 calls. Combat and eating only fire for entities co-located with targets.

**Latency requirement**: Negligible -- single HashMap probe, already O(1). Under 50ns per call.

**Result size**: Typically 0-4 entities per tile. The `SmallVec<[Entity; 4]>` inline allocation handles the common case without heap allocation.

**Current cost**: Single HashMap probe, ~15ns. No bottleneck.

**Target cost**: Keep as-is. The per-tile `spatial_index` HashMap serves this query perfectly.

### 1.3 Path Existence and Cost (A* Pathfinding)

**Callers**: `find_path` in `tile_map.rs`, called by `run_wander` in Phase 4.

**Semantics**: Find shortest 8-directional path from start to goal on the walkability grid. Returns `Option<Vec<(i32, i32)>>`. Capped at 32,768 expanded nodes (MAX_EXPANDED).

**Frequency per tick**:
- At 360 entities: ~40 calls (only entities whose cooldown expired, Walk gait = 9 ticks between moves, so ~360/9 = 40 entities move per tick).
- At 4K entities: ~444 calls (4000/9).

**Latency requirement**: Must complete all pathfinding within ~3ms. At 444 calls, that's ~7us per path on average.

**Result size**: Path length varies from 1 (adjacent tile) to ~60 (WANDER_RANGE=30, diagonal traversal). A* typically expands 100-5,000 nodes for paths within WANDER_RANGE.

**Current cost**: Highly variable, 0.01-0.8ms per path depending on distance and obstacle complexity. The HashMap-based open/closed sets are the bottleneck for longer paths.

**Target cost**: For Phase B/C, the current A* with 32K node cap is adequate. Phase D introduces HPA* (SCALE-D03) for cross-district pathfinding. The spatial index does NOT interact with pathfinding -- pathfinding reads tile walkability directly from TileMap chunks, not from the entity spatial index.

### 1.4 Zone Membership

**Callers**: Phase C zone framework (SCALE-C02, not yet implemented).

**Semantics**: Given a camera/focal position and the district hierarchy (36 quartiers, ~600 blocks), determine which zone (Active/Nearby/Statistical) each entity or district belongs to.

**Frequency per tick**: Recomputed on camera movement, not every tick. Camera typically moves at most once per frame (60fps), and many frames the camera is stationary. When it does move, zone transitions for ~100 entities per tick (hydration/dehydration batch size).

**Latency requirement**: Zone classification itself is cheap. The expensive part is hydration/dehydration triggered by zone changes.

**Result size**: 36 quartier classifications (Active/Nearby/Statistical). Each quartier maps to a set of blocks and buildings.

### 1.5 Nearest-of-Type

**Callers**: `select_eat_target` and `select_attack_target` in `decisions.rs`.

**Semantics**: Among entities in range with a specific component (nutritions or combat_stats+healths), find the one with minimum Chebyshev distance (ties broken by secondary criteria: highest nutrition for food, lowest health for enemies, then entity ID).

**Frequency per tick**: Same as range sensing -- this is a refinement pass on the range-sensing result. After Layer 2 caching (nearby-entity cache), the nearest-of-type query operates on the already-collected nearby list, not a fresh spatial scan.

**Latency requirement**: Subsumed by range sensing -- if we have the nearby list cached, nearest-of-type is a linear scan of ~100 entities with component lookups. Under 1us.

### 1.6 Batch Spawn-at-Position (Hydration)

**Callers**: Phase D hydration system (SCALE-D01, not yet implemented).

**Semantics**: Spawn ~100 entities per tick in newly-active zone buildings. Requires: (1) identify buildings in the target zone, (2) select floor tiles within those buildings, (3) insert new entities at those positions into all property tables and the spatial index.

**Frequency per tick**: ~100 entity spawns per tick during zone transitions. Zero otherwise.

**Latency requirement**: Must complete within ~1ms per batch of 100. That's 10us per entity spawn, which is generous -- spawning an entity involves ~15 HashMap inserts (one per property table) plus spatial index update.

**Result size**: N/A (write operation, not read).

---

## 2. Spatial Density Analysis

### 2.1 Global Density

4,000 entities in 30M tiles = 1 entity per 7,500 tiles. This is extremely sparse globally. A naive full-map spatial structure would be mostly empty.

But this global density is meaningless because entities are not uniformly distributed. The Active zone is ~150 tile radius from camera, covering ~70K tiles (pi * 150^2, roughly). 4K entities in 70K tiles = 1 entity per 17.5 tiles. Still sparse by tile count, but entities cluster in buildings.

### 2.2 Building-Level Clustering

Building footprints range from 5 to 2,000+ tiles, with median ~50 tiles (from the `superficie` field, in m^2, which equals tile count at 1m/tile). A typical quartier has ~300 buildings totaling ~15,000 tiles of building space.

With ~4K entities in the Active zone, distributed across roughly 10-15 quartiers (the Active zone of 150-tile radius encompasses several quartiers given that quartier dimensions range from ~200-600 tiles across), the density is:
- ~300-400 entities per quartier in the camera vicinity
- ~1-3 entities per building on average
- Peak density in large buildings (hotels, markets, 500+ tiles): 10-30 entities

### 2.3 What Clustering Means for Grid Cell Sizing

Entities occupy building interior tiles (Floor, Door terrain). Building footprints are contiguous polygonal regions. A 16-tile cell (16x16 = 256 tiles) typically covers:
- 0-3 small buildings entirely
- Part of 1 large building
- Surrounding road tiles (entity-free except during transit)

With ~400 entities in a quartier spanning ~200x200 tiles, and cells of 16x16:
- ~12x12 = 144 cells cover the quartier
- ~2.8 entities per cell on average
- Most cells: 0-1 entities (road cells, empty building cells)
- Dense cells (market, large buildings): 5-15 entities

This clustering means a coarse grid with cell_size=16 has highly non-uniform cell occupancy. The range query (R=30) probes a 5x5 cell neighborhood containing ~25 cells, of which maybe 8-10 have any entities at all. The HashMap-based coarse grid naturally handles this -- empty cells don't exist in the map, so probing them returns `None` immediately.

### 2.4 Cache Behavior

With cell_size=16, each cell's entity list is a `Vec<Entity>`. Entity is 8 bytes (u64). A cell with 4 entities = 32 bytes of entity IDs, fitting in a single cache line. The 5x5 cell probe for a range query touches ~25 HashMap buckets (each ~64-128 bytes including SmallVec inline storage or Vec pointer+len+cap), for a total of ~2-3KB of memory access. This fits comfortably in L1 cache (32-64KB typical).

The distance-check filter after cell retrieval reads `body.positions` for each candidate entity. At ~100 candidates in the 5x5 neighborhood, that's 100 HashMap lookups into the positions table. Each Position is 8 bytes (two i32s). The HashMap buckets are scattered in memory, so this is ~100 cache misses at ~5-10ns each = ~0.5-1us.

### 2.5 Range Query Selectivity

A Chebyshev-30 query centered on a building in a dense quartier returns ~100 entities from a 5x5 cell neighborhood containing ~25 cells. After the distance filter, we get exactly those within the 61x61 tile Chebyshev square. The false-positive rate from the coarse grid (entities in cells that overlap the query square but outside Chebyshev distance) is low because cell_size=16 is small relative to range=30 -- the cell boundary only extends 15 tiles beyond the query edge at worst, adding at most a 1-cell-wide border of false positives.

Quantitatively: the 5x5 cell area is 80x80 = 6,400 tiles. The Chebyshev-30 square is 61x61 = 3,721 tiles. The false-positive area ratio is (6400-3721)/6400 = 42%. But because entity density is concentrated in buildings (not uniformly spread), and buildings at the cell boundaries are partially inside the query range, the actual false-positive entity rate is lower -- perhaps 20-30% of candidates fail the distance check and are discarded. This is acceptable; 100 integer comparisons cost ~100ns total.

---

## 3. Grid Cell Sizing: The Math

The optimization target is minimizing total work per tick: `rebuild_cost + total_query_cost`.

### 3.1 Rebuild Cost

Rebuilding the coarse grid iterates all N entities, computing `cell_key(pos.x, pos.y)` and inserting into a HashMap. Cost: N HashMap inserts.

For a HashMap with K expected keys (cells), each insert is ~20-50ns (hash computation + bucket lookup + possible resize). At N=4,000:
- Rebuild cost = 4,000 * ~30ns = ~120us

Cell size does not significantly affect rebuild cost -- the number of inserts is always N regardless of cell granularity. HashMap resize thresholds depend on load factor, but with `clear()` + re-insert (current approach), the internal capacity is retained between ticks. At cell_size=16 with 4K entities spread over ~1,000 distinct cells, the HashMap is well within its comfort zone.

### 3.2 Query Cost

For a range-R Chebyshev query with cell_size=C:

```
cells_per_axis = floor(2*R/C) + 2   (accounting for alignment)
cells_probed = cells_per_axis^2
entities_checked = cells_probed * avg_entities_per_cell
```

At C=16, R=30:
```
cells_per_axis = floor(60/16) + 2 = 3 + 2 = 5
cells_probed = 25
entities_checked = 25 * (N_local / cells_local)
```

Where `N_local` is entities in the Active zone area near the query center, and `cells_local` is occupied cells in that area.

The cost per query: `cells_probed * probe_cost + entities_checked * filter_cost`.
- probe_cost = ~15ns (HashMap lookup, mostly hitting populated buckets)
- filter_cost = ~10ns (position table lookup + distance comparison)

At C=16, R=30, 4K entities:
```
25 * 15ns + 100 * 10ns = 375ns + 1000ns = 1.375us per query
```

### 3.3 Varying Cell Size

| Cell Size | Cells Probed (R=30) | Entities/Cell (4K in ~1000 cells) | Filter Work | Total/Query |
|-----------|--------------------|------------------------------------|-------------|-------------|
| 4         | 17x17 = 289        | ~1.0                               | ~289 * 10ns | ~7.2us      |
| 8         | 9x9 = 81           | ~2.0                               | ~162 * 10ns | ~2.8us      |
| 16        | 5x5 = 25           | ~4.0                               | ~100 * 10ns | ~1.4us      |
| 32        | 3x3 = 9            | ~8.0                               | ~72 * 10ns  | ~0.9us      |
| 64        | 2x2 = 4            | ~16.0                              | ~64 * 10ns  | ~0.7us      |

The cost curve flattens rapidly above C=16. Going from C=16 to C=32 saves ~0.5us per query, but C=32 means each cell covers an entire chunk (64x64 at chunk boundary alignment). This is too coarse for the Nearby zone's simplified movement -- entities in a 32x32 cell can't be efficiently queried for short-range interactions.

Going from C=16 to C=8 doubles the probed cells but halves entities per cell. The crossover happens around C=12-16. Since 16 is a power of 2 (bit-shift division: `x >> 4`), it wins on implementation simplicity.

### 3.4 Optimal Cell Size: 16

Cell size 16 is the correct choice for R=30 range queries. The math converges on this: `C = R / 2` is the classical rule of thumb for spatial hashing, and `30 / 2 = 15`, rounded to the nearest power of 2 = 16. This gives 5x5 cell probes per query, which is the sweet spot between probe count and per-cell entity count.

If SENSE_RANGE were later reduced to 15 (a possible gameplay change), cell_size=8 would become optimal. If SENSE_RANGE increased to 60, cell_size=32 would be better. The cell size should be a configurable constant, not hardcoded.

---

## 4. Rebuild vs Incremental Update

### 4.1 Current Approach: Full Rebuild

`rebuild_spatial_index()` calls `self.spatial_index.clear()` then iterates all positions and inserts. Called twice per tick (before Phase 3 decisions, before Phase 4 combat/eating). Cost at 4K entities: ~120us per rebuild, ~240us total.

### 4.2 Movement Frequency Analysis

Movement is gated by the gait cooldown system. At Walk gait (default for all entities), movement occurs every 9 ticks. At 4K entities:
- Entities that move per tick: ~4,000 / 9 = ~444
- Entities that remain stationary: ~3,556 (89%)

On any given tick, 89% of entities do not change position. A full rebuild re-inserts all 4,000 entities into the spatial index, doing 3,556 insertions of unchanged data.

### 4.3 Incremental Update Cost

An incremental approach tracks which entities moved and updates only their entries:
1. For each moved entity: remove from old cell, insert into new cell.
2. For stationary entities: no work.

Cost per moved entity: 1 HashMap remove + 1 HashMap insert + 1 old-position lookup (to know which cell to remove from) = ~60ns.

At 444 moved entities: 444 * 60ns = ~27us per incremental update.

Savings vs full rebuild: 120us - 27us = ~93us per rebuild. With two rebuilds per tick: ~186us saved.

### 4.4 Breakeven Analysis

The incremental approach requires storing the previous position of each entity (or the previous cell key). This adds 8 bytes per entity (an `(i32, i32)` for the old cell key) = 32KB at 4K entities. Trivial memory cost.

The breakeven point is where `move_fraction * N * incremental_cost_per_entity < N * rebuild_cost_per_entity`. With the numbers above:

```
0.111 * N * 60ns < N * 30ns
6.67ns < 30ns  -- always true
```

Incremental update is cheaper for ANY entity count when move_fraction < 0.5 (i.e., more than half of entities are stationary per tick). With Walk gait at 9 ticks/move, the move fraction is 11.1%, so incremental update is always the right choice.

However, there is a critical caveat: the current architecture rebuilds the index twice per tick because movement (Phase 4 `run_wander`) happens between the two rebuild points. The first rebuild (before Phase 3 decisions) reflects positions from the previous tick's movement. The second rebuild (before Phase 4 combat/eating) reflects this tick's movement from `run_wander`.

### 4.5 Eliminating the Double Rebuild

Looking at the actual phase ordering in `main.rs`:

```
spatial1 (rebuild)
temperature (Phase 1)
hunger (Phase 2)
fatigue (Phase 2)
decisions (Phase 3) -- reads spatial index for range sensing
wander (Phase 4) -- modifies positions
spatial2 (rebuild)
eating (Phase 4) -- reads spatial index for exact tile lookup
combat (Phase 4) -- reads spatial index for exact tile lookup
death (Phase 5)
```

The second rebuild exists because `run_wander` moves entities, and `run_eating`/`run_combat` need to see updated positions for same-tile detection. But `run_eating` and `run_combat` only use `entities_at(x, y)` -- exact tile lookups, not range queries.

With incremental update, the second rebuild becomes: "update the spatial index for entities that moved in `run_wander`." Since `run_wander` already collects its moves in a `Vec<(Entity, Position)>` before applying them, this move list can drive the incremental update directly. No need to re-scan all entities.

**Proposed architecture change**: After `run_wander` applies moves, iterate the move list and update the spatial index incrementally. This eliminates the full second rebuild entirely. The combined cost of one full rebuild (120us at 4K) plus one incremental update (27us) = 147us, versus two full rebuilds (240us). A 39% reduction, and the code becomes clearer about when the spatial index is valid.

Alternatively, both rebuilds could become incremental if position changes between ticks are tracked. The first rebuild currently happens after `run_death` clears the previous tick's dead entities, so it must at minimum remove despawned entities' stale entries. With a "pending removes" list from `despawn()`, this too becomes incremental.

### 4.6 Recommended Approach

**Phase C (immediate)**: Keep one full rebuild per tick at the start (simple, 120us, well within budget). Replace the second rebuild with an incremental update driven by `run_wander`'s move list. This requires `run_wander` to return or store its move list for the spatial index update step.

**Phase D (later, if needed)**: Both rebuilds become incremental. Track position changes via a `dirty_positions: Vec<Entity>` on World, populated by any system that modifies `body.positions`. Clear at tick start after the spatial index is updated. This requires discipline (every position mutation must push to `dirty_positions`), which is manageable given that only `run_wander` currently modifies positions.

---

## 5. Pathfinding Interaction

### 5.1 Spatial Index vs Pathfinding: Separate Concerns

The spatial index and pathfinding operate on different data:
- **Spatial index**: Maps positions to entities. Used for "who is near me?" queries.
- **Pathfinding**: Maps tile coordinates to walkability (Terrain enum). Used for "how do I get from A to B?" queries.

These are fundamentally different access patterns. The spatial index is entity-centric (keyed by position, returning entities). Pathfinding is tile-centric (keyed by position, returning terrain/walkability). They share the coordinate space but not the data.

### 5.2 A* and the Tile Grid

The current A* implementation in `TileMap::find_path` reads terrain walkability via `self.is_walkable(nx, ny)`, which routes through the chunk system: `chunk_and_local(x, y)` -> `chunks[idx].get_terrain(lx, ly)`. This is O(1) per tile access -- flat Vec indexing into the chunk array, then array indexing within the chunk. No HashMap involved.

The A* uses `HashMap<(i32, i32), u32>` for g_scores, `HashMap<(i32, i32), (i32, i32)>` for came_from, and `HashSet<(i32, i32)>` for the closed set. These are per-path allocations, created and dropped within each `find_path` call. At MAX_EXPANDED=32,768 nodes, these HashMaps consume ~1-2MB transiently.

### 5.3 Should Pathfinding Use the Same Spatial Structure?

No. The spatial entity index and the pathfinding walkability grid serve different purposes and have different performance characteristics. The pathfinding grid is read-only within a tick (terrain doesn't change during simulation), while the entity spatial index is written and read within the same tick. Merging them would create unnecessary coupling and complicate the Phase D HPA* transition.

### 5.4 Obstacle Queries During Path Execution

During path execution in `run_wander`, the entity follows a precomputed path (one step per movement cooldown). The path was computed against the static walkability grid, which doesn't change between ticks. Dynamic obstacles (other entities) are NOT considered by pathfinding -- entities can share tiles (the simulation allows co-location, which is how combat works: entities fight when on the same tile).

If future design requires entity-entity collision avoidance, that would be a separate "local avoidance" system operating on the spatial index, not a pathfinding concern. The spatial index would support this via `entities_at(next_step_x, next_step_y)` to check for crowding.

### 5.5 HPA* and Precomputed Graphs (Phase D)

HPA* (SCALE-D03) precomputes a chunk-level navigation graph: border nodes at chunk edges, precomputed intra-chunk paths. This is built once at map load (or when terrain changes, which never happens during simulation). The precomputed graph lives alongside the TileMap, not the spatial index. It has no interaction with entity positions or the spatial grid.

---

## 6. LOD Zone Spatial Structure

### 6.1 Zone Definition

Zones are concentric regions around the camera/focal point:
- **Active**: ~150-tile radius. Full AI.
- **Nearby**: ~500-tile radius. Simplified systems.
- **Statistical**: Everything else. Aggregate equations.

### 6.2 Zone Membership Computation

Option A: Per-entity distance check each tick.
- Cost: 4K entities * 1 distance computation = ~4K * 5ns = 20us. Negligible.
- Problem: Doesn't handle the Statistical zone (no individual entities exist there).

Option B: Per-quartier zone classification.
- Each of the 36 quartiers has a bounding box (derivable from building positions at load time). Classify each quartier as Active/Nearby/Statistical based on distance from camera to quartier centroid.
- Cost: 36 distance comparisons = ~180ns. Negligible.
- Advantage: Statistical zone entities never need individual classification. The zone system operates on districts, not entities.

Option C: Pre-baked tile-level zone map.
- A `zone_id: [u8; CHUNK_AREA]` layer in each chunk, updated when camera moves. Each tile knows its zone.
- Cost: Too expensive. Updating ~70K tiles per camera move is overkill when we only need ~36 quartier classifications.

**Recommended**: Option B with block-level refinement. Classify quartiers first (36 comparisons). For quartiers on zone boundaries, classify individual blocks within that quartier (~30-100 blocks per quartier, ~600 total). Total cost per camera move: <100 comparisons, <1us. Cache the classification in a `Vec<ZoneKind>` indexed by quartier_id (or block_id). Recompute only when camera moves more than a threshold distance (e.g., 32 tiles = 2 chunks).

### 6.3 Quartier Bounding Boxes

Quartier bounds are not explicitly in the data but are trivially derivable. At map load (or in the preprocessor), compute the axis-aligned bounding box for each quartier from the positions of all buildings with that quartier name:

```rust
struct QuartierBounds {
    min_x: i32, max_x: i32,
    min_y: i32, max_y: i32,
    centroid_x: i32, centroid_y: i32,
    building_count: u32,
    total_area: f32,
}
```

This is computed once and stored on `GisTables`. Memory: 36 quartiers * ~32 bytes = ~1KB. The centroid is used for zone classification distance checks. The bounding box is used for hydration (finding buildings to populate).

### 6.4 Entity-to-Zone Mapping

For Active zone entities, zone membership can be cached as a component: `HashMap<Entity, ZoneKind>`. Updated when camera moves. At 4K entities, this is ~32KB. Systems check zone membership to skip irrelevant entities: `if zone != Active { continue; }` for combat, etc.

For Nearby zone entities, the same cache applies. Nearby entities exist as real Entity values with simplified components. Zone transitions (Active->Nearby, Nearby->Statistical) are detected by comparing the cached zone with the freshly-computed quartier zone classification.

---

## 7. Hydration Spatial Requirements

### 7.1 The Hydration Pipeline

When a quartier transitions from Statistical to Active (camera moves into range):

1. **Identify target buildings**: All buildings in the quartier. Available via `BuildingData.quartier` field in the BuildingRegistry. At ~300 buildings per quartier, this is a linear scan of the registry or a precomputed index.

2. **Select floor tiles**: Each building has a `tiles: Vec<(i32, i32)>` listing all its tile coordinates. Filter for Floor/Door tiles (not Wall). At median ~50 tiles per building with ~60% interior, ~30 Floor/Door tiles per building.

3. **Place entities**: Distribute entities across floor tiles according to population density rules (from SCALE-C05). At ~400 entities per quartier, batch ~100 per tick = 4 ticks to fully hydrate a quartier.

4. **Insert into spatial index**: Each spawned entity gets inserted into both the fine-grained `spatial_index` and the coarse `spatial_grid`.

### 7.2 Data Structures for Efficient Hydration

**Precomputed building-to-quartier index**: `HashMap<String, Vec<BuildingId>>` keyed by quartier name, listing all building IDs in that quartier. Built once at load time from the BuildingRegistry. Memory: 36 entries * ~300 BuildingIds * 4 bytes = ~43KB.

Alternatively, since quartier_id is stored per-tile in the chunk system, and buildings already have a `quartier` field, the mapping is available without a separate index. But a precomputed index avoids scanning 15,000 buildings to find the ~300 in one quartier.

**Precomputed floor tile lists per building**: Already available -- `BuildingData.tiles` stores all tile coordinates. At hydration time, filter for walkable tiles (check terrain via TileMap). This filtering could also be precomputed and cached to avoid repeated terrain lookups, but the cost is low: 300 buildings * 50 tiles * 1 terrain check = 15,000 chunk accesses at O(1) each = ~75us. Acceptable for a once-per-quartier-transition operation.

**Batch insert into spatial index**: Inserting 100 entities into the spatial index is 100 HashMap inserts for the fine-grained index + 100 for the coarse grid = 200 inserts at ~30ns each = 6us. Negligible.

The bottleneck in hydration is not spatial queries but entity construction: allocating Entity IDs, inserting ~15 components per entity into their respective HashMaps, generating names/stats from distributions. At 100 entities * 15 table inserts * 30ns = 45us per batch. Well within budget.

### 7.3 Building-to-Zone Precomputation

Should building-zone mappings be precomputed? Yes, partially. The `QuartierBounds` structure (Section 6.3) enables O(1) zone classification per quartier. Given a quartier's zone, all its buildings inherit that zone. No per-building spatial query needed.

For boundary quartiers (partially in Active, partially in Nearby), block-level refinement determines which buildings are in which zone. The `BlockData.buildings` field maps blocks to building lists. This is already available in the BlockRegistry.

---

## 8. Memory Budget

### 8.1 Fine-Grained Spatial Index (Existing)

`HashMap<(i32, i32), SmallVec<[Entity; 4]>>`

At 4K entities on a 30M-tile map, there are at most 4K occupied tiles (entities can share tiles, so typically fewer). Each HashMap entry: ~80 bytes (key: 8 bytes, SmallVec inline: 32 bytes, HashMap overhead: ~40 bytes). Total: ~4,000 * 80 = 320KB.

This grows linearly with entity count, not map size. At 40K entities (hypothetical): ~3.2MB.

### 8.2 Coarse Spatial Grid

`HashMap<(i32, i32), Vec<Entity>>`

At cell_size=16 with 4K entities, the number of occupied cells depends on entity distribution. With entities spread across ~1,000 distinct cells: each entry is ~64 bytes (key: 8, Vec header: 24, HashMap overhead: ~32) plus ~32 bytes of entity IDs per cell (average 4 entities). Total: ~1,000 * 96 = ~96KB.

The HashMap approach means only occupied cells consume memory. A full-map flat grid at cell_size=16 would be:
- (6309/16 + 1) * (4753/16 + 1) = 395 * 298 = ~118K cells
- At 8 bytes per cell pointer: ~944KB
- At 64 bytes per cell (Vec + capacity): ~7.5MB
- Most cells would be empty, wasting memory.

The HashMap-based coarse grid is strictly better than a flat grid for this use case because entities occupy <0.02% of map tiles. The HashMap only allocates for occupied cells.

At 40K entities (Nearby zone expansion): ~10,000 occupied cells * 96 bytes = ~960KB. Acceptable.

### 8.3 Full-Map Structures

For zone transitions anywhere on the map, the spatial index only needs to cover entities that exist -- Active and Nearby zones. Statistical zone entities don't have individual positions, so they don't appear in the spatial index. The spatial index scales with entity count, not map size.

The TileMap itself is the only full-map structure: 30M tiles * ~5 bytes per tile (terrain, temperature, building_id, block_id, quartier_id across chunks) = ~150MB. This is already loaded and isn't part of the spatial query infrastructure.

### 8.4 Summary

| Structure | Memory at 4K | Memory at 40K | Scales With |
|-----------|-------------|---------------|-------------|
| Fine spatial index | 320KB | 3.2MB | Entity count |
| Coarse spatial grid | 96KB | 960KB | Entity count |
| Quartier bounds | 1KB | 1KB | Fixed (36) |
| Building-quartier index | 43KB | 43KB | Fixed (15K) |
| Zone cache per entity | 32KB | 320KB | Entity count |
| **Total spatial overhead** | **~490KB** | **~4.5MB** | Entity count |

All values are within budget for a simulation running on a modern desktop.

---

## 9. The Two-Rebuild Question

### 9.1 Current Phase Ordering

```
spatial1 (rebuild)      -- positions reflect last tick's movement
temperature             -- Phase 1, no entity positions
hunger                  -- Phase 2, no spatial queries
fatigue                 -- Phase 2, no spatial queries
decisions               -- Phase 3, READS spatial index (range sensing)
wander                  -- Phase 4, WRITES positions (movement)
spatial2 (rebuild)      -- positions reflect this tick's movement
eating                  -- Phase 4, READS spatial index (exact tile lookup)
combat                  -- Phase 4, READS spatial index (exact tile lookup)
death                   -- Phase 5, removes entities
```

### 9.2 Why Two Rebuilds Exist

The first rebuild (`spatial1`) makes the spatial index consistent with entity positions for Phase 3 decisions. Phase 3 needs accurate range sensing to evaluate FoodNearby/EnemyNearby considerations and select targets.

Phase 4 `run_wander` then moves entities, invalidating the spatial index.

The second rebuild (`spatial2`) makes the spatial index consistent again for Phase 4 eating/combat, which need accurate same-tile detection.

### 9.3 Can We Reduce to One Rebuild?

Yes, with the incremental update approach from Section 4.5. The architecture change:

1. `spatial1` remains a full rebuild at tick start (or incremental if tracking dirty positions from previous tick).
2. `run_wander` stores its move list in a temporary field or returns it.
3. After `run_wander`, apply incremental spatial index updates from the move list. No full rebuild.

This gives two consistent spatial index states per tick with the cost of one full rebuild + one incremental update, instead of two full rebuilds.

### 9.4 Reordering Phases to Avoid the Problem

Could we reorder phases to need only one rebuild? If `run_wander` (movement) happened before `run_decisions` (sensing):

```
spatial1 (rebuild)
wander           -- Phase 4 moved before Phase 3
decisions        -- Phase 3 now sees current positions
eating           -- Phase 4, same spatial index
combat           -- Phase 4, same spatial index
death            -- Phase 5
```

This breaks the phase contract. Decisions (Phase 3) are supposed to read environment and write intentions. Wander (Phase 4) is supposed to read intentions and change external state. If wander runs before decisions, entities move before deciding -- movement would use the PREVIOUS tick's intentions, making entities always one tick behind in their responses.

The existing phase ordering is correct. The two-rebuild structure is inherent to the Phase 3 (read spatial) -> Phase 4 (write positions) -> Phase 4 (read spatial) pipeline. The incremental update is the right solution.

---

## 10. Concrete Data Structure Proposals

### 10.1 Structure A: Per-Tile Spatial Index (Existing, Keep)

**Rust type**:
```rust
pub spatial_index: HashMap<(i32, i32), SmallVec<[Entity; 4]>>
```

**Purpose**: Exact tile lookups for combat and eating.

**Rebuild cost**: O(N) where N = alive entity count. At 4K: ~4,000 HashMap inserts, ~120us.

**Range query cost**: O((2R+1)^2) HashMap probes. At R=30: 3,721 probes, ~56us. DO NOT use this for range queries -- it is the bottleneck.

**Exact lookup cost**: O(1) amortized. Single HashMap probe, ~15ns.

**Memory**: ~80 bytes per occupied tile. At 4K entities: ~320KB.

**Compatibility**: Fully compatible. This is the existing structure. `entities_at(x, y)` returns `&[Entity]` via borrowed SmallVec slice. Combat and eating use this API. No changes needed.

### 10.2 Structure B: Coarse Spatial Grid (New, Add)

**Rust type**:
```rust
pub spatial_grid: HashMap<(i32, i32), Vec<Entity>>
// cell_key: (x >> 4, y >> 4), i.e., 16-tile cells
```

**Purpose**: Range queries for decisions system.

**Rebuild cost**: O(N) HashMap inserts with bit-shift key computation. At 4K: ~4,000 inserts, ~120us. Rebuilt alongside Structure A in `rebuild_spatial_index()`.

**Range query cost**: O(C^2 + M) where C = cells probed, M = entities in those cells. At R=30, C=25, M~100: 25 HashMap probes + 100 distance checks = ~1.4us.

**Exact lookup cost**: Not designed for exact tile lookups. Would require filtering cell contents by exact position. Use Structure A instead.

**Memory**: ~96 bytes per occupied cell. At 4K entities, ~1,000 cells: ~96KB.

**Compatibility**: Fully compatible with Wulfaz constraints. It is a HashMap (same data structure family, satisfying the CLAUDE.md constraint). Added as a new field on World, populated by `rebuild_spatial_index()`. `entities_in_range()` rewritten to use this instead of per-tile iteration. No API changes for callers -- the return type `impl Iterator<Item = Entity>` is preserved.

**Determinism**: No impact. The coarse grid returns the same entity set as the fine-grained scan (distance filter is identical). Callers already sort or use deterministic tiebreakers.

### 10.3 Structure C: Quartier Spatial Bounds (New, Add to GisTables)

**Rust type**:
```rust
pub struct QuartierBounds {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
    pub centroid_x: i32,
    pub centroid_y: i32,
    pub building_count: u32,
    pub total_floor_area: f32,
}

// On GisTables:
pub quartier_bounds: Vec<QuartierBounds>  // indexed by quartier_id - 1
```

**Purpose**: Zone classification. Hydration target selection.

**Build cost**: O(B) where B = total buildings. Single pass at map load. At 15K buildings: ~5ms one-time. Not rebuilt per tick.

**Zone classification cost**: O(Q) where Q = 36 quartiers. 36 distance comparisons: ~200ns.

**Memory**: 36 * ~32 bytes = ~1.2KB.

**Compatibility**: Fully compatible. Added to GisTables, computed at load time in `loading_gis.rs`. No per-tick cost. Removed via `GisTables::remove()` if needed (though it has no per-entity data).

### 10.4 Structure D: Building-to-Quartier Index (New, Add to GisTables)

**Rust type**:
```rust
pub quartier_buildings: Vec<Vec<BuildingId>>  // indexed by quartier_id - 1
```

**Purpose**: Hydration -- quickly find all buildings in a quartier without scanning the full BuildingRegistry.

**Build cost**: O(B) single pass at load. ~1ms.

**Lookup cost**: O(1) by quartier_id. Returns a Vec of ~300 BuildingIds.

**Memory**: 36 * ~300 * 4 bytes = ~43KB.

**Compatibility**: Fully compatible. Immutable after load. Used by the hydration system.

### 10.5 Structure E: Dirty Position Tracking (Phase D, Deferred)

**Rust type**:
```rust
pub dirty_positions: Vec<(Entity, (i32, i32), (i32, i32))>
// (entity, old_cell_key, new_cell_key) -- populated by any position mutation
```

**Purpose**: Incremental spatial index update instead of full rebuild.

**Cost per movement**: O(1) push to Vec. ~5ns.

**Incremental update cost**: O(D) where D = moved entities. At ~444 moved per tick: 444 * 60ns = ~27us.

**Memory**: At 444 entries * 20 bytes = ~9KB per tick (transient, cleared each tick).

**Compatibility**: Requires discipline -- every `body.positions.insert()` must also push to `dirty_positions`. Currently only `run_wander` modifies positions, so the discipline burden is minimal. Future position-modifying systems (knockback, teleportation) must follow the same pattern.

---

## 11. Combined Performance Projection

### 11.1 At 4K Entities (Phase C Target)

Using Structures A + B, with Layer 1 score pruning and Layer 2 nearby-entity cache from `perf-decisions-design.md`:

| Component | Cost |
|-----------|------|
| Spatial rebuild (A + B) | 240us (2x full rebuild) |
| Decisions: scoring (pruned) | 3ms |
| Decisions: spatial queries (~200 * 1.4us) | 0.28ms |
| Wander: A* pathfinding (~444 paths) | 2-3ms |
| Combat + Eating | <0.5ms |
| Hunger + Fatigue + Temperature | <0.5ms |
| Death | <0.1ms |
| **Total** | **~6.5-7.5ms** |

This is within the 10ms budget with ~2.5ms headroom for LOD zone management, hydration batches, and future systems.

### 11.2 At 4K Entities with Incremental Update

Replace second full rebuild with incremental update:

| Component | Cost |
|-----------|------|
| Spatial rebuild (1x full + 1x incremental) | 147us |
| Everything else | Same |
| **Total** | **~6.4-7.4ms** |

Modest savings (93us), but it establishes the pattern needed for Phase D where entity churn from hydration/dehydration makes incremental updates more valuable.

### 11.3 At Scale with Zone Transitions

During a zone transition (camera moves to a new quartier):
- Zone reclassification: ~200ns
- Hydration batch (100 entities): ~50us (entity construction) + 6us (spatial insert)
- Dehydration batch (100 entities): ~30us (remove from tables) + 6us (spatial remove)
- Per-tick transition overhead: <100us

Zone transitions are amortized over ~40 ticks (4 quartiers * 10 ticks to fully hydrate each). The per-tick cost is negligible relative to the simulation budget.

---

## 12. What This Infrastructure Does NOT Need

### 12.1 No Quadtree or k-d Tree

These structures are designed for efficient nearest-neighbor queries in continuous 2D/3D space. Wulfaz operates on a discrete integer grid with Chebyshev distance. The coarse HashMap grid with bit-shift cell keys is simpler, faster for this specific access pattern, and compatible with the HashMap-based EAV architecture.

Quadtrees would add complexity (balancing, pointer chasing, cache unfriendly) for no benefit when cell_size=16 already achieves <2us per range query.

### 12.2 No R-tree

R-trees are for range queries on axis-aligned bounding boxes in databases. Entity positions are points, not rectangles. The coarse grid handles point-in-range queries efficiently.

### 12.3 No Spatial Hashing with Open Addressing

The Rust `HashMap` already uses Robin Hood hashing with good cache behavior. A custom spatial hash with open addressing might save 10-20% on lookup time but adds maintenance burden and fragility. Not worth it when the total spatial query cost is already under 0.5ms.

### 12.4 No Parallel Spatial Queries

The simulation is single-threaded by architectural constraint. Parallel spatial queries would require either unsafe aliasing of the spatial index or Arc/Mutex overhead that negates the parallelism benefit at 4K entities. The sequential budget (10ms) is sufficient.

---

## 13. Implementation Ordering

The structures should be implemented in this order, each independently testable:

1. **Structure B (Coarse Spatial Grid)**: The single highest-impact change. Add `spatial_grid` to World, update `rebuild_spatial_index()`, rewrite `entities_in_range()`. Expected impact: range queries drop from 56us to 1.4us each. Combined with score pruning (Layer 1), decisions drops from 40ms to ~3ms at 360 entities.

2. **Structure C (Quartier Bounds)**: Required for Phase C zone classification. Computed once at load. Low risk, low effort, no per-tick cost.

3. **Structure D (Building-Quartier Index)**: Required for Phase D hydration. Same characteristics as Structure C.

4. **Incremental Update (Structure E)**: Replace second per-tick rebuild with incremental update. Requires `run_wander` to expose its move list. Modest performance benefit now, essential for Phase D hydration churn.

Structures C and D can be built in parallel with or after Structure B, as they serve different phases of the project. Structure E is a refinement that becomes important only when hydration/dehydration adds entity churn.

---

## 14. Conclusion

The spatial query infrastructure for Wulfaz at city scale consists of exactly five structures:

1. **Per-tile spatial index** (existing HashMap): serves exact tile lookups for combat/eating. O(1) per query. No change needed.

2. **Coarse spatial grid** (new HashMap, cell_size=16): serves range sensing for decisions. O(25 probes + 100 distance checks) per query = ~1.4us. Replaces the catastrophic O(3,721 probes) = 56us per query.

3. **Quartier spatial bounds** (new Vec, 36 entries): serves zone classification. O(36) per camera move. ~200ns.

4. **Building-quartier index** (new Vec of Vec, 36 * ~300 entries): serves hydration. O(1) per quartier. ~43KB.

5. **Dirty position tracking** (new Vec, transient): serves incremental spatial update. O(moved entities) per tick. ~27us.

Total memory overhead: ~490KB at 4K entities. Total per-tick spatial cost: ~360us (one full rebuild + one incremental update + range query amortization). This leaves 9.6ms of the 10ms budget for actual simulation logic.

The current spatial index is not broken in design -- it is broken in granularity. The per-tile HashMap is the correct structure for exact lookups. It is the wrong structure for range queries spanning 3,721 tiles. Adding a second index at 16-tile granularity, alongside algorithmic pruning in the decisions system, transforms spatial queries from the dominant bottleneck (40ms, 400% over budget) into a negligible cost (0.36ms, 3.6% of budget). No exotic data structures are needed. No architectural changes beyond adding one HashMap field to World and one Vec field to GisTables. The simulation scales from 360 to 4,000 entities within the existing single-threaded, deterministic, HashMap-based architecture.
