# Spatial Infrastructure Failure Analysis

An exhaustive catalog of failure modes, anti-patterns, emergent complexity traps,
and architectural time bombs in Wulfaz's spatial infrastructure -- both current
and as it scales toward Phase C/D.

---

## 1. The Stale Index Semantic Gap

### 1.1 Two-Rebuild Phase Ordering Creates an Implicit Contract Nobody Documents

The main loop rebuilds the spatial index twice per tick:

```
spatial1 (rebuild)
  temperature  (Phase 1 -- no spatial reads)
  hunger       (Phase 2 -- no spatial reads)
  fatigue      (Phase 2 -- no spatial reads)
  decisions    (Phase 3 -- reads spatial index for range queries)
  wander       (Phase 4 -- MUTATES positions, does NOT read spatial index)
spatial2 (rebuild)
  eating       (Phase 4 -- reads spatial index for tile lookups)
  combat       (Phase 4 -- reads spatial index for tile lookups)
  death        (Phase 5)
```

The implicit contract: decisions sees pre-movement positions; eating/combat see
post-movement positions. This works today because only `run_wander` mutates
positions, and it sits between the two rebuilds. The contract is not enforced
by any mechanism -- it is an accident of current system ordering.

The compounding failure: every new system that touches positions must be manually
placed between the correct rebuild points. A future system like `run_flee` (Phase
4, entities run from attackers) or `run_knockback` (Phase 4, combat pushes
entities) would need to run between spatial1 and spatial2. If placed after
spatial2, eating and combat see pre-flee positions. If placed before spatial2
but after wander, the ordering works -- but only if the developer knows the
contract. There is no compile-time or runtime check for this.

The deeper problem: the two-rebuild model implicitly couples the number of
spatial snapshots to the number of position-mutating systems. With one movement
system, two rebuilds suffice. With two movement systems that need to see each
other's results (flee reads wander's output to avoid fleeing into another
entity's path), you need three rebuilds. With N independent position mutators,
you need N+1 rebuilds. But each rebuild costs O(entities), and the rebuild
count becomes part of the tick budget. At 4K entities with a coarse grid,
each rebuild is ~0.1ms. At 10 rebuilds (five position-mutating systems), that
is 1ms just for index maintenance -- 10% of the tick budget doing no actual
simulation work.

### 1.2 Within-Phase Stale Reads in Combat and Eating

`run_combat` iterates combatants sorted by entity ID. Entity A attacks entity
B, killing B (pushed to `pending_deaths`). Entity C, processed later in the
same system pass, queries `entities_at(x, y)` -- and still sees B in the
spatial index, because the index was rebuilt before combat started and B's
position was not removed. The `pending_deaths` filter catches this for the
combatant collection, but the `entities_at` call in the fallback path (line
122-131 of combat.rs) filters `pending_deaths` manually.

This works now because `pending_deaths` is a `Vec<Entity>` checked with
`.contains()`. At 4K entities, if 200 die per tick, each `pending_deaths.contains`
is a linear scan of up to 200 entries. With 4K combatants each checking
pending_deaths, that is 800K linear-scan operations. The `pending_deaths`
check is O(N*D) where N is entities processed and D is deaths so far this tick.
It masquerades as a constant-time filter but becomes quadratic in the number
of deaths.

### 1.3 Spawn-During-Tick Invisibility

Entities spawned during Phase 4 (e.g., by a future `run_craft` that creates
items, or by hydration batches) are inserted into `body.positions` but are
not in the spatial index -- neither spatial1 nor spatial2 included them.
Combat and eating in the same tick cannot see these entities via spatial queries.
They become visible next tick after the next rebuild.

For hydration specifically: 100 entities spawned per tick at building positions.
All 100 are invisible to spatial queries for the remainder of the tick they are
spawned. If hydration runs before spatial2, they miss eating/combat. If it runs
after spatial2, they miss everything. If it runs before spatial1 of the next
tick, they are properly indexed. But the design document says hydration batches
at ~100/tick -- meaning 100 entities exist for one tick in a Schrodinger state:
alive, positioned, but spatially non-existent. Any system that naively checks
"is there an entity here?" via the spatial index gets the wrong answer.

The subtle failure: a hydrated entity at building position (100, 200) is
invisible to `entities_at(100, 200)`, but IS visible to `body.positions`
iteration. Systems that iterate `body.positions` directly (like the spatial
index rebuild itself) see it; systems that go through the spatial index do
not. This creates two different "truths" about entity locations within the
same tick.

---

## 2. Determinism Leaks in the Spatial Pipeline

### 2.1 HashMap Iteration Order Propagates Through SmallVec Insertion Order

`rebuild_spatial_index` iterates `body.positions` (a `HashMap<Entity, Position>`)
and pushes entities into `SmallVec<[Entity; 4]>` per cell. The iteration order
of `body.positions` is non-deterministic across Rust versions, platforms, and
even across runs with different ASLR layouts (since Rust's default hasher uses
random state).

The claim is that callers sort by entity ID, so this does not matter. Examining
the actual call sites:

- `entities_in_range` returns an `impl Iterator`. Callers like `read_input`
  for FoodNearby do `.filter().count()` -- count is order-independent, safe.
- `select_eat_target` calls `entities_in_range` then `.min_by()` with entity
  ID as final tiebreaker -- safe.
- `select_attack_target` same pattern -- safe.
- `entities_at` in combat (line 122-131) collects into a Vec and
  `sort_unstable_by_key` -- safe.
- `entities_at` in eating (line 47-62) collects into a Vec and
  `sort_unstable_by_key` -- safe.

All current callers are safe. But the vulnerability is structural: any future
caller of `entities_at` or `entities_in_range` that uses the first result
without sorting, or that uses the iterator in a way that depends on order
(e.g., `find`, `any`, `take`), introduces a determinism leak. The API returns
unsorted results but relies on callers to sort. This is a convention, not a
guarantee.

### 2.2 Coarse Grid Does Not Fix This -- It Reshuffles It

The proposed coarse spatial grid (from `perf-decisions-design.md`) stores
`Vec<Entity>` per cell. The Vec is populated by iterating `body.positions`
(HashMap, non-deterministic order). Within a coarse cell, entities appear
in HashMap iteration order. The `entities_in_range` rewrite iterates cells
in a deterministic geometric order (min_cell_y to max_cell_y, min_cell_x to
max_cell_x), but within each cell, the entity order is still non-deterministic.

For scoring (count-based), this is irrelevant. For target selection (min_by
with tiebreaker), this is safe because the tiebreaker is on entity ID. But
the concern is not the current callers -- it is that the data structure
presents itself as ordered when it is not, and new callers will assume
ordering they do not have.

### 2.3 Cache Behavior Non-Determinism

Even when the simulation produces identical outputs regardless of spatial
index iteration order, the performance characteristics differ.
HashMap internal layout depends on insertion order (which bucket each entry
falls into, how collision chains form). Two runs with the same seed produce
the same entity positions, but if the HashMap rebuilds in different iteration
orders (because the HashMap randomizes its hasher on construction), the
spatial index's internal memory layout differs. This means cache line access
patterns during range queries differ between runs. The simulation is
deterministic; the performance is not.

This creates a testing trap: profiling run A shows 2ms for decisions;
profiling run B with identical inputs shows 3ms. The developer hunts for a
bug that does not exist -- it is just cache layout variance. This wastes
debugging time proportional to the frequency of performance measurements,
which increases exactly when performance is being optimized (Phase C/D).

---

## 3. The Empty-Cell Dominance Problem

### 3.1 Range Queries Pay for Geometric Area, Not Entity Density

`entities_in_range(pos, 30)` iterates (2*30+1)^2 = 3,721 cells in the
fine-grained spatial index. With 360 entities on a ~300x200 tile area, the
occupancy rate is 360/60,000 = 0.6%. The range query probes 3,721 cells of
which ~22 contain entities. 99.4% of HashMap probes return `None`.

The coarse grid reduces this to 25 cell lookups, of which maybe 10 contain
entities. Better, but the fundamental problem persists: the query cost is
O(cells_in_range), not O(entities_in_range). As SENSE_RANGE grows or as
systems need larger awareness radii (e.g., a merchant who scans for customers
within 100m, or a fire system that checks for flammable buildings within 50m),
the query cost grows quadratically with range while entity density remains
sparse.

### 3.2 Entity Clustering Creates Bimodal Query Cost

GIS-spawned entities cluster in buildings. A dense building might have 20
entities on 50 tiles; adjacent road tiles are empty. A range query centered
on a road intersection sees mostly empty cells. A range query centered inside
a dense building sees 20+ entities on nearby tiles, plus whatever is in
adjacent buildings.

The bimodal distribution means average-case analysis is misleading. The
average query might return 5 entities, but the worst case returns 200 (all
entities in a dense block of adjacent buildings). Post-query processing
(scoring, target selection) iterates all returned entities. If a scoring
function does per-entity work proportional to returned count, the worst-case
entity in a dense neighborhood takes 40x longer than the average entity.
This entity dominates the tick -- a single entity evaluation can cost more
than all other entity evaluations combined.

With the utility AI's geometric-mean scoring, each entity's score evaluation
iterates all nearby entities for FoodNearby and EnemyNearby considerations.
An entity in a dense neighborhood with 200 nearby entities does 400 filter
operations (200 for food + 200 for enemies). An entity in a sparse area does
10. If 5% of entities are in dense areas, they consume ~67% of the scoring
budget. Optimizing for the average case (pruning, caching) does not help
the dense-neighborhood outliers that dominate wall-clock time.

### 3.3 The Density Inversion at Scale

At 360 entities in one quartier (~300x200 tiles), the spatial density is
manageable. At 4K entities in the Active zone (~300x300 tiles visible, 150
tile radius), density per building increases roughly 10x. Buildings that had
2 entities now have 20. The per-building entity count follows the occupant
distribution in the SoDUCo data, which is heavy-tailed: most buildings have
1-3 occupants, but some (hotels, workshops, large shops) have 15-30.

SmallVec<[Entity; 4]> inlines up to 4 entities. At 4K entities with building
clustering, many tiles have 5+ entities, causing SmallVec to spill to heap
allocation. The coarse grid is worse: a 16x16 cell covering a dense building
block might contain 50+ entities in a single Vec. The Vec is heap-allocated
regardless of size, so the inline optimization of SmallVec is irrelevant for
the coarse grid.

The cache impact: at 360 entities, SmallVec inline storage means spatial
index entries are ~32 bytes each (key + inline array). At 4K entities with
frequent spilling, many entries are 16 bytes (key + pointer) plus a separate
heap allocation. Pointer chasing through heap-allocated SmallVecs has
fundamentally worse cache behavior than inline storage.

---

## 4. Pending Deaths as a Quadratic Ghost Filter

### 4.1 Linear Scan Accumulates Within a Tick

`pending_deaths` is a `Vec<Entity>`. Every system filters it with
`.filter(|e| !world.pending_deaths.contains(e))`. `Vec::contains` is O(N)
where N is the current length of pending_deaths.

Within a single tick, pending_deaths grows monotonically. It starts empty
(cleared by run_death at the end of the previous tick). Phase 2 systems
(hunger, fatigue) can push deaths. Phase 3 (decisions) does not push deaths
but filters against them. Phase 4 (wander, eating, combat) can push deaths
and filters against them.

The worst case: 4K entities, 10% die per tick (400 deaths). If 200 die in
combat and 100 die from hunger and 100 from fatigue:

- hunger processes 4K entities, each checking pending_deaths (0 to 100 entries
  growing): ~4K * 50avg = 200K comparisons.
- fatigue processes 4K entities, pending_deaths now 100 entries: 4K * 100 = 400K.
- decisions processes 4K entities, each with 2-4 spatial queries, each filtered
  against 200 pending deaths: 4K * 3 * 200 = 2.4M comparisons.
- combat processes 4K entities, pending_deaths 200-400 entries: 4K * 300avg = 1.2M.

Total: ~4.2M linear-scan comparisons per tick. At ~1ns per comparison (cache-
friendly sequential scan of u64s), that is ~4ms. Nearly half the tick budget,
spent on death bookkeeping.

A HashSet would eliminate this, but the architecture specifies
`pending_deaths: Vec<Entity>` and all systems assume Vec semantics (push,
contains, drain). Changing to HashSet requires updating every system and test.
The type is load-bearing across the entire codebase -- seven source files
directly reference it.

### 4.2 Dying Entities in the Spatial Index Create Ghost Interactions

The spatial index is rebuilt before Phase 3 (decisions) and before Phase 4
(eating/combat). Entities killed in Phase 2 (hunger starvation, fatigue
exhaustion) are in `pending_deaths` but still alive -- they have not been
despawned. They are included in the spatial index rebuild because
`rebuild_spatial_index` checks `self.alive.contains(&entity)`, and pending
entities are still alive.

So decisions sees dying-but-not-yet-dead entities in range queries. The
`pending_deaths` filter catches them for scoring (FoodNearby, EnemyNearby
counts exclude pending deaths). But the spatial index contains them. If a
future system queries the spatial index without the pending_deaths filter --
because the developer does not know about this implicit contract -- it will
interact with ghost entities.

This gets worse with compound deaths: entity A dies from hunger in Phase 2.
Entity B, in Phase 3, selects A as an eat target (A has a nutrition component
but is in pending_deaths). The pending_deaths filter in select_eat_target
catches this. But if the eat target selection were cached from a previous tick
and reused without re-filtering (an optimization someone might try for
performance), the stale target A would be eaten, creating a double-death
(food's pending death + eaten death).

---

## 5. LOD Zone Boundaries as Simulation Discontinuities

### 5.1 The Sensing Horizon Artifact

An entity at the edge of the Active zone has SENSE_RANGE=30. Its sensing
circle extends 30 tiles into the Nearby zone. Entities in the Nearby zone
run simplified systems (no decisions, no combat). The active entity's range
query returns Nearby entities -- they exist as real Entity values -- but
those entities did not run decisions, so they have no intentions, no current
action, and stale action states.

The active entity sees a food item 25 tiles away in the Nearby zone. It
selects this food as an eat target and starts pathfinding toward it. The
food exists (it has a position and nutrition component) but is in a zone
where eating does not run. When the active entity reaches the food's tile
(now itself near or crossing the zone boundary), does eating trigger? If
the entity is still Active, yes. But the food might be Nearby (different
zone classification applies to the food based on its position, not the
eater's position).

The zone classification is per-entity-position. Two entities on the same
tile can be in different zones if the zone boundary passes through that tile.
This is impossible for quartier-aligned boundaries (boundaries follow building
polygons), but possible for radius-based boundaries (distance from camera).
If the boundary is defined as "150 tiles from camera center," entities at tile
150 are Active and entities at tile 151 are Nearby. A range query from tile
150 extends to tile 180, crossing the boundary. The returned entities are a
mix of Active and Nearby -- but the querying system does not know which zone
each returned entity belongs to.

### 5.2 Hydration/Dehydration Boundary Oscillation

Camera panning causes the Active zone to sweep across the map. A slow pan
moves the zone boundary at ~1 tile/frame. At 60fps with a 1-tile-per-frame
pan speed, the boundary sweeps through a building in 20-30 frames.

Dehydration collapses active entities to district aggregates. Hydration
spawns entities from the distribution. If the camera oscillates (the player
pans back and forth near a quartier boundary), entities are repeatedly
hydrated and dehydrated. Each cycle:

1. Hydrate: spawn 100 entities at building positions from statistical
   distribution. Costs ~0.5ms spawn time + spatial index rebuild.
2. Simulate for 50-200 ticks: entities accumulate state (hunger, fatigue,
   action states, wander targets, cooldowns).
3. Dehydrate: collapse all entity state to district averages. Individual
   state is lost.
4. Hydrate again: fresh entities with district-average initial state.

The oscillation destroys information. An entity that was 80% through eating
is dehydrated and replaced with a fresh entity at district-average hunger.
The more the player pans, the more state is churned. This is not just a
visual glitch -- it changes simulation outcomes. Entities near oscillating
boundaries have effectively random reset points for their state machines.

### 5.3 The Boundary Shadow Problem

When the Active zone moves east, the western edge dehydrates and the eastern
edge hydrates. During the transition, the spatial index contains a mix of:
- Fully simulated entities from the stable Active zone center
- Freshly hydrated entities with default state at the eastern edge
- About-to-be-dehydrated entities with evolved state at the western edge

Range queries crossing the eastern edge return freshly hydrated entities
with zero hunger, full health, no action state. These entities have not
run decisions yet (they were just spawned). Active entities near the boundary
see a neighborhood of "perfect" entities -- healthy, not hungry, no
intentions. This biases their own decision-making. An entity deciding
whether to attack sees neighbors with full health and lower aggression
(fresh defaults), making attack score lower. The simulation behaves
differently near zone boundaries than in the zone interior.

This is not a bug that manifests once. It is a persistent bias affecting
every entity within SENSE_RANGE (30 tiles) of a zone boundary, every tick.
With 4K active entities in a ~300-tile-diameter zone, entities within 30
tiles of the boundary constitute ~36% of the active population (annular
area / total area). Over a third of all active entities are permanently
affected by boundary artifacts.

---

## 6. Pathfinding as the Hidden Spatial Bottleneck

### 6.1 A* Cost Is Not Spatial-Index-Addressable

`find_path` uses A* on the tile grid with a HashMap for g_scores and a
HashSet for the closed set. This is completely independent of the entity
spatial index. Improving the spatial index for range queries has zero effect
on pathfinding performance.

A single A* call expands up to 32,768 tiles (MAX_EXPANDED). With octile
heuristic, typical expansion for a 30-tile path is 100-500 tiles. But
pathfinding through narrow corridors (alleys between buildings, passages
through blocks) can expand thousands of tiles as the search explores dead
ends in the building geometry.

At 4K entities, ~1/9 move per tick (Walk gait = 9-tick cooldown). That is
~444 pathfinding calls per tick. At 200 tile expansions average, that is
~89K tile expansions. Each expansion does 8 neighbor lookups, 8 walkability
checks (chunk-indexed, fast), 8 HashMap operations for g_score, 8 HashSet
operations for closed. Total: ~712K HashMap/HashSet operations per tick just
for pathfinding.

HashMap operations for A* (i32, i32) keys are approximately 30-50ns each
(hash + probe + compare). 712K * 40ns = ~28ms. That is nearly 3x the entire
tick budget, from pathfinding alone.

The spatial index optimization for decisions creates a false sense of progress.
The developer fixes decisions (40ms -> 2ms), then discovers pathfinding is
28ms. The architecture has the whack-a-mole property where fixing one spatial
bottleneck reveals another that was masked by the first.

### 6.2 HPA* Precomputation Assumes Static Topology

The planned HPA* precomputes chunk-level navigation graphs with intra-chunk
shortest paths between border nodes. This works for static terrain (buildings
do not move). But it does not account for dynamic obstacles (entities blocking
tiles).

Currently, A* does not check for blocking entities -- it only checks tile
walkability. Two entities can occupy the same tile. But if a future collision
avoidance system is added (which is common in agent simulations of this type),
pathfinding needs to avoid tiles occupied by other entities. The spatial index
provides this information, but:

1. The spatial index is stale during pathfinding (rebuilt before movement,
   but pathfinding happens during movement).
2. HPA*'s precomputed paths assume no dynamic obstacles. An entity blocking
   a narrow corridor invalidates the precomputed path but HPA* does not know.
3. Re-planning when a precomputed path is blocked requires falling back to
   fine-grained A*, eliminating the HPA* benefit for exactly the cases where
   pathfinding is most expensive (narrow corridors in dense neighborhoods).

### 6.3 Pathfinding Allocation Pattern

Each A* call allocates three collections: `BinaryHeap`, `HashMap<(i32,i32), (i32,i32)>`,
`HashMap<(i32,i32), u32>`, `HashSet<(i32,i32)>`. At 444 calls per tick, that
is 1,776 HashMap/HashSet allocations per tick, each growing to ~200-500
entries. Rust's allocator handles this, but the allocation-deallocation churn
creates memory fragmentation pressure. After 10,000 ticks (100 seconds of
simulation), millions of pathfinding allocations have been created and
destroyed. The allocator's free list grows, heap fragmentation increases,
and allocation latency becomes less predictable. This manifests as tick-time
variance: most ticks complete in 8ms, but every ~100 ticks one takes 15ms
due to allocator pressure.

---

## 7. The Dual-Index Coherence Trap

### 7.1 Two Spatial Indexes That Must Agree

The proposed design (from perf-decisions-design.md) maintains two spatial
indexes simultaneously:

```rust
pub spatial_index: HashMap<(i32, i32), SmallVec<[Entity; 4]>>,  // fine-grained, for entities_at
pub spatial_grid: HashMap<(i32, i32), Vec<Entity>>,              // coarse, for entities_in_range
```

Both are rebuilt from the same source (`body.positions`) in the same function.
They must agree: every entity in the fine index must be in the coarse index,
and vice versa. This is enforced by construction (single rebuild function),
but:

1. If a future optimization incrementally updates one index but not the other
   (e.g., "only update the coarse grid since decisions already ran and won't
   run again this tick"), the indexes diverge.
2. If a third spatial structure is added (e.g., a quadtree for LOD zone
   classification, or a separate index for food-only entities), the coherence
   requirement expands combinatorially.
3. Each rebuild duplicates the iteration over `body.positions`. Two rebuilds =
   2x the cost. Three structures = 3x. The rebuild becomes the bottleneck it
   was supposed to solve.

### 7.2 API Surface Tension

`entities_at` returns `&[Entity]` -- a borrowed slice from SmallVec. This is
zero-copy and fast. The coarse grid cannot provide this for exact-tile queries
because entities at a specific tile are scattered within the coarse cell's Vec.
The dual-index design keeps the fine-grained index specifically to preserve
this API.

But the API locks in the data structure. Any optimization that wants to remove
the fine-grained index (to save rebuild time and memory) must change
`entities_at` to return an owned Vec or an iterator, breaking all callers.
Combat and eating directly pattern-match on the returned slice. The borrowed-
slice return type is load-bearing across three source files.

This creates a situation where the data structure cannot be changed without a
coordinated multi-file refactor, but the only reason the data structure exists
is to avoid a multi-file refactor. The architectural inertia compounds: each
new caller of `entities_at` adds another dependent that must be updated if
the return type changes.

---

## 8. Memory Topology and Cache Cliff Edges

### 8.1 The L2 Boundary at ~2K Entities

At 360 entities, the spatial index contains ~350 HashMap entries (some
entities share tiles). Each entry is ~64 bytes (bucket metadata + key + value
inline). Total: ~22KB. This fits in L1 cache (typically 32-48KB). Range
queries that probe 25 coarse cells achieve near-perfect L1 hit rates.

At 2K entities, the spatial index grows to ~120KB (fine-grained) + ~30KB
(coarse). Total: ~150KB. This spills from L1 (32-48KB) into L2 (256-512KB).
L2 hit latency is ~4x L1. Range queries slow by ~2-3x without any algorithmic
change. The developer sees a performance regression at ~2K entities that does
not correspond to any code change -- it is purely a cache topology effect.

At 4K entities, the spatial index is ~300KB. Still fits in L2 on most CPUs.
But `body.positions` (4K entries * ~24 bytes each = ~96KB), `body.healths`
(~32KB), `mind.hungers` (~32KB), `mind.action_states` (~256KB with cooldown
HashMaps), etc. total ~800KB for property tables. A range query that returns
100 entities and then checks their health, hunger, combat_stats, and nutrition
touches data scattered across multiple HashMaps. Each HashMap has its own
bucket array in separate memory regions. A single entity evaluation during
scoring touches 5-6 different HashMap bucket arrays. This is 5-6 cache lines
per entity, scattered across ~800KB of total property table space.

At 4K entities, the working set for scoring a single entity (spatial index +
5 property table lookups) spans ~1MB. This exceeds L2 on many CPUs. The
scoring loop processes 4K entities, each touching ~6 cache lines in different
memory regions. Total cache pressure: 4K * 6 * 64 bytes = ~1.5MB. Without
locality optimization, every property table lookup is a cache miss after the
first ~50 entities evict earlier entries.

### 8.2 SmallVec Spill Creates Bimodal Access Patterns

SmallVec<[Entity; 4]> uses inline storage for up to 4 entities. Access
pattern: read the SmallVec discriminant, then either read inline data (same
cache line) or follow a pointer to heap (different cache line, likely cache
miss).

At low entity density (360 entities), most tiles have 0-1 entities. SmallVec
almost never spills. Access is uniform and predictable.

At high density (4K entities), building tiles frequently have 5+ entities.
SmallVec spills for ~30% of occupied cells (estimated from SoDUCo occupant
distribution: 15-30 occupants per building, 50-100 tiles per building, so
some tiles have 2+ entities from the same building). The access pattern
becomes bimodal: 70% of lookups read inline (fast), 30% chase a pointer
(slow). The CPU branch predictor learns the majority case (inline) but
mispredicts on spills. Branch misprediction penalty is ~15 cycles on modern
CPUs. With 3,721 lookups per range query (fine-grained) or 25 (coarse),
misprediction cost is negligible for coarse but adds ~56K penalty cycles
for fine-grained queries.

---

## 9. Hydration Batch Spawning as a Transient Load Spike

### 9.1 100 Entities/Tick Spawn Concentrates Allocation

Hydration spawns ~100 entities per tick. Each entity requires insertion into
~15 HashMaps (positions, healths, hungers, fatigues, combat_stats,
gait_profiles, current_gaits, move_cooldowns, icons, names, nutritions,
intentions, action_states, home_buildings, workplaces). That is ~1,500
HashMap insertions per hydration tick.

HashMap insertion amortizes to O(1) but occasionally triggers a resize (table
doubling). If the HashMap was at capacity before hydration, 100 insertions
can trigger a resize that copies the entire existing table. At 4K entities,
a positions HashMap resize copies ~4K entries * 24 bytes = ~96KB. This is a
one-time cost, but it happens for each of the ~15 HashMaps independently. If
multiple HashMaps happen to resize on the same tick, the transient allocation
spike can reach several hundred KB of copying, pushing that tick's duration
to 15-20ms -- a missed deadline that causes a visible stutter.

The timing is unpredictable: it depends on the internal load factor of each
HashMap, which depends on the sequence of insertions and deletions since
program start. A HashMap that was resized at 2,048 entries will not resize
again until ~4,096. But hydration adds 100 entries per tick, so the resize
happens every ~20 ticks for that particular HashMap. If 15 HashMaps resize
independently, the probability of at least one resize per tick is high (about
once every 1.3 ticks on average). The probability of 3+ concurrent resizes
(the stutter case) is lower but non-negligible over thousands of ticks.

### 9.2 Dehydration Leaves HashMap Capacity Permanently Inflated

When dehydration removes 500 entities, it calls `.remove()` on each HashMap.
Rust's HashMap does not shrink its allocation on remove. The bucket array
stays at its peak capacity. After several hydration/dehydration cycles, every
property HashMap is sized for the peak entity count ever reached, even if the
current entity count is much lower.

With 4K active entities and 100/tick hydration/dehydration churn, the peak
might reach 4.5K during a hydration burst before dehydration catches up. All
HashMaps are permanently allocated for 4.5K entities. The memory overhead is
small (extra ~10KB per HashMap * 15 HashMaps = ~150KB), but the cache impact
is real: HashMap iteration for spatial index rebuild iterates all buckets
including empty ones. More buckets = more cache lines touched during rebuild.

The `spatial_index.clear()` call in `rebuild_spatial_index` does not
deallocate -- it keeps the bucket array. After a hydration burst that
created 500 spatial index entries, the HashMap has capacity for 500+ entries.
Even if only 200 entities remain after dehydration, the rebuild iterates
the full bucket array of the positions HashMap (4.5K capacity) and inserts
into a spatial_index HashMap with 500+ capacity. Both operations touch more
memory than necessary for the current entity count.

---

## 10. The Waterbed Effect Across Optimization Layers

### 10.1 Optimizing Decisions Reveals Pathfinding

The perf-decisions-design.md proposes three layers of optimization:
1. Score pruning (~40ms -> ~2ms)
2. Nearby-entity cache (~2ms -> ~1ms)
3. Coarse spatial grid (~1ms -> ~0.3ms)

After all three, decisions drops from 40ms to 0.3ms. The tick budget is
10ms. But the remaining systems are:

| System      | Current (360e) | Projected (4K) |
|-------------|---------------|----------------|
| spatial1    | ~0.2ms        | ~0.5ms         |
| temperature | ~0.1ms        | ~0.1ms (tile-based, entity-independent) |
| hunger      | ~0.1ms        | ~1ms           |
| fatigue     | ~0.1ms        | ~1ms           |
| decisions   | ~0.3ms (optimized) | ~3ms      |
| wander      | ~1ms (A* dominated) | ~28ms    |
| spatial2    | ~0.2ms        | ~0.5ms         |
| eating      | ~0.1ms        | ~0.5ms         |
| combat      | ~0.1ms        | ~0.5ms         |
| death       | ~0.01ms       | ~0.1ms         |
| **Total**   | **~2.3ms**    | **~35ms**      |

Wander (pathfinding) is 28ms of a 35ms tick. The spatial index optimization
that reduced decisions from 40ms to 0.3ms is irrelevant to the actual
bottleneck at 4K entities. The architecture creates an illusion of progress:
the most-measured problem is solved while the actual critical path shifts
to a different subsystem.

### 10.2 HPA* Introduces a New Class of Spatial State

HPA* requires precomputed chunk-level navigation graphs. This is a new
spatial data structure on World:

```rust
pub hpa_graph: HpaGraph,  // chunk border nodes, inter-chunk edges, intra-chunk paths
```

This structure is precomputed and static (terrain does not change). But it
must be queried during pathfinding, adding another data structure to the
working set. The HPA graph for ~7,400 chunks with ~20 border nodes each is
~148K nodes and ~1M edges. At ~16 bytes per edge, that is ~16MB. This does
not fit in any cache level. HPA* pathfinding on the chunk graph is itself an
A* call on a graph with ~148K nodes -- potentially slow for cross-city paths.

The pathfinding working set grows from ~100KB (current A* per call) to
~100KB + 16MB (current A* for local search + HPA graph for global search).
The HPA graph is accessed sparsely (only nodes along the path are touched),
but the chunk-to-node lookup is O(1) and accesses a contiguous array, so
cache behavior depends on path locality. Cross-city paths touch nodes in
many distant chunks, creating cache misses proportional to path length.

### 10.3 Each Fix Creates a New Implicit Coupling

Score pruning (Layer 1) relies on consideration ordering. If a developer
reorders considerations in the KDL config without understanding the pruning
interaction, the spatial query count can increase 10-100x. The optimization
is invisible at the config layer -- it is a performance dependency between
the scoring algorithm in decisions.rs and the data layout in the config file.

The nearby-entity cache (Layer 2) creates a temporal coupling: the cache is
valid for exactly one entity's evaluation within exactly one tick. If the
scoring is ever parallelized (violating the single-threaded constraint but
tempting at 4K entities), the cache cannot be shared between threads without
synchronization.

The coarse grid (Layer 3) creates a spatial coupling: the cell size (16
tiles) must be larger than any range query's radius divided by the target
cell-probe count. If SENSE_RANGE increases to 50, the coarse grid needs
cell_size >= 10 to keep probe count under 100. But smaller cells mean more
cells, more HashMap entries, more memory, worse cache behavior. The cell size
is a hidden parameter that constrains future system design -- any system
that needs a sensing range > 50m implicitly requires a cell size review.

---

## 11. The All-State-On-World Constraint as Spatial Infrastructure Lock-In

### 11.1 No Per-System Spatial Acceleration

The architecture mandates "All state on World. No per-system persistent state."
This means a system cannot maintain its own spatial acceleration structure
tuned to its access pattern.

Decisions needs coarse range queries. Combat needs exact tile lookups.
Pathfinding needs walkability checks (tile grid, not entity spatial index).
A future crowd-flow system might need directional density queries (how many
entities are moving eastward in this corridor?). Each use case benefits from
a different spatial structure, but the architecture forces all spatial
queries through one or two shared indexes on World.

The constraint is well-motivated (deterministic replay requires all state to
be inspectable and serializable), but it creates a lowest-common-denominator
spatial infrastructure. The shared index must be general enough for all
callers, which means it cannot be optimized for any specific caller. The
coarse grid is a partial exception (it serves decisions specifically), but
adding a third structure for crowd flow, a fourth for LOD zone classification,
a fifth for pathfinding obstacle avoidance, leads to five redundant
representations of entity locations, each rebuilt every tick.

### 11.2 Rebuild-From-Scratch Prevents Amortization

The rebuild-from-scratch model means spatial index cost is paid every tick
regardless of how much entity state changed. If only 10 entities moved this
tick (the rest are on cooldown), the spatial index still rebuilds from all
4K entities. The rebuild cost is O(N) regardless of movement count.

An incremental model (update only entities that moved) would be O(M) where M
is moving entities per tick. With Walk gait at 9-tick cooldown, ~11% of
entities move per tick. At 4K entities, incremental updates would process
~444 moves instead of rebuilding 4K entries. That is ~10x cheaper.

But incremental updates require tracking which entities moved -- which
requires per-system state (a "moved this tick" set or a position delta log).
This violates the no-per-system-state constraint. The alternative is to put
a `position_changed: bool` flag on each entity in the positions HashMap, but
that adds overhead to every position mutation and every position read, and
the HashMap entry grows from 24 bytes to 25 bytes (with padding, 32 bytes),
increasing cache pressure by 33%.

The architectural constraint forces a rebuild model that scales linearly
with total entity count, not with per-tick change volume. At 4K entities,
this is ~0.5ms per rebuild. At 40K entities (Phase D expanded active zone),
it is ~5ms per rebuild. With two rebuilds per tick, that is 10ms -- the
entire tick budget consumed by spatial index maintenance.

---

## Summary: The Interconnection Topology

These failure modes are not independent. They form a connected graph of
mutual reinforcement:

- **Stale index (1) + LOD boundaries (5)**: hydrated entities are invisible
  to spatial queries for one tick, creating ghost neighborhoods at zone edges.
- **Determinism leaks (2) + pathfinding (6)**: pathfinding allocation churn
  creates non-deterministic performance variance, making profiling unreliable.
- **Empty-cell dominance (3) + entity clustering (3.2) + cache cliffs (8)**:
  dense neighborhoods cause SmallVec spills and cache misses exactly where
  query cost is highest, creating worst-case performance in the most
  populated (and thus most important) areas of the simulation.
- **Pending deaths (4) + dual index (7)**: ghost entities in the spatial
  index require every caller to independently filter pending_deaths, and the
  dual-index design doubles the ghost surface area.
- **Pathfinding (6) + waterbed (10)**: fixing the spatial index for range
  queries masks the pathfinding bottleneck, which is addressed by HPA*, which
  introduces a 16MB spatial structure that degrades cache behavior for
  everything else.
- **All-state-on-World (11) + rebuild model (11.2)**: the rebuild-from-scratch
  model scales linearly with entity count, and the no-per-system-state
  constraint prevents the incremental alternative, creating an infrastructure
  that becomes proportionally more expensive as the simulation succeeds at
  scaling.

The fundamental tension: the architecture optimizes for simplicity and
determinism (HashMap EAV, rebuild-from-scratch, single-threaded, all state
on World) at the cost of spatial infrastructure flexibility. Each scaling
step (360 -> 4K -> 40K entities) tightens the constraints further. The
spatial infrastructure that works at 360 entities creates cascading failures
at 4K, not because any single component fails, but because the interactions
between components (stale indexes, ghost entities, cache pressure, allocation
churn) multiply nonlinearly with entity count.
