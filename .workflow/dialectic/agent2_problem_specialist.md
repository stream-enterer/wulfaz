# Performance Budget Analysis: 10ms/tick at Scale

## Problem Statement

A single-threaded simulation engine running at 100 ticks/sec has a hard budget
of 10,000 microseconds per tick. The architecture is fixed: HashMap-based EAV
property tables, deterministic replay via sorted iteration, collect-then-apply
mutation, sequential phase execution, and two spatial index rebuilds per tick.
The question is what concrete rules on data structure selection, per-system
complexity ceilings, and computation scheduling keep every system within budget
as entity count scales from 1k to 50k.

The benchmark evidence is unambiguous about where time goes:

| System | 50k entities | % of tick |
|---|---|---|
| wander (A* per entity) | 75,558us (before), 9,120us (after cache) | 91.2% / 91.2% |
| eating (spatial queries) | 28,551us / 25,004us | -- |
| combat (spatial queries) | 5,821us | -- |
| spatial rebuild x2 | 5,801us / 5,520us | -- |
| hunger (iterate+mutate) | 2,170us | -- |
| fatigue (iterate+mutate) | 726us | -- |
| temperature (tile-only) | 9us | -- |

Even after path caching brought wander from 75ms to 9ms, the total tick at 50k
still exceeds 10ms. The problem is not any single system -- it is the sum of
systems that each seem cheap in isolation.

---

## Section 1: Complexity Ceilings by Phase

### Phase 1 (Environment): O(tiles), amortized O(1)

Phase 1 operates on the tile map, not on entities. The existing temperature
system demonstrates the correct pattern: chunk-level equilibrium flags make
steady-state cost O(1). At 256x256 (65,536 tiles, 64 chunks), even full-scan
cost is ~9us -- negligible.

**Rule: Phase 1 systems must operate on tiles/chunks only, never iterate
entities. Any Phase 1 system that touches entities is misclassified and must
move to Phase 2+.**

**Rule: Phase 1 systems must implement chunk-level dirty/equilibrium flags.
Full tile iteration is permitted only for chunks not at equilibrium. Steady-state
cost must be O(active_chunks), not O(total_tiles).**

### Phase 2 (Needs): Strict O(n) with constant-factor discipline

Hunger and fatigue are pure O(n): iterate the relevant table, compute
arithmetic on each entry, collect changes, apply. At 50k entities, hunger
takes 2,170us and fatigue takes 726us. The 3x difference between them
(despite identical algorithmic complexity) comes from hunger pushing events
per entity while fatigue only pushes events conditionally.

The important observation: at 50k entities, a system that does 50ns of work
per entity costs 2,500us. Two such systems already consume 5,000us --
half the budget. Phase 2 cannot afford any system doing more than ~100ns
per entity.

**Rule: Phase 2 systems must be O(n) with no HashMap lookups beyond the
primary iteration table. Each entity's computation must be arithmetic on
values already in hand (the table entry being iterated, plus at most one
secondary lookup). No spatial queries. No pathfinding. No allocation.**

**Rule: Phase 2 systems that push events per entity must account for the
EventLog push cost in their budget. At 50k entities, unconditional event
pushes add ~1,000us. Push events only on state transitions (value crosses
a threshold), not on every increment.**

**Concrete ceiling: Any Phase 2 system exceeding 3,000us at 50k entities
needs optimization or must move computation to a less-frequent schedule.**

### Phase 3 (Decisions): O(n x k) where k is bounded action count, plus O(n x spatial) for target selection

The decisions system has two distinct cost profiles:

1. **Scoring**: For each entity, evaluate k actions x c considerations each.
   With k=4 actions and c~3 considerations, this is O(n x 12) -- effectively
   O(n). The pruning (zero-product cutoff, max-score cutoff) reduces this
   further. Pure arithmetic; cheap.

2. **Spatial queries for input axes**: `FoodNearby` and `EnemyNearby` each
   call `entities_in_range(pos, 30)`. With 16m cells, range=30 scans
   ceil(30/16)=2 cells per axis = 4 cells. At uniform density on 256x256,
   50k entities yield ~12 entities/cell, so each query touches ~48 entities.
   Two queries per entity = 96 comparisons per entity = 4.8M comparisons total.

3. **Target selection**: `select_eat_target` and `select_attack_target` each
   do another `entities_in_range(30)` query plus sorting by distance. This is
   only called for entities that chose Eat or Attack (typically a subset), but
   in the worst case (all entities hungry with food nearby), it doubles the
   spatial query load.

**Rule: Phase 3 scoring must use only O(1) per-entity lookups for input axes
that read entity state (HungerRatio, HealthRatio, Aggression). Spatial input
axes (FoodNearby, EnemyNearby) are the expensive ones and must be bounded.**

**Rule: Spatial input axes in Phase 3 must use SENSE_RANGE as small as gameplay
permits. Every doubling of SENSE_RANGE quadruples the number of cells scanned.
Current SENSE_RANGE=30 with CELL_SIZE=16 scans 4 cells; SENSE_RANGE=64 would
scan 25 cells -- 6x more expensive.**

**Rule: Target selection (select_eat_target, select_attack_target) must not
re-scan the spatial index if the scoring phase already determined proximity.
Cache the nearest-target result from the spatial query used by the input axis,
and pass it through to target selection. This eliminates a redundant O(n) scan.**

**Concrete ceiling: Phase 3 must stay under 4,000us at 50k. The current
implementation likely exceeds this when most entities score Eat or Attack
(three spatial queries each). The fix is query deduplication.**

### Phase 4 (Actions): O(n_active x path_cost), amortized via caching

This is the catastrophic phase. Before optimization, wander alone was 75ms
at 50k -- 7.5x the entire tick budget. Path caching brought it to 9ms,
but that is still 90% of budget consumed by one system.

The key insight from the wander system is that most entities are on cooldown.
At Walk gait (9 ticks/tile), only 1/9 of entities attempt movement on any
given tick. Of those that move, most have cached paths and consume a single
Vec step -- O(1). Only entities that need a fresh A* call are expensive.

A* with flat arrays and MAX_EXPANDED=32,768 on a 256x256 map costs up to
~32K node expansions. Each expansion does 8 neighbor checks with bounds/walkability
tests. Empirically, a single find_path call on this map costs 50-200us depending
on path length and obstacle density.

**Rule: Phase 4 systems must not call find_path per entity per tick. Paths
must be cached as components (CachedPath) and reused across ticks until
invalidated. A fresh A* is computed only when: (a) no cached path exists,
(b) the goal changed, or (c) the path is blocked (obstacle appeared on
next step).**

**Rule: A* calls per tick must be capped. At 200us per call, 50 calls
consume the entire tick budget. The system must limit fresh pathfinding to
at most N_PATH_BUDGET calls per tick (suggest: 30-50). Entities that exceed
the budget fall back to random stepping and retry next tick.**

**Rule: For tracking intentions (Eat/Attack), do NOT invalidate the cached
path every tick just because the target might have moved. Instead, check
if the target moved more than K tiles from the cached goal. If K=3
(one step's worth of distance), re-path. Otherwise, reuse. The current
code invalidates every tick for tracking targets -- this is the performance
hole that makes eating at 25ms at 50k.**

**Rule: Movement cooldowns naturally throttle pathfinding. At Walk gait (9
ticks/tile), only ~11% of entities request movement per tick. Systems must
not bypass this natural rate-limiter. Never pre-compute paths for entities
still on cooldown.**

**Concrete ceiling: Phase 4 must stay under 3,000us at 50k entities. This
requires: (a) path caching with lazy invalidation, (b) per-tick A* call cap,
(c) cooldown-gated processing.**

### Phase 5 (Consequences): O(n_affected), not O(n_total)

The death system iterates only `pending_deaths`, which is typically a tiny
fraction of entities. Combat iterates combatants at same tile. Eating iterates
hungry entities at food tiles. These are naturally bounded by the number of
state changes in Phase 4.

**Rule: Phase 5 systems must iterate only the affected set, never the full
entity population. Use pending_deaths, spatial co-location, or event-driven
triggers to identify the affected set.**

**Concrete ceiling: Phase 5 should be under 500us at 50k entities under
normal conditions. Mass death events (e.g., all entities starving) may spike
this, but such conditions are transient.**

---

## Section 2: Data Structure Decision Tree

Given the fixed constraint that property tables are HashMap<Entity, T>,
the question is when to use alternatives for auxiliary data.

### Decision: HashMap<Entity, T> (Property Tables)

**Use when:**
- Key is Entity (unbounded, sparse ID space)
- Access pattern is point lookup by entity ID
- Mutation is per-entity, per-tick or on-change
- This is the default for all per-entity state

**Cost model:**
- Single lookup: ~50-80ns (hash + probe, cache-cold)
- Iteration: O(n) but cache-unfriendly (pointer chasing)
- Iteration + sort: O(n log n) due to determinism requirement

**The sort tax**: Every system that iterates a HashMap and needs determinism
pays O(n log n) for sorting keys. At 50k entities, n log n = ~780k comparisons.
At ~5ns per comparison, that is ~3,900us -- nearly 40% of budget on sorting
alone. This is the single largest hidden cost in the architecture.

**Rule: Systems must sort entity IDs only once per system invocation, not per
inner loop. Collect keys into a Vec, sort once, then iterate the sorted Vec
for all processing.**

**Rule: If a system iterates the same sorted entity set as a preceding system
in the same phase, consider passing the sorted Vec between them (as a field
on World or a return value). Do NOT re-sort.**

### Decision: Flat Array (Indexed by Tile Coordinate)

**Use when:**
- Key space is bounded and dense (tile coordinates within map dimensions)
- Access pattern is point lookup or local neighborhood scan
- Example: A* g_score/came_from/closed arrays, temperature data, walkability

**Cost model:**
- Single lookup: ~5ns (array index, cache-friendly)
- Neighborhood scan (8 neighbors): ~40ns
- Allocation: O(width x height) per call -- amortize by reusing buffers

**Rule: When key space maps to tile coordinates (bounded, dense), always use
a flat array indexed by y * width + x. Never use HashMap<(i32, i32), T> for
tile-indexed data.**

**Rule: A* must use flat arrays for g_score, came_from, and closed set. The
existing implementation already does this correctly. HashMap-based A* would be
~10x slower due to hash overhead per node expansion.**

**Rule: If A* is called multiple times per tick, consider pooling the flat
arrays (allocate once, clear between calls) rather than allocating per call.
At 256x256, each vec![u32::MAX; 65536] allocates 256KB. Pooling eliminates
allocation overhead for subsequent calls.**

### Decision: SpatialGrid (HashMap<(i32, i32), Vec<(Entity, i32, i32)>>)

**Use when:**
- Query pattern is "all entities near position (x, y)"
- Range is small relative to cell size (1-3 cells per axis)
- Used by: eating (same-tile), combat (same-tile), decisions (SENSE_RANGE)

**Cost model:**
- Rebuild: O(n) -- iterate all positions, insert into grid. ~2,700us at 50k.
- Point query (entities_at): O(density) per cell, typically O(1)-O(20)
- Range query (entities_in_range): O(cells_scanned x density_per_cell)
  - SENSE_RANGE=30, CELL_SIZE=16: scans 4 cells, ~48 entities
  - SENSE_RANGE=3, CELL_SIZE=16: scans 1 cell, ~12 entities

**Rule: The spatial index must be rebuilt at most twice per tick (before
needs/decisions, before eating/combat). Each rebuild costs ~2,700us at 50k.
Never add a third rebuild without profiling evidence that the cost is justified.**

**Rule: Same-tile queries (entities_at) should be preferred over range queries
(entities_in_range) when the gameplay mechanic requires co-location (eating,
combat). Same-tile scans one cell; range queries scan up to (2*ceil(range/cell)+1)^2
cells.**

### Decision: Pre-sorted Vec (Cached Sorted Entity Lists)

**Use when:**
- Multiple systems need the same sorted entity set
- The set changes slowly (entities spawn/die, but the living set is mostly stable)

**Cost model:**
- Initial sort: O(n log n) = ~3,900us at 50k
- Incremental maintenance: O(log n) per insert/remove with binary search

**Rule: If the sort-by-entity-ID cost exceeds 2,000us in profiling, consider
maintaining a pre-sorted alive entity list on World, updated incrementally
in spawn() and despawn(). This amortizes the sort across all systems.**

### Decision: HashSet<Entity>

**Use when:**
- Membership test: "is entity X in set Y?"
- No iteration needed, or iteration order does not matter
- Example: pending_deaths, alive, consumed (in eating)

**Cost model:**
- Insert/lookup: ~50-80ns
- Iteration: O(n) but unordered

**Rule: Use HashSet for membership-test-only sets (pending_deaths, consumed).
Never use Vec::contains for membership testing -- it is O(n) per check,
turning an O(n) system into O(n^2). The benchmark showed eating at 28ms
partially because of this pattern.**

---

## Section 3: Computation Scheduling Patterns

Not every entity needs every computation every tick. The simulation has natural
rate-limiters built into its mechanics that can be exploited.

### Pattern 1: Cooldown-Gated Processing

**Observation:** Movement cooldown at Walk gait means entities move once every
9 ticks. For 8 of 9 ticks, the wander system checks cooldown > 0, decrements,
and skips -- O(1) per entity. Only on the 9th tick does the entity need
pathfinding or movement resolution.

**Rule: Any system with a built-in cooldown/timer mechanism should test the
timer first and early-exit. The "decrement and skip" path must be O(1) with
no secondary lookups, no spatial queries, and no allocation.**

**Quantitative impact:** At Walk gait, only 11% of entities are active per
tick. At 50k entities, 5,556 are active. If each active entity costs 200us
of A* time, the amortized cost is (5,556 x 200 x probability_of_needing_fresh_path).
With cached paths, the probability drops further to ~10% (only entities
reaching their goal or having invalidated paths). This yields: 5,556 x 0.1 x
200us = ~111us amortized. The theoretical floor.

### Pattern 2: Batch-Accumulate for Linear Growth

**Observation:** Hunger increases by 1.0/tick. If a system only cares about
hunger crossing a threshold (e.g., hunger > 50% for eating behavior), it does
not need per-tick resolution. A "hunger_next_threshold_tick" component could
skip the entity entirely until tick T when hunger crosses the threshold.

**Rule: For needs that grow linearly and are consumed discretely (hunger,
fatigue recovery), consider computing the next-event tick at the time of the
last state change, and skipping the entity until that tick.**

**Implementation:** Add a field `next_check_tick: Tick` to the component. On
each tick, the system filters entities where `tick >= next_check_tick`. When
hunger is consumed (eating), recompute: `next_check_tick = current_tick +
(threshold - current_hunger) / rate`. This turns Phase 2 from O(n) to
O(events_this_tick).

**Caveat:** This breaks the simple "iterate table, apply arithmetic" pattern
and introduces tick-prediction complexity. It is only worth doing when the
O(n) cost of the simple approach exceeds ~2,000us. At 50k entities with hunger
at 2,170us, this is marginal. At 100k entities, it would be essential.

**Rule: Do not batch-accumulate unless profiling shows the simple O(n) approach
exceeds 2,000us for that system. Premature optimization here adds complexity
without meaningful gain.**

### Pattern 3: Staggered Entity Processing

**Observation:** Not all entities need decisions every tick. An entity that
chose Wander and is mid-path does not need to re-evaluate utility scores
until: (a) it arrives at its destination, (b) a significant state change
occurs (health drops, food appears nearby), or (c) a fixed interval elapses.

**Rule: The decisions system should support an "inertia period" -- entities
whose action did not change for N consecutive ticks skip scoring for the
next M ticks. The inertia_bonus mechanism already encourages stability;
this extends it to skip computation entirely.**

**Implementation:** Add `next_decision_tick: Tick` to ActionState. After
scoring, if the action is unchanged and has been stable for 10+ ticks, set
`next_decision_tick = current_tick + 5`. The system skips entities where
`tick < next_decision_tick`. This reduces the scoring population by up to
80% in stable simulations.

**Caveat for determinism:** The skip decision must be purely a function of
state on World (no external input). Since next_decision_tick is stored on
the entity and updated deterministically, replay is preserved.

### Pattern 4: Spatial Query Amortization

**Observation:** The decisions system makes 2-3 spatial queries per entity
(FoodNearby, EnemyNearby, target selection). These queries overlap -- they
scan the same cells for the same entity position. The results can be cached
within the system's execution.

**Rule: Within a single system invocation, cache spatial query results per
entity position. If multiple entities are at the same position (or within the
same spatial cell), reuse the query result. At high density, many entities
share cells.**

**Implementation:** Build a local `HashMap<(i32, i32), (food_count, enemy_count,
Option<Entity>, Option<Entity>)>` keyed by cell coordinate. On first query from
a cell, populate it. Subsequent entities in the same cell read from cache. At
uniform density with 50k entities on 256x256, each 16x16 cell contains ~12
entities. If 12 entities share a query result, spatial query cost drops by 12x.

**Rule: For decisions specifically, compute all spatial inputs and target
selections in a single pass over the spatial index, then use the results
during scoring. Do not interleave spatial queries with scoring loops.**

---

## Section 4: Cache-as-Component Pattern

### When to Cache

**Rule: Cache an expensive computation as a component when ALL of the following
hold:**
1. The computation costs more than 100us per invocation (e.g., A* pathfinding)
2. The result is valid for multiple ticks (e.g., a path remains valid until
   the goal changes or an obstacle appears)
3. The invalidation condition can be checked in O(1) (e.g., "did the goal
   position change?")

**Current instances:**
- `CachedPath` (steps + goal): caches A* result, valid until goal changes or
  path steps are exhausted. Cost avoided: 50-200us per pathfinding call.
- `WanderTarget` (goal_x, goal_y): caches the random destination, valid until
  arrival. Cost avoided: 5 random rolls + walkability checks.

### What Invalidates the Cache

Under the blackboard model, any system can write any state. This makes cache
invalidation non-trivial because the writing system does not know who depends
on the cached value.

**Rule: Cache invalidation must be checked by the consuming system, not the
producing system. The system that reads CachedPath is responsible for checking
if the path is still valid before using it.**

**Rule: Invalidation checks must compare the cache's assumptions against
current world state. For CachedPath:**
- `cached.goal != current_goal` --> invalidate (goal changed)
- `!tiles.is_walkable(cached.steps[0])` --> invalidate (obstacle on next step)
- `cached.steps.is_empty()` --> invalidate (path exhausted)

**Rule: Never invalidate a cache based on tick count alone ("re-path every N
ticks"). This wastes computation when the path is still valid and misses
invalidation when the path becomes invalid before N ticks.**

### Expressing Invalidation Safely

**Rule: Every cached component must include the assumptions it was computed
under. CachedPath includes `goal: (i32, i32)` -- this is the assumption. If
the entity's goal changes (intention target moved, new wander target picked),
the consuming system detects the mismatch and recomputes.**

**Rule: For caches that depend on spatial state (e.g., "is there food within
SENSE_RANGE?"), store a summary of the spatial state at computation time
(e.g., `food_count_at_decision: u8`). On subsequent ticks, compare against
current spatial state. Only recompute if the summary differs. This avoids
full spatial rescans for cache validation.**

### New Cache Opportunities

**Nearest-target cache for decisions:** Store `NearestFood(Entity, Tick)` and
`NearestEnemy(Entity, Tick)` as components. The decisions system writes them;
subsequent ticks read them and only rescan if the target entity is dead or the
querying entity has moved more than CELL_SIZE tiles from where the scan was done.

**Influence map for spatial queries:** Instead of per-entity spatial queries
in decisions, maintain a coarse "food density" and "enemy density" grid at
spatial-cell resolution, updated once per tick during spatial rebuild. Each
cell stores count of food items and combatants. Input axes read from this grid
in O(1) instead of scanning neighbors.

---

## Section 5: Spatial Query Cost Model

### The Fundamental Equation

Total spatial query cost per tick = N_queries x cells_per_query x entities_per_cell x cost_per_comparison

With current parameters:
- N_queries = n_entities x queries_per_entity
- cells_per_query = (2 * ceil(range / cell_size) + 1)^2
- entities_per_cell = n_entities / (map_area / cell_area)
- cost_per_comparison = ~10-20ns (distance check + filter predicates)

At 50k entities, SENSE_RANGE=30, CELL_SIZE=16, map=256x256:
- cells_per_query = (2 * 2 + 1)^2 = 25... wait, let me recompute.
  range=30, cell_size=16. min_cell = (cx-30) >> 4, max_cell = (cx+30) >> 4.
  Cell span = floor((cx+30)/16) - floor((cx-30)/16) + 1.
  For cx=128: (128+30)/16=9, (128-30)/16=6, span=4. Both axes: 4x4=16 cells.
  But for cx near cell boundary: could be 5x5=25 cells.
- entities_per_cell at uniform density: 50000 / (256/16)^2 = 50000 / 256 = ~195.
  Wait. 256x256 map with 16x16 cells = 16x16 = 256 cells. 50000/256 = 195 entities/cell.

This is much worse than the prompt's estimate of 12/cell. Let me recheck.
The spatial cell size is 2^4 = 16 tiles. Map is 256x256. Number of cells =
(256/16)x(256/16) = 16x16 = 256 cells. 50k entities / 256 cells = 195
entities per cell at uniform density.

A SENSE_RANGE=30 query scans ~16 cells x 195 entities = 3,120 entity
comparisons per query. At 50k entities each doing 2 queries (FoodNearby +
EnemyNearby), that is 50k x 2 x 3,120 = 312M comparisons. At 10ns each,
that is 3.12 seconds. This is obviously impossible -- the benchmark shows
decisions running in single-digit milliseconds, so either the density is not
uniform (entities cluster in walkable areas, which is denser) or the pruning
(zero-product cutoff) eliminates most spatial queries.

**Revised analysis:** The utility scoring prunes aggressively. If the first
consideration for Eat is HungerRatio and the entity is not hungry, the
zero-product cutoff skips the FoodNearby evaluation entirely. Similarly for
Attack. In practice, only a fraction of entities evaluate spatial input axes.

But in the worst case (all entities hungry, food present), the spatial query
cost is catastrophic. This is exactly what the 25ms eating benchmark shows.

**Rule: Spatial queries at SENSE_RANGE scale must NEVER be called for all n
entities. The calling system must pre-filter to a subset using O(1) checks
(hunger threshold, action cooldown, intention type) before issuing spatial
queries.**

**Rule: SPATIAL_CELL_SHIFT should be tuned to keep entities_per_cell reasonable.
With 50k entities on 256x256, CELL_SIZE=16 gives 195/cell. CELL_SIZE=4 would
give 50000 / (64*64) = 12/cell but 4096 cells. The optimal cell size balances
entities_per_cell (lower = cheaper per query) against cells_per_query (smaller
cells = more cells scanned). For SENSE_RANGE=30 and target density ~20/cell:
CELL_SIZE = sqrt(50000 * 16^2 / (20 * 256^2))... the math suggests CELL_SIZE=8
(SPATIAL_CELL_SHIFT=3) is better: 50000/1024=49/cell, query scans (30/8+1)^2=25
cells. Total = 25 * 49 = 1,225 comparisons. Much better than 3,120.**

**Rule: When entity count exceeds 10k, SPATIAL_CELL_SHIFT must be reviewed.
The optimal value is approximately: cell_size = sqrt(entity_count / target_density)
/ (map_size / target_query_range). For practical purposes, CELL_SHIFT=3 (8m cells)
is better than CELL_SHIFT=4 (16m cells) for entity counts above 5k on a 256x256
map.**

---

## Section 6: Scaling Walls

### Wall 1: Sort Tax -- O(n log n) per system

Every system that iterates a HashMap sorts by entity ID for determinism. At
50k entities, sorting a Vec<Entity> takes ~1,500us (measured by extrapolation:
50k x log2(50k) x 5ns = 50k x 16 x 5ns = 4,000us). With 5+ systems each
sorting independently, the total sort cost approaches 8,000-20,000us.

**Mitigation:** Maintain a single pre-sorted alive list on World. Update it
in spawn() (binary-search insert, O(log n)) and despawn() (binary-search
remove, O(n) for shift, but despawn is rare). Systems read this list instead
of collecting+sorting HashMap keys.

### Wall 2: Spatial Rebuild -- O(n) per rebuild, twice per tick

Two rebuilds at ~2,700us each = 5,400us. This is structural: every position
must be inserted into the grid. At 50k entities, 5,400us is 54% of the budget.

**Mitigation: Incremental spatial index.** Instead of clearing and rebuilding,
track position changes from the wander system (it already collects `moves: Vec<(Entity, Position)>`).
Remove moved entities from their old cells, insert into new cells. Cost =
O(moved_entities) per rebuild instead of O(all_entities). At Walk gait,
~11% of entities move per tick = 5,500 moves. Incremental update: 5,500 x 2
operations x ~50ns = ~550us. 10x improvement over full rebuild.

The first rebuild (before decisions) still needs to be full if entities
spawned or died since last tick. But spawn/death counts are typically tiny
compared to the living population.

**Rule: The spatial index must support incremental updates. Full rebuild is
only needed on the first tick or after structural changes (mass spawn/despawn).
The wander system must output its move list for the subsequent incremental
rebuild.**

### Wall 3: HashMap Iteration Cache Hostility

HashMap iteration touches memory in hash-table order, not insertion order. At
50k entries, each HashMap has ~50k slots spread across ~64KB-128KB of memory
(Entity is 8 bytes, typical value is 8-32 bytes, bucket overhead). Iterating
body.positions touches ~400KB. Iterating mind.hungers touches ~400KB. Each
system that iterates a different table thrashes L1/L2 cache.

**Mitigation:** This is structural to the HashMap choice and cannot be
eliminated without changing to SoA arrays. The practical mitigation is to
minimize the number of distinct table iterations per tick. Systems should
iterate one "primary" table and do point lookups (get) into secondary tables,
rather than iterating multiple tables and joining them.

**Rule: Each system must have exactly one "driving table" that it iterates.
All other data access must be via point lookup (HashMap::get). Never iterate
two tables and join them -- the cache cost is multiplicative.**

### Wall 4: A* Allocation -- O(map_size) per call

Each find_path call allocates three vectors of map_size (width x height):
g_score, came_from, closed. At 256x256, that is 3 x 65,536 x 4 bytes =
768KB per call. At 50 calls per tick, that is 37.5MB of allocation and
deallocation per tick.

**Mitigation:** Pool the A* buffers. Allocate once, clear between calls.
Use `vec.fill(u32::MAX)` and `vec.fill(false)` instead of reallocating.
The fill cost is O(map_size) but with no allocator overhead.

Better: use a generation counter instead of clearing. Each call increments a
generation. Instead of `closed[i] = false` for all i, check `closed_gen[i] ==
current_gen`. This makes "clear" O(1).

**Rule: A* pathfinding must use pooled buffers with generation-counter clearing.
Never allocate fresh vectors per find_path call. Store the buffers on TileMap
or in a dedicated PathfindingPool struct on World.**

### Wall 5: Collect-Then-Apply Allocation

Every system collects changes into Vecs before applying. At 50k entities, a
Vec<(Entity, f32, f32)> for hunger changes is 50k x 20 bytes = 1MB. Multiple
systems each allocating 1MB Vecs per tick creates allocation pressure.

**Mitigation:** Pre-allocate change buffers on World or use `Vec::with_capacity`
based on the expected entity count. Retain the Vec across ticks (clear, don't
drop). This eliminates per-tick allocation.

**Rule: Systems should accept or store pre-allocated buffers for their collect
phase. Use `buffer.clear()` at the start of each tick, not `let mut buffer =
Vec::new()`. This eliminates allocator overhead for stable entity counts.**

---

## Section 7: Concrete CLAUDE.md Rules (Summary)

These are the rules distilled from the above analysis, formatted for direct
inclusion in project documentation.

### Complexity Ceiling Rules

1. **Phase 1 systems must not iterate entities.** Operate on tiles/chunks only.
   Implement chunk-level dirty flags for O(active_chunks) steady-state cost.

2. **Phase 2 systems must be O(n) with at most one secondary HashMap lookup
   per entity.** No spatial queries. No pathfinding. No allocation beyond the
   collect buffer. Budget: 3,000us at 50k entities.

3. **Phase 3 must pre-filter entities with O(1) checks before issuing spatial
   queries.** Never call entities_in_range for all n entities unconditionally.
   Budget: 4,000us at 50k entities.

4. **Phase 4 must not call find_path per entity per tick.** Cache paths as
   components; invalidate on goal change or path blockage. Cap fresh A* calls
   at 50 per tick. Entities exceeding the cap fall back to random stepping.
   Budget: 3,000us at 50k entities.

5. **Phase 5 must iterate only affected entities (pending_deaths, co-located
   pairs), never the full population.** Budget: 500us at 50k entities.

### Data Structure Rules

6. **Use flat arrays (indexed by y * width + x) for all tile-coordinate-keyed
   data.** Never HashMap<(i32, i32), T> for spatial data.

7. **Use HashSet for membership testing (pending_deaths, consumed sets).**
   Never Vec::contains for O(n) membership checks.

8. **Pool A* buffers with generation-counter clearing.** Never allocate fresh
   vectors per find_path call.

9. **Pre-allocate collect-then-apply buffers.** Use buffer.clear(), not
   Vec::new(), per tick.

10. **Maintain a pre-sorted alive entity list on World if sort cost exceeds
    2,000us per system at target entity count.** Update incrementally in
    spawn()/despawn().

### Scheduling Rules

11. **Systems with cooldown mechanics must test the timer first and early-exit
    in O(1).** No secondary lookups for entities on cooldown.

12. **Decision scoring must support inertia-based skip: entities stable in
    their action for 10+ ticks skip scoring for 5 ticks.** Store
    next_decision_tick on ActionState.

13. **Spatial input axes in decisions must cache results per cell, not per
    entity.** Entities in the same spatial cell share query results.

14. **Cache invalidation must be checked by the consuming system, comparing
    cached assumptions against current world state.** Never invalidate on
    timer alone.

### Spatial Index Rules

15. **At most two spatial index rebuilds per tick.** Never add a third without
    profiling justification.

16. **Consider incremental spatial updates when entity count exceeds 10k.**
    Track moved entities from wander; update cells O(moved) instead of
    rebuild O(all).

17. **Review SPATIAL_CELL_SHIFT when entity count changes by 4x or more.**
    Optimal cell size depends on entity density and query range.

18. **SENSE_RANGE must be the minimum required by gameplay.** Every doubling
    of range quadruples spatial query cost. Document the gameplay justification
    for any SENSE_RANGE > 16.

---

## Section 8: Budget Allocation at 50k Entities

Given a 10,000us budget:

| Component | Budget (us) | Current (us) | Status |
|---|---|---|---|
| Spatial rebuild x2 | 2,000 | 5,520 | OVER -- needs incremental update |
| Phase 1 (temperature) | 100 | 9 | OK |
| Phase 2 (hunger + fatigue) | 2,000 | 2,896 | MARGINAL -- event push optimization |
| Phase 3 (decisions) | 2,500 | ~3,000 est | MARGINAL -- query dedup needed |
| Phase 4 (wander) | 2,500 | 9,120 | OVER -- needs A* budget cap + pooling |
| Phase 4 (eating + combat) | 500 | ~800 est | OK if spatial is incremental |
| Phase 5 (death) | 100 | <50 | OK |
| Sort tax | 300 | ~2,000 est | OVER -- needs pre-sorted alive list |
| Headroom | 0 | -- | No margin |

The total current cost at 50k exceeds 23,000us. Reaching 10,000us requires:
1. Incremental spatial index: saves ~4,000us
2. A* buffer pooling + call cap: saves ~5,000us
3. Pre-sorted alive list: saves ~1,500us
4. Decision query deduplication: saves ~1,000us
5. Conditional event pushing in hunger: saves ~500us

Total savings: ~12,000us, bringing the estimated cost to ~11,000us. The
remaining 1,000us gap requires either reducing SENSE_RANGE, implementing
decision-tick skipping, or accepting that 50k is the ceiling for 10ms/tick
without architectural changes.

The honest conclusion: 50k entities at 10ms/tick is achievable but requires
implementing ALL of the above optimizations. Missing any single one leaves
the system over budget. The architecture's fixed constraints (HashMap property
tables, deterministic sort, collect-then-apply) impose a structural floor of
approximately 6,000-7,000us at 50k even with perfect optimization. The
remaining 3,000-4,000us must accommodate all gameplay logic. This is tight
but feasible with disciplined per-system complexity budgets.
