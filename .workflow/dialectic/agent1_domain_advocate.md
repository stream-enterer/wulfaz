# Performance Engineering for LLM-Authored Blackboard EAV Simulations

A treatise on rules, data structure selection, complexity budgets, and system
authoring patterns for the Wulfaz architecture.

## 1. The Fundamental Cost Model

This architecture is not a game engine. It is not an ECS. It is a blackboard
system with HashMap-keyed property tables, sequential phase ordering, and a
deterministic replay invariant. Every performance argument must begin from
these three facts:

1. **Every system iterates at least one HashMap per tick.** HashMap iteration
   is O(n) but with a constant factor of ~30ns per `.get()` due to hashing,
   cache-line straddling, and pointer chasing. A system that touches 3 tables
   for 10k entities does ~30k hash lookups = ~0.9ms before any computation.

2. **Mutation requires collect-then-apply.** You cannot modify a table while
   iterating it. Every write path allocates a Vec, fills it, then iterates
   it again. This doubles the iteration cost and adds allocation pressure.

3. **Determinism requires sorting.** HashMap iteration order is
   non-deterministic. Every system that processes entities must sort by
   `Entity.0` before the processing loop, costing O(n log n). This is
   unavoidable without changing the data structure.

The per-tick budget at 100 ticks/sec is 10ms. With 8 systems, 2 spatial index
rebuilds, and a debug validation pass, each system gets roughly 1ms. At 10k
entities, that is **100ns per entity per system**. This is the fundamental
constraint that governs every design decision.

### 1.1 What Fits in 100ns

| Operation | Approximate Cost | Fits? |
|-----------|-----------------|-------|
| HashMap::get | 25-40ns | Yes (2-3 per entity) |
| HashMap::insert | 40-60ns | Yes (1 per entity) |
| Vec::push (amortized) | 5ns | Yes |
| f32 arithmetic (add/mul/cmp) | 1-3ns | Yes (dozens) |
| f32::powf | 15-25ns | Yes (a few) |
| HashSet::contains | 20-35ns | Yes (1-2 per entity) |
| A* pathfinding (256x256 map) | 50,000-200,000ns | No (500-2000x over) |
| A* pathfinding (8192x7424 map) | 500,000-5,000,000ns | No (5000-50000x over) |
| entities_in_range (SENSE_RANGE=30) | 1,000-10,000ns | Marginal |
| sort 10k entities | 200,000-400,000ns | Amortized over all entities |

The table reveals the architecture's pressure points: spatial queries and
pathfinding are the only operations that routinely exceed budget. Everything
else -- arithmetic, table lookups, Vec operations -- fits comfortably.

## 2. Rules for HashMap Iteration and Table Access

### Rule PERF-ITER-01: Iterate the Smallest Relevant Table

A system should iterate the table with the fewest entries that still captures
all entities it needs to process. The hunger system iterates `hungers`, not
`positions`. The combat system iterates `combat_stats`, not `alive`. This
minimizes the base iteration cost.

**Verifiable check:** The primary iteration table of a system must be the
narrowest table that defines the system's domain. If the system touches
entities with components A, B, and C, iterate whichever of A, B, C has the
fewest expected entries.

### Rule PERF-ITER-02: Filter Before Collecting

When building the candidate list, chain `.filter()` calls on the iterator
before `.collect()`. Each filter that rejects an entity avoids the downstream
`.get()` calls on secondary tables for that entity. The filter ordering should
be: cheapest rejection first.

Preferred ordering:
1. `pending_deaths.contains()` -- HashSet O(1), rejects dead entities
2. `.contains_key()` on a required secondary table -- HashMap O(1)
3. `.get()` with value predicate -- HashMap O(1) + comparison

**Verifiable check:** Every system's candidate collection must filter
`pending_deaths` before any secondary table lookup.

### Rule PERF-ITER-03: Avoid Redundant Table Lookups in the Processing Loop

If the iteration phase already extracted a value from a table (via `.get()`
during filtering or mapping), pass that value into the processing loop rather
than looking it up again. The current codebase is inconsistent here: `run_combat`
extracts aggression during candidate collection but then re-reads `combat_stats`
in `compute_fatigue_damage`. This is acceptable only when the value might have
changed between collection and processing (which cannot happen in a
collect-then-apply system within the same phase).

**Verifiable check:** Within a single system function, no `HashMap::get` call
should read a value that was already extracted in the same function and cannot
have changed.

### Rule PERF-ITER-04: Pre-size Collection Vecs

When the number of changes is bounded by the number of candidates (which it
always is in this architecture), use `Vec::with_capacity(candidates.len())`
for the changes Vec. This eliminates reallocation during the collect phase.

**Verifiable check:** Every `Vec::new()` used for collecting changes must be
replaced with `Vec::with_capacity()` sized to the candidate count, OR must be
justified with a comment explaining why the expected size is much smaller than
the candidate count.

## 3. Rules for the Deterministic Sort

### Rule PERF-SORT-01: Sort Once Per System, Not Per Sub-Operation

The sort by `Entity.0` must happen exactly once: on the primary candidate list.
Subsequent sub-operations within the same system (e.g., finding a target from
a spatial query) must sort their own candidate lists independently only if
those sub-lists are derived from a non-deterministic source (spatial index,
secondary HashMap iteration). Do not re-sort the main candidate list.

**Verifiable check:** Each system has exactly one `sort_by_key(|e| e.0)` on
its primary candidate Vec. Secondary sorts are permitted only on Vecs derived
from spatial queries or secondary HashMap iterations within the per-entity
processing loop.

### Rule PERF-SORT-02: Systems That Do Not Branch on Entity Order May Skip Sort

A system whose per-entity computation is purely independent -- meaning the
result for entity A does not depend on whether entity B was processed first --
does not need the sort for correctness. However, it still needs the sort for
determinism if it consumes RNG. The hunger system (`run_hunger`) does not use
RNG and its per-entity computation is independent. It could skip the sort. But
the current implementation sorts anyway. The cost is ~0.4ms for 10k entities.

**Verifiable check:** A system may omit the sort ONLY if it satisfies ALL of:
(a) does not use `world.rng`, (b) per-entity results are independent of
processing order, (c) it does not push to `pending_deaths` (which would affect
downstream skip checks). If any condition fails, the sort is mandatory.

### Rule PERF-SORT-03: Consider Pre-Sorted Entity Lists for Hot Paths

If multiple systems iterate the same set of entities (e.g., all entities with
`positions` + `gait_profiles`), the sort is duplicated. The architecture could
provide a `World::sorted_alive() -> Vec<Entity>` cached per tick. At 10k
entities, each sort costs ~0.2-0.4ms. With 6 systems sorting, that is 1.2-2.4ms
of pure sorting per tick. A cached sorted list would reduce this to ~0.4ms total.

**Verifiable check:** This is an architectural recommendation, not a per-system
rule. If tick profiling shows >15% of tick budget in sorting, implement
per-tick sorted entity caches.

## 4. Rules for Spatial Queries

### Rule PERF-SPATIAL-01: Use Direct Table Lookup When You Know the Entity

If a system already has a reference to a target entity (e.g., from an
`Intention.target`), look up its position with `world.body.positions.get(&target)`.
Do not use the spatial index to find it. Direct lookup: ~30ns. Spatial query:
~1000ns minimum.

The combat and eating systems already follow this pattern: they check the
preferred target via direct lookup first, then fall back to spatial query only
if no preferred target exists.

**Verifiable check:** If a system has access to a specific `Entity` ID, it must
use `positions.get(&entity)` rather than a spatial query to find that entity's
location.

### Rule PERF-SPATIAL-02: Bound Spatial Query Range to Minimum Necessary

`entities_in_range` scans a grid of spatial cells. The cost is O(cells_scanned *
entities_per_cell). With SPATIAL_CELL_SHIFT=4 (16m cells) and SENSE_RANGE=30m,
the query scans a ~4x4 grid of cells = 16 cell lookups. At 10 entities per cell
average (urban density), that is ~160 entity comparisons per query. At 10k
entities all running decisions with spatial queries, the total is 1.6M
comparisons.

The SENSE_RANGE of 30m is already the correct scale for the simulation
(roughly one city block). Do not increase it without understanding the quadratic
cost implication.

**Verifiable check:** Every call to `entities_in_range` must use the minimum
range that satisfies the system's design. The range must have a `const` with a
comment stating the real-world distance and justification.

### Rule PERF-SPATIAL-03: Spatial Index Rebuild Cost is O(n)

`rebuild_spatial_index` iterates all positions and inserts into the grid. At
10k entities, this is ~10k hash insertions + alive checks = ~0.5ms. It is
called twice per tick (before needs/decisions and after movement). This 1ms
cost is essentially a tax on the phase-ordered architecture. It is acceptable
as long as the entity count stays under ~50k. Beyond that, consider incremental
updates (maintaining a dirty set of moved entities).

**Verifiable check:** `rebuild_spatial_index` is called exactly twice per tick:
once before Phase 2 (needs) and once before Phase 4 (actions that depend on
post-movement positions). Adding a third call requires justification.

### Rule PERF-SPATIAL-04: Spatial Queries in Phase 3 Must Not Scale Quadratically

The decisions system calls `entities_in_range` once per entity for `FoodNearby`
and once for `EnemyNearby`. At 10k entities, that is 20k spatial queries per
tick. Each query examines ~160 entities. Total: 3.2M entity comparisons. This
is the single most expensive operation in the tick loop.

Mitigation strategies (in order of implementation priority):
1. **Early-exit counting:** The current code counts matches and caps at 3.
   This is correct. Do not remove the `.count().min(3)` cap.
2. **Stagger queries across ticks:** Entities that have not changed intention
   in the last N ticks can skip the decision phase entirely. This requires an
   "intention freshness" timestamp on `ActionState`.
3. **Spatial query result caching:** For entities that have not moved, the
   nearby entity set is unchanged. Cache the count and invalidate on movement.

**Verifiable check:** Every spatial query in a Phase 3 system must have either
(a) a result cap (e.g., `.count().min(3)`) or (b) a caching/staggering
mechanism documented in a comment.

## 5. Rules for Expensive Computation Amortization

### Rule PERF-CACHE-01: Cache Pathfinding Results as Components

A* on the Paris map (8192x7424) with MAX_EXPANDED=32768 can examine up to 32k
nodes. Even with flat-array g_score (no HashMap), this is ~100-500us per call.
At 10k entities, if even 10% need a fresh path per tick, that is 1000 * 200us
= 200ms -- 20x the entire tick budget.

The `CachedPath` component exists precisely for this. The wander system reuses
cached paths when the goal has not changed. For tracking targets (Eat/Attack),
it does not cache because the target moves.

**Verifiable check:** Any system that calls `find_path` MUST:
(a) Check `cached_paths` first for the same goal.
(b) Store the result in `cached_paths` if the goal is stable.
(c) Invalidate `cached_paths` when the goal changes.
Systems tracking moving targets may skip caching but must document why.

### Rule PERF-CACHE-02: Amortize Per-Entity Work Across Ticks Using Cooldowns

The gait/cooldown system is the canonical example: movement costs N ticks per
tile. During the N-1 ticks where `remaining > 0`, the system decrements a
counter (cost: ~60ns) instead of computing a path (cost: ~200,000ns). This
is a 3000x cost reduction for 8/9 ticks (at Walk gait).

Any system where per-entity computation exceeds ~500ns should consider a
cooldown or stagger pattern:
- **Movement:** already implemented via `MoveCooldown`.
- **Decision-making:** should stagger. Not every entity needs to re-evaluate
  every tick. An entity with inertia bonus on its current action can skip
  re-evaluation for `ticks_in_action < 10` unless a high-priority stimulus
  (damage taken, food appeared) fires.
- **Spatial sensing:** the `FoodNearby`/`EnemyNearby` inputs could be cached
  for entities that have not moved.

**Verifiable check:** If a system's per-entity computation exceeds 500ns
(measured or estimated from operation counts), it must implement one of:
cooldown skip, tick staggering, or result caching. The mechanism must be
documented in the system's doc comment.

### Rule PERF-CACHE-03: Use Dirty Flags for Tile-Based Computation

The temperature system demonstrates the pattern: chunks track
`at_equilibrium`, and the system skips equilibrium chunks entirely. At steady
state, the temperature system is O(1) regardless of map size.

Any future tile-based system (influence maps, pathfinding cost overlays,
visibility) must use chunk-level dirty flags to skip stable regions.

**Verifiable check:** Every system that iterates tiles must either (a) use
chunk-level dirty/equilibrium flags to skip stable chunks, or (b) justify in
a comment why full iteration is necessary (e.g., the computation depends on
neighboring chunks that may have changed).

## 6. Data Structure Selection Principles

### Rule PERF-DS-01: Bounded Dense Key Spaces Use Flat Arrays

If the key space is known at compile time or initialization and is dense
(most keys are populated), use a flat `Vec<T>` or `[T; N]` indexed directly.

This applies to:
- **Tile data:** The TileMap uses `[Terrain; CHUNK_AREA]` and
  `[f32; CHUNK_AREA]`. Correct.
- **A* internal state:** `find_path` uses `vec![u32::MAX; w*h]` for g_score
  and came_from. Correct.
- **Per-gait data:** `GaitProfile` uses `[u32; 6]` indexed by gait enum
  discriminant. Correct.

This does NOT apply to:
- **Entity property tables.** Entity IDs are sparse (monotonically increasing,
  never reused, with gaps from despawned entities). A Vec indexed by Entity.0
  would waste memory proportional to max_entity_id, not alive_count. HashMap
  is correct here.
- **Spatial index.** Cell coordinates are sparse (most of the coordinate space
  is empty). HashMap is correct.

**Verifiable check:** When adding a new data structure, determine whether the
key space is (a) bounded and dense -- use flat array, or (b) sparse or
unbounded -- use HashMap. Document the choice in a comment.

### Rule PERF-DS-02: HashSet for Membership Tests, HashMap for Value Lookups

`pending_deaths` is `HashSet<Entity>`, not `HashMap<Entity, ()>`. The `alive`
set is `HashSet<Entity>`. This is correct: they are pure membership-test
structures. Do not use a HashMap where only `.contains()` is needed.

**Verifiable check:** If a HashMap's values are never read (only
`.contains_key()` is called), it should be a HashSet.

### Rule PERF-DS-03: BTreeMap Only for Ordered Iteration

The `UtilityConfig.actions` uses `BTreeMap<ActionId, ActionDef>`. This is
correct because the scorer iterates actions in a deterministic order. If
ordering is not needed, use HashMap (faster by ~2x for random access).

**Verifiable check:** BTreeMap is justified only when iteration order matters
for determinism or correctness. Otherwise use HashMap.

### Rule PERF-DS-04: Vec for Small Collections in Inner Loops

The `consumed` set in `run_eating` was originally a `Vec` with `.contains()`
(O(n) per call). It was changed to `HashSet`. For collections that stay under
~20 elements, Vec with linear scan is faster than HashSet due to allocation
overhead and cache locality. But the rule should target worst-case: if the
collection can grow unbounded with entity density, use HashSet.

**Verifiable check:** Inner-loop membership-test collections must use HashSet
if the collection size could exceed 50 elements under maximum expected density.
For collections provably bounded under 20 elements, Vec with `.contains()` is
acceptable but must include a comment stating the bound.

## 7. The Invisible O(n^2) Problem

### Rule PERF-QUAD-01: Identify Nested Entity Iteration

An O(n^2) pattern occurs when a system iterates all entities and, for each
entity, iterates all entities again (or a subset proportional to n). The
canonical example:

```rust
for attacker in &combatants {
    for defender in &combatants {  // O(n^2)
        if same_tile(attacker, defender) { ... }
    }
}
```

The correct pattern uses the spatial index to reduce the inner loop to
O(entities_per_cell):

```rust
for attacker in &combatants {
    for defender in world.entities_at(ax, ay) {  // O(k), k << n
        ...
    }
}
```

Both combat and eating already use the spatial index for the inner lookup.

**Verifiable check:** No system may contain a nested loop where both the outer
and inner loops iterate entity collections of size proportional to total entity
count. The inner loop must be bounded by a spatial query, a fixed-size
collection, or a constant.

### Rule PERF-QUAD-02: Target Worst-Case for Growing Collections

The `consumed` HashSet in eating grows with the number of eat actions per tick.
In a pathological case (all entities eating at the same tile), it could reach
n entries, making each `.contains()` call O(1) amortized but the total
insertions O(n). This is fine. The problem would be if `consumed` were a Vec:
`.contains()` would be O(n) per call, making the total O(n^2).

The general rule: if a collection used for membership testing inside a loop
grows proportionally to the loop count, it must be a HashSet or sorted Vec
with binary search.

**Verifiable check:** Every `.contains()` call inside a loop must be on a
HashSet, BTreeSet, or sorted Vec. Plain Vec `.contains()` inside a loop is
permitted only with a comment proving the Vec size is bounded by a constant.

### Rule PERF-QUAD-03: Watch for Hidden Quadratics in Spatial Queries

`entities_in_range` with SENSE_RANGE=30 on a dense map can return hundreds of
entities. If the caller then does O(k) work per returned entity (e.g., another
spatial query), the total is O(n * k^2). The decisions system avoids this by
only counting nearby entities, not doing per-neighbor work.

**Verifiable check:** If a spatial query result is iterated with per-element
work exceeding O(1), the total complexity must be documented in a comment and
must not exceed O(n * k) where k is the expected spatial query result size.

## 8. System Authoring Patterns for Complexity Budget Compliance

### Pattern PERF-PAT-01: The Cheap System

Systems like `run_hunger` and `run_fatigue` do O(1) work per entity: one table
read, one arithmetic operation, one table write. Total cost: ~100ns per entity.
These systems will never be bottlenecks.

**Template:**
```
collect changes (iterate primary table, filter pending_deaths, compute new value)
sort changes by entity ID
apply changes (iterate changes Vec, mutate primary table)
```

### Pattern PERF-PAT-02: The Spatial System

Systems like `run_eating` and `run_combat` need same-tile or nearby-entity
information. Cost: ~30ns for primary table iteration + ~1000ns for spatial
query per entity that needs one.

**Template:**
```
collect candidates (iterate primary table, filter, extract position)
sort candidates by entity ID
for each candidate:
    check preferred target via direct lookup (O(1))
    if no preferred target: spatial query with bounded range
    collect action into changes Vec
apply changes
```

### Pattern PERF-PAT-03: The Expensive System

Systems like `run_wander` (with A* pathfinding) have per-entity costs that
can exceed 100,000ns. These must use amortization.

**Template:**
```
collect candidates (iterate primary table, filter, extract position + cooldown)
sort candidates by entity ID
for each candidate:
    if cooldown > 0: decrement cooldown, skip (cost: 60ns)
    if cached result valid: use cached result (cost: 100ns)
    else: compute expensive result (cost: 100,000ns+)
    collect changes
apply changes
```

The critical insight: if 90% of entities are in cooldown or cache-hit state,
the effective per-entity cost drops from 100,000ns to ~10,000ns average.

### Pattern PERF-PAT-04: The Environment System

Systems like `run_temperature` operate on tiles, not entities. Cost model is
completely different: O(tiles_needing_update), not O(entities). With dirty
flags, steady-state cost is O(1).

**Template:**
```
for each chunk:
    if chunk.at_equilibrium: skip
    for each tile in chunk:
        if tile at target: skip
        collect change
    if no changes: mark chunk equilibrium
apply changes
```

## 9. The LLM Agent Constraint

These rules exist because the development agent reasons from CLAUDE.md rules,
not from profiler output. An LLM cannot run `perf` or `cargo bench`. It can
only reason about complexity from code structure. Therefore:

### Rule PERF-LLM-01: Complexity Must Be Statically Visible

Every loop's iteration count must be traceable to a named collection. Nested
loops must have their combined complexity stated in a comment. An LLM can
verify "this loop iterates `candidates` (n entities) and calls `entities_at`
(O(k) per cell)" but cannot verify "this runs in 0.3ms on my machine."

**Verifiable check:** Every system's doc comment must state its expected
per-tick complexity in big-O notation, parameterized by entity count (n),
spatial density (k), and map size (w*h) as appropriate.

### Rule PERF-LLM-02: Expensive Operations Must Be Named Constants

A* search limit is `MAX_EXPANDED = 32_768`. Spatial sense range is
`SENSE_RANGE = 30`. Wander range is `WANDER_RANGE = 30`. These are the
performance-critical tuning knobs. An LLM can reason about the cost
implications of changing `SENSE_RANGE` from 30 to 100 (spatial query cost
increases ~11x) because the constant is named and documented.

**Verifiable check:** Every value that affects computational complexity must be
a named `const` with a comment stating its performance implication.

### Rule PERF-LLM-03: The 500ns Threshold Test

When authoring a system, the LLM should mentally estimate per-entity cost:
- Count HashMap lookups: ~30ns each
- Count arithmetic operations: ~3ns each
- Count spatial queries: ~1000-5000ns each
- Count pathfinding calls: ~100,000-500,000ns each

If the estimated total exceeds 500ns, the system must implement amortization.
If it exceeds 5000ns, the system must implement aggressive amortization
(cooldowns with >90% skip rate).

**Verifiable check:** New systems must include a comment estimating per-entity
cost based on operation counts. If the estimate exceeds 500ns, the amortization
strategy must be documented.

## 10. Concrete Recommendations for the Current Codebase

### 10.1 Decisions System: Stagger Re-evaluation

`run_decisions` is the most expensive system per tick due to spatial queries
for every entity every tick. Recommendation: entities with `ticks_in_action < 5`
and no high-priority stimulus should skip re-evaluation and retain their current
intention. This would reduce spatial query volume by ~80% at steady state.

Implementation: in the scoring loop, before evaluating considerations, check
if `ticks_in_action < STAGGER_THRESHOLD` and no "interrupt" condition holds
(damage taken this tick, health below threshold). If skip conditions are met,
re-insert the current intention and continue.

### 10.2 Wander System: Pool A* Allocations

`find_path` allocates three `Vec`s per call: `g_score`, `came_from`, and
`closed`. On the Paris map (8192x7424 = 60M tiles), these are 60M * (4+4+1) =
540MB per call. This is the actual bottleneck -- not the search itself but the
allocation.

Recommendation: add a reusable `PathfindingWorkspace` struct to World with
pre-allocated flat arrays. `find_path` takes a `&mut PathfindingWorkspace`
instead of allocating internally. The workspace is cleared (not deallocated)
between calls.

### 10.3 Pre-Tick Sorted Entity Cache

Add `World::sorted_entities: Vec<Entity>` rebuilt once per tick (in the same
pass as `rebuild_spatial_index`). Systems can borrow this instead of collecting
and sorting independently. Saves ~1.5ms per tick at 10k entities (6 sorts
eliminated).

### 10.4 The `clone()` in Decisions

`run_decisions` calls `world.mind.utility_config.clone()` to work around
borrow checker constraints (it needs `&world` for reads and `&mut world` for
writes within the same function). The `UtilityConfig` contains `BTreeMap` with
`Vec<Consideration>` per action. This clone allocates. At 4 actions with 2-3
considerations each, the allocation is small (~200 bytes). But it happens every
tick. Recommendation: restructure to extract config as a local reference before
the mutable phase, or accept the cost with a comment.

## 11. Summary of All Verifiable Rules

| Rule ID | One-Line Summary |
|---------|-----------------|
| PERF-ITER-01 | Iterate the smallest relevant table |
| PERF-ITER-02 | Filter pending_deaths before secondary lookups |
| PERF-ITER-03 | Do not re-read values already extracted in the same function |
| PERF-ITER-04 | Pre-size collection Vecs with `with_capacity` |
| PERF-SORT-01 | One sort per system on primary candidates |
| PERF-SORT-02 | Omit sort only if no RNG, independent results, no deaths |
| PERF-SORT-03 | Consider per-tick sorted entity cache if sorting > 15% budget |
| PERF-SPATIAL-01 | Direct lookup when entity ID is known |
| PERF-SPATIAL-02 | Minimum necessary range with documented const |
| PERF-SPATIAL-03 | Exactly two spatial rebuilds per tick |
| PERF-SPATIAL-04 | Spatial queries in Phase 3 must have result caps or caching |
| PERF-CACHE-01 | Cache pathfinding results as components |
| PERF-CACHE-02 | Amortize >500ns per-entity work across ticks |
| PERF-CACHE-03 | Tile systems must use chunk dirty flags |
| PERF-DS-01 | Bounded dense keys use flat arrays |
| PERF-DS-02 | HashSet for membership, HashMap for values |
| PERF-DS-03 | BTreeMap only when iteration order matters |
| PERF-DS-04 | HashSet for inner-loop membership if size > 50 |
| PERF-QUAD-01 | No nested entity loops without spatial index |
| PERF-QUAD-02 | Growing loop collections must be HashSet |
| PERF-QUAD-03 | Document complexity of spatial query consumers |
| PERF-LLM-01 | State big-O in system doc comments |
| PERF-LLM-02 | Named constants for complexity-affecting values |
| PERF-LLM-03 | 500ns threshold test for amortization requirement |
