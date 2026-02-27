# CLAUDE.md — Wulfaz

## What This Is

Emergent simulation engine. Blackboard architecture over HashMap-based EAV.
Single-threaded phase-ordered sequential loop. Rust.

## Architecture (Non-Negotiable)

- **World** groups property tables into sub-structs (`BodyTables`, `MindTables`,
  `GisTables`), each containing `HashMap<Entity, T>` fields. Plus infrastructure:
  `TileMap`, `EventLog`, `StdRng`, spatial index.
- **Systems** are plain functions: `fn run_x(world: &mut World, tick: Tick)`.
- **One system per file** in `src/systems/`.
- Systems communicate **only** through shared state on World. No message passing.
  No traits between systems. No direct system-to-system calls.
- The main loop calls systems in **phase order**. Order matters.
  No system registry or scheduler.
- All randomness goes through `world.rng` (a seeded `StdRng`). Never use
  `thread_rng()` or any other RNG source. This guarantees deterministic replay.
- All shared state lives on World. No mutable state outside it.
- Single-threaded. Do not add concurrency to the simulation loop.
- **Data-driven:** creature/item types defined in `data/*.kdl`, not hardcoded.
  `loading.rs` maps KDL nodes to spawned entities. To add a new creature/item
  type: add a node to the relevant KDL file. No code changes needed.

## Core Types

`Entity(pub u64)` is the entity ID. `Tick(pub u64)` is the simulation tick.
Do not cast between Entity and Tick. Do not use raw integers for either.

Key infrastructure fields on World (not in property-table sub-structs):
- `alive: HashSet<Entity>` — NOT a HashMap. Tracks living entities.
- `pending_deaths: Vec<Entity>` — entities marked for death this tick.
- `events: EventLog` — ring buffer, not Vec.

## Entity Lifecycle

**NEVER** manually `.remove()` an entity from individual tables. All removal
goes through `World::despawn()`. This is the single most important rule.

Only `run_death` calls `World::despawn()`. No other system despawns.

Entities may be spawned in any phase. New entities will not be processed by
earlier phases until the next tick.

## System Iteration and Mutation

Systems MUST skip entities already in `pending_deaths`.

Systems MUST collect changes first, then apply. Never mutate a table while
iterating over it.

Systems MUST sort entity collections by `e.0` before processing. HashMap
iteration order is non-deterministic; without sorting, deterministic replay
breaks even with a seeded RNG.

## Main Loop Phases

```
Phase 1: Environment — tile/environmental state. No entity interaction.
Phase 2: Needs      — reads environment, writes entity internal state.
Phase 3: Decisions  — reads needs + environment, writes intentions only.
Phase 4: Actions    — reads intentions, changes external world state.
Phase 5: Consequences — derives from state changed this tick.
                        run_death() is ALWAYS last.
Debug:   #[cfg(debug_assertions)] validate_world(&world);
```

**Phase classification:**
- Phase 1 reads/writes tiles and environmental state. Does not touch entity needs.
- Phase 2 reads environment. Writes entity internal state (needs, emotions).
- Phase 3 reads needs + environment. Writes intentions. No external state changes.
- Phase 4 changes external world state (position, HP, inventory).
- Phase 5 derives consequences from changes already made this tick.

## Code Rules

- Missing table entry = skip that entity silently (`if let Some`). Never
  `.unwrap()` on table lookups.
- Helper functions shared across systems go as methods on `World` or its
  sub-structs in `world.rs`. Do not create `utils.rs` or `helpers.rs`.
- Do not use `#[allow(...)]` or `#[expect(...)]` to suppress warnings. Fix the
  cause: remove dead code, delete unused imports, prefix unused bindings with
  `_`, apply the clippy suggestion. If a warning is a genuine false positive
  (FFI naming, conditional compilation), add a comment explaining why before
  suppressing.
- Do not use `unsafe` without explicit approval.
- Do not replace HashMap with another data structure without profiling data
  showing >5ms per tick for that system.

## Spatial Scale

**1 tile = 1 meter.** This is non-negotiable.

- Every spatial constant MUST have a comment with real-world units.

## Gait System

Movement speed uses DF-style gait tiers. All creatures share the same slow
gaits (Creep/Stroll/Walk); fast gaits differ by body plan (biped vs quadruped).

Each entity has a `GaitProfile` (cooldown array) and a `current_gait` (Gait enum).
Cooldown = ticks to wait between 1-tile moves. Lower = faster.

| Gait    | Biped (ticks/tile) | Quadruped (ticks/tile) | Tiles/sec @100fps |
|---------|-------------------|----------------------|------------------|
| Creep   | 29                | 29                   | 3.4              |
| Stroll  | 19                | 19                   | 5.3              |
| Walk    | 9                 | 9                    | 11.1 (DF default)|
| Hustle  | 7 (jog)           | 4 (trot)             | 14.3 / 25.0      |
| Run     | 5 (run)           | 3 (canter)           | 20.0 / 33.3      |
| Sprint  | 3 (sprint)        | 2 (gallop)           | 33.3 / 50.0      |

Diagonal moves cost `base_cooldown × 141 / 100` (√2 fixed-point).

All creatures default to Walk gait at spawn. Fast gaits are used situationally
(fleeing, charging) — not as permanent speed. `gaits` field in KDL selects
the profile: `"biped"` or `"quadruped"`.

## Performance Rules

Hard budget: **10,000μs per tick** at 100 ticks/sec. LOD caps active entities
at ~4K; entity-count costs are bounded by the active zone.

### Iteration

- Iterate exactly **one driving table per system.** Access all other data via
  point lookup (`HashMap::get`). Choose the smallest table that captures the
  needed entities.
- Order filter chains **cheapest rejection first.** Chain `.filter()` calls
  from cheapest (pending_deaths check) to most expensive (spatial query).
- Test cooldown timers first and skip in O(1). No secondary lookups or
  allocation for entities still on cooldown.
- Never nest O(n) entity loops. Bound inner loops by a spatial query,
  fixed-size collection, or constant.

### Data Structures

- Use direct lookup (`positions.get(&entity)`) when you have an Entity ID.
  Never spatial-scan for a known entity.
- Index tile data with flat `Vec` (`y * width + x`). Never
  `HashMap<(i32, i32), T>` for tile-indexed data.
- Use `HashSet` for membership testing. Never `Vec::contains()` inside a loop.

### Spatial Queries

- Use the minimum range for each `entities_in_range` call. Define fixed ranges
  as named consts. For per-entity ranges, comment the field in `components.rs`.
  Both need real-world unit justification.
- Run O(1) checks before spatial queries. Never call `entities_in_range`
  unconditionally for all n entities.

### Pathfinding

- Check `cached_paths` before calling `find_path`. Never path per entity per
  tick. Reuse until invalidated by goal change, blockage, or path exhaustion.
- Pool A* workspace buffers with generation-counter clearing. Never allocate
  fresh per call — flat arrays on the production map (6309×4753) cost ~270MB.
- Invalidate cached paths when terrain walkability changes. Store assumptions
  (goal, terrain generation) in the cache. Never invalidate by tick count alone.
- Re-path for moving targets only when the target has moved >K tiles from the
  cached goal, not every tick.

### Phase Budgets

- **Phase 1:** Use chunk-level dirty flags. O(active_chunks), not
  O(total_tiles).
- **Phase 2:** O(n) with at most one secondary lookup per entity. No spatial
  queries, no pathfinding. Emit events only on state transitions, not every
  tick.
- **Phase 3:** Cap or stagger every spatial query result set.
- **Phase 4:** Do not insert systems between `run_wander` and
  `rebuild_spatial_index` — the spatial index is stale until rebuilt.
- **Phase 5:** Iterate only the affected set (pending_deaths, event triggers).
  Never scan the full entity population.

### Scaling Constraints

Production map: 6,309 × 4,753 tiles (~30M tiles), ~1M statistical population.
LOD caps active entities at ~4K; the rest are district aggregate models.

- Always cache A* paths — the full-map workspace costs ~270MB per call.
  Account for 2–5× extra node expansion in tight geometry (doorways, bridges,
  winding streets).
- Keep district-model systems O(district_count), not O(statistical_population).
- Treat each new property table as a cost: it adds work to `despawn()` and
  future LOD hydration. Each new system adds per-tick sort + lookup cost
  proportional to active entity count.

## Adding a New System

1. Create `src/systems/new_system.rs`
2. Add `pub mod new_system;` to `src/systems/mod.rs`
3. Add the call to the correct phase in `main.rs`
4. Ensure deterministic iteration order — sort by `e.0` where it matters
5. Write a unit test (see Testing)
6. `cargo build` + run debug mode to confirm `validate_world()` passes

## Adding a New Property Table

When adding a new `HashMap<Entity, T>` to a sub-struct in World, do ALL of these:

1. Add the struct in `src/components.rs`
2. Add the `HashMap<Entity, T>` field to the appropriate sub-struct in `world.rs`
3. Add `.remove(&entity)` in that sub-struct's `remove()` method
4. Add an alive-check in `validate_world()`
5. Initialize to `HashMap::new()` in the sub-struct's `new()`

Skip any step and you will create zombie entity bugs.

## Adding a New Event Type

1. Add the variant to `Event` in `src/events.rs`
2. Every variant MUST include `tick: Tick`
3. For lethal events: push AFTER the decision, BEFORE `pending_deaths.push()`
4. For non-lethal events: push immediately after the state change

## Testing

Every new system MUST ship with a unit test. Construct a minimal World
with `World::new_with_seed(42)`, spawn an entity, insert components,
run the system, assert state change.

- Property-based tests in `tests/invariants.rs`: no zombie entities,
  food conservation, deterministic replay with same seed.
