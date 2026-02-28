# Performance Failure Analysis: Wulfaz Simulation Engine

## Preamble: Why This Analysis Exists

Three performance regressions were discovered in this codebase only through benchmarking at 50k entities. All three compiled cleanly, passed all tests, and produced correct simulation output. They were invisible to code review because each looked like a reasonable implementation choice in isolation:

1. The wander system recomputed A\* per entity per move tick, discarding the full path after extracting one step. At 50k entities this cost 75ms/tick.
2. The A\* internals used HashMap for g\_score and came\_from on a bounded grid where flat `Vec<u32>` arrays suffice.
3. The eating system used `Vec::contains` for deduplication, which is O(n) per check but only over a small consumed-food set.

These are the *found* problems. The following catalog concerns the *unfound* problems --- the performance diseases that future systems will introduce through reasonable-seeming decisions, and that will remain invisible until the entity count crosses a critical threshold or the system count reaches a tipping point.

---

## 1. Silent Quadratic Emergence Through System Interaction

### Mechanism of Entry

A system author writes a Phase 3 decision system --- say, `run_social` --- that evaluates social bonds. For each entity, it calls `entities_in_range` to find nearby entities, then for each nearby entity, looks up 2-3 property tables to evaluate bond strength. This is textbook ECS. It compiles, it's correct, and when tested with 500 entities on a 256x256 map it finishes in microseconds.

### Compounding Effect

The decisions system already does this. For each entity, `read_input` for `FoodNearby` and `EnemyNearby` calls `entities_in_range`, which scans all spatial grid cells within a 30-tile Chebyshev radius. With 16-tile cells, that's up to 16 cells (4x4 grid of cells). Each cell contains a Vec of entities in that cell. At 50k entities uniformly distributed over 256x256 tiles, that's roughly 0.76 entities per tile, or about 12 entities per cell. So each `entities_in_range` call touches ~192 entities. Two input axes means ~384 entity examinations per decision-making entity.

Now add `run_social` with its own `entities_in_range` call per entity. The per-entity cost of the decision phase silently goes from 384 to 576 entity examinations. But the real problem is not the constant factor --- it's what happens when entities cluster.

On the real Paris map, entities spawn inside buildings. A quartier like Arcis might pack hundreds of entities into a few city blocks. In a dense block, a single 16x16 spatial cell might contain 200+ entities. Each `entities_in_range` call from that cell now examines the same 200 entities repeatedly --- once per caller in the same cell. The cost of `run_social` in that cell alone is O(d^2) where d is local density. Two systems that each call `entities_in_range` produce O(2 * d^2) rather than O(d + d), because the spatial query iterates the full cell contents for every caller.

### Invisibility

Unit tests spawn 5-10 entities. The benchmark spawns entities uniformly on a 256x256 grid. Neither scenario produces density hotspots. The O(d^2) behavior only manifests when entities cluster, which happens on the real map but not in any test fixture.

### Interaction with Other Failure Modes

This compounds with Failure Mode 10 (Benchmark Blindness). It also interacts with Failure Mode 2: if the system author adds a third `entities_in_range` call inside the per-neighbor loop (e.g., "for each nearby entity, check if *their* neighbors include me"), the cost becomes O(d^3) in dense cells, which at density 200 is 8 million operations per cell per tick.

---

## 2. Spatial Index Abuse: The "Free Lookup" Illusion

### Mechanism of Entry

The spatial index is rebuilt twice per tick (lines 229 and 238 of `main.rs`). Rebuilding is O(n): iterate all positions, insert into the HashMap grid. At 50k entities this is fast --- a few hundred microseconds. A system author observes this and concludes that spatial queries are cheap. They then use `entities_at` or `entities_in_range` as a general-purpose "who is near me?" primitive, calling it inside per-entity loops without concern.

### Compounding Effect

The rebuild cost is O(n) but it allocates. `spatial_index.clear()` deallocates all the inner Vecs. Then each `entry().or_default()` call potentially allocates a new Vec. With 50k entities across ~4000 cells (256x256 map, 16x16 cells), that's 4000 Vec allocations per rebuild, twice per tick. The Vecs themselves are small (averaging ~12.5 entities per cell), but the allocation traffic is 8000 Vec allocs/frees per tick.

A new system that triggers a third rebuild (e.g., a Phase 4 system that needs post-its-own-movement positions) adds 4000 more alloc/free cycles per tick. But more insidiously, it also means the spatial index is stale for any system that ran between rebuild 2 and rebuild 3 --- which creates a correctness concern that the system author "fixes" by adding yet another rebuild. Each rebuild is cheap individually. Four rebuilds at 50k entities is still only ~1ms. But this sets a precedent: the spatial index is "just rebuilt when needed." By the time there are 15 systems and 5 rebuilds per tick, the cumulative rebuild cost is 2.5ms, which is 25% of the 10ms budget, for a pure infrastructure operation.

A separate vector of abuse is `entities_in_range` with large ranges. The current SENSE_RANGE is 30 tiles. The spatial cell is 16 tiles. A 30-tile Chebyshev range queries a 4x4 block of cells. But if a future system uses a 100-tile sense range (e.g., for a "flee from distant threat" behavior), it queries a 14x14 block of cells, examining up to 196 cells with all their contents. At high density, a single such query could touch thousands of entities. Inside a per-entity loop, this is disastrous.

### Invisibility

The rebuild is timed separately in the benchmark. A system author adding a third `rebuild_spatial_index()` call will see the individual timing and conclude it's cheap. They won't see the cumulative allocation pressure or the interaction with the allocator's fragmentation behavior over thousands of ticks.

### Interaction with Other Failure Modes

Interacts with Failure Mode 5 (Allocation Pressure). Each spatial rebuild generates thousands of tiny Vec allocations. These alternate with the per-system collect-then-apply Vecs, creating a fragmented allocation pattern that degrades allocator performance over time.

---

## 3. Cache Invalidation Cascades and Stale Path Accumulation

### Mechanism of Entry

The `CachedPath` component stores a sequence of (x,y) steps toward a goal. This was introduced to avoid recomputing A\* every move tick. The wander system correctly invalidates cached paths when the goal changes or when the entity is tracking a moving target. But no system invalidates cached paths when the *map* changes.

### Compounding Effect

Consider a future Phase 1 system that models environmental change: flooding tiles during rain, collapsing buildings, or simply opening and closing doors based on time of day. Each such change modifies the walkability of tiles. Every cached path that passes through a now-unwalkable tile is stale. Entities following stale paths will attempt to move into walls, fail, and fall back to random stepping. This is a correctness issue, but it is also a performance issue because:

1. The random fallback step still costs an A\* attempt (which fails, allocating and freeing the 3 flat arrays of size `width * height`) before falling back.
2. Each failed A\* on the production 11000x7000 map allocates `3 * 11000 * 7000 * 4 bytes = ~924 MB` per call. Even with the 32,768 expansion limit, the allocation happens unconditionally at the start of `find_path`. At 50k entities, if even 1% have stale paths that trigger a re-pathfind per tick, that's 500 pathfinding calls per tick, each allocating 924 MB and freeing it immediately.

Wait. Let me re-examine. The flat arrays are `vec![u32::MAX; total]` and `vec![false; total]` where `total = width * height`. On the production map (11000x7000), `total = 77,000,000`. That means:
- `g_score`: 77M * 4 bytes = 308 MB
- `came_from`: 77M * 4 bytes = 308 MB
- `closed`: 77M * 1 byte = 77 MB

That is 693 MB allocated and freed per A\* call. Even a single unnecessary A\* call per tick is catastrophic. With path caching, only entities that need a fresh path pay this cost. But stale caches that silently fail force re-pathfinding, converting a cache hit (free) into a cache miss (693 MB alloc+init+free).

### Invisibility

The benchmark uses a 256x256 map. On that map, the A\* flat arrays are `256 * 256 * 9 bytes = ~590 KB` per call --- negligible. The 693 MB cost only manifests on the production 11000x7000 map, which no benchmark or test exercises for pathfinding.

Furthermore, the stale path problem only manifests when a tile changes walkability *after* paths are cached through it. No current system changes terrain at runtime (temperature changes don't affect walkability). So the problem is entirely latent, waiting for the first environmental mutation system.

### Interaction with Other Failure Modes

Interacts with Failure Mode 5 (Allocation Pressure). A single unnecessary A\* call on the production map allocates and frees 693 MB. Rust's allocator (likely `malloc`/`free` via system allocator) may or may not return this to the OS. If it doesn't, the process RSS balloons. If it does, the next A\* call triggers a fresh mmap. Either way, the TLB is trashed.

Also interacts with Failure Mode 1: if stale paths cause many entities to simultaneously re-pathfind, the pathfinding cost is multiplied by the number of affected entities. A single terrain change affecting a busy corridor could force thousands of simultaneous re-paths.

---

## 4. HashMap Iteration Tax: Death by a Thousand Lookups

### Mechanism of Entry

Every system iterates at least one `HashMap<Entity, T>`. The hunger system iterates `hungers` and checks `pending_deaths`. The decisions system iterates `action_states`, checks `pending_deaths`, `positions`, `hungers`, `healths`, `fatigues`, `combat_stats`, `nutritions` (via spatial query), and writes `intentions`. Each `.get(&e)` on a HashMap is O(1) amortized, but the constant factor includes: hash the Entity, probe the bucket array, follow the chain, compare keys.

### Compounding Effect

With the current 8 systems, a single tick at 50k entities performs roughly:
- `run_hunger`: 50k iterations of hungers, 50k `.contains()` checks on pending\_deaths. ~100k HashMap ops.
- `run_fatigue`: 50k iterations + 50k pending\_deaths checks + conditional health lookups. ~120k ops.
- `run_decisions`: 50k iterations + 50k pending\_deaths + per-entity scoring (3-4 table lookups per consideration, 4 actions, 2-3 considerations each). That's 50k * 4 * 3 * 1 = ~600k ops, plus spatial query lookups.
- `run_wander`: 50k iterations + per-entity lookups into positions, gait\_profiles, current\_gaits, move\_cooldowns, intentions, wander\_targets, cached\_paths. That's ~350k ops.
- `run_eating`: similar scale.
- `run_combat`: similar scale.

Conservative estimate: ~2 million HashMap operations per tick at 50k entities across 8 systems.

Each new system adds at minimum one full table iteration plus several cross-table lookups per entity. A system that cross-references 4 tables adds ~200k HashMap ops. With 15 systems, the cumulative HashMap traffic could exceed 5 million ops per tick. Each op involves hashing Entity(u64), which itself involves SipHash (the default Rust hasher), which is cryptographically robust but not fast. SipHash processes ~1 GB/s. Hashing 8 bytes takes ~8ns. 5 million hashes = 40ms. That alone exceeds the 10ms tick budget.

The reality is less dire because Rust's HashMap uses a faster hasher (aHash since hashbrown), but the scaling direction is clear: HashMap lookup cost grows linearly with system count and entity count, and at some system count, the base tax of "just looking things up" dominates all computation.

### Invisibility

Each system's lookups are O(1) individually. No system shows as a hotspot. The cost is distributed across all systems equally. Profiling shows "HashMap::get" consuming 60% of tick time, but there is no single call site to blame. The disease is architectural.

### Interaction with Other Failure Modes

Interacts with Failure Mode 8 (Table Proliferation). Each new table adds a `.remove()` call to despawn and a full iteration to `validate_world`. But it also means that systems that need to cross-reference the new table add yet more HashMap lookups per entity. The cost grows as the *product* of system count and table count, not their sum.

---

## 5. Collect-Then-Apply Allocation Pressure

### Mechanism of Entry

The architecture mandates collect-then-apply: systems cannot mutate a table while iterating it. Every system collects changes into a `Vec`, then applies them. This is correct and necessary. But each Vec allocation is proportional to the entity count.

### Compounding Effect

At 50k entities per tick:
- `run_hunger` collects a `Vec<(Entity, f32, f32)>` of ~50k entries = 50k * 24 bytes = 1.2 MB.
- `run_fatigue` collects a `Vec<(Entity, f32)>` of ~50k entries = 50k * 12 bytes = 600 KB.
- `run_decisions` collects cooldown decrements plus results. Results: `Vec<(Entity, ActionId, Option<Entity>)>` of ~50k entries = 50k * 24 bytes = 1.2 MB.
- `run_wander` collects four Vecs: moves, cooldown\_updates, wander\_target\_updates, cached\_path\_updates. Total ~50k * 4 * ~20 bytes = ~4 MB.
- `run_eating`, `run_combat` collect smaller Vecs.

Total per-tick allocation: roughly 8-10 MB of Vec buffers, allocated and freed every 10ms. Over one second (100 ticks), that is 800 MB - 1 GB of allocation churn. This is not a memory leak --- the Vecs are dropped at function exit. But the allocator must service this churn: find free blocks, return them, potentially coalesce.

A new system that adds even one `Vec<(Entity, SomeStruct)>` of 50k entries adds another ~1 MB per tick. By system 20, the allocation churn is 20+ MB per tick. The allocator's free-list becomes fragmented. Large allocations (like the A\* flat arrays) may fail to reuse freed Vec memory because the sizes don't match, causing fresh mmap calls and RSS growth.

### Invisibility

`Vec::push` is O(1) amortized. The Vecs are dropped promptly. No memory leak. No test detects allocation overhead. The benchmark measures wall time but doesn't decompose it into computation vs allocation. A system that takes 500us might be spending 200us on actual work and 300us on allocation, but the author attributes all 500us to "the algorithm."

### Interaction with Other Failure Modes

Directly interacts with Failure Mode 3 (A\* allocation). The A\* flat arrays are ~693 MB on the production map. When these are freed, the allocator has a huge free block. When the next system allocates 50k-entry Vecs (1 MB each), the allocator splits the huge block. When A\* runs again, the huge block no longer exists contiguously. This fragmentation cascade means the second A\* call requires a fresh mmap. Over many ticks, the process RSS grows monotonically even though live memory usage is bounded.

---

## 6. Sort Cost Accumulation: The Determinism Tax

### Mechanism of Entry

Every system sorts its entity collection by `e.0` for deterministic replay. This is mandatory and correct. `sort_by_key(|e| e.0)` is O(n log n).

### Compounding Effect

At 50k entities with 10 systems, each sorting once:
- 10 * 50,000 * log2(50,000) = 10 * 50,000 * 15.6 = 7.8 million comparisons per tick.
- Each comparison involves accessing the Entity's u64 ID. This is a single `cmp` instruction, but the data access pattern matters. The entities are being sorted *in a Vec*, which means the sort is comparing elements and swapping them within contiguous memory. This is cache-friendly for small entities (Entity is 8 bytes, fits in a cache line).

The direct sorting cost at 50k entities across 10 systems is probably ~2-3ms (rough estimate: 7.8M comparisons at ~3ns each including memory access = ~23ms is too pessimistic; Rust's sort is pdqsort which is ~10-20ns per element for 50k elements, giving 10 * 50k * 15ns = 7.5ms). This is significant: 75% of the tick budget.

But the real cost is not the sort itself --- it's the *re-sorting*. Each system independently collects its entity keys from a HashMap iteration (random order), then sorts them. The *same* set of entity IDs is sorted from scratch in every system. With 10 systems, the same 50k IDs are sorted 10 times per tick.

At 20 systems, the sort tax doubles to ~15ms, exceeding the tick budget on sorting alone.

### Invisibility

Each individual sort takes ~750us at 50k entities --- well within budget. No single system looks slow. But the cumulative sort time across all systems is not measured by the benchmark (it's folded into each system's timing). A developer adding system 11 sees their system's sort contribute ~750us, which seems fine, unaware that 9 other systems are paying the same cost.

### Interaction with Other Failure Modes

Interacts with Failure Mode 4 (HashMap Iteration Tax). The entities are collected from HashMap iteration (random order), then sorted. If the entities were stored in a sorted structure (e.g., BTreeMap), the collection would already be sorted. But the architecture mandates HashMap for O(1) lookup. The sort cost is the tax for using HashMap instead of a sorted container.

Also interacts with Failure Mode 8 (Table Proliferation): systems that iterate multiple tables may need to sort the intersection or union of entity sets from different tables, requiring set operations that are themselves O(n log n).

---

## 7. Determinism Fragility: RNG Consumption Ordering

### Mechanism of Entry

Deterministic replay requires that `world.rng` is consumed in identical order on every run with the same seed. Systems that consume RNG do so in entity-sorted order. But RNG consumption is data-dependent: an entity might consume 0, 1, or 2 RNG calls depending on its state.

### Compounding Effect

In `run_wander`, the RNG consumption per entity depends on:
1. Whether the entity is on cooldown (0 RNG calls if cooling down).
2. Whether it has a cached path (0 RNG calls if cache hit).
3. Whether it needs a new wander target (up to 5 RNG calls for the search loop at lines 111-112).
4. Whether A\* fails (1 RNG call for random fallback direction at line 264).

In `run_fatigue`, the RNG is consumed conditionally: only entities with fatigue > 200 trigger a `world.rng.random()` call (line 50).

In `run_combat`, the RNG is consumed per combatant: `world.rng.random()` at line 97.

A new system that consumes RNG in a data-dependent way adds another branch in the RNG consumption tree. If that system also reads state that was written by a *previous* system in the same tick, and the previous system's output is itself data-dependent, then the RNG consumption order becomes a function of the simulation state, not just the entity order.

This is fragile in a specific way: a change to system A's behavior (say, tweaking a threshold) can change which entities consume RNG in system A, which shifts the RNG stream for all subsequent systems, which changes their behavior, which changes combat outcomes, which changes who dies, which changes the entity set for the next tick. Deterministic replay is preserved (same seed produces same result), but *bisecting* a behavior change becomes impossible: every parameter tweak produces a butterfly effect through the RNG stream.

### Invisibility

The determinism test (`test_wander_deterministic_with_seed`) only verifies that the same seed produces the same result. It does not detect that adding a single RNG call in a new system changes the output of every downstream system. There is no test that verifies cross-system RNG stability.

### Interaction with Other Failure Modes

This is not a performance failure per se, but it interacts with Failure Mode 10 (Benchmark Blindness): if a parameter change causes different entities to take different code paths that consume different amounts of RNG, the benchmark results become non-comparable between runs. A "performance improvement" might actually be a different entity distribution caused by an RNG stream shift, producing a different density pattern that happens to be faster.

It also compounds with any future system that uses RNG-based load: if a system rolls dice to decide whether to do expensive work, and the RNG stream shifts, the set of entities doing expensive work changes unpredictably.

---

## 8. Component Table Proliferation: The Linear Tax on Everything

### Mechanism of Entry

Adding a new property table is documented as a 5-step process in CLAUDE.md. Each step is simple. The temptation is to add tables freely, because each individual table has negligible overhead.

### Compounding Effect

Currently there are 20 property table fields across `BodyTables` (9), `MindTables` (7), and `GisTables` (2 per-entity). Each table adds:

1. **Despawn cost**: One `HashMap::remove()` per table per entity death. Currently 20 removes per despawn. At 50k entities with a 1% death rate per tick, that's 500 * 20 = 10,000 HashMap removes per tick. At 40 tables, that doubles to 20,000.

2. **Validation cost** (debug builds): `validate_world` iterates every key of every table and checks `alive.contains()`. With 20 tables at 50k entries each, that's 1 million HashSet lookups per tick. At 40 tables, 2 million. At ~8ns per lookup (SipHash on Entity), validation alone costs ~16ms at 40 tables --- exceeding the tick budget in debug mode.

3. **Structural overhead**: Each HashMap has a minimum allocation (bucket array, metadata). With 40 tables, that's 40 HashMap allocations at initialization, 40 HashMap clears at world reset, and 40 HashMap drops at program exit. More importantly, iterating over the *world* (e.g., for serialization or snapshot) requires visiting 40 tables.

4. **Cache pollution**: Each `HashMap::get` on a different table touches a different region of memory. A system that cross-references 5 tables per entity accesses 5 different memory regions per entity. With 50k entities, that's 250k random memory accesses, spread across 5 different hash table bucket arrays. The L1/L2 cache cannot hold all 5 tables simultaneously at 50k entries (a HashMap of 50k entries with 8-byte keys and ~16-byte values occupies ~1.5 MB including metadata, times 5 = 7.5 MB, exceeding typical L2 cache).

### Invisibility

Each new table adds ~50us to the tick (from one extra remove in despawn, one extra iteration in validate). Fifty microseconds is invisible. But the accumulation is not: 20 additional tables add 1ms, and the cache pollution effect is non-linear --- it only manifests when the total working set exceeds cache size.

### Interaction with Other Failure Modes

Interacts with Failure Mode 4 (HashMap Tax). More tables means more cross-table lookups per system. The total HashMap operations per tick grows as (systems * entities * tables-referenced-per-system).

Interacts with Failure Mode 6 (Sort Cost). If a new table serves as the iteration source for a new system, that system must sort its keys. More tables means more systems means more sorts.

---

## 9. Phase Boundary Violations: Temporal Coupling as Performance Disease

### Mechanism of Entry

The phase model is clear in documentation but unenforced in code. Nothing prevents a Phase 3 system from writing to `positions` (Phase 4 territory) or a Phase 4 system from reading `hungers` (Phase 2 territory). The compiler does not enforce phase semantics. Only code review does.

### Compounding Effect

A Phase 3 system that reads the spatial index sees pre-movement positions (spatial rebuild 1). A Phase 4 system that reads the spatial index after `run_wander` sees stale data (spatial rebuild 2 hasn't happened yet --- it occurs after wander). If a new Phase 4 system is inserted between `run_wander` and `rebuild_spatial_index` (spatial2), it sees an inconsistent spatial index: some entities have moved, the index reflects pre-movement positions.

The "fix" is to add another spatial rebuild. This costs O(n) per rebuild but sets the precedent described in Failure Mode 2. More perniciously, the system author might not realize the index is stale and instead add compensating logic: "check if the entity is really at the position the spatial index says." This compensating logic adds per-entity overhead that would be unnecessary with correct phase ordering.

The deepest form of this failure is *unnecessary recomputation*. If Phase 3's decisions are based on Phase 2's needs, and a Phase 4 system modifies needs (violating the phase contract), then Phase 3's decisions become stale within the same tick. A "fix" is to run Phase 3 again after Phase 4. This doubles the decision cost. Worse, if decisions consume RNG, running them twice breaks determinism unless carefully handled.

### Invisibility

Phase violations produce correct results (the simulation is self-consistent at tick end). The performance cost is invisible because it manifests as "this system needs an extra spatial rebuild" or "we need to re-run decisions after movement." Each individual accommodation is cheap. The cumulative cost of accommodating multiple phase violations is not tracked.

### Interaction with Other Failure Modes

Interacts with Failure Mode 2 (Spatial Index Abuse): phase violations are the primary driver of unnecessary spatial rebuilds.

Interacts with Failure Mode 7 (Determinism Fragility): if a phase violation causes a system to run twice, the RNG stream is consumed twice, breaking replay unless the RNG state is saved and restored.

---

## 10. Benchmark Blindness: What the 256x256 Uniform Grid Cannot See

### Mechanism of Entry

The benchmark (`bin/bench.rs`) uses a 256x256 map with random terrain (73% Road, 3% Water, 3% Wall, etc.) and uniformly distributed entities. This is the only scalability test.

### Compounding Effect

The production map is 11000x7000 tiles. The benchmark map is 256x256. The ratio is ~1200:1 in area. This means:

1. **A\* cost is invisible**: On 256x256, the flat arrays for A\* are 590 KB. On 11000x7000, they are 693 MB. The benchmark cannot detect A\* allocation cost because it's 1000x smaller than production. An A\* regression that costs 100us on the benchmark map costs 100ms on the production map (dominated by `vec![u32::MAX; 77_000_000]` initialization).

2. **Density distribution is wrong**: Uniform random placement on a grid produces roughly Poisson-distributed density. The real map has entities clustered inside buildings within a quartier. The Arcis quartier might have 500 entities in a 200x200 tile area, with many sharing the same building's floor tiles. This means spatial cells in that area contain 50-100 entities, not 12. All quadratic-in-density algorithms are 10-60x worse than the benchmark predicts.

3. **Terrain topology is wrong**: Random terrain creates a "swiss cheese" pattern --- small isolated walls and water tiles. Real terrain has connected structures: contiguous buildings, continuous river, winding streets. A\* on swiss-cheese terrain rarely needs to path around obstacles (most paths are nearly straight). A\* on real terrain must navigate around buildings, through doorways, across bridges. Path lengths are 2-5x longer, and the search space is larger due to constricted passages that force the search to expand many nodes before finding a path through a doorway.

4. **Entity type mix is wrong**: The benchmark spawns uniform creatures with identical stats. The real simulation has citizens (with occupations, home buildings, workplaces) and food items. The decision system's behavior differs for entities with different component sets. An entity without `combat_stats` skips the Attack action entirely. An entity without `hungers` skips the Eat action. The benchmark gives every entity the full component set, which exercises the maximum-cost code path for decisions. This could make the benchmark *pessimistic* for decisions but *optimistic* for systems that have fast-path early exits on missing components.

5. **Movement patterns are wrong**: Uniformly distributed entities wander randomly. Real entities will have goal-directed movement (home-to-work commutes) that produces correlated movement patterns: rush-hour clustering at doors, congestion on bridges. These patterns create density spikes that stress spatial queries, pathfinding, and collision systems in ways that random wandering does not.

### Invisibility

The benchmark exists and produces numbers. The numbers look reasonable. The system author runs the benchmark, sees their new system adds 200us at 50k entities, and concludes it's fine. They cannot know that the same system will cost 20ms on the production map with clustered entities, because no test exercises that scenario.

### Interaction with Other Failure Modes

Benchmark blindness is the meta-failure mode that makes all other failure modes invisible. Every failure mode in this document was identified by reasoning about the code, not by observing benchmark results. The benchmark would not have caught any of them at the current map size and entity distribution.

---

## 11. The `utility_config.clone()` Cascade

### Mechanism of Entry

Line 246 of `decisions.rs` clones the entire `UtilityConfig` every tick. This is a `BTreeMap<ActionId, ActionDef>` where each `ActionDef` contains a `Vec<Consideration>`. The clone allocates a new BTreeMap, clones each ActionDef, which clones each Vec of Considerations. Currently with 4 actions and 2-3 considerations each, this is ~10 heap allocations per tick.

### Compounding Effect

This clone happens once per tick regardless of entity count --- it's O(actions * considerations), not O(entities). But as the utility AI grows (more action types, more considerations per action, more complex curves), the per-tick clone cost grows. With 20 actions averaging 5 considerations each, that's ~100 heap allocations per tick just for the clone. Over 100 ticks per second, that's 10,000 allocations per second for data that never changes during a tick.

More critically, this pattern of "clone shared config to avoid borrow conflicts" may be replicated in future systems. A system that needs to iterate entities while also reading a config struct on `World` faces the same borrow problem. If each such system clones its config, the per-tick allocation grows with each new system that uses this pattern.

### Invisibility

The clone is fast for the current config size. Profiling shows it as negligible. The allocation is not tracked separately from the system's computation. It only becomes visible when the config grows large enough to appear in allocation profiling, by which point the pattern has been established across multiple systems.

---

## 12. Event Log as Hidden O(n) Tax

### Mechanism of Entry

The `EventLog` ring buffer receives events from multiple systems. The hunger system pushes one `HungerChanged` event per entity per tick. At 50k entities, that's 50k events per tick from hunger alone. The wander system pushes one `Moved` event per entity that moves. With walk cooldown of 9 ticks, roughly 1/9 of entities move per tick at 50k, producing ~5500 events per tick. Combat and eating produce events proportional to combat and eating encounters.

### Compounding Effect

The ring buffer has a capacity of 10,000 (line 63 of events.rs). At 50k entities, the hunger system alone produces 50k events per tick, meaning the ring buffer wraps 5 times per tick and only retains the last 10k events. The first 40k events are immediately overwritten. Every `push` call writes to the buffer and increments the write position --- this is O(1) per push, but the 50k pushes per tick from hunger alone represent ~50k cache-line-touching writes to a ~240 KB buffer (10k events at ~24 bytes each).

A future system that pushes per-entity events (e.g., `TemperatureChanged { entity, old, new, tick }`) adds another 50k pushes per tick, doubling the event throughput through a buffer that already wraps multiple times per tick. The events are never read during simulation (they're for the upcoming UI). The 100k writes per tick to a buffer that can hold 10k events means 90% of writes are immediately overwritten. This is pure waste: the computation to construct the Event struct, the write to the ring buffer, and the potential cache-line eviction all happen for events that are destroyed before any consumer reads them.

### Invisibility

Each `events.push()` is one line, O(1), and takes <1ns. But 50k pushes per tick occupy ~50us. With 5 systems each pushing per-entity events, that's 250k pushes per tick, occupying ~250us. This is 2.5% of the tick budget, attributed to no single system.

### Interaction with Other Failure Modes

Interacts with Failure Mode 5 (Allocation Pressure) if Event variants grow to include heap-allocated data (e.g., `String` descriptions). Each push of a heap-allocating Event that is immediately overwritten creates allocation churn for zero benefit.

---

## Synthesis: The Compounding Problem

No single failure mode in this catalog is fatal in isolation. The wander system's A\* allocation is the worst individual offender, and it was already caught and partially fixed. The remaining failure modes are all moderate: 200us here, 500us there, 750us of sorting.

The catastrophe is multiplicative. At 8 systems, the cumulative overhead from HashMap tax + sort tax + allocation churn + spatial queries is perhaps 4-5ms per tick, leaving 5-6ms for actual computation. At 15 systems, those same taxes scale to 8-10ms, leaving zero headroom for the systems' own logic. At 20 systems, the infrastructure overhead alone exceeds the tick budget, and no individual system is to blame.

The three previously discovered regressions were each attributable to a single system. The failures described here are distributed: they emerge from the interaction of the architecture with scale, not from any single bad decision. They are the ground truth of what it costs to run a HashMap-based EAV with deterministic sorted iteration at 50k+ entities and 100 ticks/second. They cannot be caught by unit tests, code review, or benchmarks that test the wrong scale and distribution. They can only be caught by profiling the full production scenario at target entity count, on the production map, with realistic entity distribution and movement patterns.
