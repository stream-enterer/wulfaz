# CLAUDE.md — Wulfaz

## What This Is

Emergent simulation engine. Blackboard architecture over HashMap-based EAV.
Single-threaded phase-ordered sequential loop. Rust.

## Architecture (Non-Negotiable)

- **World** is a struct of `HashMap<Entity, T>` property tables plus a `TileMap`.
- **Systems** are plain functions: `fn run_x(world: &mut World, tick: Tick)`.
- **One system per file** in `src/systems/`.
- Systems communicate **only** through shared state on World. No message passing.
  No traits between systems. No direct system-to-system calls.
- The main loop calls systems in **phase order**. Order matters.
- All randomness goes through `world.rng` (a seeded `StdRng`). Never use
  `thread_rng()` or any other RNG source. This guarantees deterministic replay.

## Core Types

```rust
pub struct Entity(pub u64);   // entity ID — never use raw u64
pub struct Tick(pub u64);     // simulation tick — never use raw u64
```

Key World fields (not property tables):
- `alive: HashSet<Entity>` — NOT a HashMap. Tracks living entities.
- `pending_deaths: Vec<Entity>` — entities marked for death this tick.
- `rng: StdRng` — deterministic seeded RNG. The ONLY RNG source.
- `events: EventLog` — ring buffer, not Vec.
- `tiles: TileMap` — flat Vec arrays for grid data.

Do not cast between Entity and Tick. Do not use raw integers for either.

## Entity Lifecycle

```
Spawn:    let e = world.spawn();
          world.positions.insert(e, pos);
          world.hungers.insert(e, hunger);

Kill:     world.pending_deaths.push(entity);
          world.events.push(Event::Died { ... });

Despawn:  ONLY run_death calls world.despawn(). No other system despawns.
```

**NEVER** manually `.remove()` an entity from individual tables. All removal
goes through `World::despawn()`. This is the single most important rule.

Entities may be spawned in any phase. New entities will not be processed by
earlier phases until the next tick.

## Pending-Death Rule

Systems MUST skip entities that are already marked for death. When iterating,
filter out pending deaths:

```rust
for (&entity, hunger) in &world.hungers {
    if world.pending_deaths.contains(&entity) { continue; }
    // ... process entity
}
```

## System Mutation Pattern

Systems MUST collect changes first, then apply. Never mutate a table while
iterating over it.

```rust
let changes: Vec<(Entity, f32)> = world.hungers.iter()
    .filter(|(&e, _)| !world.pending_deaths.contains(&e))
    .filter_map(|(&e, h)| {
        let new_val = h.current + 1.0;
        Some((e, new_val))
    })
    .collect();

for (e, new_val) in changes {
    if let Some(h) = world.hungers.get_mut(&e) {
        h.current = new_val;
    }
}
```

## Main Loop Phases

```rust
// === Phase 1: Environment ===
// Weather, temperature, plant growth, decay, fluid flow.
// Reads/writes tile data and environmental state.

// === Phase 2: Needs ===
// Hunger, thirst, tiredness, emotions.
// Reads environment. Writes entity internal state.

// === Phase 3: Decisions ===
// AI planning, pathfinding, task selection.
// Reads needs + environment. Writes intentions.

// === Phase 4: Actions ===
// Movement, eating, combat, building, crafting.
// Changes external world state: positions, HP, inventory.

// === Phase 5: Consequences ===
// Injury, relationship updates, reputation, death.
// Derives consequences from state changed this tick.
// run_death() is ALWAYS last in this phase.

// === Debug Validation ===
// #[cfg(debug_assertions)] validate_world(&world);
```

**Phase rule:** If a system changes external world state (position, HP,
inventory), it is Phase 4. If it derives consequences from changes already
made this tick (death check, relationship recalc), it is Phase 5.

## Adding a New System

1. Create `src/systems/new_system.rs`
2. Write `pub fn run_new_system(world: &mut World, tick: Tick) { ... }`
3. Add `pub mod new_system;` to `src/systems/mod.rs`
4. Add the call to the correct phase in `main.rs`
5. Write a unit test: construct minimal World, run system, assert state change
6. `cargo build` + run debug mode to confirm `validate_world()` passes

## Adding a New Property Table

When adding a new `HashMap<Entity, T>` to World, do ALL of these:

1. Add the struct in `src/components.rs`
2. Add `HashMap<Entity, T>` field to `World` in `world.rs`
3. Add `.remove(&entity)` in `World::despawn()`
4. Add an alive-check in `validate_world()`
5. Initialize to `HashMap::new()` in `World::new()`

Skip any step and you will create zombie entity bugs.

## Adding a New Event Type

1. Add the variant to `Event` in `src/events.rs`
2. Every variant MUST include `tick: Tick`
3. For lethal events: push AFTER the decision, BEFORE `pending_deaths.push()`
4. For non-lethal events: push immediately after the state change

## Event Log

EventLog is a ring buffer with configurable max depth (default: 10,000).
Old events are overwritten, not accumulated unboundedly. Do not use
`Vec<Event>` directly. Use the EventLog API: `world.events.push(event)`,
`world.events.iter()`, `world.events.recent(n)`.

## TileMap

Grid data lives in `TileMap` using flat `Vec<T>` arrays. Never HashMap.

```rust
world.tiles.get_terrain(x, y)
world.tiles.set_temperature(x, y, temp)
// Internally indexed by y * width + x
```

Systems that read/write tile data use these methods. Do not index the
Vec arrays directly.

## Data Files (KDL)

Content is defined in `data/*.kdl`. The engine does not hardcode entity types.
Parsed with the `kdl` crate (https://github.com/kdl-org/kdl-rs).

```kdl
creature "Goblin" {
    icon "g"
    max_hunger 100
    aggression 0.8
    speed 2
}

creature "Troll" {
    icon "T"
    max_hunger 200
    aggression 0.6
    speed 1
}
```

To add a new creature/item type: add a node to the relevant KDL file.
No code changes needed. `loading.rs` maps KDL nodes to spawned entities.

## Code Rules

- Never `.unwrap()` on table lookups. Always `if let Some(x) = world.table.get(&entity)`.
- Helper functions shared across systems go as methods on `World` in `world.rs`.
  Do not create `utils.rs` or `helpers.rs`.
- If a table entry is missing for an entity, skip that entity. Do not log, do not panic.

## What NOT To Do

- Do not add traits, interfaces, or abstraction layers between systems.
- Do not create a system registry or scheduler.
- Do not use message passing or event channels between systems.
- Do not manually remove entities from individual HashMap tables.
- Do not put multiple systems in one file.
- Do not create shared mutable state outside of World.
- Do not use `unsafe` without explicit approval.
- Do not use `thread_rng()` or any unseeded RNG.
- Do not use `Vec<Event>` for the event log. Use EventLog.
- Do not replace HashMap with another data structure without profiling data
  showing >5ms per tick for that system.
- Do not add concurrency to the simulation loop.

## Testing

Every new system MUST ship with a unit test:

```rust
#[test]
fn test_hunger_increases() {
    let mut world = World::new_with_seed(42);
    let e = world.spawn();
    world.hungers.insert(e, Hunger { current: 0.0, max: 100.0 });
    run_hunger(&mut world, Tick(0));
    assert!(world.hungers[&e].current > 0.0);
}
```

- Property-based tests in `tests/invariants.rs`: no zombie entities,
  food conservation, deterministic replay with same seed.
- `validate_world()` runs every tick in debug builds.

## Growth Patterns

When World exceeds ~25 fields, group into sub-structs:
`world.body.positions`, `world.mind.emotions`, `world.social.friendships`.
This is a readability change, not an architectural change.

When the main loop exceeds ~30 systems, group phases into functions:
`run_environment_phase(&mut world, tick)`. Same phase rules still apply.

At 15+ systems, build `src/bin/analyze_systems.rs` to extract read/write
dependencies from source code.

## Project Structure

```
CLAUDE.md
Cargo.toml
.workflow/
  features.md            # immutable feature contract
  phases.md              # work phases and session protocol
  progress.jsonl         # cross-session state tracker
data/
  creatures.kdl          # creature definitions
  items.kdl              # item definitions
  terrain.kdl            # terrain definitions
src/
  main.rs                # phased main loop
  world.rs               # World struct, spawn, despawn, validate
  events.rs              # Event enum + EventLog ring buffer
  components.rs          # property structs (Position, Hunger, etc.)
  tile_map.rs            # TileMap with flat Vec arrays
  loading.rs             # KDL parsing, entity spawning
  render.rs              # display output
  rng.rs                 # deterministic seeded RNG wrapper
  systems/
    mod.rs
    hunger.rs
    wander.rs
    eating.rs
    combat.rs
    death.rs             # ALWAYS last in Phase 5
tests/
  invariants.rs          # property-based cross-system tests
  determinism.rs         # replay/seed tests
```

## Workflow Tracking

`.workflow/` tracks feature implementation across sessions. The files are
self-documenting — read them before starting on a new feature.

Immutability rule: MAY mark features complete after verification. MAY NOT
modify acceptance criteria, delete features, or mark features not applicable.

