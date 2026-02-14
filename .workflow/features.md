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
  - [x] World struct exists in `src/world.rs`
  - [x] World contains only `HashMap<Entity, T>` fields for entity properties
  - [x] World contains a `TileMap` field for grid data
  - [x] World contains `alive: HashSet<Entity>` (NOT HashMap)
  - [x] World contains `pending_deaths: Vec<Entity>`
  - [x] World contains `rng: StdRng`
  - [x] World contains `events: EventLog`
  - [x] No simulation state exists outside the World struct
- **Dependencies:** CORE-002, CORE-003, TILE-001, EVT-001, RNG-001
- **Status:** `[x]`

### CORE-002 — Entity Newtype

- **Category:** Core Architecture
- **Description:** `Entity(pub u64)` newtype wrapper. All entity references use
  this type. Raw `u64` is never used where an entity ID is meant.
- **Acceptance Criteria:**
  - [x] `pub struct Entity(pub u64)` is defined
  - [x] Entity implements Hash, Eq, PartialEq, Clone, Copy
  - [x] No raw u64 is used in place of Entity anywhere in the codebase
  - [x] Entity is never cast to or from Tick
- **Dependencies:** None
- **Status:** `[x]`

### CORE-003 — Tick Newtype

- **Category:** Core Architecture
- **Description:** `Tick(pub u64)` newtype wrapper. All tick/time references use
  this type. Raw `u64` is never used where a tick count is meant.
- **Acceptance Criteria:**
  - [x] `pub struct Tick(pub u64)` is defined
  - [x] No raw u64 is used in place of Tick anywhere in the codebase
  - [x] Tick is never cast to or from Entity
  - [x] Every Event variant includes a `tick: Tick` field
- **Dependencies:** None
- **Status:** `[x]`

### CORE-004 — Blackboard Architecture

- **Category:** Core Architecture
- **Description:** Systems communicate only through shared state on World.
  No message passing, no traits between systems, no direct system-to-system
  calls. World is the sole communication channel.
- **Acceptance Criteria:**
  - [x] No system imports or calls another system directly
  - [x] No message-passing channels exist between systems
  - [x] No trait definitions connect systems to each other
  - [x] All inter-system data flow passes through World fields
- **Dependencies:** CORE-001, SYS-001
- **Status:** `[x]`

### CORE-005 — Single-Threaded Sequential Loop

- **Category:** Core Architecture
- **Description:** The simulation runs as a single-threaded phase-ordered
  sequential loop. No concurrency in the simulation loop.
- **Acceptance Criteria:**
  - [x] main.rs runs a sequential tick loop
  - [x] No threads, async tasks, or parallel iterators in the simulation loop
  - [x] Systems are called in deterministic phase order each tick
  - [x] Phase order is explicitly defined in main.rs
- **Dependencies:** CORE-001
- **Status:** `[x]`

---

## Domain: Entity Lifecycle

### LIFE-001 — Entity Spawning via World::spawn()

- **Category:** Entity Lifecycle
- **Description:** New entities are created exclusively through `world.spawn()`,
  which returns an Entity with a unique ID and adds it to the alive set.
  Property tables are populated after spawning.
- **Acceptance Criteria:**
  - [x] `World::spawn()` method exists and returns `Entity`
  - [x] spawn() generates a unique Entity ID (monotonically increasing or equivalent)
  - [x] spawn() inserts the new Entity into `world.alive`
  - [x] After spawn(), caller inserts into relevant property tables
  - [x] Entities may be spawned in any phase
  - [x] Newly spawned entities are not processed by earlier phases until next tick
- **Dependencies:** CORE-001, CORE-002
- **Status:** `[x]`

### LIFE-002 — Entity Kill via pending_deaths

- **Category:** Entity Lifecycle
- **Description:** To kill an entity, push a death event and then push the
  entity to `world.pending_deaths`. Only `run_death` actually despawns. No
  other system calls despawn. Event ordering follows ADD-003: lethal events
  are pushed AFTER the kill decision, BEFORE `pending_deaths.push()`.
- **Acceptance Criteria:**
  - [x] Killing an entity means `world.pending_deaths.push(entity)`
  - [x] A death event is pushed BEFORE `pending_deaths.push()` (per ADD-003 rule)
  - [x] No system other than run_death calls `world.despawn()`
  - [x] pending_deaths is cleared after run_death processes all entries
- **Dependencies:** CORE-001, LIFE-001, EVT-002
- **Status:** `[x]`

### LIFE-003 — Entity Despawn via World::despawn()

- **Category:** Entity Lifecycle
- **Description:** `World::despawn()` removes an entity from ALL property
  tables and from the alive set. This is the ONLY mechanism for removing
  entities from tables. Manual `.remove()` on individual tables is forbidden.
- **Acceptance Criteria:**
  - [x] `World::despawn(entity)` method exists
  - [x] despawn removes entity from `world.alive`
  - [x] despawn calls `.remove(&entity)` on EVERY HashMap property table
  - [x] No code outside `World::despawn()` calls `.remove()` on property tables
  - [x] Only `run_death` system calls `world.despawn()`
- **Dependencies:** CORE-001, LIFE-001
- **Status:** `[x]`

### LIFE-004 — Pending-Death Filtering Rule

- **Category:** Entity Lifecycle
- **Description:** Every system that iterates over entities MUST skip entities
  present in `world.pending_deaths`. This prevents dead-entity processing.
- **Acceptance Criteria:**
  - [x] Every system's iteration loop checks `world.pending_deaths.contains(&entity)`
  - [x] Entities in pending_deaths are skipped (continue), not processed
  - [x] No system processes an entity that is already marked for death
- **Dependencies:** LIFE-002, SYS-001
- **Status:** `[x]`

---

## Domain: Systems Framework

### SYS-001 — System Function Signature

- **Category:** Systems Framework
- **Description:** Systems are plain functions with the signature
  `pub fn run_x(world: &mut World, tick: Tick)`. No traits, no structs,
  no closures. Plain functions only.
- **Acceptance Criteria:**
  - [x] Every system is a `pub fn` with exactly `(world: &mut World, tick: Tick)` params
  - [x] No system is a method on a struct or trait impl
  - [x] No system uses closures as its primary entry point
  - [x] Function names follow the `run_*` convention
- **Dependencies:** CORE-001, CORE-003
- **Status:** `[x]`

### SYS-002 — One System Per File

- **Category:** Systems Framework
- **Description:** Each system lives in its own file under `src/systems/`.
  No file contains multiple system functions.
- **Acceptance Criteria:**
  - [x] `src/systems/` directory exists
  - [x] Each `.rs` file in `src/systems/` contains exactly one `pub fn run_*` function
  - [x] `src/systems/mod.rs` re-exports all system modules
  - [x] No system function exists outside `src/systems/`
- **Dependencies:** SYS-001
- **Status:** `[x]`

### SYS-003 — Collect-Then-Apply Mutation Pattern

- **Category:** Systems Framework
- **Description:** Systems MUST collect changes into a Vec first, then apply
  them in a second pass. Never mutate a HashMap while iterating over it.
- **Acceptance Criteria:**
  - [x] No system mutates a property table while iterating over it
  - [x] Changes are collected into a Vec (or similar) before application
  - [x] Application phase uses `get_mut()` or `insert()`, not direct indexing
  - [x] Application phase uses `if let Some(x)` for safety, not unwrap
- **Dependencies:** SYS-001, RULE-001
- **Status:** `[x]`

### SYS-004 — No Inter-System Communication

- **Category:** Systems Framework
- **Description:** Systems do not communicate with each other directly. No
  message passing, no event channels between systems, no direct function calls
  between systems, no shared traits.
- **Acceptance Criteria:**
  - [x] No system file imports another system's module
  - [x] No channel, queue, or mailbox exists for system-to-system messages
  - [x] No trait is defined to be implemented by multiple systems
  - [x] No system calls another system's run function
- **Dependencies:** CORE-004
- **Status:** `[x]`

---

## Domain: Main Loop Phases

### PHASE-001 — Phase 1: Environment

- **Category:** Main Loop Phases
- **Description:** Environment phase handles weather, temperature, plant growth,
  decay, and fluid flow. Reads/writes tile data and environmental state.
- **Acceptance Criteria:**
  - [x] Environment systems run first in the tick loop
  - [x] Environment systems read and write TileMap data
  - [x] Environment systems handle weather, temperature, growth, decay, fluid flow
  - [x] No environment system modifies entity external state (position, HP, inventory)
- **Dependencies:** CORE-005, TILE-001
- **Status:** `[x]`

### PHASE-002 — Phase 2: Needs

- **Category:** Main Loop Phases
- **Description:** Needs phase handles hunger, thirst, tiredness, emotions.
  Reads environment. Writes entity internal state (need values).
- **Acceptance Criteria:**
  - [x] Needs systems run after environment phase
  - [x] Needs systems read environment/tile state but do not write it
  - [x] Needs systems write entity internal state (hunger, thirst, tiredness, emotions)
  - [x] Hunger system exists: `src/systems/hunger.rs`
- **Dependencies:** CORE-005, PHASE-001
- **Status:** `[x]`

### PHASE-003 — Phase 3: Decisions

- **Category:** Main Loop Phases
- **Description:** Decision phase handles AI planning, pathfinding, task
  selection. Reads needs and environment. Writes intentions (what entity
  will attempt this tick).
- **Acceptance Criteria:**
  - [x] Decision systems run after needs phase
  - [x] Decision systems read needs and environment state
  - [x] Decision systems write intention/goal state on entities
  - [x] Decision systems do not modify external world state
- **Dependencies:** CORE-005, PHASE-002
- **Status:** `[x]`

### PHASE-004 — Phase 4: Actions

- **Category:** Main Loop Phases
- **Description:** Action phase handles movement, eating, combat, building,
  crafting. Changes external world state: positions, HP, inventory. Any system
  that changes external world state belongs here.
- **Acceptance Criteria:**
  - [x] Action systems run after decision phase
  - [x] Action systems change external world state (positions, HP, inventory)
  - [x] Wander system exists: `src/systems/wander.rs` (movement)
  - [x] Eating system exists: `src/systems/eating.rs`
  - [x] Combat system exists: `src/systems/combat.rs`
  - [x] Phase assignment rule: external state change = Phase 4
- **Dependencies:** CORE-005, PHASE-003
- **Status:** `[x]`

### PHASE-005 — Phase 5: Consequences

- **Category:** Main Loop Phases
- **Description:** Consequence phase derives consequences from state changed
  this tick: injury, relationship updates, reputation, death. `run_death()`
  is ALWAYS the last system in this phase.
- **Acceptance Criteria:**
  - [x] Consequence systems run after action phase
  - [x] Consequence systems derive state from changes made this tick
  - [x] `run_death()` is the final system call in Phase 5
  - [x] Death system exists: `src/systems/death.rs`
  - [x] Phase assignment rule: consequence derivation = Phase 5
  - [x] run_death processes all pending_deaths and calls world.despawn()
- **Dependencies:** CORE-005, PHASE-004, LIFE-002, LIFE-003
- **Status:** `[x]`

### PHASE-006 — Debug Validation Phase

- **Category:** Main Loop Phases
- **Description:** After all five phases, `validate_world()` runs in debug
  builds (`#[cfg(debug_assertions)]`). Checks for zombie entities and
  other invariants.
- **Acceptance Criteria:**
  - [x] `validate_world(&world)` is called after Phase 5 in the main loop
  - [x] validate_world is gated behind `#[cfg(debug_assertions)]`
  - [x] validate_world checks that no entity in any property table is missing from alive
  - [x] validate_world checks that no entity in alive is missing expected components
  - [x] validate_world panics on invariant violations in debug builds
- **Dependencies:** CORE-005, PHASE-005, VALID-001
- **Status:** `[x]`

### PHASE-007 — Phase Assignment Rules

- **Category:** Main Loop Phases
- **Description:** Formal rules for determining which phase a system belongs
  to. If a system changes external world state (position, HP, inventory), it
  is Phase 4. If it derives consequences from changes already made this tick
  (death check, relationship recalc), it is Phase 5.
- **Acceptance Criteria:**
  - [x] Every system is assigned to exactly one phase
  - [x] Systems that modify external world state are in Phase 4
  - [x] Systems that derive consequences are in Phase 5
  - [x] Phase assignment is documented in main.rs comments
  - [x] No system violates its phase's read/write contract
- **Dependencies:** PHASE-001 through PHASE-005
- **Status:** `[x]`

---

## Domain: Adding New Components (Checklists)

### ADD-001 — Adding a New System (6-Step Checklist)

- **Category:** Checklists
- **Description:** Defined procedure for adding any new system to the engine.
  All six steps must be completed for a system addition to be valid.
- **Acceptance Criteria:**
  - [x] Step 1: Create `src/systems/new_system.rs`
  - [x] Step 2: Write `pub fn run_new_system(world: &mut World, tick: Tick)`
  - [x] Step 3: Add `pub mod new_system;` to `src/systems/mod.rs`
  - [x] Step 4: Add the call to the correct phase in `main.rs`
  - [x] Step 5: Write a unit test (construct minimal World, run system, assert state change)
  - [x] Step 6: `cargo build` + debug mode confirms `validate_world()` passes
- **Dependencies:** SYS-001, SYS-002, PHASE-007, TEST-001
- **Status:** `[x]`

### ADD-002 — Adding a New Property Table (5-Step Checklist)

- **Category:** Checklists
- **Description:** Defined procedure for adding a new `HashMap<Entity, T>` to
  World. All five steps must be completed or zombie entity bugs will result.
- **Acceptance Criteria:**
  - [x] Step 1: Add the struct in `src/components.rs`
  - [x] Step 2: Add `HashMap<Entity, T>` field to World in `world.rs`
  - [x] Step 3: Add `.remove(&entity)` in `World::despawn()`
  - [x] Step 4: Add an alive-check in `validate_world()`
  - [x] Step 5: Initialize to `HashMap::new()` in `World::new()`
  - [x] Skipping any step produces a zombie entity bug
- **Dependencies:** CORE-001, LIFE-003, VALID-001
- **Status:** `[x]`

### ADD-003 — Adding a New Event Type (4-Step Checklist)

- **Category:** Checklists
- **Description:** Defined procedure for adding a new variant to the Event enum.
- **Acceptance Criteria:**
  - [x] Step 1: Add the variant to `Event` in `src/events.rs`
  - [x] Step 2: Every variant MUST include `tick: Tick`
  - [x] Step 3: For lethal events, push AFTER the decision, BEFORE `pending_deaths.push()`
  - [x] Step 4: For non-lethal events, push immediately after the state change
- **Dependencies:** CORE-003, EVT-001, EVT-002
- **Status:** `[x]`

---

## Domain: Events

### EVT-001 — EventLog Ring Buffer

- **Category:** Events
- **Description:** EventLog is a ring buffer with configurable max depth
  (default 10,000). Old events are overwritten, not accumulated unboundedly.
  `Vec<Event>` must never be used for the event log.
- **Acceptance Criteria:**
  - [x] EventLog struct exists in `src/events.rs`
  - [x] EventLog is a ring buffer, not a Vec
  - [x] Default max depth is 10,000 events
  - [x] Max depth is configurable at construction time
  - [x] Old events are silently overwritten when capacity is reached
  - [x] EventLog API: `push(event)`, `iter()`, `recent(n)`
  - [x] No code uses `Vec<Event>` for event storage
- **Dependencies:** CORE-003
- **Status:** `[x]`

### EVT-002 — Event Enum with Tick Field

- **Category:** Events
- **Description:** The Event enum defines all event types. Every variant
  must include a `tick: Tick` field for temporal ordering.
- **Acceptance Criteria:**
  - [x] `Event` enum exists in `src/events.rs`
  - [x] Every variant includes `tick: Tick`
  - [x] Died variant exists for entity death events
  - [x] Events are pushed via `world.events.push(event)`
- **Dependencies:** CORE-003
- **Status:** `[x]`

### EVT-003 — Event Ordering Rules

- **Category:** Events
- **Description:** Events must be pushed at specific points relative to state
  changes. Lethal events: push AFTER the decision, BEFORE `pending_deaths.push()`.
  Non-lethal events: push immediately after the state change.
- **Acceptance Criteria:**
  - [x] Lethal event push precedes `pending_deaths.push()` in all kill sites
  - [x] Non-lethal event push immediately follows the state change it describes
  - [x] No event is pushed without the corresponding state change occurring
- **Dependencies:** EVT-001, EVT-002, LIFE-002
- **Status:** `[x]`

---

## Domain: TileMap

### TILE-001 — TileMap with Flat Vec Arrays

- **Category:** TileMap
- **Description:** Grid data lives in TileMap using flat `Vec<T>` arrays.
  Never HashMap for grid data. Internally indexed by `y * width + x`.
- **Acceptance Criteria:**
  - [x] `TileMap` struct exists in `src/tile_map.rs`
  - [x] Grid data stored as flat `Vec<T>` arrays, not HashMap
  - [x] Internal indexing uses `y * width + x`
  - [x] TileMap stores width and height dimensions
  - [x] No HashMap is used for grid/tile data
- **Dependencies:** None
- **Status:** `[x]`

### TILE-002 — TileMap Accessor Methods

- **Category:** TileMap
- **Description:** Systems access tile data through TileMap methods only.
  Direct Vec indexing is forbidden. Methods include get/set for terrain,
  temperature, and other tile properties.
- **Acceptance Criteria:**
  - [x] `get_terrain(x, y)` method exists
  - [x] `set_temperature(x, y, temp)` method exists
  - [x] No system indexes TileMap's internal Vec arrays directly
  - [x] Accessor methods handle bounds checking
  - [x] All tile reads/writes go through TileMap methods
- **Dependencies:** TILE-001
- **Status:** `[x]`

---

## Domain: Data Pipeline

### DATA-001 — KDL Data File Format

- **Category:** Data Pipeline
- **Description:** Content (creatures, items, terrain) is defined in `data/*.kdl`
  files. The engine does not hardcode entity types. Parsed with the `kdl` crate.
- **Acceptance Criteria:**
  - [x] `data/` directory exists at project root
  - [x] `data/creatures.kdl` defines creature types
  - [x] `data/items.kdl` defines item types
  - [x] `data/terrain.kdl` defines terrain types
  - [x] KDL format is used (not TOML, JSON, YAML)
  - [x] `kdl` crate is a dependency in Cargo.toml
  - [x] No entity type is hardcoded in Rust source
- **Dependencies:** None
- **Status:** `[x]`

### DATA-002 — KDL Loading and Entity Spawning

- **Category:** Data Pipeline
- **Description:** `src/loading.rs` parses KDL data files and maps nodes to
  spawned entities. Adding a new creature/item type requires only a KDL
  file change, not code changes.
- **Acceptance Criteria:**
  - [x] `src/loading.rs` exists
  - [x] loading.rs reads and parses `data/*.kdl` files
  - [x] KDL nodes are mapped to entity spawns with correct property table entries
  - [x] Adding a new creature type to KDL requires no Rust code changes
  - [x] KDL node attributes (icon, max_hunger, aggression, speed, etc.) map to components
  - [x] Parse errors are handled gracefully (no unwrap on KDL parsing)
- **Dependencies:** CORE-001, LIFE-001, COMP-001, DATA-001
- **Status:** `[x]`

---

## Domain: Deterministic RNG

### RNG-001 — Seeded StdRng on World

- **Category:** Deterministic RNG
- **Description:** All randomness goes through `world.rng`, a seeded `StdRng`.
  Never use `thread_rng()` or any other RNG source. This guarantees
  deterministic replay given the same seed.
- **Acceptance Criteria:**
  - [x] `world.rng` is of type `StdRng`
  - [x] `World::new_with_seed(seed)` constructor accepts a seed value
  - [x] All systems use `world.rng` for random decisions
  - [x] No code uses `thread_rng()`, `OsRng`, or any other RNG source
  - [x] Same seed produces identical simulation runs
- **Dependencies:** None
- **Status:** `[x]`

### RNG-002 — RNG Wrapper Module

- **Category:** Deterministic RNG
- **Description:** `src/rng.rs` provides a deterministic seeded RNG wrapper.
  May contain helpers for common random operations while ensuring all
  randomness flows through the single seeded source.
- **Acceptance Criteria:**
  - [x] `src/rng.rs` exists
  - [x] RNG wrapper uses StdRng internally
  - [x] Wrapper enforces single RNG source constraint
  - [x] Helper functions for common operations (range, choice, etc.) if needed
- **Dependencies:** RNG-001
- **Status:** `[x]`

---

## Domain: Components

### COMP-001 — Property Structs in components.rs

- **Category:** Components
- **Description:** All property/component structs (Position, Hunger, etc.)
  are defined in `src/components.rs`. These are the `T` in
  `HashMap<Entity, T>` property tables.
- **Acceptance Criteria:**
  - [x] `src/components.rs` exists
  - [x] Position struct is defined (at minimum x, y fields)
  - [x] Hunger struct is defined (current and max fields)
  - [x] All property structs used as values in World's HashMaps live here
  - [x] No property struct is defined in a system file
- **Dependencies:** CORE-002
- **Status:** `[x]`

---

## Domain: Existing Systems

### ESYS-001 — Hunger System

- **Category:** Existing Systems
- **Description:** `src/systems/hunger.rs` implements the hunger system.
  Phase 2 (Needs). Increases hunger over time for entities with a Hunger
  component.
- **Acceptance Criteria:**
  - [x] `src/systems/hunger.rs` exists
  - [x] Contains `pub fn run_hunger(world: &mut World, tick: Tick)`
  - [x] Iterates over `world.hungers` and increases hunger values
  - [x] Skips entities in `world.pending_deaths`
  - [x] Uses collect-then-apply mutation pattern
  - [x] Has a unit test
- **Dependencies:** SYS-001, SYS-003, LIFE-004, COMP-001
- **Status:** `[x]`

### ESYS-002 — Wander System

- **Category:** Existing Systems
- **Description:** `src/systems/wander.rs` implements random movement.
  Phase 4 (Actions). Moves entities to adjacent positions.
- **Acceptance Criteria:**
  - [x] `src/systems/wander.rs` exists
  - [x] Contains `pub fn run_wander(world: &mut World, tick: Tick)`
  - [x] Moves entity positions using `world.rng` for direction
  - [x] Skips entities in `world.pending_deaths`
  - [x] Uses collect-then-apply mutation pattern
  - [x] Has a unit test
- **Dependencies:** SYS-001, SYS-003, LIFE-004, RNG-001
- **Status:** `[x]`

### ESYS-003 — Eating System

- **Category:** Existing Systems
- **Description:** `src/systems/eating.rs` implements the eating action.
  Phase 4 (Actions). Entities consume food to reduce hunger.
- **Acceptance Criteria:**
  - [x] `src/systems/eating.rs` exists
  - [x] Contains `pub fn run_eating(world: &mut World, tick: Tick)`
  - [x] Reduces hunger when entity eats
  - [x] Skips entities in `world.pending_deaths`
  - [x] Uses collect-then-apply mutation pattern
  - [x] Has a unit test
- **Dependencies:** SYS-001, SYS-003, LIFE-004, COMP-001
- **Status:** `[x]`

### ESYS-004 — Combat System

- **Category:** Existing Systems
- **Description:** `src/systems/combat.rs` implements combat.
  Phase 4 (Actions). Resolves combat between entities.
- **Acceptance Criteria:**
  - [x] `src/systems/combat.rs` exists
  - [x] Contains `pub fn run_combat(world: &mut World, tick: Tick)`
  - [x] Resolves combat interactions between entities
  - [x] Skips entities in `world.pending_deaths`
  - [x] Uses collect-then-apply mutation pattern
  - [x] Uses `world.rng` for combat randomness
  - [x] Has a unit test
- **Dependencies:** SYS-001, SYS-003, LIFE-004, RNG-001
- **Status:** `[x]`

### ESYS-005 — Death System

- **Category:** Existing Systems
- **Description:** `src/systems/death.rs` processes pending_deaths.
  Phase 5 (Consequences). ALWAYS the last system called in Phase 5.
  Calls `world.despawn()` for each entity in pending_deaths.
- **Acceptance Criteria:**
  - [x] `src/systems/death.rs` exists
  - [x] Contains `pub fn run_death(world: &mut World, tick: Tick)`
  - [x] Iterates over `world.pending_deaths` and calls `world.despawn()` for each
  - [x] Clears `world.pending_deaths` after processing
  - [x] Is the LAST system called in Phase 5
  - [x] No other system calls `world.despawn()`
  - [x] Has a unit test
- **Dependencies:** SYS-001, LIFE-003, PHASE-005
- **Status:** `[x]`

### ESYS-006 — Systems Module Re-export

- **Category:** Existing Systems
- **Description:** `src/systems/mod.rs` declares all system submodules and
  re-exports them.
- **Acceptance Criteria:**
  - [x] `src/systems/mod.rs` exists
  - [x] Contains `pub mod hunger;`
  - [x] Contains `pub mod wander;`
  - [x] Contains `pub mod eating;`
  - [x] Contains `pub mod combat;`
  - [x] Contains `pub mod death;`
  - [x] Every system file has a corresponding `pub mod` entry
- **Dependencies:** SYS-002
- **Status:** `[x]`

---

## Domain: Testing & Validation

### TEST-001 — Unit Tests Per System

- **Category:** Testing & Validation
- **Description:** Every system MUST ship with a unit test. Tests construct
  a minimal World with `World::new_with_seed(42)`, spawn an entity, populate
  relevant properties, run the system, and assert state changes.
- **Acceptance Criteria:**
  - [x] Every system file contains at least one `#[test]` function
  - [x] Tests use `World::new_with_seed(42)` for deterministic setup
  - [x] Tests spawn entities and populate required property tables
  - [x] Tests call the system function and assert observable state changes
  - [x] Tests do not depend on other systems having run
- **Dependencies:** SYS-001, LIFE-001, RNG-001
- **Status:** `[x]`

### TEST-002 — Property-Based Tests (invariants.rs)

- **Category:** Testing & Validation
- **Description:** `tests/invariants.rs` contains property-based tests that
  verify cross-system invariants: no zombie entities, food conservation,
  deterministic replay with same seed.
- **Acceptance Criteria:**
  - [x] `tests/invariants.rs` exists
  - [x] Tests verify no zombie entities (entity in table but not in alive)
  - [x] Tests verify food conservation (or equivalent resource invariant)
  - [x] Tests verify deterministic replay (same seed produces identical results)
  - [x] Tests run multiple ticks and check invariants hold across time
  - [x] Tests use seeded RNG for reproducibility
- **Dependencies:** TEST-001, VALID-001, RNG-001
- **Status:** `[x]`

### TEST-003 — Determinism Tests (determinism.rs)

- **Category:** Testing & Validation
- **Description:** `tests/determinism.rs` verifies deterministic replay.
  Running the simulation twice with the same seed produces identical results.
- **Acceptance Criteria:**
  - [x] `tests/determinism.rs` exists
  - [x] Tests run simulation with seed X, record final state
  - [x] Tests run simulation again with seed X, compare final state
  - [x] States must be identical for determinism to pass
  - [x] Tests exercise multiple ticks and multiple systems
- **Dependencies:** RNG-001, CORE-005
- **Status:** `[x]`

### VALID-001 — validate_world() Debug Assertion

- **Category:** Testing & Validation
- **Description:** `validate_world()` runs every tick in debug builds. Checks
  that no entity exists in any property table without being in `world.alive`.
  Defined in `src/world.rs`.
- **Acceptance Criteria:**
  - [x] `validate_world(world: &World)` function exists in `src/world.rs`
  - [x] Called in main loop gated by `#[cfg(debug_assertions)]`
  - [x] Checks every property table: all keys must be in `world.alive`
  - [x] Checks every entity in `world.alive` for expected component presence
  - [x] Panics with descriptive message on any violation
- **Dependencies:** CORE-001, LIFE-003
- **Status:** `[x]`

---

## Domain: Code Invariants (Rules)

### RULE-001 — No unwrap() on Table Lookups

- **Category:** Code Invariants
- **Description:** Never use `.unwrap()` on property table lookups. Always
  use `if let Some(x) = world.table.get(&entity)`. Missing entries mean
  skip, not panic.
- **Acceptance Criteria:**
  - [x] No `.unwrap()` call on any `HashMap::get()` result in system code
  - [x] All table lookups use `if let Some(x)` or equivalent safe pattern
  - [x] Missing table entries cause the entity to be silently skipped
  - [x] No logging or error reporting for missing entries
- **Dependencies:** SYS-001
- **Status:** `[x]`

### RULE-002 — Helpers as Methods on World

- **Category:** Code Invariants
- **Description:** Helper functions shared across systems are methods on World
  in `world.rs`. No `utils.rs`, `helpers.rs`, or separate utility modules.
- **Acceptance Criteria:**
  - [x] No `utils.rs` or `helpers.rs` file exists
  - [x] Shared helper logic is implemented as `impl World` methods in `world.rs`
  - [x] Systems call helpers via `world.method()` syntax
- **Dependencies:** CORE-001
- **Status:** `[x]`

### RULE-003 — Skip Missing Table Entries

- **Category:** Code Invariants
- **Description:** If a table entry is missing for an entity, skip that entity.
  Do not log, do not panic. Silent skip.
- **Acceptance Criteria:**
  - [x] Systems that cross-reference tables skip entities missing from secondary tables
  - [x] No warning or error is logged for missing entries
  - [x] No panic occurs for missing entries
- **Dependencies:** RULE-001
- **Status:** `[x]`

---

## Domain: Prohibitions (What NOT To Do)

### PROHIB-001 — No Traits/Interfaces Between Systems

- **Category:** Prohibitions
- **Description:** Do not add traits, interfaces, or abstraction layers between
  systems.
- **Acceptance Criteria:**
  - [x] No trait definition exists that is implemented by system functions or modules
  - [x] No abstraction layer wraps system invocation
- **Dependencies:** SYS-004
- **Status:** `[x]`

### PROHIB-002 — No System Registry or Scheduler

- **Category:** Prohibitions
- **Description:** Do not create a system registry or scheduler. Systems are
  called directly in main.rs in explicit order.
- **Acceptance Criteria:**
  - [x] No struct or data structure stores a list of system functions
  - [x] No dynamic dispatch selects which systems to run
  - [x] Systems are called as direct function calls in main.rs
- **Dependencies:** CORE-005
- **Status:** `[x]`

### PROHIB-003 — No Message Passing Between Systems

- **Category:** Prohibitions
- **Description:** Do not use message passing or event channels between systems.
  All communication goes through World.
- **Acceptance Criteria:**
  - [x] No channel (mpsc, crossbeam, etc.) exists for inter-system communication
  - [x] No queue or mailbox connects systems
  - [x] EventLog is for observation, not system-to-system signaling
- **Dependencies:** CORE-004
- **Status:** `[x]`

### PROHIB-004 — No Manual Entity Removal from Tables

- **Category:** Prohibitions
- **Description:** Do not manually remove entities from individual HashMap
  tables. All removal goes through `World::despawn()`.
- **Acceptance Criteria:**
  - [x] No `.remove()` call on any property table outside of `World::despawn()`
  - [x] grep confirms: only despawn() calls .remove on property tables
- **Dependencies:** LIFE-003
- **Status:** `[x]`

### PROHIB-005 — No Multiple Systems Per File

- **Category:** Prohibitions
- **Description:** Do not put multiple systems in one file. One system
  per file in `src/systems/`.
- **Acceptance Criteria:**
  - [x] Each file in `src/systems/` (excluding mod.rs) has exactly one `pub fn run_*`
  - [x] No system file re-exports or contains another system's logic
- **Dependencies:** SYS-002
- **Status:** `[x]`

### PROHIB-006 — No Shared Mutable State Outside World

- **Category:** Prohibitions
- **Description:** Do not create shared mutable state outside of World.
  Global statics, lazy statics, thread-locals for simulation state are forbidden.
- **Acceptance Criteria:**
  - [x] No `static mut`, `lazy_static!`, or `thread_local!` holds simulation state
  - [x] All mutable simulation state lives inside World
- **Dependencies:** CORE-001
- **Status:** `[x]`

### PROHIB-007 — No unsafe Without Approval

- **Category:** Prohibitions
- **Description:** Do not use `unsafe` without explicit user approval.
  Exception: fontconfig FFI in `src/font.rs` has existing approval.
- **Acceptance Criteria:**
  - [x] No new `unsafe` blocks in simulation code without documented approval
  - [x] Existing `unsafe` in font.rs FFI is the only approved usage
- **Dependencies:** None
- **Status:** `[x]`

### PROHIB-008 — No Unseeded RNG

- **Category:** Prohibitions
- **Description:** Do not use `thread_rng()` or any unseeded RNG. All
  randomness must flow through `world.rng`.
- **Acceptance Criteria:**
  - [x] `thread_rng()` does not appear anywhere in the codebase
  - [x] `OsRng` does not appear anywhere in the codebase
  - [x] No RNG is constructed without an explicit seed except `world.rng`
- **Dependencies:** RNG-001
- **Status:** `[x]`

### PROHIB-009 — No Vec<Event> for Event Log

- **Category:** Prohibitions
- **Description:** Do not use `Vec<Event>` for the event log. Use EventLog
  (ring buffer).
- **Acceptance Criteria:**
  - [x] `Vec<Event>` is not used as a field in World or anywhere for event storage
  - [x] All event storage goes through EventLog
- **Dependencies:** EVT-001
- **Status:** `[x]`

### PROHIB-010 — No HashMap Replacement Without Profiling

- **Category:** Prohibitions
- **Description:** Do not replace HashMap with another data structure without
  profiling data showing >5ms per tick for that system.
- **Acceptance Criteria:**
  - [x] Property tables use `HashMap<Entity, T>`
  - [x] Any proposal to change data structure includes profiling data
  - [x] Profiling must show >5ms per tick for the specific system
- **Dependencies:** CORE-001
- **Status:** `[x]`

### PROHIB-011 — No Concurrency in Simulation Loop

- **Category:** Prohibitions
- **Description:** Do not add concurrency to the simulation loop. Single-threaded
  sequential execution only.
- **Acceptance Criteria:**
  - [x] No `thread::spawn` in the simulation loop
  - [x] No `async` runtime in the simulation loop
  - [x] No `rayon::par_iter` or parallel iteration in systems
  - [x] Simulation loop is purely sequential
- **Dependencies:** CORE-005
- **Status:** `[x]`

---

## Domain: Growth & Scaling Patterns

### GROW-001 — Sub-Struct Grouping at 25+ Fields

- **Category:** Growth & Scaling
- **Description:** When World exceeds ~25 fields, group into sub-structs:
  `world.body.positions`, `world.mind.emotions`, `world.social.friendships`.
  Readability change, not architectural change.
- **Acceptance Criteria:**
  - [x] Threshold trigger: World has >25 HashMap fields
  - [x] Fields grouped by domain: body, mind, social (or similar)
  - [x] Sub-structs are plain structs, not trait objects
  - [x] All existing code patterns still apply (despawn removes from all sub-tables)
  - [x] This is a refactor, not a redesign
- **Dependencies:** CORE-001
- **Status:** `[x]`

### GROW-002 — Phase Function Grouping at 30+ Systems

- **Category:** Growth & Scaling
- **Description:** When the main loop exceeds ~30 systems, group phase calls
  into functions: `run_environment_phase(&mut world, tick)`. Same phase rules
  still apply.
- **Acceptance Criteria:**
  - [x] Threshold trigger: main loop has >30 system calls
  - [x] Each phase gets its own function: `run_X_phase(world, tick)`
  - [x] Phase functions are plain functions, not trait impls
  - [x] Phase ordering rules are preserved
  - [x] Phase functions live in main.rs or a dedicated phase module
- **Dependencies:** CORE-005, PHASE-007
- **Status:** `[x]`

### GROW-003 — System Dependency Analyzer at 15+ Systems

- **Category:** Growth & Scaling
- **Description:** At 15+ systems, build `src/bin/analyze_systems.rs` to
  extract read/write dependencies from source code. Static analysis tool.
- **Acceptance Criteria:**
  - [x] Threshold trigger: 15+ system files in `src/systems/`
  - [x] `src/bin/analyze_systems.rs` exists
  - [x] Analyzer reads system source files
  - [x] Outputs which World fields each system reads and writes
  - [x] Helps detect phase violations and ordering issues
- **Dependencies:** SYS-002
- **Status:** `[x]`

---

## Domain: Rendering

### REND-001 — Font Rendering Pipeline (FreeType + R8Unorm Atlas + WGSL)

- **Category:** Rendering
- **Description:** Kitty-style font rendering pipeline. FreeType rasterizes
  glyphs into an R8Unorm glyph atlas texture. WGSL shaders render text
  with sRGB-correct color blending.
- **Acceptance Criteria:**
  - [x] `src/font.rs` exists with `FontRenderer` struct
  - [x] FreeType library rasterizes ASCII 32-126 glyphs
  - [x] Glyph atlas is R8Unorm texture uploaded to GPU
  - [x] Shelf-packing algorithm arranges glyphs in atlas
  - [x] `freetype-rs` crate is a dependency
- **Dependencies:** REND-002, REND-003, REND-004
- **Status:** `[x]`

### REND-002 — WGSL Text Shader

- **Category:** Rendering
- **Description:** `src/text.wgsl` implements vertex and fragment shaders for
  text rendering. Fragment shader performs sRGB-to-linear conversion, BT.709
  luminance calculation, and kitty-style contrast adjustment. Output is
  premultiplied alpha.
- **Acceptance Criteria:**
  - [x] `src/text.wgsl` exists
  - [x] Vertex shader transforms pixel coordinates to clip space via projection matrix
  - [x] Fragment shader samples R8Unorm atlas for glyph alpha
  - [x] sRGB-to-linear conversion (IEC 61966-2-1) in fragment shader
  - [x] BT.709 luminance calculation for contrast adjustment
  - [x] Kitty-style contrast adjustment formula
  - [x] Premultiplied alpha output
- **Dependencies:** None
- **Status:** `[x]`

### REND-003 — Fontconfig FFI Integration

- **Category:** Rendering
- **Description:** Direct fontconfig FFI (no fontconfig crate) queries system
  fonts and hinting configuration. Falls back to hardcoded paths if fontconfig
  is unavailable.
- **Acceptance Criteria:**
  - [x] fontconfig queried via `#[link(name = "fontconfig")]` FFI
  - [x] Queries font family, file path, hinting, and hintstyle
  - [x] Supports family fallback chain (Noto Sans Mono, monospace, DejaVu, Liberation)
  - [x] Falls back to hardcoded TTF paths if fontconfig fails
  - [x] Hinting config maps to FreeType load flags (NO_HINTING, TARGET_LIGHT, TARGET_NORMAL)
- **Dependencies:** None
- **Status:** `[x]`

### REND-004 — GPU State and Window Management

- **Category:** Rendering
- **Description:** wgpu-based GPU state management with winit window. Handles
  surface creation, resize, scale factor changes, and render loop.
- **Acceptance Criteria:**
  - [x] `GpuState` struct manages wgpu surface, device, queue, config
  - [x] Window created via winit with "Wulfaz" title
  - [x] Surface resize handled on WindowEvent::Resized
  - [x] Scale factor changes trigger font re-rasterization
  - [x] Render loop clears with Gruvbox Dark background (#282828)
  - [x] sRGB-to-linear conversion for clear color
  - [x] Escape key and close button exit the application
  - [x] wgpu, winit, pollster, bytemuck crates are dependencies
- **Dependencies:** STRUCT-002
- **Status:** `[x]`

### REND-005 — FontRenderer API

- **Category:** Rendering
- **Description:** FontRenderer provides prepare() and render() methods.
  prepare() builds vertex data for text at given position. render() issues
  draw calls. Vertex buffer grows dynamically.
- **Acceptance Criteria:**
  - [x] `FontRenderer::new()` takes device, queue, surface_format, font_size, scale_factor
  - [x] `prepare()` accepts text, position, screen dimensions, fg/bg colors
  - [x] `prepare()` returns vertex count for subsequent render call
  - [x] `render()` sets pipeline, bind group, vertex buffer, and draws
  - [x] Vertex buffer grows dynamically (next_power_of_two) when text exceeds capacity
  - [x] Unknown glyphs fall back to '?' glyph
  - [x] Zero-dimension glyphs are skipped
  - [x] Monospace grid layout with integer-pixel snapping
- **Dependencies:** REND-001, REND-002
- **Status:** `[x]`

### REND-006 — Render Module (render.rs)

- **Category:** Rendering
- **Description:** `src/render.rs` handles display output for the simulation.
  Distinct from font.rs GPU pipeline -- this module translates simulation
  state into renderable output.
- **Acceptance Criteria:**
  - [x] `src/render.rs` exists
  - [x] Reads World state to produce visual output
  - [x] Translates entity positions, types, and states into renderable text/graphics
  - [x] Does not modify World state (read-only access)
- **Dependencies:** CORE-001, REND-001
- **Status:** `[x]`

---

## Domain: Project Structure

### STRUCT-001 — File Layout

- **Category:** Project Structure
- **Description:** The project follows a specific file layout as defined in
  the CLAUDE.md project structure section. Every file has a defined purpose.
- **Acceptance Criteria:**
  - [x] `CLAUDE.md` exists at project root
  - [x] `Cargo.toml` exists at project root
  - [x] `data/` directory with `creatures.kdl`, `items.kdl`, `terrain.kdl`
  - [x] `src/main.rs` — phased main loop
  - [x] `src/world.rs` — World struct, spawn, despawn, validate
  - [x] `src/events.rs` — Event enum + EventLog ring buffer
  - [x] `src/components.rs` — property structs
  - [x] `src/tile_map.rs` — TileMap with flat Vec arrays
  - [x] `src/loading.rs` — KDL parsing, entity spawning
  - [x] `src/render.rs` — display output
  - [x] `src/rng.rs` — deterministic seeded RNG wrapper
  - [x] `src/systems/mod.rs` — system module declarations
  - [x] `src/systems/hunger.rs`
  - [x] `src/systems/wander.rs`
  - [x] `src/systems/eating.rs`
  - [x] `src/systems/combat.rs`
  - [x] `src/systems/death.rs`
  - [x] `tests/invariants.rs` — property-based cross-system tests
  - [x] `tests/determinism.rs` — replay/seed tests
- **Dependencies:** CORE-001, COMP-001, EVT-001, TILE-001, DATA-001, DATA-002, RNG-002, SYS-002, ESYS-006, VALID-001, TEST-002, TEST-003, REND-006, LOOP-001
- **Status:** `[x]`

### STRUCT-002 — Cargo.toml Dependencies

- **Category:** Project Structure
- **Description:** Cargo.toml declares all required dependencies for the
  project including simulation and rendering crates.
- **Acceptance Criteria:**
  - [x] `wgpu` dependency for GPU rendering
  - [x] `winit` dependency for windowing
  - [x] `pollster` dependency for async blocking
  - [x] `env_logger` and `log` dependencies for logging
  - [x] `freetype-rs` dependency for font rasterization
  - [x] `bytemuck` dependency with "derive" feature for GPU data
  - [x] `kdl` dependency for data file parsing
  - [x] `rand` dependency for seeded RNG
- **Dependencies:** None
- **Status:** `[x]`

---

## Domain: Main Loop Integration

### LOOP-001 — Phased Main Loop in main.rs

- **Category:** Main Loop Integration
- **Description:** `src/main.rs` contains the main simulation loop that calls
  all systems in phase order each tick. Currently also contains the windowing
  and render loop; simulation loop will be integrated.
- **Acceptance Criteria:**
  - [x] main.rs contains a tick loop that increments Tick each iteration
  - [x] Phase 1 (Environment) systems called first
  - [x] Phase 2 (Needs) systems called second
  - [x] Phase 3 (Decisions) systems called third
  - [x] Phase 4 (Actions) systems called fourth
  - [x] Phase 5 (Consequences) systems called fifth, with run_death() last
  - [x] Debug validation called after Phase 5 (`#[cfg(debug_assertions)]`)
  - [x] Comments in main.rs clearly delineate phase boundaries
- **Dependencies:** PHASE-001 through PHASE-006, ESYS-001 through ESYS-005
- **Status:** `[x]`

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
