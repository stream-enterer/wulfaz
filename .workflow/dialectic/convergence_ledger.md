# Convergence Ledger: Round 4 Final Scores

## Process Overview

This ledger is the product of a 4-round dialectic process applied to 73 propositions
about performance guidelines for the Wulfaz simulation engine:

1. **Round 1** -- Three independent agents each produced propositions scored on four axes:
   defensibility, specificity, robustness, and compatibility. Agent 1 (25 propositions)
   focused on per-system optimization rules. Agent 2 (25 propositions) focused on
   phase-level budget constraints and caching strategy. Agent 3 (23 propositions) focused
   on architectural failure modes and scaling limits.
2. **Round 2** -- A conflict map identified 35 tensions across the 73 propositions,
   each with a severity score (0.50--0.92).
3. **Round 3** -- Each tension cluster went through prosecution/defense/adjudication passes.
   The adjudication produced final deltas to the original scores.
4. **Round 4 (this document)** -- Deltas applied, composite scores computed, propositions
   categorized as Survivor/Wounded/Contested/Fallen.

**Composite** = mean of all four axes after deltas.

| Category | Criteria | Count |
|----------|----------|-------|
| Survivor | composite >= 0.75 AND no axis below 0.50 | 69 |
| Wounded | composite 0.50--0.74 OR any axis below 0.50 | 4 |
| Contested | wounded + high-severity unresolved tension + mixed deltas | 0 |
| Fallen | composite < 0.50 OR defensibility < 0.30 | 0 |

---

## Final Scoreboard

Sorted by composite score (descending).

| # | ID | Composite | Def | Spec | Rob | Compat | Category |
|---|-----|-----------|-----|------|-----|--------|----------|
| 1 | a1-09 | **0.94** | 0.95 | 0.95 | 0.93 | 0.95 | SURVIVOR |
| 2 | a2-14 | **0.94** | 0.95 | 0.95 | 0.92 | 0.95 | SURVIVOR |
| 3 | a2-01 | **0.93** | 0.97 | 0.93 | 0.88 | 0.95 | SURVIVOR |
| 4 | a2-18 | **0.93** | 0.95 | 0.92 | 0.90 | 0.95 | SURVIVOR |
| 5 | a3-05 | **0.92** | 0.94 | 0.93 | 0.92 | 0.90 | SURVIVOR |
| 6 | a3-18 | **0.92** | 0.96 | 0.88 | 0.93 | 0.92 | SURVIVOR |
| 7 | a3-14 | **0.92** | 0.95 | 0.90 | 0.91 | 0.92 | SURVIVOR |
| 8 | a2-15 | **0.92** | 0.96 | 0.95 | 0.85 | 0.90 | SURVIVOR |
| 9 | a2-19 | **0.92** | 0.93 | 0.88 | 0.90 | 0.95 | SURVIVOR |
| 10 | a3-04 | **0.92** | 0.96 | 0.95 | 0.90 | 0.85 | SURVIVOR |
| 11 | a1-02 | **0.91** | 0.92 | 0.88 | 0.90 | 0.95 | SURVIVOR |
| 12 | a1-03 | **0.91** | 0.90 | 0.92 | 0.88 | 0.95 | SURVIVOR |
| 13 | a1-17 | **0.91** | 0.93 | 0.90 | 0.90 | 0.92 | SURVIVOR |
| 14 | a2-02 | **0.91** | 0.92 | 0.90 | 0.85 | 0.95 | SURVIVOR |
| 15 | a3-01 | **0.91** | 0.97 | 0.92 | 0.88 | 0.85 | SURVIVOR |
| 16 | a2-10 | **0.90** | 0.92 | 0.85 | 0.88 | 0.95 | SURVIVOR |
| 17 | a1-16 | **0.90** | 0.92 | 0.88 | 0.83 | 0.95 | SURVIVOR |
| 18 | a2-08 | **0.90** | 0.95 | 0.95 | 0.78 | 0.90 | SURVIVOR |
| 19 | a1-06 | **0.89** | 0.88 | 0.90 | 0.85 | 0.92 | SURVIVOR |
| 20 | a1-20 | **0.89** | 0.90 | 0.88 | 0.85 | 0.92 | SURVIVOR |
| 21 | a2-06 | **0.89** | 0.93 | 0.88 | 0.82 | 0.92 | SURVIVOR |
| 22 | a1-10 | **0.89** | 0.88 | 0.90 | 0.84 | 0.92 | SURVIVOR |
| 23 | a2-22 | **0.89** | 0.92 | 0.85 | 0.85 | 0.92 | SURVIVOR |
| 24 | a3-13 | **0.89** | 0.93 | 0.93 | 0.86 | 0.82 | SURVIVOR |
| 25 | a3-03 | **0.88** | 0.96 | 0.94 | 0.75 | 0.88 | SURVIVOR |
| 26 | a3-11 | **0.88** | 0.90 | 0.91 | 0.85 | 0.87 | SURVIVOR |
| 27 | a2-05 | **0.88** | 0.90 | 0.88 | 0.83 | 0.90 | SURVIVOR |
| 28 | a3-20 | **0.87** | 0.95 | 0.86 | 0.88 | 0.80 | SURVIVOR |
| 29 | a2-11 | **0.87** | 0.90 | 0.88 | 0.78 | 0.92 | SURVIVOR |
| 30 | a1-18 | **0.87** | 0.85 | 0.90 | 0.82 | 0.90 | SURVIVOR |
| 31 | a3-17 | **0.87** | 0.93 | 0.81 | 0.88 | 0.85 | SURVIVOR |
| 32 | a3-23 | **0.87** | 0.92 | 0.84 | 0.86 | 0.85 | SURVIVOR |
| 33 | a3-10 | **0.86** | 0.89 | 0.84 | 0.85 | 0.88 | SURVIVOR |
| 34 | a1-13 | **0.86** | 0.90 | 0.90 | 0.75 | 0.90 | SURVIVOR |
| 35 | a2-12 | **0.86** | 0.88 | 0.90 | 0.82 | 0.85 | SURVIVOR |
| 36 | a3-19 | **0.86** | 0.87 | 0.89 | 0.82 | 0.86 | SURVIVOR |
| 37 | a2-03 | **0.86** | 0.88 | 0.85 | 0.80 | 0.90 | SURVIVOR |
| 38 | a3-06 | **0.86** | 0.93 | 0.85 | 0.83 | 0.82 | SURVIVOR |
| 39 | a1-04 | **0.85** | 0.85 | 0.87 | 0.80 | 0.90 | SURVIVOR |
| 40 | a1-15 | **0.85** | 0.87 | 0.85 | 0.80 | 0.90 | SURVIVOR |
| 41 | a1-23 | **0.85** | 0.90 | 0.92 | 0.78 | 0.82 | SURVIVOR |
| 42 | a2-07 | **0.85** | 0.90 | 0.90 | 0.77 | 0.85 | SURVIVOR |
| 43 | a3-08 | **0.85** | 0.94 | 0.85 | 0.87 | 0.75 | SURVIVOR |
| 44 | a1-12 | **0.85** | 0.87 | 0.90 | 0.75 | 0.88 | SURVIVOR |
| 45 | a3-02 | **0.85** | 0.88 | 0.90 | 0.82 | 0.80 | SURVIVOR |
| 46 | a3-21 | **0.84** | 0.93 | 0.88 | 0.72 | 0.85 | SURVIVOR |
| 47 | a2-13 | **0.84** | 0.92 | 0.88 | 0.85 | 0.72 | SURVIVOR |
| 48 | a2-21 | **0.84** | 0.88 | 0.90 | 0.70 | 0.88 | SURVIVOR |
| 49 | a2-24 | **0.84** | 0.85 | 0.88 | 0.75 | 0.88 | SURVIVOR |
| 50 | a3-09 | **0.84** | 0.86 | 0.87 | 0.80 | 0.83 | SURVIVOR |
| 51 | a3-15 | **0.83** | 0.86 | 0.83 | 0.80 | 0.85 | SURVIVOR |
| 52 | a1-07 | **0.83** | 0.90 | 0.93 | 0.75 | 0.75 | SURVIVOR |
| 53 | a3-07 | **0.83** | 0.85 | 0.90 | 0.78 | 0.80 | SURVIVOR |
| 54 | a3-16 | **0.83** | 0.85 | 0.80 | 0.78 | 0.90 | SURVIVOR |
| 55 | a2-04 | **0.83** | 0.85 | 0.92 | 0.67 | 0.88 | SURVIVOR |
| 56 | a2-23 | **0.82** | 0.85 | 0.85 | 0.70 | 0.90 | SURVIVOR |
| 57 | a3-22 | **0.82** | 0.85 | 0.80 | 0.77 | 0.88 | SURVIVOR |
| 58 | a1-11 | **0.82** | 0.83 | 0.88 | 0.70 | 0.88 | SURVIVOR |
| 59 | a2-09 | **0.82** | 0.88 | 0.90 | 0.65 | 0.85 | SURVIVOR |
| 60 | a1-19 | **0.81** | 0.82 | 0.87 | 0.70 | 0.85 | SURVIVOR |
| 61 | a1-05 | **0.81** | 0.75 | 0.90 | 0.65 | 0.92 | SURVIVOR |
| 62 | a1-25 | **0.80** | 0.80 | 0.90 | 0.62 | 0.88 | SURVIVOR |
| 63 | a2-25 | **0.80** | 0.79 | 0.92 | 0.63 | 0.85 | SURVIVOR |
| 64 | a3-12 | **0.79** | 0.84 | 0.85 | 0.70 | 0.78 | SURVIVOR |
| 65 | a2-16 | **0.79** | 0.86 | 0.90 | 0.62 | 0.78 | SURVIVOR |
| 66 | a2-17 | **0.79** | 0.80 | 0.95 | 0.60 | 0.80 | SURVIVOR |
| 67 | a1-01 | **0.78** | 0.77 | 0.93 | 0.58 | 0.85 | SURVIVOR |
| 68 | a1-14 | **0.76** | 0.75 | 0.85 | 0.60 | 0.85 | SURVIVOR |
| 69 | a1-24 | **0.76** | 0.72 | 0.85 | 0.68 | 0.80 | SURVIVOR |
| 70 | a1-21 | **0.75** | 0.75 | 0.92 | 0.50 | 0.82 | WOUNDED |
| 71 | a2-20 | **0.74** | 0.75 | 0.88 | 0.53 | 0.82 | WOUNDED |
| 72 | a1-22 | **0.74** | 0.82 | 0.88 | 0.58 | 0.68 | WOUNDED |
| 73 | a1-08 | **0.70** | 0.62 | 0.88 | 0.53 | 0.78 | WOUNDED |

---

## Survivors

69 propositions survived the dialectic. These should inform the CLAUDE.md ruleset.

### Core Rules (high specificity + high robustness)

These propositions have specificity >= 0.85 AND robustness >= 0.80 after deltas.
They are concrete, actionable, and hold across scenarios. Strongest candidates for
direct inclusion in CLAUDE.md as hard rules.

#### a1-09 (composite: 0.94)
> If a system has access to a specific Entity ID, it must use positions.get(&entity) rather than a spatial query to find that entity's location, because direct lookup costs ~30ns versus ~1000ns minimum for a spatial query (PERF-SPATIAL-01).

> Scores: def=0.95 spec=0.95 rob=0.93 compat=0.95

#### a2-14 (composite: 0.94)
> When key space maps to tile coordinates (bounded, dense), always use a flat array indexed by y * width + x; never use HashMap<(i32, i32), T> for tile-indexed data, because flat-array lookup costs ~5ns versus HashMap's ~50-80ns.

> Scores: def=0.95 spec=0.95 rob=0.92 compat=0.95

#### a2-01 (composite: 0.93)
> A single-threaded simulation engine running at 100 ticks/sec has a hard budget of 10,000 microseconds per tick, and even after path caching brought wander from 75ms to 9ms, the total tick at 50k entities still exceeds 10ms because the problem is the sum of systems that each seem cheap in isolation.

> Scores: def=0.97 spec=0.93 rob=0.88 compat=0.95
> Tensions survived: t-14, t-15, t-24, t-28 (budget disagreement cluster)

#### a2-18 (composite: 0.93)
> Use HashSet for membership testing (pending_deaths, consumed sets); never use Vec::contains for O(n) membership checks, because the benchmark showed eating at 28ms partially due to this O(n^2) pattern.

> Scores: def=0.95 spec=0.92 rob=0.90 compat=0.95

#### a3-05 (composite: 0.92)
> The benchmark's 256x256 map makes A* flat arrays cost 590 KB per call, which is 1175x smaller than the production map's 693 MB, meaning A* performance regressions are invisible in the benchmark and can only be detected by profiling on the production map geometry.

> Scores: def=0.94 spec=0.93 rob=0.92 compat=0.90

#### a3-18 (composite: 0.92)
> The benchmark exercises the wrong scale (256x256 vs 11000x7000), the wrong density distribution (uniform vs clustered-in-buildings), the wrong terrain topology (random swiss-cheese vs connected urban), and the wrong entity type mix (uniform creatures vs heterogeneous citizens and items), making it structurally incapable of detecting any of the failure modes that will manifest in production.

> Scores: def=0.96 spec=0.88 rob=0.93 compat=0.92
> Tensions survived: t-23, t-25 (profiling blindness)

#### a3-14 (composite: 0.92)
> The benchmark uses uniform random entity placement on a 256x256 grid, which produces Poisson-distributed density (~12 entities per spatial cell), while the production Paris map concentrates entities in buildings where a single 16x16 spatial cell can contain 200+ entities, making all density-dependent algorithms 10-60x more expensive than the benchmark predicts.

> Scores: def=0.95 spec=0.90 rob=0.91 compat=0.92
> Tensions survived: t-10 through t-13, t-26 (spatial optimizations vs production reality)

#### a2-15 (composite: 0.92)
> A* pathfinding must use pooled buffers with generation-counter clearing rather than allocating fresh vectors per find_path call, because at 256x256 each call allocates 768KB (3 x 65,536 x 4 bytes), and 50 calls per tick means 37.5MB of allocation churn.

> Scores: def=0.96 spec=0.95 rob=0.85 compat=0.90
> Tensions survived: t-21, t-22 (pathfinding workspace design)

#### a2-19 (composite: 0.92)
> Systems with cooldown mechanics must test the timer first and early-exit in O(1), with no secondary lookups, spatial queries, or allocation for entities on cooldown.

> Scores: def=0.93 spec=0.88 rob=0.90 compat=0.95

#### a3-04 (composite: 0.92)
> A single A* call on the production 11000x7000 map allocates and initializes 693 MB of flat arrays (308 MB g_score + 308 MB came_from + 77 MB closed), making every unnecessary pathfinding call catastrophic: even 1% of 50k entities re-pathfinding per tick means 500 calls each allocating and freeing 693 MB.

> Scores: def=0.96 spec=0.95 rob=0.90 compat=0.85

#### a1-02 (composite: 0.91)
> A system should iterate the table with the fewest entries that still captures all entities it needs to process (PERF-ITER-01).

> Scores: def=0.92 spec=0.88 rob=0.90 compat=0.95

#### a1-03 (composite: 0.91)
> Every system's candidate collection must filter pending_deaths before any secondary table lookup (PERF-ITER-02), with filter ordering: cheapest rejection first.

> Scores: def=0.90 spec=0.92 rob=0.88 compat=0.95

#### a1-17 (composite: 0.91)
> No system may contain a nested loop where both the outer and inner loops iterate entity collections of size proportional to total entity count; the inner loop must be bounded by a spatial query, a fixed-size collection, or a constant (PERF-QUAD-01).

> Scores: def=0.93 spec=0.90 rob=0.90 compat=0.92

#### a2-02 (composite: 0.91)
> Phase 1 (Environment) systems must operate on tiles/chunks only, never iterate entities; any Phase 1 system that touches entities is misclassified and must move to Phase 2+.

> Scores: def=0.92 spec=0.90 rob=0.85 compat=0.95

#### a3-01 (composite: 0.91)
> When entities cluster spatially (as they will on the real Paris map inside buildings), every system calling entities_in_range produces O(d^2) cost per spatial cell where d is local density, and multiple systems calling it independently produce O(k * d^2) rather than O(k * d), because each caller re-scans the full cell contents for every entity in that cell.

> Scores: def=0.97 spec=0.92 rob=0.88 compat=0.85
> Tensions survived: t-10 through t-13, t-26 (spatial optimizations vs production reality)

#### a2-10 (composite: 0.90)
> Movement cooldowns naturally throttle pathfinding: at Walk gait (9 ticks/tile), only ~11% of entities request movement per tick, and systems must not bypass this rate-limiter by pre-computing paths for entities still on cooldown.

> Scores: def=0.92 spec=0.85 rob=0.88 compat=0.95

#### a1-16 (composite: 0.90)
> Entity property tables must use HashMap because entity IDs are sparse (monotonically increasing, never reused, with gaps from despawned entities), and a Vec indexed by Entity.0 would waste memory proportional to max_entity_id rather than alive_count (PERF-DS-01).

> Scores: def=0.92 spec=0.88 rob=0.83 compat=0.95
> Tensions survived: t-27 (HashMap structural ceiling)

#### a1-06 (composite: 0.89)
> The sort by Entity.0 must happen exactly once per system on the primary candidate list; secondary sorts are permitted only on Vecs derived from spatial queries or secondary HashMap iterations within the per-entity processing loop (PERF-SORT-01).

> Scores: def=0.88 spec=0.90 rob=0.85 compat=0.92

#### a1-20 (composite: 0.89)
> Every value that affects computational complexity must be a named const with a comment stating its performance implication (PERF-LLM-02).

> Scores: def=0.90 spec=0.88 rob=0.85 compat=0.92

#### a2-06 (composite: 0.89)
> Phase 3 (Decisions) must pre-filter entities with O(1) checks before issuing spatial queries; spatial queries at SENSE_RANGE scale must never be called for all n entities unconditionally.

> Scores: def=0.93 spec=0.88 rob=0.82 compat=0.92

#### a1-10 (composite: 0.89)
> Every call to entities_in_range must use the minimum range that satisfies the system's design, with the range defined as a named const with a comment stating the real-world distance and justification (PERF-SPATIAL-02).

> Scores: def=0.88 spec=0.90 rob=0.84 compat=0.92
> Tensions survived: t-35 (SENSE_RANGE minimization methodology gap)

#### a2-22 (composite: 0.89)
> Cache invalidation must be checked by the consuming system (not the producing system), comparing cached assumptions against current world state; every cached component must include the assumptions it was computed under.

> Scores: def=0.92 spec=0.85 rob=0.85 compat=0.92

#### a3-13 (composite: 0.89)
> The EventLog ring buffer with 10k capacity receives 50k+ events per tick from the hunger system alone, meaning 80%+ of constructed Event structs are immediately overwritten before any consumer reads them, wasting the CPU cycles spent constructing those events and polluting cache lines with writes to a buffer that wraps 5+ times per tick.

> Scores: def=0.93 spec=0.93 rob=0.86 compat=0.82
> Tensions survived: t-30 (event push frequency)

#### a3-11 (composite: 0.88)
> Each new property table added to World increases despawn cost by one HashMap::remove per entity death, validation cost by one full-table iteration in debug builds, and cache pollution by one additional disjoint memory region per cross-table lookup, and at 40 tables with 50k entities the validation pass alone costs ~16ms per tick in debug mode, exceeding the tick budget.

> Scores: def=0.90 spec=0.91 rob=0.85 compat=0.87

#### a2-05 (composite: 0.88)
> Phase 2 systems that push events per entity must push events only on state transitions (value crosses a threshold), not on every tick increment, because unconditional event pushes at 50k entities add approximately 1,000us of overhead.

> Scores: def=0.90 spec=0.88 rob=0.83 compat=0.90
> Tensions survived: t-30 (event push frequency)

#### a3-20 (composite: 0.87)
> The determinism test (test_wander_deterministic_with_seed) only verifies that the same seed produces the same result, but does not detect that adding a single RNG call in a new system changes the output of every downstream system, meaning the test suite provides false confidence that determinism is robust when it is actually fragile to any system modification.

> Scores: def=0.95 spec=0.86 rob=0.88 compat=0.80
> Tensions survived: t-01 through t-05 (sort/determinism cluster)

#### a1-18 (composite: 0.87)
> Every .contains() call inside a loop must be on a HashSet, BTreeSet, or sorted Vec with binary search; plain Vec .contains() inside a loop is permitted only with a comment proving the Vec size is bounded by a constant (PERF-QUAD-02).

> Scores: def=0.85 spec=0.90 rob=0.82 compat=0.90

#### a2-12 (composite: 0.86)
> The sort-by-entity-ID requirement for deterministic replay imposes a hidden O(n log n) tax per system: at 50k entities, sorting costs approximately 3,900us per system, and with 5+ systems sorting independently, the total sort cost approaches 8,000-20,000us.

> Scores: def=0.88 spec=0.90 rob=0.82 compat=0.85

#### a3-19 (composite: 0.86)
> If a Phase 4 system is inserted between run_wander and rebuild_spatial_index, it sees an inconsistent spatial index where some entities have moved but the index reflects pre-movement positions, and the natural 'fix' (adding another rebuild) costs O(n) per additional rebuild while setting a precedent that accumulates rebuilds over time.

> Scores: def=0.87 spec=0.89 rob=0.82 compat=0.86

#### a2-03 (composite: 0.86)
> Phase 1 systems must implement chunk-level dirty/equilibrium flags so that steady-state cost is O(active_chunks) rather than O(total_tiles), with full tile iteration permitted only for chunks not at equilibrium.

> Scores: def=0.88 spec=0.85 rob=0.80 compat=0.90

#### a3-06 (composite: 0.86)
> The cumulative HashMap operation count across all systems grows as (system_count * entity_count * tables_referenced_per_system), and at 50k entities with 15 systems each referencing 4-5 tables, the total exceeds 5 million hash operations per tick, where even with aHash the per-operation cost of hashing + probing + cache misses across disjoint table memory regions becomes a dominant fraction of tick budget.

> Scores: def=0.93 spec=0.85 rob=0.83 compat=0.82
> Tensions survived: t-14, t-15, t-24, t-28 (budget disagreement cluster), t-27 (HashMap structural ceiling)

#### a1-04 (composite: 0.85)
> Within a single system function, no HashMap::get call should read a value that was already extracted in the same function and cannot have changed (PERF-ITER-03).

> Scores: def=0.85 spec=0.87 rob=0.80 compat=0.90

#### a1-15 (composite: 0.85)
> Every system that iterates tiles must either use chunk-level dirty/equilibrium flags to skip stable chunks, or justify in a comment why full iteration is necessary (PERF-CACHE-03).

> Scores: def=0.87 spec=0.85 rob=0.80 compat=0.90

#### a3-08 (composite: 0.85)
> The shared seeded RNG creates a butterfly effect where any change to one system's conditional RNG consumption (e.g., tweaking a threshold that changes which entities roll dice) shifts the RNG stream for all downstream systems, making it impossible to bisect behavior changes because every parameter tweak produces a global cascade through combat outcomes, movement choices, and entity deaths.

> Scores: def=0.94 spec=0.85 rob=0.87 compat=0.75
> Tensions survived: t-01 through t-05 (sort/determinism cluster), t-18 through t-20, t-34 (decision staggering vs RNG butterfly)

#### a3-02 (composite: 0.85)
> The spatial index rebuild (clear + reinsert all entities) performs 4000+ Vec allocations and deallocations per rebuild on a 256x256 map, and each additional rebuild_spatial_index call added by a new system doubles this allocation traffic, establishing a precedent where 'just rebuild when needed' accumulates to consume 25%+ of tick budget by the time 5 rebuilds exist.

> Scores: def=0.88 spec=0.90 rob=0.82 compat=0.80

#### a2-13 (composite: 0.84)
> Systems must sort entity IDs only once per system invocation, not per inner loop, and if multiple systems in the same phase iterate the same sorted entity set, the sorted Vec should be passed between them rather than re-sorted.

> Scores: def=0.92 spec=0.88 rob=0.85 compat=0.72
> Tensions survived: t-01 through t-05 (sort/determinism cluster)

#### a3-09 (composite: 0.84)
> The collect-then-apply pattern mandated by the architecture generates 8-10 MB of Vec buffer allocations per tick at 50k entities across 8 systems, producing 800 MB to 1 GB of allocation churn per second that fragments the allocator's free list and causes fresh mmap calls when large allocations (A* arrays) can no longer find contiguous free blocks.

> Scores: def=0.86 spec=0.87 rob=0.80 compat=0.83

### Conditional Guidance (high defensibility, lower robustness)

These propositions scored well overall but have specificity < 0.85 or robustness < 0.80.
They are sound in principle but need qualification, conditions, or caveats before
inclusion in CLAUDE.md.

#### a2-08 (composite: 0.90)
> Phase 4 must not call find_path per entity per tick; paths must be cached as components (CachedPath) and reused across ticks until invalidated by goal change, path blockage, or path exhaustion, with fresh A* calls capped at 30-50 per tick.

> Scores: def=0.95 spec=0.95 rob=0.78 compat=0.90
> Deltas applied: robustness -0.07
> Tensions: t-06 through t-09 (path caching vs terrain invalidation)

#### a3-03 (composite: 0.88)
> No system currently invalidates cached paths when terrain walkability changes, so the first system that modifies tile walkability at runtime (flooding, doors, building collapse) will silently corrupt all cached paths passing through affected tiles, causing mass path cache misses that each trigger a full A* recomputation.

> Scores: def=0.96 spec=0.94 rob=0.75 compat=0.88
> Deltas applied: defensibility +0.03; specificity +0.03
> Tensions: t-06 through t-09 (path caching vs terrain invalidation)

#### a2-11 (composite: 0.87)
> Phase 5 systems must iterate only the affected set (pending_deaths, co-located pairs, event-driven triggers), never the full entity population, with a budget ceiling of 500us at 50k entities under normal conditions.

> Scores: def=0.90 spec=0.88 rob=0.78 compat=0.92

#### a3-17 (composite: 0.87)
> No individual failure mode in this architecture produces a single-system hotspot attributable by profiling to one call site; the costs are distributed across all systems equally (HashMap lookups, sorting, allocation), making the performance disease architectural and undiagnosable by standard profiling techniques that attribute cost to individual functions.

> Scores: def=0.93 spec=0.81 rob=0.88 compat=0.85
> Deltas applied: defensibility +0.03; specificity +0.03
> Tensions: t-16 (per-system estimation vs distributed overhead), t-23, t-25 (profiling blindness)

#### a3-23 (composite: 0.87)
> The HashMap operation cost grows as the product of system count and table count (not their sum), because each new system that references a new table adds (entity_count) additional cross-table lookups, meaning the scaling is multiplicative and the per-system marginal cost increases as the architecture grows rather than remaining constant.

> Scores: def=0.92 spec=0.84 rob=0.86 compat=0.85
> Deltas applied: defensibility +0.04
> Tensions: t-27 (HashMap structural ceiling)

#### a3-10 (composite: 0.86)
> Phase boundary violations are unenforced by the compiler: nothing prevents a Phase 3 system from writing positions (Phase 4 territory) or a Phase 4 system from reading hungers (Phase 2 territory), and each violation triggers compensating workarounds (extra spatial rebuilds, re-running decision phases) that silently accumulate infrastructure cost.

> Scores: def=0.89 spec=0.84 rob=0.85 compat=0.88

#### a1-13 (composite: 0.86)
> Any system that calls find_path must check cached_paths first for the same goal, store the result in cached_paths if the goal is stable, and invalidate cached_paths when the goal changes; systems tracking moving targets may skip caching but must document why (PERF-CACHE-01).

> Scores: def=0.90 spec=0.90 rob=0.75 compat=0.90
> Deltas applied: robustness -0.10; specificity -0.02
> Tensions: t-06 through t-09 (path caching vs terrain invalidation), t-31 (cache invalidation completeness)

#### a1-23 (composite: 0.85)
> find_path allocates three Vecs per call (g_score, came_from, closed) totaling 540MB on the Paris map (8192x7424); a reusable PathfindingWorkspace struct with pre-allocated flat arrays should be added to World, cleared between calls rather than reallocated.

> Scores: def=0.90 spec=0.92 rob=0.78 compat=0.82
> Deltas applied: robustness -0.07
> Tensions: t-21, t-22 (pathfinding workspace design)

#### a2-07 (composite: 0.85)
> SENSE_RANGE must be the minimum required by gameplay because every doubling of range quadruples the number of spatial cells scanned; current SENSE_RANGE=30 with CELL_SIZE=16 scans 16 cells, while SENSE_RANGE=64 would scan 25 cells.

> Scores: def=0.90 spec=0.90 rob=0.77 compat=0.85
> Deltas applied: robustness -0.03
> Tensions: t-35 (SENSE_RANGE minimization methodology gap)

#### a1-12 (composite: 0.85)
> Every spatial query in a Phase 3 system must have either a result cap (e.g., .count().min(3)) or a caching/staggering mechanism documented in a comment, because the decisions system's spatial queries are the single most expensive operation in the tick loop at 3.2M entity comparisons for 10k entities (PERF-SPATIAL-04).

> Scores: def=0.87 spec=0.90 rob=0.75 compat=0.88
> Deltas applied: robustness -0.07
> Tensions: t-10 through t-13, t-26 (spatial optimizations vs production reality)

#### a3-21 (composite: 0.84)
> A single terrain walkability change affecting a busy corridor could force thousands of entities to simultaneously invalidate their cached paths and re-pathfind, creating a per-tick spike where thousands of A* calls each allocate 693 MB on the production map, which is unrecoverable within a single tick's budget.

> Scores: def=0.93 spec=0.88 rob=0.72 compat=0.85
> Deltas applied: defensibility +0.03
> Tensions: t-06 through t-09 (path caching vs terrain invalidation)

#### a2-21 (composite: 0.84)
> Within a single system invocation, spatial query results should be cached per cell coordinate so that multiple entities in the same spatial cell share the query result, reducing spatial query cost by up to 12x at uniform density with 50k entities.

> Scores: def=0.88 spec=0.90 rob=0.70 compat=0.88
> Deltas applied: robustness -0.08
> Tensions: t-10 through t-13, t-26 (spatial optimizations vs production reality)

#### a2-24 (composite: 0.84)
> Each system must have exactly one 'driving table' that it iterates; all other data access must be via point lookup (HashMap::get), because iterating two tables and joining them has multiplicative cache cost.

> Scores: def=0.85 spec=0.88 rob=0.75 compat=0.88

#### a3-15 (composite: 0.83)
> The benchmark's random swiss-cheese terrain produces nearly-straight A* paths, while the production map's connected buildings, continuous river, and winding streets force A* to expand 2-5x more nodes due to constricted passages (doorways, bridges), meaning pathfinding cost on real terrain is 2-5x higher than benchmark terrain even at the same map scale.

> Scores: def=0.86 spec=0.83 rob=0.80 compat=0.85

#### a1-07 (composite: 0.83)
> A system may omit the deterministic sort ONLY if it satisfies ALL of: (a) does not use world.rng, (b) per-entity results are independent of processing order, (c) it does not push to pending_deaths (PERF-SORT-02).

> Scores: def=0.90 spec=0.93 rob=0.75 compat=0.75
> Deltas applied: robustness -0.12; compatibility -0.07
> Tensions: t-01 through t-05 (sort/determinism cluster)

#### a3-07 (composite: 0.83)
> The mandatory deterministic sort (sort_by_key on entity ID) costs O(n log n) per system per tick, and since every system independently re-sorts the same entity set from HashMap's random iteration order, the total sort cost at 50k entities across 10 systems is approximately 7.5ms (75% of the 10ms tick budget), doubling to 15ms at 20 systems.

> Scores: def=0.85 spec=0.90 rob=0.78 compat=0.80
> Deltas applied: specificity +0.02
> Tensions: t-01 through t-05 (sort/determinism cluster)

#### a3-16 (composite: 0.83)
> At 8 systems the cumulative infrastructure overhead (HashMap tax + sort tax + allocation churn + spatial queries) consumes 4-5ms of the 10ms tick budget, leaving 5-6ms for actual computation, and this overhead scales linearly with system count such that at 15 systems the infrastructure alone consumes 8-10ms, leaving zero headroom for system logic.

> Scores: def=0.85 spec=0.80 rob=0.78 compat=0.90
> Deltas applied: defensibility +0.02
> Tensions: t-14, t-15, t-24, t-28 (budget disagreement cluster)

#### a2-04 (composite: 0.83)
> Phase 2 systems must be O(n) with no HashMap lookups beyond the primary iteration table, at most one secondary lookup per entity, no spatial queries, no pathfinding, and no allocation beyond the collect buffer, with a budget ceiling of 3,000us at 50k entities.

> Scores: def=0.85 spec=0.92 rob=0.67 compat=0.88
> Deltas applied: robustness -0.05
> Tensions: t-32 (Phase 2 ceiling vs table proliferation)

#### a2-23 (composite: 0.82)
> Never invalidate a cache based on tick count alone ('re-path every N ticks'); this wastes computation when the path is still valid and misses invalidation when the path becomes invalid before N ticks.

> Scores: def=0.85 spec=0.85 rob=0.70 compat=0.90
> Deltas applied: robustness -0.10; defensibility -0.05
> Tensions: t-06 through t-09 (path caching vs terrain invalidation)

#### a3-22 (composite: 0.82)
> Future goal-directed movement (home-to-work commutes) will produce correlated movement patterns with rush-hour clustering at doorways and bridge congestion, creating density spikes that stress spatial queries, pathfinding, and collision systems in ways that random wandering cannot exercise, and no current test or benchmark models these patterns.

> Scores: def=0.85 spec=0.80 rob=0.77 compat=0.88

#### a1-11 (composite: 0.82)
> rebuild_spatial_index is called exactly twice per tick (before Phase 2 and before Phase 4) at a cost of ~0.5ms each; a third call requires explicit justification, and beyond ~50k entities incremental updates should be considered (PERF-SPATIAL-03).

> Scores: def=0.83 spec=0.88 rob=0.70 compat=0.88
> Deltas applied: robustness -0.05
> Tensions: t-10 through t-13, t-26 (spatial optimizations vs production reality)

#### a2-09 (composite: 0.82)
> For tracking intentions (Eat/Attack), cached paths should not be invalidated every tick when the target moves; instead, re-path only when the target has moved more than K tiles (suggested K=3) from the cached goal position.

> Scores: def=0.88 spec=0.90 rob=0.65 compat=0.85
> Deltas applied: robustness -0.10
> Tensions: t-06 through t-09 (path caching vs terrain invalidation)

#### a1-19 (composite: 0.81)
> Every system's doc comment must state its expected per-tick complexity in big-O notation, parameterized by entity count (n), spatial density (k), and map size (w*h) as appropriate (PERF-LLM-01).

> Scores: def=0.82 spec=0.87 rob=0.70 compat=0.85
> Deltas applied: robustness -0.05
> Tensions: t-23, t-25 (profiling blindness)

#### a1-05 (composite: 0.81)
> Every Vec::new() used for collecting changes must be replaced with Vec::with_capacity() sized to the candidate count, or justified with a comment explaining why the expected size is much smaller (PERF-ITER-04).

> Scores: def=0.75 spec=0.90 rob=0.65 compat=0.92
> Deltas applied: robustness -0.07; defensibility -0.03
> Tensions: t-17 (Vec::with_capacity vs allocation churn)

#### a1-25 (composite: 0.80)
> HashMap::get costs 25-40ns per call due to hashing, cache-line straddling, and pointer chasing; a system touching 3 tables for 10k entities incurs approximately 0.9ms in hash lookups alone before any computation.

> Scores: def=0.80 spec=0.90 rob=0.62 compat=0.88
> Deltas applied: robustness -0.08
> Tensions: t-14, t-15, t-24, t-28 (budget disagreement cluster)

#### a2-25 (composite: 0.80)
> The total current cost at 50k entities exceeds 23,000us, and reaching the 10,000us target requires implementing ALL of: incremental spatial index (~4,000us savings), A* buffer pooling + call cap (~5,000us), pre-sorted alive list (~1,500us), decision query deduplication (~1,000us), and conditional event pushing (~500us), with the architecture's structural floor at approximately 6,000-7,000us even with perfect optimization.

> Scores: def=0.79 spec=0.92 rob=0.63 compat=0.85
> Deltas applied: robustness -0.05; defensibility -0.03
> Tensions: t-14, t-15, t-24, t-28 (budget disagreement cluster), t-23, t-25 (profiling blindness)

#### a3-12 (composite: 0.79)
> The utility_config.clone() in decisions.rs clones the entire BTreeMap<ActionId, ActionDef> every tick regardless of entity count, establishing a pattern ('clone shared config to avoid borrow conflicts') that will be replicated in future systems, scaling per-tick clone cost with the number of systems and the complexity of their configs.

> Scores: def=0.84 spec=0.85 rob=0.70 compat=0.78
> Deltas applied: defensibility +0.02
> Tensions: t-33 (utility_config.clone() severity)

#### a2-16 (composite: 0.79)
> The spatial index must support incremental updates rather than full rebuild: track moved entities from the wander system, remove from old cells and insert into new cells at O(moved_entities) cost instead of O(all_entities), reducing two rebuilds from ~5,400us to ~550us at 50k entities.

> Scores: def=0.86 spec=0.90 rob=0.62 compat=0.78
> Deltas applied: robustness -0.10; defensibility -0.02
> Tensions: t-10 through t-13, t-26 (spatial optimizations vs production reality), t-29 (incremental spatial updates vs rebuild allocation)

#### a2-17 (composite: 0.79)
> At 50k entities on a 256x256 map with CELL_SIZE=16, uniform density yields ~195 entities per cell, making SENSE_RANGE=30 queries scan 16 cells x 195 entities = 3,120 comparisons per query; reducing CELL_SIZE to 8 (CELL_SHIFT=3) gives 49/cell and 25 cells scanned = 1,225 comparisons.

> Scores: def=0.80 spec=0.95 rob=0.60 compat=0.80
> Deltas applied: robustness -0.10; defensibility -0.05
> Tensions: t-10 through t-13, t-26 (spatial optimizations vs production reality)

#### a1-01 (composite: 0.78)
> At 100 ticks/sec with 8 systems, 2 spatial index rebuilds, and a debug validation pass, each system gets roughly 1ms, yielding a per-entity budget of 100ns at 10k entities.

> Scores: def=0.77 spec=0.93 rob=0.58 compat=0.85
> Deltas applied: robustness -0.10; defensibility -0.05
> Tensions: t-14, t-15, t-24, t-28 (budget disagreement cluster)

#### a1-14 (composite: 0.76)
> If a system's per-entity computation exceeds 500ns (measured or estimated from operation counts), it must implement cooldown skip, tick staggering, or result caching, with the mechanism documented in the system's doc comment (PERF-CACHE-02).

> Scores: def=0.75 spec=0.85 rob=0.60 compat=0.85
> Deltas applied: robustness -0.12; defensibility -0.05
> Tensions: t-14, t-15, t-24, t-28 (budget disagreement cluster)

#### a1-24 (composite: 0.76)
> The clone() of UtilityConfig in run_decisions allocates every tick; the cost is small (~200 bytes for 4 actions with 2-3 considerations each) but should be restructured to extract config as a local reference before the mutable phase, or accepted with a comment.

> Scores: def=0.72 spec=0.85 rob=0.68 compat=0.80
> Deltas applied: robustness -0.02
> Tensions: t-33 (utility_config.clone() severity)

---

## Wounded

4 propositions are wounded -- they contain valid insights but need
qualification, conditions, or significant rework before they can be trusted as rules.

### a1-21 (composite: 0.75)
> When authoring a system, the LLM should estimate per-entity cost by summing: HashMap lookups at ~30ns each, arithmetic at ~3ns each, spatial queries at ~1000-5000ns each, and pathfinding calls at ~100,000-500,000ns each; if the total exceeds 500ns, amortization is required, and if it exceeds 5000ns, aggressive amortization with >90% skip rate is required (PERF-LLM-03).

> Scores: def=0.75 spec=0.92 rob=0.50 compat=0.82
> Weak axes: robustness
> Deltas: robustness -0.10; defensibility -0.03; robustness -0.05

### a2-20 (composite: 0.74)
> The decisions system should support inertia-based skip: entities whose action has been stable for 10+ consecutive ticks skip scoring for the next 5 ticks, reducing the scoring population by up to 80% in stable simulations.

> Scores: def=0.75 spec=0.88 rob=0.53 compat=0.82
> Weak axes: robustness
> Deltas: robustness -0.12; defensibility -0.05

### a1-22 (composite: 0.74)
> The decisions system (run_decisions) should stagger re-evaluation: entities with ticks_in_action < 5 and no high-priority stimulus (damage taken, health below threshold) should skip re-evaluation and retain their current intention, reducing spatial query volume by approximately 80% at steady state.

> Scores: def=0.82 spec=0.88 rob=0.58 compat=0.68
> Weak axes: robustness, compatibility
> Deltas: robustness -0.10; compatibility -0.07

### a1-08 (composite: 0.70)
> If tick profiling shows sorting consuming more than 15% of the tick budget, a per-tick cached sorted entity list (World::sorted_entities rebuilt once per tick) should be implemented, saving approximately 1.5ms per tick at 10k entities by eliminating 6 redundant sorts (PERF-SORT-03).

> Scores: def=0.62 spec=0.88 rob=0.53 compat=0.78
> Weak axes: defensibility, robustness
> Deltas: robustness -0.12; defensibility -0.08; defensibility -0.05

---

## Contested

No propositions remain in the contested category. All tensions were resolved
to either survivor, wounded, or fallen status through the adjudication process.

---

## Fallen

No propositions fell below thresholds.

---

## Tension Resolution Map

How each major tension cluster was resolved in Round 3 adjudication.

### t-01 through t-05 (sort/determinism cluster)

**Affected propositions and deltas:**

- **a1-07** [SURVIVOR]: robustness -0.12, compatibility -0.07
- **a1-08** [WOUNDED]: robustness -0.12, defensibility -0.08
- **a2-13** [SURVIVOR]: compatibility -0.08
- **a3-07** [SURVIVOR]: specificity +0.02
- **a3-08** [SURVIVOR]: defensibility +0.03
- **a3-20** [SURVIVOR]: defensibility +0.03

**Resolution:** The defense is correct that a1-07's conditions are statically checkable via grep, which mitigates the 'prove a negative' concern. However, the prosecution's point about the missing test safety net (a3-20) is decisive: even if the conditions are checkable at write time, there is no automated enforcement. In a codebase where an LLM agent writes systems, static analysis is the developer's responsibility, and mistakes happen.

### t-06 through t-09 (path caching vs terrain invalidation)

**Affected propositions and deltas:**

- **a1-13** [SURVIVOR]: robustness -0.10
- **a2-08** [SURVIVOR]: robustness -0.07
- **a2-09** [SURVIVOR]: robustness -0.10
- **a2-23** [SURVIVOR]: robustness -0.10, defensibility -0.05
- **a3-03** [SURVIVOR]: defensibility +0.03, specificity +0.03
- **a3-21** [SURVIVOR]: defensibility +0.03

**Resolution:** The defense makes a valid point: terrain mutation does not currently exist, so penalizing rules for not handling it is partially speculative. However, the adjudicator resolves this in favor of the more concrete, verifiable proposition (a3-03), for three reasons: (1) This is a simulation of Paris with buildings, bridges, and streets. Doors, flooding, and building collapse are not exotic features but natural extensions of the simulation's core premise.

### t-14, t-15, t-24, t-28 (budget disagreement cluster)

**Affected propositions and deltas:**

- **a1-01** [SURVIVOR]: robustness -0.10, defensibility -0.05
- **a1-14** [SURVIVOR]: robustness -0.12, defensibility -0.05
- **a1-21** [WOUNDED]: robustness -0.10
- **a1-25** [SURVIVOR]: robustness -0.08
- **a2-01** [SURVIVOR]: defensibility +0.02
- **a2-25** [SURVIVOR]: robustness -0.05
- **a3-06** [SURVIVOR]: defensibility +0.03
- **a3-16** [SURVIVOR]: defensibility +0.02

**Resolution:** The defense is correct that a1-01 provides a framework, not just a number. However, the prosecution's core point is decisive: when a1-01's 100ns budget is used downstream by a1-14 and a1-21 as calibration, the 10k assumption propagates silently. A developer reading a1-14 ('500ns threshold') does not re-derive the budget from entity count; they use the number as given. The rules must be self-contained or explicitly parameterized by entity count.

### t-10 through t-13, t-26 (spatial optimizations vs production reality)

**Affected propositions and deltas:**

- **a1-11** [SURVIVOR]: robustness -0.05
- **a1-12** [SURVIVOR]: robustness -0.07
- **a2-16** [SURVIVOR]: robustness -0.10
- **a2-17** [SURVIVOR]: robustness -0.10, defensibility -0.05
- **a2-21** [SURVIVOR]: robustness -0.08
- **a3-01** [SURVIVOR]: defensibility +0.02
- **a3-14** [SURVIVOR]: defensibility +0.02

**Resolution:** The defense makes two valid points: (1) optimizations should be evaluated in combination, not isolation, and (2) a2-21's 12x claim is for downstream processing specifically. However, the prosecution's core objection stands for a2-17: the improvement under clustering is categorically different from the improvement under uniform density, and the proposition's specificity score (0.95) implies the numbers are reliable when they are actually conditional on an unrealistic assumption.

### t-18 through t-20, t-34 (decision staggering vs RNG butterfly)

**Affected propositions and deltas:**

- **a1-22** [WOUNDED]: robustness -0.10, compatibility -0.07
- **a2-20** [WOUNDED]: robustness -0.12, defensibility -0.05
- **a3-08** [SURVIVOR]: specificity +0.03

**Resolution:** The defense raises a strong point: the RNG butterfly applies to all conditional optimizations, not just decision staggering. This is correct and the prosecution was selectively applying the standard. However, decision staggering is qualitatively different from other conditional optimizations because (1) it affects the most RNG-heavy system (decisions involve multiple random evaluations per entity), (2) the skip condition is data-dependent on the full population's action history, creating maximal...

### t-27 (HashMap structural ceiling)

**Affected propositions and deltas:**

- **a1-16** [SURVIVOR]: robustness -0.05
- **a3-06** [SURVIVOR]: defensibility +0.03
- **a3-23** [SURVIVOR]: defensibility +0.04

**Resolution:** The defense is largely correct: a1-16's scope is the data structure choice, and it answers that question soundly. Penalizing it for not also doing ceiling analysis is unfair. However, a small robustness penalty is warranted because a1-16's framing ('HashMap is correct for entity tables') without qualification could be read as 'HashMap is permanently correct,' which a3-06 and a3-23 show is conditional on scale.

### t-23, t-25 (profiling blindness)

**Affected propositions and deltas:**

- **a1-08** [WOUNDED]: defensibility -0.05
- **a1-19** [SURVIVOR]: robustness -0.05
- **a2-25** [SURVIVOR]: defensibility -0.03
- **a3-17** [SURVIVOR]: defensibility +0.03
- **a3-18** [SURVIVOR]: defensibility +0.02

**Resolution:** The defense is correct that the right response to benchmark blindness is to fix the benchmark, not to abandon measurement-gated decisions. However, the prosecution's point stands that propositions gated on profiling results cannot currently be validated because the benchmark does not exercise production conditions. This creates a temporal problem: the rules exist now, the production benchmark does not. Until the benchmark is fixed, profiling-gated rules are effectively dead letters.

### t-17 (Vec::with_capacity vs allocation churn)

**Affected propositions and deltas:**

- **a1-05** [SURVIVOR]: robustness -0.07, defensibility -0.03

**Resolution:** The defense's point about demand-paging is technically correct for physical memory but misses the mmap/munmap cycle cost, which is ~1-5us per call on Linux. With 8 systems each doing a large allocation and deallocation per tick, that is 80-400us of syscall overhead per tick — meaningful at the 10ms budget. The prosecution's 'qualitative inversion' framing is too strong: the escape hatch genuinely permits smaller allocations.

### t-21, t-22 (pathfinding workspace design)

**Affected propositions and deltas:**

- **a1-23** [SURVIVOR]: robustness -0.07
- **a2-15** [SURVIVOR]: robustness -0.03, defensibility +0.03

**Resolution:** The defense's sparse-clearing point is strong: clearing only the explored set eliminates the full-map memset concern. However, a1-23's specific proposal uses 'pre-allocated flat arrays cleared between calls,' which reads as full clearing. The proposition should specify sparse clearing or generation counters, not flat clearing. Moderate penalty for the specification gap.

### t-29 (incremental spatial updates vs rebuild allocation)

**Affected propositions and deltas:**

- **a2-16** [SURVIVOR]: defensibility -0.02

**Resolution:** The defense is correct that variable-cost fast/slow paths are standard. The prosecution's concern is valid but minor. Tiny penalty. Note: the larger penalties for a2-16 were already applied in the spatial optimizations cluster (t-10). This tension adds only a marginal additional concern.

### t-30 (event push frequency)

**Affected propositions and deltas:**

- **a2-05** [SURVIVOR]: robustness -0.02
- **a3-13** [SURVIVOR]: defensibility +0.02

**Resolution:** The defense makes a devastating point: the ring buffer already drops 80% of events, so per-tick consumers are already broken. However, the prosecution's concern that no consumer audit was performed remains valid as a process issue. The optimization is likely safe (and the defense explains why) but the reasoning should explicitly note the ring-buffer overflow as justification, not leave it implicit. Minor penalties/boosts.

### t-31 (cache invalidation completeness)

**Affected propositions and deltas:**

- **a1-13** [SURVIVOR]: specificity -0.02

**Resolution:** The defense is right that double-counting is unfair. However, a2-22's principle (assumptions must be stored in the cache) exposes that a1-13's specificity is narrowly correct but incomplete. A very small specificity penalty for the unstated assumption, acknowledging that the terrain gap is primarily an issue of robustness (already penalized in t-06 cluster).

### t-32 (Phase 2 ceiling vs table proliferation)

**Affected propositions and deltas:**

- **a2-04** [SURVIVOR]: robustness -0.05

**Resolution:** The defense correctly notes the effect is gradual. The prosecution correctly notes the ceiling is calibrated to current conditions and will erode. Split the difference: moderate penalty reflecting the non-constant-cost assumption.

### t-33 (utility_config.clone() severity)

**Affected propositions and deltas:**

- **a1-24** [SURVIVOR]: robustness -0.02
- **a3-12** [SURVIVOR]: defensibility +0.02

**Resolution:** The defense is right on the numbers: even replicated, the clone cost is negligible. a3-12's concern is about pattern hygiene, not performance. Tiny penalties reflecting that the risk is real but the impact is minimal. Both propositions survive with essentially unchanged scores.

### t-16 (per-system estimation vs distributed overhead)

**Affected propositions and deltas:**

- **a1-21** [WOUNDED]: defensibility -0.03, robustness -0.05
- **a3-17** [SURVIVOR]: specificity +0.03

**Resolution:** The defense is right that preventing per-system catastrophes is independently valuable. a1-21 does help an LLM avoid O(n^2) systems and pathfinding-every-tick mistakes. The prosecution is right that a1-21's framing implies comprehensive cost estimation when it only captures per-system costs. The surviving rule should carry a caveat: 'This estimation captures per-system algorithmic cost.

### t-35 (SENSE_RANGE minimization methodology gap)

**Affected propositions and deltas:**

- **a1-10** [SURVIVOR]: robustness -0.01
- **a2-07** [SURVIVOR]: robustness -0.03

**Resolution:** a1-10's real-world justification requirement is a genuine methodology, as the defense argues. a2-07's 'minimum required by gameplay' is vaguer and less actionable. Minimal penalty for a1-10, slightly larger for a2-07. Both survive; the tension is low-severity.

---

## Summary Statistics

- **Total propositions:** 73
- **Survivors:** 69 (37 core rules, 32 conditional guidance)
- **Wounded:** 4
- **Contested:** 0
- **Fallen:** 0
- **Propositions with deltas applied:** 44
- **Total individual axis deltas:** 64

- **Average composite score:** 0.855
- **Highest composite:** a1-09 at 0.945
- **Lowest composite:** a1-08 at 0.703
