# Spatial Data Structures for Wulfaz: A First-Principles Analysis

## 1. Characterization of the Problem

The spatial problem in Wulfaz is not a generic game-engine spatial problem. It has a specific and unusual combination of constraints that eliminates most textbook solutions and points toward a narrow family of optimal designs. Let me enumerate what makes this problem distinctive before evaluating any data structure.

**Extreme sparsity.** The map is 6,309 x 4,753 = ~30 million tiles. At 4,000 active entities, that is one entity per 7,500 tiles. At 360 entities (current), one per 83,000 tiles. Any data structure that allocates per-tile is wasteful by a factor of 10,000. Any data structure that partitions space at tile granularity (like a dense 2D array of entity lists) is out of the question on memory alone: 30M entries x 8 bytes minimum = 240 MB for an empty grid.

**Small entity counts, expensive queries.** The bottleneck is not "too many entities to iterate" -- 4,000 entities is trivially iterable in microseconds. The bottleneck is per-entity spatial queries. Each entity in the decisions system calls `entities_in_range` with R=30, which probes (2*30+1)^2 = 3,721 HashMap buckets. At 360 entities, that is 360 x 3,721 = 1.34 million HashMap lookups per rebuild cycle. At 4,000 entities it would be 14.9 million lookups. This is the actual performance wall.

**Bulk rebuild, read-heavy.** The spatial index is rebuilt from scratch twice per tick (before Phase 3 decisions, before Phase 4 combat/eating). Between rebuilds, it is read-only. This is the ideal access pattern for structures that are expensive to build incrementally but cheap to query -- the exact opposite of balanced trees, which are cheap to update but have per-query overhead from pointer chasing.

**Blackboard architecture constraint.** The spatial index lives as a field on `World`. Systems borrow `&self` (for queries) or `&mut self` (for rebuilds) on World. The index cannot maintain internal mutability or lazy-evaluation state because that would require interior mutability patterns (`RefCell`, `Cell`) which conflict with the borrow checker's guarantees about the rest of World. The index must be a simple owned data structure that supports `&self` queries and `&mut self` rebuilds.

**HashMap-based source data.** Entity positions live in `HashMap<Entity, Position>`. The rebuild must iterate this map. HashMap iteration order is non-deterministic, but that is acceptable because the spatial index's query results are sorted by entity ID by callers. The rebuild's cost is dominated by the iteration + insertion, not by the iteration order.

**Determinism is a non-constraint for the index itself.** Callers sort results by `e.0`. The index only needs to be correct (return all entities in the queried region), not deterministically ordered. This frees us from sorted containers.

## 2. Analysis of the Current Design and Its Failure Mode

The current spatial index is `HashMap<(i32, i32), SmallVec<[Entity; 4]>>`. This is a sparse tile-granularity index.

**Rebuild cost:** O(N) where N = entity count. Iterate `positions`, insert each into the HashMap. At 360 entities this is ~360 HashMap insertions, which is fast (microseconds). At 4,000 entities, still fast.

**Exact tile lookup:** O(1) amortized. Single HashMap probe. This is optimal.

**Range query:** O(R^2) HashMap probes, where R is the Chebyshev radius. For R=30, that is 3,721 probes. Each probe is a HashMap lookup with hashing + potential collision chain traversal. The HashMap's buckets are heap-allocated, scattered across memory. At 3,721 probes, the CPU is doing 3,721 cache-missing memory accesses per query.

**Why this is the bottleneck:** At 360 entities, each running one range query, that is 1.34M cache-missing lookups per decisions phase. At ~3ns per L3 cache miss (AMD Zen 3/4), that alone is 4ms. But HashMap lookup is worse than a single cache miss: it hashes the key, indexes into the bucket array, follows a pointer to the bucket, and compares keys. Realistic cost is 10-30ns per lookup depending on load factor and collision rate, giving 13-40ms for 360 entities. The measured ~40ms aligns with ~30ns per lookup. At 4,000 entities: 150-450ms per decisions phase. Unacceptable at 100 ticks/sec (10ms budget).

**The fundamental problem:** Tile-granularity probing turns an O(N) problem (iterate all entities, filter by distance) into an O(R^2) problem per query. When R^2 >> N (3,721 >> 360), the brute-force approach of "iterate all entities and compute distance" would actually be faster. But as N grows toward 4,000 and beyond, brute force becomes O(N^2) across all entities. The correct solution is neither tile-level probing nor brute force, but a coarser spatial partitioning that reduces probes to O(K) where K is the number of cells overlapping the query region.

## 3. The Optimal Primary Structure: Coarse Uniform Grid

### 3.1 Why a Uniform Grid

A uniform grid with cell size C covers the query region of radius R with ceil(2R/C + 1)^2 cell lookups. Setting C = 32 (32 meters per cell) reduces a R=30 query from 3,721 probes to ceil(60/32 + 1)^2 = 3^2 = 9 cell lookups. Setting C = 64 (matching chunk size) reduces it to ceil(60/64 + 1)^2 = 2^2 = 4 cell lookups. This is a 400-900x reduction in probe count.

The grid cells contain entity lists, not presence flags. At 4,000 entities across 30M tiles, most cells are empty. Using a `HashMap<(i32, i32), SmallVec<[Entity; 4]>>` keyed by cell coordinate (not tile coordinate) preserves sparsity while reducing the keyspace from 30M possible keys to ~7,400 possible keys (at C=64, matching chunks).

This is architecturally identical to the current design -- same type signature, same rebuild logic, same query pattern -- but with a different coordinate divisor. The change is approximately 10 lines of code.

### 3.2 Cell Size Selection

The cell size must be chosen to minimize total work = (rebuild cost) + (query cost per entity x entity count).

- **C = 16 (16m cells):** R=30 query spans ceil(60/16+1)^2 = 5^2 = 25 cells. At 4K entities: 100K lookups per decisions phase. Good but not great.
- **C = 32 (32m cells):** R=30 query spans 3^2 = 9 cells. At 4K entities: 36K lookups. Excellent.
- **C = 64 (64m cells, matching chunks):** R=30 query spans 2^2 = 4 cells. At 4K entities: 16K lookups. Best probe count, but each cell covers 4,096 tiles and may contain more entities, requiring more filtering per cell.

At 4K entities in the active zone, the average cell occupancy at C=64 is 4,000 / ~100 occupied cells = ~40 entities per cell. Filtering 40 entities x 4 cells = 160 distance checks per query. At 4K queries: 640K distance checks. Each distance check is a Chebyshev comparison (two subtracts, one max, one compare) -- ~2ns. Total: ~1.3ms. Plus 16K HashMap lookups at ~15ns each: ~0.24ms. Total: ~1.5ms.

At C=32: average cell occupancy ~10 entities. 10 x 9 = 90 distance checks per query. 360K distance checks + 36K HashMap lookups. Total: ~1.3ms. Similar.

Both are dramatically better than the current ~40ms and project well to 4K entities. The C=64 option has the advantage of aligning with the existing chunk coordinate system, simplifying mental model and enabling potential future optimizations (chunk-based LOD filtering).

**Recommendation: C=64, aligned with TileMap chunks.** Entity cell coordinate is `(pos.x / 64, pos.y / 64)` -- a trivial shift. The index type remains `HashMap<(i32, i32), SmallVec<[Entity; 4]>>` (though the SmallVec inline capacity should increase to 8 or 16 given higher per-cell counts). The conceptual model becomes "which chunk is this entity in?" which is natural for the existing chunk-based architecture.

### 3.3 Maintaining Exact Tile Lookup

The coarse grid does not support O(1) exact tile lookup. To find entities at tile (x, y), you must look up cell (x/64, y/64) and filter. At average 40 entities per cell, this is 40 comparisons -- fast but not O(1).

Combat and eating systems use `entities_at(x, y)` for same-tile checks. These are called only for combatant pairs and hungry entities -- not for all 4K entities. Typical call count: 10-50 per tick. At 40 comparisons per call: 200-2000 comparisons total. Negligible.

If exact tile lookup becomes a bottleneck (it will not at 4K entities), a secondary tile-level index can be added as a separate HashMap. But this is premature optimization given the call frequency.

## 4. Analysis of Alternative Structures

### 4.1 Quadtree

A quadtree recursively subdivides space into quadrants. For a 6,309 x 4,753 map, the tree depth is ~13 levels (log2(6309) = 12.6).

**Rebuild cost:** O(N log N) for N entity insertions. Each insertion traverses ~13 levels. At 4K entities: 52K operations. Comparable to HashMap insertions but with worse cache behavior (pointer chasing through tree nodes).

**Range query cost:** O(K + R) where K is the number of nodes visited and R is the result size. For a R=30 query (60x60 tile region) in a map of 6,309 x 4,753, the query rectangle is tiny relative to the map. The quadtree prunes aggressively: at most 4 x 13 = 52 nodes visited to reach the relevant leaf region, plus examination of entities in overlapping leaves. With 4K entities, the leaves near the query region contain maybe 5-20 entities. Total work per query: ~100 operations. Very good.

**Problems for Wulfaz:**
1. **Pointer-heavy, cache-unfriendly.** Each quadtree node is a heap allocation with 4 child pointers. Traversal chases pointers through scattered memory. At 200 rebuilds/sec, the allocation/deallocation cost is non-trivial.
2. **Complexity for marginal gain.** The uniform grid at C=64 achieves 4 cell lookups + 160 distance checks per query. The quadtree achieves ~100 operations per query. The quadtree is faster per-query, but the uniform grid's per-query cost is already ~0.4 microseconds. At 4K queries: quadtree saves ~0.4ms. Not worth the added code complexity.
3. **Rebuild cost is higher.** Quadtree insertion traverses 13 levels per entity. HashMap insertion is O(1) amortized. At 4K entities, quadtree rebuild is ~3x slower.

**Verdict:** Quadtree is theoretically superior for range queries but practically worse for Wulfaz's specific parameters (small N, moderate R, frequent rebuilds). Reject.

### 4.2 R-tree

R-trees organize bounding boxes into a balanced tree with configurable fanout. Good for spatial databases with mixed-size objects and complex query shapes.

**Problems for Wulfaz:** Entities are points, not rectangles. Point R-trees exist (just degenerate bounding boxes) but add complexity. R-tree bulk loading (STR packing) is O(N log N) and produces excellent query performance, but the implementation complexity is high. More importantly, R-trees are designed for datasets that change incrementally. Wulfaz rebuilds from scratch every tick. The rebuild cost of an R-tree (even with bulk loading) is higher than a HashMap-based uniform grid, and query performance at 4K points is not meaningfully better. **Reject.**

### 4.3 k-d Tree

k-d trees partition points by alternating axes. Excellent for nearest-neighbor queries in low dimensions.

**Rebuild cost:** O(N log N) with median-of-medians partitioning. At 4K: ~48K operations. The tree can be built in a flat array (no pointer chasing during queries).

**Range query cost:** O(sqrt(N) + K) where K is the result count. For N=4K: ~63 node visits plus result iteration. Very fast.

**Advantages:** Can be built in a contiguous `Vec<KdNode>` with index-based children, giving excellent cache locality. Rebuild is a single allocation + sort-based partitioning.

**Problems for Wulfaz:**
1. **Exact tile lookup is O(log N)**, not O(1). For combat's `entities_at(x, y)`, this means 12 tree traversal steps instead of 1 HashMap probe. At 50 calls per tick this is negligible, but it is architecturally inelegant.
2. **Rebuild cost is higher than a HashMap grid.** O(N log N) vs O(N). At 4K entities the difference is 48K vs 4K operations.
3. **Range query advantage is marginal.** k-d tree: ~63 visits. Grid C=64: ~160 comparisons. At ~2ns per operation, the difference is ~0.2 microseconds per query. Over 4K queries: ~0.8ms savings. Not enough to justify the complexity.

**Verdict:** Viable. Better than quadtree. But the uniform grid is simpler and good enough. Reject unless profiling shows the grid becoming a bottleneck, which is unlikely before 40K entities.

### 4.4 Flat Sorted Array with Binary Search

Store all entity positions in a `Vec<(i32, i32, Entity)>` sorted by (x, y). Range queries use binary search to find the x-range, then linear scan for y-range.

**Rebuild cost:** O(N log N) for sorting. At 4K: ~48K operations.

**Range query cost:** O(log N + R_x * avg_density) where R_x is the x-range width. For R=30, R_x=60. With 4K entities on 6,309 x-tiles, average density is 0.63 entities per x-column. Scanning 60 x-columns: ~38 entities examined. Very fast.

**Advantages:** Excellent cache locality (single contiguous Vec). No allocation during rebuild (reuse Vec, just re-sort). Trivially correct.

**Problems:**
1. **Exact tile lookup is O(log N + scan)**, not O(1).
2. **Sorting by (x, y) makes y-range queries suboptimal.** Entities in the x-range but outside the y-range are visited and discarded. At low density this is fine; at high density (clustered entities) it degrades.
3. **Marginal improvement over grid.** The sorted array's cache locality is better than a HashMap, but the grid's HashMap is probed only 4-9 times. The bottleneck is not cache behavior of the grid; it is the sheer number of probes in the current tile-level design.

**Verdict:** Interesting as a secondary optimization if the grid's HashMap itself becomes a bottleneck. Could replace the grid's HashMap with a flat Vec of cells. But as a primary structure, it does not offer enough advantage. Reject as primary; consider as implementation detail.

### 4.5 Spatial Hashing with Larger Cells (= Uniform Grid with HashMap)

This is literally what I am recommending in Section 3. The current design is spatial hashing with cell size 1. The recommendation is spatial hashing with cell size 64. The name is different, the principle is the same: hash (x/C, y/C) into a HashMap.

### 4.6 Zone-Based Partitioning (GIS Quartier/Block Boundaries)

Use the existing quartier and block geometry to partition entities. Each quartier has a `Vec<Entity>`. Range queries check the current quartier plus adjacent quartiers.

**Advantages:** Zero rebuild cost if entities maintain a `quartier_id` field that is updated on movement. Quartier adjacency is precomputed.

**Problems:**
1. **Irregular shapes.** Quartiers are not axis-aligned rectangles. A range query centered near a quartier boundary may need to check 3-4 quartiers, each of which may contain hundreds of entities. The filtering cost depends on quartier size, not query radius.
2. **Quartier sizes vary wildly.** Palais de Justice: 265 buildings. Temple: 2,391 buildings. Entity counts per quartier may range from 50 to 500. A range query in a dense quartier scans 500 entities; in a sparse one, 50. Unpredictable query cost.
3. **Does not support sub-quartier queries well.** A R=30 query in a quartier that spans 1,000 meters scans all entities in the quartier even though only 60x60 meters are relevant.
4. **Maintenance cost.** Updating the quartier ID on every entity move requires a tile lookup (`tile_map.get_quartier_id`). This is O(1) per move but adds complexity.

**Verdict:** Useful for LOD zone membership queries (C02-C03), not for spatial range queries. The granularity is too coarse for R=30 sensing. Adopt for LOD filtering, not as the primary spatial index. Could serve as a pre-filter: "only run range queries on entities in the active zone" -- but that is a system-level filter, not an index structure.

### 4.7 Hierarchical Grid (Two-Level)

Level 0: coarse grid (C=64). Level 1: fine grid (C=8) within each occupied coarse cell. Exact tile lookup: probe L0 cell, then L1 cell, then linear scan. Range query: probe L0 cells, for each occupied L0 cell probe relevant L1 cells.

**Advantages:** Best of both worlds: coarse grid for range queries, fine grid for exact lookups. At 4K entities, L0 has ~100 occupied cells, each L1 grid is 8x8 = 64 sub-cells. Fine-grained filtering without HashMap overhead.

**Problems:**
1. **Rebuild is more complex.** Two levels of insertion. At 4K entities, both levels must be populated.
2. **Exact tile lookup improvement is marginal.** L1 at C=8 still requires scanning ~5 entities per sub-cell (at 40 entities per L0 cell, 64 L1 sub-cells: 0.6 entities per sub-cell on average). The current cost of scanning 40 entities is ~80ns. Improving to ~2ns is not meaningful at 50 calls per tick.
3. **Over-engineering.** The single-level grid at C=64 already solves the range query problem. The exact tile lookup problem does not exist at current and projected scale.

**Verdict:** Unnecessary complexity. Reject.

## 5. The Recommended Architecture

### 5.1 Primary Structure: Chunk-Aligned Spatial Grid

```rust
/// Spatial index using chunk-aligned cells (64x64 tiles per cell).
/// Rebuilt from positions each tick. Query cost: O(K * entities_per_cell)
/// where K = number of cells overlapping the query region (typically 4-9).
pub spatial_index: HashMap<(i32, i32), SmallVec<[Entity; 8]>>,
```

**Rebuild:**
```rust
pub fn rebuild_spatial_index(&mut self) {
    self.spatial_index.clear();
    for (&entity, pos) in &self.body.positions {
        if self.alive.contains(&entity) {
            let cell = (pos.x >> 6, pos.y >> 6); // divide by 64 via shift
            self.spatial_index
                .entry(cell)
                .or_default()
                .push(entity);
        }
    }
}
```

**Range query:**
```rust
pub fn entities_in_range(
    &self,
    cx: i32,
    cy: i32,
    range: i32,
) -> impl Iterator<Item = Entity> + '_ {
    let min_cell_x = (cx - range) >> 6;
    let max_cell_x = (cx + range) >> 6;
    let min_cell_y = (cy - range) >> 6;
    let max_cell_y = (cy + range) >> 6;

    (min_cell_y..=max_cell_y).flat_map(move |cell_y| {
        (min_cell_x..=max_cell_x).flat_map(move |cell_x| {
            self.spatial_index
                .get(&(cell_x, cell_y))
                .map(SmallVec::as_slice)
                .unwrap_or(&[])
                .iter()
                .copied()
                .filter(move |&e| {
                    if let Some(pos) = self.body.positions.get(&e) {
                        let dx = (pos.x - cx).abs();
                        let dy = (pos.y - cy).abs();
                        dx.max(dy) <= range
                    } else {
                        false
                    }
                })
        })
    })
}
```

**Exact tile lookup (unchanged in behavior, slightly more filtering):**
```rust
pub fn entities_at(&self, x: i32, y: i32) -> impl Iterator<Item = Entity> + '_ {
    let cell = (x >> 6, y >> 6);
    self.spatial_index
        .get(&cell)
        .map(SmallVec::as_slice)
        .unwrap_or(&[])
        .iter()
        .copied()
        .filter(move |e| {
            self.body.positions
                .get(e)
                .is_some_and(|pos| pos.x == x && pos.y == y)
        })
}
```

Note: The `entities_at` return type changes from `&[Entity]` to `impl Iterator<Item = Entity>`. Callers that currently use the slice must be updated to collect or iterate. This is a minor API change affecting `run_combat` and `run_eating`.

### 5.2 Performance Projections

**Rebuild cost at 4K entities:** 4,000 HashMap insertions. Each insertion: hash (i32, i32), probe bucket, push to SmallVec. ~50ns per insertion. Total: ~0.2ms. Two rebuilds per tick: ~0.4ms. Well within budget.

**Range query cost at 4K entities (R=30):** 4 cell lookups. Average 40 entities per cell. 160 entities examined. Each examination: HashMap lookup for position (~15ns) + Chebyshev comparison (~2ns) = ~17ns. Total per query: ~2.7 microseconds. At 4K queries: ~10.8ms. This is a ~4x improvement over current at 360 entities and projects linearly with entity count rather than quadratically.

Wait -- that is still 10.8ms at 4K entities. Let me re-examine. The position lookup (`self.body.positions.get(&e)`) inside the range query filter is a HashMap probe for each candidate entity. At 160 candidates per query and 4K queries, that is 640K HashMap probes. At ~15ns each: ~9.6ms. This is the new bottleneck.

**Optimization: cache position in the spatial index.** Store `(Entity, i32, i32)` instead of just `Entity` in the spatial index cells. The range filter then checks cached coordinates directly, avoiding the positions HashMap lookup.

```rust
pub spatial_index: HashMap<(i32, i32), SmallVec<[(Entity, i32, i32); 8]>>,
```

Now range query filtering is purely arithmetic: load (entity, ex, ey) from the SmallVec (cache-line-local), compute Chebyshev distance. No HashMap probe. Cost per candidate: ~2ns. At 160 candidates x 4K queries: 640K * 2ns = 1.3ms. Two phases: ~2.6ms. With rebuild: ~3.0ms total spatial overhead per tick. Excellent.

**Memory cost:** 4K entities x 16 bytes per entry (Entity: 8 bytes, two i32: 8 bytes) = 64 KB for entity data. HashMap overhead (buckets, metadata): ~2-4 KB for ~100 occupied cells. Total: ~68 KB. Fits comfortably in L2 cache (256 KB-1 MB on modern CPUs).

### 5.3 Projected Scaling

| Entity count | Rebuild (ms) | Range queries (ms) | Total spatial (ms) | Budget headroom |
|---|---|---|---|---|
| 360 (current) | 0.02 | 0.1 | 0.3 | 97% of 10ms |
| 4,000 (Phase C) | 0.2 | 1.3 | 3.0 | 70% of 10ms |
| 10,000 | 0.5 | 3.2 | 7.4 | 26% of 10ms |
| 40,000 | 2.0 | 13.0 | 30.0 | Over budget |

The design scales linearly to ~10K entities before spatial queries consume the majority of the tick budget. At 40K entities, the range query cost dominates because each entity still performs 160 candidate examinations. This can be addressed by:
1. Reducing `SENSE_RANGE` from 30 to 15 (probe 4 cells instead of 4, but ~40 candidates per cell instead of 40 -- wait, same cells, just tighter distance filter. Actually at R=15: ceil(30/64 + 1)^2 = 2^2 = 4 cells still, but filter discards more. Marginal improvement.)
2. LOD filtering: only entities in the Active zone run range queries. At 40K total entities, if only 4K are Active, query cost stays at 1.3ms.
3. Staggered queries: not every entity needs to sense every tick. Run decisions for 1/4 of entities per tick. 4x cost reduction.

The LOD filtering (option 2) is already planned for Phase C (SCALE-C03). This is the correct scaling strategy: do not optimize the spatial index for 40K range queries; instead, ensure only ~4K entities ever perform range queries.

## 6. Incremental Updates vs. Full Rebuild

Should the spatial index support incremental updates (entity moved from cell A to cell B) instead of full rebuilds?

**Arguments for incremental updates:**
- At 4K entities, ~400 entities move per tick (Walk gait: cooldown 9, so 1/9 entities move each tick = 444). Updating 444 entities is cheaper than rebuilding 4,000.
- Avoids clearing and reallocating SmallVecs.

**Arguments against incremental updates:**
- **Complexity.** Must track previous cell for each entity. Must handle spawn/despawn (entities entering/leaving the index outside the movement path). Must ensure no stale entries survive a position update that crosses cells. The current design's correctness is trivially verifiable: clear + rebuild = always consistent.
- **Movement phase mutability conflict.** The wander system mutates `body.positions` while iterating entities. If the spatial index must be updated on each position change, it must be mutated during the same phase that writes positions. This requires careful ordering or a deferred-update pattern (collect moves, apply moves, update index) -- which is effectively a rebuild but more complex.
- **Savings are modest.** Full rebuild at 4K entities: 0.2ms. Incremental update of 444 entities: ~0.04ms. Savings: 0.16ms per tick. Not meaningful.
- **The `spatial_index.clear()` call reuses allocated memory.** HashMap::clear does not deallocate buckets. SmallVec inline storage is not heap-allocated. The rebuild's cost is dominated by HashMap insertions, not allocation.

**Verdict:** Full rebuild is the correct model for Wulfaz's scale. The simplicity guarantee (index is always consistent after rebuild, no stale entries possible) is worth the 0.2ms cost. Revisit only if entity count exceeds 20K active entities, which the LOD design explicitly prevents.

## 7. Integration with LOD Zones

The Phase C LOD framework (SCALE-C02) partitions the map into Active/Nearby/Statistical zones. Only Active-zone entities run full systems including range queries.

**Zone membership query:** "Is this entity/tile in the Active zone?" This should NOT be handled by the spatial index. It should be a per-entity flag (`LocomotionZone` enum on a new property table) or derived from the entity's quartier and the active zone's quartier set. Zone transitions happen on camera movement, not every tick. The flag is updated during the zone-transition system (Phase 1 or pre-phase), not during spatial index rebuild.

**Spatial index filtering by zone:** The spatial index should contain ALL positioned entities (Active + Nearby). Statistical entities have no positions (they are aggregates, not individual entities on the map). The systems that perform range queries should filter by zone flag:

```rust
// In decisions system:
.filter(|e| world.is_active(e))  // skip Nearby entities for sensing
```

This keeps the spatial index zone-agnostic and pushes filtering to the system level, consistent with the blackboard architecture (systems decide what to process, the index just answers spatial queries).

**Hydration/dehydration and the spatial index:** When entities are hydrated (spawned from statistical aggregates), they are added to `body.positions`. On the next spatial index rebuild, they appear in the index automatically. No special handling needed. When entities are dehydrated, they are despawned. On the next rebuild, they vanish from the index. The full-rebuild model makes hydration/dehydration trivially correct.

## 8. Memory Layout Considerations

The `HashMap<(i32, i32), SmallVec<[(Entity, i32, i32); 8]>>` layout:
- HashMap bucket array: contiguous allocation, ~100 entries at 4K entities. ~800 bytes. Cache-friendly for cell lookups.
- SmallVec with inline capacity 8: entities 0-8 are stored inline in the HashMap value, no heap allocation. At average 40 entities per cell, the SmallVec spills to heap. The heap allocation is a single contiguous block of `40 * 16 = 640 bytes`, which fits in a cache line pair.

For range queries scanning 4 cells: 4 HashMap probes (bucket array access + SmallVec access). The SmallVec heap pointer is followed, loading 640 bytes of entity data. Total memory touched: ~2.6 KB per range query. At 4K queries: ~10.4 MB of memory traffic. This fits within L3 cache bandwidth.

**Alternative: flat Vec-based grid.** Instead of HashMap, use a flat `Vec<SmallVec<...>>` indexed by `(cell_y * chunks_x + cell_x)`. At 99 x 75 chunks: 7,425 entries x (SmallVec inline = 128 bytes with 8 inlined tuples) = ~950 KB. This is larger than the HashMap approach (which only allocates for occupied cells) but provides O(1) lookup with zero hashing overhead.

At 4K entities and ~100 occupied cells, the HashMap has ~100 entries. The flat Vec has 7,425 entries, 7,325 of which are empty SmallVecs (24 bytes each for the empty inline variant = ~176 KB wasted). The HashMap approach is more memory-efficient for sparse occupation.

**The constraint interpretation matters here.** CLAUDE.md says "Do not replace HashMap with another data structure without profiling data showing >5ms per tick for that system." The spatial index is currently a HashMap. Changing the cell size is a parameter change, not a data structure change. The type remains `HashMap<(i32, i32), SmallVec<...>>`. This change does not violate the constraint.

If we wanted to use a flat Vec instead of HashMap for the grid: this would replace a HashMap with a Vec. The constraint applies. We would need profiling data showing the HashMap-based coarse grid exceeds 5ms per tick before switching to Vec. Given the projections (~3ms total at 4K entities), this threshold is unlikely to be reached. The HashMap-based coarse grid is the correct choice within the existing constraints.

## 9. Pathfinding Infrastructure

Pathfinding (A* on the tile grid) is orthogonal to the entity spatial index. A* queries the TileMap for walkability, not the spatial index for entity positions. However, two spatial concerns arise:

**Entity collision during pathfinding:** Currently, entities can stack on the same tile. If future systems prevent tile stacking (blocking movement into occupied tiles), the spatial index must support "is tile (x, y) occupied?" queries efficiently. The coarse grid handles this via the same filtering approach as `entities_at`: check the cell, filter by exact tile. At low call frequency (only during movement phase), this is fine.

**HPA* (Phase D) and chunk alignment:** HPA* precomputes a graph of chunk border nodes with intra-chunk path costs. This aligns naturally with the chunk-aligned spatial grid. The same chunk coordinate system (`pos >> 6`) serves both the spatial index and the HPA* hierarchy. This is a design advantage of choosing C=64.

## 10. What Breaks and When

**At 4K entities (Phase C target):** Nothing breaks. The coarse grid handles this with ~3ms total spatial overhead per tick. LOD filtering ensures only ~4K entities are Active. The design is comfortable.

**At 10K active entities:** Range queries consume ~7ms of the 10ms tick budget. This is tight but viable if other systems are lean. The mitigation is staggered sensing (not every entity senses every tick).

**At 40K active entities:** The spatial index itself is no longer the problem (rebuild is 2ms, manageable). The problem is 40K range queries at 3.2 microseconds each = 128ms. This is fundamentally O(N^2) behavior (each of N entities queries a region containing O(N/cells) entities). No spatial index can fix this; the solution is to reduce the number of queries (LOD filtering, staggered computation, approximate sensing).

**If the entire 30M tile map had entities:** At 1M entities (the full statistical population), rebuild is ~50ms and range queries are astronomically expensive. This scenario is explicitly prevented by the LOD design. The spatial index is designed for the Active zone only.

## 11. Summary of Recommendations

1. **Change the spatial index cell size from 1 tile to 64 tiles (chunk-aligned).** Same HashMap type, same rebuild logic, different coordinate divisor. Reduces range query probes from 3,721 to 4-9.

2. **Store `(Entity, i32, i32)` tuples in index cells** instead of bare `Entity`. Eliminates positions HashMap lookups during range query filtering.

3. **Keep full rebuild, do not implement incremental updates.** Rebuild cost at 4K entities is 0.2ms. Correctness guarantee of clear+rebuild is worth the cost.

4. **Change `entities_at` return type** from `&[Entity]` to `impl Iterator<Item = Entity>` that filters the cell by exact tile coordinates. Minor API change.

5. **Do not add zone awareness to the spatial index.** Keep it zone-agnostic. Push LOD filtering to the system level.

6. **Do not adopt quadtrees, R-trees, k-d trees, or hierarchical grids.** The complexity/benefit ratio is unfavorable at Wulfaz's scale. The coarse uniform grid is optimal for this specific combination of extreme sparsity, small N, moderate R, and frequent bulk rebuilds.

7. **The "HashMap constraint" in CLAUDE.md is satisfied.** The recommended change modifies cell size, not the data structure. The type remains `HashMap<(i32, i32), SmallVec<...>>`.

8. **Future escape hatch:** If Active entity count exceeds 10K (which the LOD design prevents), the next step is staggered sensing (run range queries for 1/K of entities per tick, cycling through all entities over K ticks). This is a system-level change, not an index change.

The optimal spatial infrastructure for Wulfaz is the simplest one that eliminates the measured bottleneck: a coarse uniform grid at chunk granularity, with cached positions, using the existing HashMap type. Everything else is premature complexity that the LOD framework will render unnecessary.
