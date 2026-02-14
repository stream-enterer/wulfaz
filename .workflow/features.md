# Wulfaz Feature Contract

> **Immutable contract.** Agents MAY set status to `[x]` after verification.
> Agents MAY NOT delete features, modify acceptance criteria, or mark features
> as "not applicable." If a feature seems impossible, escalate to the user.

> Built using the **Concentric Rings Method**: enumerate all features first,
> then shape (inputs/outputs/dependencies), then specify (edge cases/validation),
> then cross-check for gaps. Each pass covers ALL features before proceeding.

---

## Domain: Core Architecture

### CORE-001 — World Struct as HashMap-Based EAV

- **Category:** Core Architecture
- **Description:** World is a struct of `HashMap<Entity, T>` property tables
  plus a `TileMap`. This is the blackboard: all simulation state lives here.
  No simulation state exists outside World.
- **Acceptance Criteria:**
  - [ ] World struct exists in `src/world.rs`
  - [ ] World contains only `HashMap<Entity, T>` fields for entity properties
  - [ ] World contains a `TileMap` field for grid data
  - [ ] World contains `alive: HashSet<Entity>` (NOT HashMap)
  - [ ] World contains `pending_deaths: Vec<Entity>`
  - [ ] World contains `rng: StdRng`
  - [ ] World contains `events: EventLog`
  - [ ] No simulation state exists outside the World struct
- **Dependencies:** CORE-002, CORE-003, TILE-001, EVT-001, RNG-001
- **Status:** `[ ]`

### CORE-002 — Entity Newtype

- **Category:** Core Architecture
- **Description:** `Entity(pub u64)` newtype wrapper. All entity references use
  this type. Raw `u64` is never used where an entity ID is meant.
- **Acceptance Criteria:**
  - [ ] `pub struct Entity(pub u64)` is defined
  - [ ] Entity implements Hash, Eq, PartialEq, Clone, Copy
  - [ ] No raw u64 is used in place of Entity anywhere in the codebase
  - [ ] Entity is never cast to or from Tick
- **Dependencies:** None
- **Status:** `[ ]`

### CORE-003 — Tick Newtype

- **Category:** Core Architecture
- **Description:** `Tick(pub u64)` newtype wrapper. All tick/time references use
  this type. Raw `u64` is never used where a tick count is meant.
- **Acceptance Criteria:**
  - [ ] `pub struct Tick(pub u64)` is defined
  - [ ] No raw u64 is used in place of Tick anywhere in the codebase
  - [ ] Tick is never cast to or from Entity
  - [ ] Every Event variant includes a `tick: Tick` field
- **Dependencies:** None
- **Status:** `[ ]`

### CORE-004 — Blackboard Architecture

- **Category:** Core Architecture
- **Description:** Systems communicate only through shared state on World.
  No message passing, no traits between systems, no direct system-to-system
  calls. World is the sole communication channel.
- **Acceptance Criteria:**
  - [ ] No system imports or calls another system directly
  - [ ] No message-passing channels exist between systems
  - [ ] No trait definitions connect systems to each other
  - [ ] All inter-system data flow passes through World fields
- **Dependencies:** CORE-001, SYS-001
- **Status:** `[ ]`

### CORE-005 — Single-Threaded Sequential Loop

- **Category:** Core Architecture
- **Description:** The simulation runs as a single-threaded phase-ordered
  sequential loop. No concurrency in the simulation loop.
- **Acceptance Criteria:**
  - [ ] main.rs runs a sequential tick loop
  - [ ] No threads, async tasks, or parallel iterators in the simulation loop
  - [ ] Systems are called in deterministic phase order each tick
  - [ ] Phase order is explicitly defined in main.rs
- **Dependencies:** CORE-001
- **Status:** `[ ]`

---

## Domain: Entity Lifecycle

### LIFE-001 — Entity Spawning via World::spawn()

- **Category:** Entity Lifecycle
- **Description:** New entities are created exclusively through `world.spawn()`,
  which returns an Entity with a unique ID and adds it to the alive set.
  Property tables are populated after spawning.
- **Acceptance Criteria:**
  - [ ] `World::spawn()` method exists and returns `Entity`
  - [ ] spawn() generates a unique Entity ID (monotonically increasing or equivalent)
  - [ ] spawn() inserts the new Entity into `world.alive`
  - [ ] After spawn(), caller inserts into relevant property tables
  - [ ] Entities may be spawned in any phase
  - [ ] Newly spawned entities are not processed by earlier phases until next tick
- **Dependencies:** CORE-001, CORE-002
- **Status:** `[ ]`

### LIFE-002 — Entity Kill via pending_deaths

- **Category:** Entity Lifecycle
- **Description:** To kill an entity, push a death event and then push the
  entity to `world.pending_deaths`. Only `run_death` actually despawns. No
  other system calls despawn. Event ordering follows ADD-003: lethal events
  are pushed AFTER the kill decision, BEFORE `pending_deaths.push()`.
- **Acceptance Criteria:**
  - [ ] Killing an entity means `world.pending_deaths.push(entity)`
  - [ ] A death event is pushed BEFORE `pending_deaths.push()` (per ADD-003 rule)
  - [ ] No system other than run_death calls `world.despawn()`
  - [ ] pending_deaths is cleared after run_death processes all entries
- **Dependencies:** CORE-001, LIFE-001, EVT-002
- **Status:** `[ ]`

### LIFE-003 — Entity Despawn via World::despawn()

- **Category:** Entity Lifecycle
- **Description:** `World::despawn()` removes an entity from ALL property
  tables and from the alive set. This is the ONLY mechanism for removing
  entities from tables. Manual `.remove()` on individual tables is forbidden.
- **Acceptance Criteria:**
  - [ ] `World::despawn(entity)` method exists
  - [ ] despawn removes entity from `world.alive`
  - [ ] despawn calls `.remove(&entity)` on EVERY HashMap property table
  - [ ] No code outside `World::despawn()` calls `.remove()` on property tables
  - [ ] Only `run_death` system calls `world.despawn()`
- **Dependencies:** CORE-001, LIFE-001
- **Status:** `[ ]`

### LIFE-004 — Pending-Death Filtering Rule

- **Category:** Entity Lifecycle
- **Description:** Every system that iterates over entities MUST skip entities
  present in `world.pending_deaths`. This prevents dead-entity processing.
- **Acceptance Criteria:**
  - [ ] Every system's iteration loop checks `world.pending_deaths.contains(&entity)`
  - [ ] Entities in pending_deaths are skipped (continue), not processed
  - [ ] No system processes an entity that is already marked for death
- **Dependencies:** LIFE-002, SYS-001
- **Status:** `[ ]`

---

## Domain: Systems Framework

### SYS-001 — System Function Signature

- **Category:** Systems Framework
- **Description:** Systems are plain functions with the signature
  `pub fn run_x(world: &mut World, tick: Tick)`. No traits, no structs,
  no closures. Plain functions only.
- **Acceptance Criteria:**
  - [ ] Every system is a `pub fn` with exactly `(world: &mut World, tick: Tick)` params
  - [ ] No system is a method on a struct or trait impl
  - [ ] No system uses closures as its primary entry point
  - [ ] Function names follow the `run_*` convention
- **Dependencies:** CORE-001, CORE-003
- **Status:** `[ ]`

### SYS-002 — One System Per File

- **Category:** Systems Framework
- **Description:** Each system lives in its own file under `src/systems/`.
  No file contains multiple system functions.
- **Acceptance Criteria:**
  - [ ] `src/systems/` directory exists
  - [ ] Each `.rs` file in `src/systems/` contains exactly one `pub fn run_*` function
  - [ ] `src/systems/mod.rs` re-exports all system modules
  - [ ] No system function exists outside `src/systems/`
- **Dependencies:** SYS-001
- **Status:** `[ ]`

### SYS-003 — Collect-Then-Apply Mutation Pattern

- **Category:** Systems Framework
- **Description:** Systems MUST collect changes into a Vec first, then apply
  them in a second pass. Never mutate a HashMap while iterating over it.
- **Acceptance Criteria:**
  - [ ] No system mutates a property table while iterating over it
  - [ ] Changes are collected into a Vec (or similar) before application
  - [ ] Application phase uses `get_mut()` or `insert()`, not direct indexing
  - [ ] Application phase uses `if let Some(x)` for safety, not unwrap
- **Dependencies:** SYS-001, RULE-001
- **Status:** `[ ]`

### SYS-004 — No Inter-System Communication

- **Category:** Systems Framework
- **Description:** Systems do not communicate with each other directly. No
  message passing, no event channels between systems, no direct function calls
  between systems, no shared traits.
- **Acceptance Criteria:**
  - [ ] No system file imports another system's module
  - [ ] No channel, queue, or mailbox exists for system-to-system messages
  - [ ] No trait is defined to be implemented by multiple systems
  - [ ] No system calls another system's run function
- **Dependencies:** CORE-004
- **Status:** `[ ]`

---

## Domain: Main Loop Phases

### PHASE-001 — Phase 1: Environment

- **Category:** Main Loop Phases
- **Description:** Environment phase handles weather, temperature, plant growth,
  decay, and fluid flow. Reads/writes tile data and environmental state.
- **Acceptance Criteria:**
  - [ ] Environment systems run first in the tick loop
  - [ ] Environment systems read and write TileMap data
  - [ ] Environment systems handle weather, temperature, growth, decay, fluid flow
  - [ ] No environment system modifies entity external state (position, HP, inventory)
- **Dependencies:** CORE-005, TILE-001
- **Status:** `[ ]`

### PHASE-002 — Phase 2: Needs

- **Category:** Main Loop Phases
- **Description:** Needs phase handles hunger, thirst, tiredness, emotions.
  Reads environment. Writes entity internal state (need values).
- **Acceptance Criteria:**
  - [ ] Needs systems run after environment phase
  - [ ] Needs systems read environment/tile state but do not write it
  - [ ] Needs systems write entity internal state (hunger, thirst, tiredness, emotions)
  - [ ] Hunger system exists: `src/systems/hunger.rs`
- **Dependencies:** CORE-005, PHASE-001
- **Status:** `[ ]`

### PHASE-003 — Phase 3: Decisions

- **Category:** Main Loop Phases
- **Description:** Decision phase handles AI planning, pathfinding, task
  selection. Reads needs and environment. Writes intentions (what entity
  will attempt this tick).
- **Acceptance Criteria:**
  - [ ] Decision systems run after needs phase
  - [ ] Decision systems read needs and environment state
  - [ ] Decision systems write intention/goal state on entities
  - [ ] Decision systems do not modify external world state
- **Dependencies:** CORE-005, PHASE-002
- **Status:** `[ ]`

### PHASE-004 — Phase 4: Actions

- **Category:** Main Loop Phases
- **Description:** Action phase handles movement, eating, combat, building,
  crafting. Changes external world state: positions, HP, inventory. Any system
  that changes external world state belongs here.
- **Acceptance Criteria:**
  - [ ] Action systems run after decision phase
  - [ ] Action systems change external world state (positions, HP, inventory)
  - [ ] Wander system exists: `src/systems/wander.rs` (movement)
  - [ ] Eating system exists: `src/systems/eating.rs`
  - [ ] Combat system exists: `src/systems/combat.rs`
  - [ ] Phase assignment rule: external state change = Phase 4
- **Dependencies:** CORE-005, PHASE-003
- **Status:** `[ ]`

### PHASE-005 — Phase 5: Consequences

- **Category:** Main Loop Phases
- **Description:** Consequence phase derives consequences from state changed
  this tick: injury, relationship updates, reputation, death. `run_death()`
  is ALWAYS the last system in this phase.
- **Acceptance Criteria:**
  - [ ] Consequence systems run after action phase
  - [ ] Consequence systems derive state from changes made this tick
  - [ ] `run_death()` is the final system call in Phase 5
  - [ ] Death system exists: `src/systems/death.rs`
  - [ ] Phase assignment rule: consequence derivation = Phase 5
  - [ ] run_death processes all pending_deaths and calls world.despawn()
- **Dependencies:** CORE-005, PHASE-004, LIFE-002, LIFE-003
- **Status:** `[ ]`

### PHASE-006 — Debug Validation Phase

- **Category:** Main Loop Phases
- **Description:** After all five phases, `validate_world()` runs in debug
  builds (`#[cfg(debug_assertions)]`). Checks for zombie entities and
  other invariants.
- **Acceptance Criteria:**
  - [ ] `validate_world(&world)` is called after Phase 5 in the main loop
  - [ ] validate_world is gated behind `#[cfg(debug_assertions)]`
  - [ ] validate_world checks that no entity in any property table is missing from alive
  - [ ] validate_world checks that no entity in alive is missing expected components
  - [ ] validate_world panics on invariant violations in debug builds
- **Dependencies:** CORE-005, PHASE-005, VALID-001
- **Status:** `[ ]`

### PHASE-007 — Phase Assignment Rules

- **Category:** Main Loop Phases
- **Description:** Formal rules for determining which phase a system belongs
  to. If a system changes external world state (position, HP, inventory), it
  is Phase 4. If it derives consequences from changes already made this tick
  (death check, relationship recalc), it is Phase 5.
- **Acceptance Criteria:**
  - [ ] Every system is assigned to exactly one phase
  - [ ] Systems that modify external world state are in Phase 4
  - [ ] Systems that derive consequences are in Phase 5
  - [ ] Phase assignment is documented in main.rs comments
  - [ ] No system violates its phase's read/write contract
- **Dependencies:** PHASE-001 through PHASE-005
- **Status:** `[ ]`

---

## Domain: Adding New Components (Checklists)

### ADD-001 — Adding a New System (6-Step Checklist)

- **Category:** Checklists
- **Description:** Defined procedure for adding any new system to the engine.
  All six steps must be completed for a system addition to be valid.
- **Acceptance Criteria:**
  - [ ] Step 1: Create `src/systems/new_system.rs`
  - [ ] Step 2: Write `pub fn run_new_system(world: &mut World, tick: Tick)`
  - [ ] Step 3: Add `pub mod new_system;` to `src/systems/mod.rs`
  - [ ] Step 4: Add the call to the correct phase in `main.rs`
  - [ ] Step 5: Write a unit test (construct minimal World, run system, assert state change)
  - [ ] Step 6: `cargo build` + debug mode confirms `validate_world()` passes
- **Dependencies:** SYS-001, SYS-002, PHASE-007, TEST-001
- **Status:** `[ ]`

### ADD-002 — Adding a New Property Table (5-Step Checklist)

- **Category:** Checklists
- **Description:** Defined procedure for adding a new `HashMap<Entity, T>` to
  World. All five steps must be completed or zombie entity bugs will result.
- **Acceptance Criteria:**
  - [ ] Step 1: Add the struct in `src/components.rs`
  - [ ] Step 2: Add `HashMap<Entity, T>` field to World in `world.rs`
  - [ ] Step 3: Add `.remove(&entity)` in `World::despawn()`
  - [ ] Step 4: Add an alive-check in `validate_world()`
  - [ ] Step 5: Initialize to `HashMap::new()` in `World::new()`
  - [ ] Skipping any step produces a zombie entity bug
- **Dependencies:** CORE-001, LIFE-003, VALID-001
- **Status:** `[ ]`

### ADD-003 — Adding a New Event Type (4-Step Checklist)

- **Category:** Checklists
- **Description:** Defined procedure for adding a new variant to the Event enum.
- **Acceptance Criteria:**
  - [ ] Step 1: Add the variant to `Event` in `src/events.rs`
  - [ ] Step 2: Every variant MUST include `tick: Tick`
  - [ ] Step 3: For lethal events, push AFTER the decision, BEFORE `pending_deaths.push()`
  - [ ] Step 4: For non-lethal events, push immediately after the state change
- **Dependencies:** CORE-003, EVT-001, EVT-002
- **Status:** `[ ]`

---

## Domain: Events

### EVT-001 — EventLog Ring Buffer

- **Category:** Events
- **Description:** EventLog is a ring buffer with configurable max depth
  (default 10,000). Old events are overwritten, not accumulated unboundedly.
  `Vec<Event>` must never be used for the event log.
- **Acceptance Criteria:**
  - [ ] EventLog struct exists in `src/events.rs`
  - [ ] EventLog is a ring buffer, not a Vec
  - [ ] Default max depth is 10,000 events
  - [ ] Max depth is configurable at construction time
  - [ ] Old events are silently overwritten when capacity is reached
  - [ ] EventLog API: `push(event)`, `iter()`, `recent(n)`
  - [ ] No code uses `Vec<Event>` for event storage
- **Dependencies:** CORE-003
- **Status:** `[ ]`

### EVT-002 — Event Enum with Tick Field

- **Category:** Events
- **Description:** The Event enum defines all event types. Every variant
  must include a `tick: Tick` field for temporal ordering.
- **Acceptance Criteria:**
  - [ ] `Event` enum exists in `src/events.rs`
  - [ ] Every variant includes `tick: Tick`
  - [ ] Died variant exists for entity death events
  - [ ] Events are pushed via `world.events.push(event)`
- **Dependencies:** CORE-003
- **Status:** `[ ]`

### EVT-003 — Event Ordering Rules

- **Category:** Events
- **Description:** Events must be pushed at specific points relative to state
  changes. Lethal events: push AFTER the decision, BEFORE `pending_deaths.push()`.
  Non-lethal events: push immediately after the state change.
- **Acceptance Criteria:**
  - [ ] Lethal event push precedes `pending_deaths.push()` in all kill sites
  - [ ] Non-lethal event push immediately follows the state change it describes
  - [ ] No event is pushed without the corresponding state change occurring
- **Dependencies:** EVT-001, EVT-002, LIFE-002
- **Status:** `[ ]`

---

## Domain: TileMap

### TILE-001 — TileMap with Flat Vec Arrays

- **Category:** TileMap
- **Description:** Grid data lives in TileMap using flat `Vec<T>` arrays.
  Never HashMap for grid data. Internally indexed by `y * width + x`.
- **Acceptance Criteria:**
  - [ ] `TileMap` struct exists in `src/tile_map.rs`
  - [ ] Grid data stored as flat `Vec<T>` arrays, not HashMap
  - [ ] Internal indexing uses `y * width + x`
  - [ ] TileMap stores width and height dimensions
  - [ ] No HashMap is used for grid/tile data
- **Dependencies:** None
- **Status:** `[ ]`

### TILE-002 — TileMap Accessor Methods

- **Category:** TileMap
- **Description:** Systems access tile data through TileMap methods only.
  Direct Vec indexing is forbidden. Methods include get/set for terrain,
  temperature, and other tile properties.
- **Acceptance Criteria:**
  - [ ] `get_terrain(x, y)` method exists
  - [ ] `set_temperature(x, y, temp)` method exists
  - [ ] No system indexes TileMap's internal Vec arrays directly
  - [ ] Accessor methods handle bounds checking
  - [ ] All tile reads/writes go through TileMap methods
- **Dependencies:** TILE-001
- **Status:** `[ ]`

---

## Domain: Data Pipeline

### DATA-001 — KDL Data File Format

- **Category:** Data Pipeline
- **Description:** Content (creatures, items, terrain) is defined in `data/*.kdl`
  files. The engine does not hardcode entity types. Parsed with the `kdl` crate.
- **Acceptance Criteria:**
  - [ ] `data/` directory exists at project root
  - [ ] `data/creatures.kdl` defines creature types
  - [ ] `data/items.kdl` defines item types
  - [ ] `data/terrain.kdl` defines terrain types
  - [ ] KDL format is used (not TOML, JSON, YAML)
  - [ ] `kdl` crate is a dependency in Cargo.toml
  - [ ] No entity type is hardcoded in Rust source
- **Dependencies:** None
- **Status:** `[ ]`

### DATA-002 — KDL Loading and Entity Spawning

- **Category:** Data Pipeline
- **Description:** `src/loading.rs` parses KDL data files and maps nodes to
  spawned entities. Adding a new creature/item type requires only a KDL
  file change, not code changes.
- **Acceptance Criteria:**
  - [ ] `src/loading.rs` exists
  - [ ] loading.rs reads and parses `data/*.kdl` files
  - [ ] KDL nodes are mapped to entity spawns with correct property table entries
  - [ ] Adding a new creature type to KDL requires no Rust code changes
  - [ ] KDL node attributes (icon, max_hunger, aggression, speed, etc.) map to components
  - [ ] Parse errors are handled gracefully (no unwrap on KDL parsing)
- **Dependencies:** CORE-001, LIFE-001, COMP-001, DATA-001
- **Status:** `[ ]`

---

## Domain: Deterministic RNG

### RNG-001 — Seeded StdRng on World

- **Category:** Deterministic RNG
- **Description:** All randomness goes through `world.rng`, a seeded `StdRng`.
  Never use `thread_rng()` or any other RNG source. This guarantees
  deterministic replay given the same seed.
- **Acceptance Criteria:**
  - [ ] `world.rng` is of type `StdRng`
  - [ ] `World::new_with_seed(seed)` constructor accepts a seed value
  - [ ] All systems use `world.rng` for random decisions
  - [ ] No code uses `thread_rng()`, `OsRng`, or any other RNG source
  - [ ] Same seed produces identical simulation runs
- **Dependencies:** None
- **Status:** `[ ]`

### RNG-002 — RNG Wrapper Module

- **Category:** Deterministic RNG
- **Description:** `src/rng.rs` provides a deterministic seeded RNG wrapper.
  May contain helpers for common random operations while ensuring all
  randomness flows through the single seeded source.
- **Acceptance Criteria:**
  - [ ] `src/rng.rs` exists
  - [ ] RNG wrapper uses StdRng internally
  - [ ] Wrapper enforces single RNG source constraint
  - [ ] Helper functions for common operations (range, choice, etc.) if needed
- **Dependencies:** RNG-001
- **Status:** `[ ]`

---

## Domain: Components

### COMP-001 — Property Structs in components.rs

- **Category:** Components
- **Description:** All property/component structs (Position, Hunger, etc.)
  are defined in `src/components.rs`. These are the `T` in
  `HashMap<Entity, T>` property tables.
- **Acceptance Criteria:**
  - [ ] `src/components.rs` exists
  - [ ] Position struct is defined (at minimum x, y fields)
  - [ ] Hunger struct is defined (current and max fields)
  - [ ] All property structs used as values in World's HashMaps live here
  - [ ] No property struct is defined in a system file
- **Dependencies:** CORE-002
- **Status:** `[ ]`

---

## Domain: Existing Systems

### ESYS-001 — Hunger System

- **Category:** Existing Systems
- **Description:** `src/systems/hunger.rs` implements the hunger system.
  Phase 2 (Needs). Increases hunger over time for entities with a Hunger
  component.
- **Acceptance Criteria:**
  - [ ] `src/systems/hunger.rs` exists
  - [ ] Contains `pub fn run_hunger(world: &mut World, tick: Tick)`
  - [ ] Iterates over `world.hungers` and increases hunger values
  - [ ] Skips entities in `world.pending_deaths`
  - [ ] Uses collect-then-apply mutation pattern
  - [ ] Has a unit test
- **Dependencies:** SYS-001, SYS-003, LIFE-004, COMP-001
- **Status:** `[ ]`

### ESYS-002 — Wander System

- **Category:** Existing Systems
- **Description:** `src/systems/wander.rs` implements random movement.
  Phase 4 (Actions). Moves entities to adjacent positions.
- **Acceptance Criteria:**
  - [ ] `src/systems/wander.rs` exists
  - [ ] Contains `pub fn run_wander(world: &mut World, tick: Tick)`
  - [ ] Moves entity positions using `world.rng` for direction
  - [ ] Skips entities in `world.pending_deaths`
  - [ ] Uses collect-then-apply mutation pattern
  - [ ] Has a unit test
- **Dependencies:** SYS-001, SYS-003, LIFE-004, RNG-001
- **Status:** `[ ]`

### ESYS-003 — Eating System

- **Category:** Existing Systems
- **Description:** `src/systems/eating.rs` implements the eating action.
  Phase 4 (Actions). Entities consume food to reduce hunger.
- **Acceptance Criteria:**
  - [ ] `src/systems/eating.rs` exists
  - [ ] Contains `pub fn run_eating(world: &mut World, tick: Tick)`
  - [ ] Reduces hunger when entity eats
  - [ ] Skips entities in `world.pending_deaths`
  - [ ] Uses collect-then-apply mutation pattern
  - [ ] Has a unit test
- **Dependencies:** SYS-001, SYS-003, LIFE-004, COMP-001
- **Status:** `[ ]`

### ESYS-004 — Combat System

- **Category:** Existing Systems
- **Description:** `src/systems/combat.rs` implements combat.
  Phase 4 (Actions). Resolves combat between entities.
- **Acceptance Criteria:**
  - [ ] `src/systems/combat.rs` exists
  - [ ] Contains `pub fn run_combat(world: &mut World, tick: Tick)`
  - [ ] Resolves combat interactions between entities
  - [ ] Skips entities in `world.pending_deaths`
  - [ ] Uses collect-then-apply mutation pattern
  - [ ] Uses `world.rng` for combat randomness
  - [ ] Has a unit test
- **Dependencies:** SYS-001, SYS-003, LIFE-004, RNG-001
- **Status:** `[ ]`

### ESYS-005 — Death System

- **Category:** Existing Systems
- **Description:** `src/systems/death.rs` processes pending_deaths.
  Phase 5 (Consequences). ALWAYS the last system called in Phase 5.
  Calls `world.despawn()` for each entity in pending_deaths.
- **Acceptance Criteria:**
  - [ ] `src/systems/death.rs` exists
  - [ ] Contains `pub fn run_death(world: &mut World, tick: Tick)`
  - [ ] Iterates over `world.pending_deaths` and calls `world.despawn()` for each
  - [ ] Clears `world.pending_deaths` after processing
  - [ ] Is the LAST system called in Phase 5
  - [ ] No other system calls `world.despawn()`
  - [ ] Has a unit test
- **Dependencies:** SYS-001, LIFE-003, PHASE-005
- **Status:** `[ ]`

### ESYS-006 — Systems Module Re-export

- **Category:** Existing Systems
- **Description:** `src/systems/mod.rs` declares all system submodules and
  re-exports them.
- **Acceptance Criteria:**
  - [ ] `src/systems/mod.rs` exists
  - [ ] Contains `pub mod hunger;`
  - [ ] Contains `pub mod wander;`
  - [ ] Contains `pub mod eating;`
  - [ ] Contains `pub mod combat;`
  - [ ] Contains `pub mod death;`
  - [ ] Every system file has a corresponding `pub mod` entry
- **Dependencies:** SYS-002
- **Status:** `[ ]`

---

## Domain: Testing & Validation

### TEST-001 — Unit Tests Per System

- **Category:** Testing & Validation
- **Description:** Every system MUST ship with a unit test. Tests construct
  a minimal World with `World::new_with_seed(42)`, spawn an entity, populate
  relevant properties, run the system, and assert state changes.
- **Acceptance Criteria:**
  - [ ] Every system file contains at least one `#[test]` function
  - [ ] Tests use `World::new_with_seed(42)` for deterministic setup
  - [ ] Tests spawn entities and populate required property tables
  - [ ] Tests call the system function and assert observable state changes
  - [ ] Tests do not depend on other systems having run
- **Dependencies:** SYS-001, LIFE-001, RNG-001
- **Status:** `[ ]`

### TEST-002 — Property-Based Tests (invariants.rs)

- **Category:** Testing & Validation
- **Description:** `tests/invariants.rs` contains property-based tests that
  verify cross-system invariants: no zombie entities, food conservation,
  deterministic replay with same seed.
- **Acceptance Criteria:**
  - [ ] `tests/invariants.rs` exists
  - [ ] Tests verify no zombie entities (entity in table but not in alive)
  - [ ] Tests verify food conservation (or equivalent resource invariant)
  - [ ] Tests verify deterministic replay (same seed produces identical results)
  - [ ] Tests run multiple ticks and check invariants hold across time
  - [ ] Tests use seeded RNG for reproducibility
- **Dependencies:** TEST-001, VALID-001, RNG-001
- **Status:** `[ ]`

### TEST-003 — Determinism Tests (determinism.rs)

- **Category:** Testing & Validation
- **Description:** `tests/determinism.rs` verifies deterministic replay.
  Running the simulation twice with the same seed produces identical results.
- **Acceptance Criteria:**
  - [ ] `tests/determinism.rs` exists
  - [ ] Tests run simulation with seed X, record final state
  - [ ] Tests run simulation again with seed X, compare final state
  - [ ] States must be identical for determinism to pass
  - [ ] Tests exercise multiple ticks and multiple systems
- **Dependencies:** RNG-001, CORE-005
- **Status:** `[ ]`

### VALID-001 — validate_world() Debug Assertion

- **Category:** Testing & Validation
- **Description:** `validate_world()` runs every tick in debug builds. Checks
  that no entity exists in any property table without being in `world.alive`.
  Defined in `src/world.rs`.
- **Acceptance Criteria:**
  - [ ] `validate_world(world: &World)` function exists in `src/world.rs`
  - [ ] Called in main loop gated by `#[cfg(debug_assertions)]`
  - [ ] Checks every property table: all keys must be in `world.alive`
  - [ ] Checks every entity in `world.alive` for expected component presence
  - [ ] Panics with descriptive message on any violation
- **Dependencies:** CORE-001, LIFE-003
- **Status:** `[ ]`

---

## Domain: Code Invariants (Rules)

### RULE-001 — No unwrap() on Table Lookups

- **Category:** Code Invariants
- **Description:** Never use `.unwrap()` on property table lookups. Always
  use `if let Some(x) = world.table.get(&entity)`. Missing entries mean
  skip, not panic.
- **Acceptance Criteria:**
  - [ ] No `.unwrap()` call on any `HashMap::get()` result in system code
  - [ ] All table lookups use `if let Some(x)` or equivalent safe pattern
  - [ ] Missing table entries cause the entity to be silently skipped
  - [ ] No logging or error reporting for missing entries
- **Dependencies:** SYS-001
- **Status:** `[ ]`

### RULE-002 — Helpers as Methods on World

- **Category:** Code Invariants
- **Description:** Helper functions shared across systems are methods on World
  in `world.rs`. No `utils.rs`, `helpers.rs`, or separate utility modules.
- **Acceptance Criteria:**
  - [ ] No `utils.rs` or `helpers.rs` file exists
  - [ ] Shared helper logic is implemented as `impl World` methods in `world.rs`
  - [ ] Systems call helpers via `world.method()` syntax
- **Dependencies:** CORE-001
- **Status:** `[ ]`

### RULE-003 — Skip Missing Table Entries

- **Category:** Code Invariants
- **Description:** If a table entry is missing for an entity, skip that entity.
  Do not log, do not panic. Silent skip.
- **Acceptance Criteria:**
  - [ ] Systems that cross-reference tables skip entities missing from secondary tables
  - [ ] No warning or error is logged for missing entries
  - [ ] No panic occurs for missing entries
- **Dependencies:** RULE-001
- **Status:** `[ ]`

---

## Domain: Prohibitions (What NOT To Do)

### PROHIB-001 — No Traits/Interfaces Between Systems

- **Category:** Prohibitions
- **Description:** Do not add traits, interfaces, or abstraction layers between
  systems.
- **Acceptance Criteria:**
  - [ ] No trait definition exists that is implemented by system functions or modules
  - [ ] No abstraction layer wraps system invocation
- **Dependencies:** SYS-004
- **Status:** `[ ]`

### PROHIB-002 — No System Registry or Scheduler

- **Category:** Prohibitions
- **Description:** Do not create a system registry or scheduler. Systems are
  called directly in main.rs in explicit order.
- **Acceptance Criteria:**
  - [ ] No struct or data structure stores a list of system functions
  - [ ] No dynamic dispatch selects which systems to run
  - [ ] Systems are called as direct function calls in main.rs
- **Dependencies:** CORE-005
- **Status:** `[ ]`

### PROHIB-003 — No Message Passing Between Systems

- **Category:** Prohibitions
- **Description:** Do not use message passing or event channels between systems.
  All communication goes through World.
- **Acceptance Criteria:**
  - [ ] No channel (mpsc, crossbeam, etc.) exists for inter-system communication
  - [ ] No queue or mailbox connects systems
  - [ ] EventLog is for observation, not system-to-system signaling
- **Dependencies:** CORE-004
- **Status:** `[ ]`

### PROHIB-004 — No Manual Entity Removal from Tables

- **Category:** Prohibitions
- **Description:** Do not manually remove entities from individual HashMap
  tables. All removal goes through `World::despawn()`.
- **Acceptance Criteria:**
  - [ ] No `.remove()` call on any property table outside of `World::despawn()`
  - [ ] grep confirms: only despawn() calls .remove on property tables
- **Dependencies:** LIFE-003
- **Status:** `[ ]`

### PROHIB-005 — No Multiple Systems Per File

- **Category:** Prohibitions
- **Description:** Do not put multiple systems in one file. One system
  per file in `src/systems/`.
- **Acceptance Criteria:**
  - [ ] Each file in `src/systems/` (excluding mod.rs) has exactly one `pub fn run_*`
  - [ ] No system file re-exports or contains another system's logic
- **Dependencies:** SYS-002
- **Status:** `[ ]`

### PROHIB-006 — No Shared Mutable State Outside World

- **Category:** Prohibitions
- **Description:** Do not create shared mutable state outside of World.
  Global statics, lazy statics, thread-locals for simulation state are forbidden.
- **Acceptance Criteria:**
  - [ ] No `static mut`, `lazy_static!`, or `thread_local!` holds simulation state
  - [ ] All mutable simulation state lives inside World
- **Dependencies:** CORE-001
- **Status:** `[ ]`

### PROHIB-007 — No unsafe Without Approval

- **Category:** Prohibitions
- **Description:** Do not use `unsafe` without explicit user approval.
  Exception: fontconfig FFI in `src/font.rs` has existing approval.
- **Acceptance Criteria:**
  - [ ] No new `unsafe` blocks in simulation code without documented approval
  - [ ] Existing `unsafe` in font.rs FFI is the only approved usage
- **Dependencies:** None
- **Status:** `[ ]`

### PROHIB-008 — No Unseeded RNG

- **Category:** Prohibitions
- **Description:** Do not use `thread_rng()` or any unseeded RNG. All
  randomness must flow through `world.rng`.
- **Acceptance Criteria:**
  - [ ] `thread_rng()` does not appear anywhere in the codebase
  - [ ] `OsRng` does not appear anywhere in the codebase
  - [ ] No RNG is constructed without an explicit seed except `world.rng`
- **Dependencies:** RNG-001
- **Status:** `[ ]`

### PROHIB-009 — No Vec<Event> for Event Log

- **Category:** Prohibitions
- **Description:** Do not use `Vec<Event>` for the event log. Use EventLog
  (ring buffer).
- **Acceptance Criteria:**
  - [ ] `Vec<Event>` is not used as a field in World or anywhere for event storage
  - [ ] All event storage goes through EventLog
- **Dependencies:** EVT-001
- **Status:** `[ ]`

### PROHIB-010 — No HashMap Replacement Without Profiling

- **Category:** Prohibitions
- **Description:** Do not replace HashMap with another data structure without
  profiling data showing >5ms per tick for that system.
- **Acceptance Criteria:**
  - [ ] Property tables use `HashMap<Entity, T>`
  - [ ] Any proposal to change data structure includes profiling data
  - [ ] Profiling must show >5ms per tick for the specific system
- **Dependencies:** CORE-001
- **Status:** `[ ]`

### PROHIB-011 — No Concurrency in Simulation Loop

- **Category:** Prohibitions
- **Description:** Do not add concurrency to the simulation loop. Single-threaded
  sequential execution only.
- **Acceptance Criteria:**
  - [ ] No `thread::spawn` in the simulation loop
  - [ ] No `async` runtime in the simulation loop
  - [ ] No `rayon::par_iter` or parallel iteration in systems
  - [ ] Simulation loop is purely sequential
- **Dependencies:** CORE-005
- **Status:** `[ ]`

---

## Domain: Growth & Scaling Patterns

### GROW-001 — Sub-Struct Grouping at 25+ Fields

- **Category:** Growth & Scaling
- **Description:** When World exceeds ~25 fields, group into sub-structs:
  `world.body.positions`, `world.mind.emotions`, `world.social.friendships`.
  Readability change, not architectural change.
- **Acceptance Criteria:**
  - [ ] Threshold trigger: World has >25 HashMap fields
  - [ ] Fields grouped by domain: body, mind, social (or similar)
  - [ ] Sub-structs are plain structs, not trait objects
  - [ ] All existing code patterns still apply (despawn removes from all sub-tables)
  - [ ] This is a refactor, not a redesign
- **Dependencies:** CORE-001
- **Status:** `[ ]`

### GROW-002 — Phase Function Grouping at 30+ Systems

- **Category:** Growth & Scaling
- **Description:** When the main loop exceeds ~30 systems, group phase calls
  into functions: `run_environment_phase(&mut world, tick)`. Same phase rules
  still apply.
- **Acceptance Criteria:**
  - [ ] Threshold trigger: main loop has >30 system calls
  - [ ] Each phase gets its own function: `run_X_phase(world, tick)`
  - [ ] Phase functions are plain functions, not trait impls
  - [ ] Phase ordering rules are preserved
  - [ ] Phase functions live in main.rs or a dedicated phase module
- **Dependencies:** CORE-005, PHASE-007
- **Status:** `[ ]`

### GROW-003 — System Dependency Analyzer at 15+ Systems

- **Category:** Growth & Scaling
- **Description:** At 15+ systems, build `src/bin/analyze_systems.rs` to
  extract read/write dependencies from source code. Static analysis tool.
- **Acceptance Criteria:**
  - [ ] Threshold trigger: 15+ system files in `src/systems/`
  - [ ] `src/bin/analyze_systems.rs` exists
  - [ ] Analyzer reads system source files
  - [ ] Outputs which World fields each system reads and writes
  - [ ] Helps detect phase violations and ordering issues
- **Dependencies:** SYS-002
- **Status:** `[ ]`

---

## Domain: Rendering

### REND-001 — Font Rendering Pipeline (FreeType + R8Unorm Atlas + WGSL)

- **Category:** Rendering
- **Description:** Kitty-style font rendering pipeline. FreeType rasterizes
  glyphs into an R8Unorm glyph atlas texture. WGSL shaders render text
  with sRGB-correct color blending.
- **Acceptance Criteria:**
  - [ ] `src/font.rs` exists with `FontRenderer` struct
  - [ ] FreeType library rasterizes ASCII 32-126 glyphs
  - [ ] Glyph atlas is R8Unorm texture uploaded to GPU
  - [ ] Shelf-packing algorithm arranges glyphs in atlas
  - [ ] `freetype-rs` crate is a dependency
- **Dependencies:** REND-002, REND-003, REND-004
- **Status:** `[ ]`

### REND-002 — WGSL Text Shader

- **Category:** Rendering
- **Description:** `src/text.wgsl` implements vertex and fragment shaders for
  text rendering. Fragment shader performs sRGB-to-linear conversion, BT.709
  luminance calculation, and kitty-style contrast adjustment. Output is
  premultiplied alpha.
- **Acceptance Criteria:**
  - [ ] `src/text.wgsl` exists
  - [ ] Vertex shader transforms pixel coordinates to clip space via projection matrix
  - [ ] Fragment shader samples R8Unorm atlas for glyph alpha
  - [ ] sRGB-to-linear conversion (IEC 61966-2-1) in fragment shader
  - [ ] BT.709 luminance calculation for contrast adjustment
  - [ ] Kitty-style contrast adjustment formula
  - [ ] Premultiplied alpha output
- **Dependencies:** None
- **Status:** `[ ]`

### REND-003 — Fontconfig FFI Integration

- **Category:** Rendering
- **Description:** Direct fontconfig FFI (no fontconfig crate) queries system
  fonts and hinting configuration. Falls back to hardcoded paths if fontconfig
  is unavailable.
- **Acceptance Criteria:**
  - [ ] fontconfig queried via `#[link(name = "fontconfig")]` FFI
  - [ ] Queries font family, file path, hinting, and hintstyle
  - [ ] Supports family fallback chain (Noto Sans Mono, monospace, DejaVu, Liberation)
  - [ ] Falls back to hardcoded TTF paths if fontconfig fails
  - [ ] Hinting config maps to FreeType load flags (NO_HINTING, TARGET_LIGHT, TARGET_NORMAL)
- **Dependencies:** None
- **Status:** `[ ]`

### REND-004 — GPU State and Window Management

- **Category:** Rendering
- **Description:** wgpu-based GPU state management with winit window. Handles
  surface creation, resize, scale factor changes, and render loop.
- **Acceptance Criteria:**
  - [ ] `GpuState` struct manages wgpu surface, device, queue, config
  - [ ] Window created via winit with "Wulfaz" title
  - [ ] Surface resize handled on WindowEvent::Resized
  - [ ] Scale factor changes trigger font re-rasterization
  - [ ] Render loop clears with Gruvbox Dark background (#282828)
  - [ ] sRGB-to-linear conversion for clear color
  - [ ] Escape key and close button exit the application
  - [ ] wgpu, winit, pollster, bytemuck crates are dependencies
- **Dependencies:** STRUCT-002
- **Status:** `[ ]`

### REND-005 — FontRenderer API

- **Category:** Rendering
- **Description:** FontRenderer provides prepare() and render() methods.
  prepare() builds vertex data for text at given position. render() issues
  draw calls. Vertex buffer grows dynamically.
- **Acceptance Criteria:**
  - [ ] `FontRenderer::new()` takes device, queue, surface_format, font_size, scale_factor
  - [ ] `prepare()` accepts text, position, screen dimensions, fg/bg colors
  - [ ] `prepare()` returns vertex count for subsequent render call
  - [ ] `render()` sets pipeline, bind group, vertex buffer, and draws
  - [ ] Vertex buffer grows dynamically (next_power_of_two) when text exceeds capacity
  - [ ] Unknown glyphs fall back to '?' glyph
  - [ ] Zero-dimension glyphs are skipped
  - [ ] Monospace grid layout with integer-pixel snapping
- **Dependencies:** REND-001, REND-002
- **Status:** `[ ]`

### REND-006 — Render Module (render.rs)

- **Category:** Rendering
- **Description:** `src/render.rs` handles display output for the simulation.
  Distinct from font.rs GPU pipeline -- this module translates simulation
  state into renderable output.
- **Acceptance Criteria:**
  - [ ] `src/render.rs` exists
  - [ ] Reads World state to produce visual output
  - [ ] Translates entity positions, types, and states into renderable text/graphics
  - [ ] Does not modify World state (read-only access)
- **Dependencies:** CORE-001, REND-001
- **Status:** `[ ]`

---

## Domain: Project Structure

### STRUCT-001 — File Layout

- **Category:** Project Structure
- **Description:** The project follows a specific file layout as defined in
  the CLAUDE.md project structure section. Every file has a defined purpose.
- **Acceptance Criteria:**
  - [ ] `CLAUDE.md` exists at project root
  - [ ] `Cargo.toml` exists at project root
  - [ ] `data/` directory with `creatures.kdl`, `items.kdl`, `terrain.kdl`
  - [ ] `src/main.rs` — phased main loop
  - [ ] `src/world.rs` — World struct, spawn, despawn, validate
  - [ ] `src/events.rs` — Event enum + EventLog ring buffer
  - [ ] `src/components.rs` — property structs
  - [ ] `src/tile_map.rs` — TileMap with flat Vec arrays
  - [ ] `src/loading.rs` — KDL parsing, entity spawning
  - [ ] `src/render.rs` — display output
  - [ ] `src/rng.rs` — deterministic seeded RNG wrapper
  - [ ] `src/systems/mod.rs` — system module declarations
  - [ ] `src/systems/hunger.rs`
  - [ ] `src/systems/wander.rs`
  - [ ] `src/systems/eating.rs`
  - [ ] `src/systems/combat.rs`
  - [ ] `src/systems/death.rs`
  - [ ] `tests/invariants.rs` — property-based cross-system tests
  - [ ] `tests/determinism.rs` — replay/seed tests
- **Dependencies:** CORE-001, COMP-001, EVT-001, TILE-001, DATA-001, DATA-002, RNG-002, SYS-002, ESYS-006, VALID-001, TEST-002, TEST-003, REND-006, LOOP-001
- **Status:** `[ ]`

### STRUCT-002 — Cargo.toml Dependencies

- **Category:** Project Structure
- **Description:** Cargo.toml declares all required dependencies for the
  project including simulation and rendering crates.
- **Acceptance Criteria:**
  - [ ] `wgpu` dependency for GPU rendering
  - [ ] `winit` dependency for windowing
  - [ ] `pollster` dependency for async blocking
  - [ ] `env_logger` and `log` dependencies for logging
  - [ ] `freetype-rs` dependency for font rasterization
  - [ ] `bytemuck` dependency with "derive" feature for GPU data
  - [ ] `kdl` dependency for data file parsing
  - [ ] `rand` dependency for seeded RNG
- **Dependencies:** None
- **Status:** `[ ]`

---

## Domain: Main Loop Integration

### LOOP-001 — Phased Main Loop in main.rs

- **Category:** Main Loop Integration
- **Description:** `src/main.rs` contains the main simulation loop that calls
  all systems in phase order each tick. Currently also contains the windowing
  and render loop; simulation loop will be integrated.
- **Acceptance Criteria:**
  - [ ] main.rs contains a tick loop that increments Tick each iteration
  - [ ] Phase 1 (Environment) systems called first
  - [ ] Phase 2 (Needs) systems called second
  - [ ] Phase 3 (Decisions) systems called third
  - [ ] Phase 4 (Actions) systems called fourth
  - [ ] Phase 5 (Consequences) systems called fifth, with run_death() last
  - [ ] Debug validation called after Phase 5 (`#[cfg(debug_assertions)]`)
  - [ ] Comments in main.rs clearly delineate phase boundaries
- **Dependencies:** PHASE-001 through PHASE-006, ESYS-001 through ESYS-005
- **Status:** `[ ]`

---

## Concentric Rings Audit Trail

### Pass 1 (Enumerate) Gap Check
- Covered: World struct, Entity/Tick newtypes, blackboard architecture,
  single-threaded loop, entity lifecycle (spawn/kill/despawn), pending-death
  rule, system signature, one-system-per-file, collect-then-apply, no
  inter-system comms, all 5 phases + debug validation, phase assignment rules,
  3 checklists (add system/property/event), EventLog ring buffer, Event enum,
  event ordering, TileMap flat Vec, TileMap accessors, KDL data files,
  KDL loading, seeded RNG, RNG wrapper, components.rs, all 5 existing systems,
  systems mod.rs, unit tests, property-based tests, determinism tests,
  validate_world, 3 code rules (no unwrap, helpers on World, skip missing),
  all 11 prohibitions, 3 growth patterns, font rendering pipeline, WGSL shader,
  fontconfig FFI, GPU state, FontRenderer API, render.rs, file layout,
  Cargo.toml deps, main loop integration.
- **Gap found:** None. All CLAUDE.md sections accounted for. Font/rendering
  pipeline from memory and source code also captured.

### Pass 2 (Shape) Gap Check
- All features have inputs, outputs, and dependencies defined through
  acceptance criteria and dependency lists.
- **Gap found:** None.

### Pass 3 (Specify) Gap Check
- Edge cases captured: missing table entries (RULE-003), unknown glyphs
  (REND-005), fontconfig fallback (REND-003), zero-dimension glyphs (REND-005),
  surface errors (REND-004), entity spawned mid-tick (LIFE-001).
- **Gap found:** None.

### Pass 4 (Cross-Check) Gap Check
- Verified: LIFE-003 (despawn) connects to ADD-002 step 3 and PROHIB-004.
  EVT-001 connects to PROHIB-009. RNG-001 connects to PROHIB-008. SYS-002
  connects to PROHIB-005. Phase assignment (PHASE-007) connects to all phase
  features. VALID-001 connects to ADD-002 step 4 and PHASE-006.
- **Gap found:** None. All cross-references consistent.
