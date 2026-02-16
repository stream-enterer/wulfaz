//! Deterministic replay and seed tests.
//!
//! These tests verify that the simulation produces identical results when run
//! with the same seed, and different results with different seeds. They require
//! the crate to expose its internals as a library. Add a `src/lib.rs` that
//! re-exports the modules (components, world, systems, events, etc.) to make
//! these compile:
//!
//! ```rust
//! // src/lib.rs
//! pub mod components;
//! pub mod events;
//! pub mod rng;
//! pub mod systems;
//! pub mod tile_map;
//! pub mod world;
//! ```

use std::collections::HashMap;

use wulfaz::components::*;
use wulfaz::systems::combat::run_combat;
use wulfaz::systems::death::run_death;
use wulfaz::systems::decisions::run_decisions;
use wulfaz::systems::eating::run_eating;
use wulfaz::systems::hunger::run_hunger;
use wulfaz::systems::temperature::run_temperature;
use wulfaz::systems::wander::run_wander;
use wulfaz::world::World;

/// Spawn a creature with full components at the given position.
fn spawn_creature(world: &mut World, x: i32, y: i32) -> Entity {
    let e = world.spawn();
    world.positions.insert(e, Position { x, y });
    world.hungers.insert(
        e,
        Hunger {
            current: 20.0,
            max: 100.0,
        },
    );
    world.healths.insert(
        e,
        Health {
            current: 100.0,
            max: 100.0,
        },
    );
    world.combat_stats.insert(
        e,
        CombatStats {
            attack: 10.0,
            defense: 5.0,
            aggression: 0.6,
        },
    );
    world.speeds.insert(e, Speed { value: 1 });
    world.icons.insert(e, Icon { ch: 'c' });
    world.names.insert(
        e,
        Name {
            value: "Creature".to_string(),
        },
    );
    world.action_states.insert(
        e,
        ActionState {
            current_action: None,
            ticks_in_action: 0,
            cooldowns: HashMap::new(),
        },
    );
    e
}

/// Spawn a food item at the given position.
fn spawn_food(world: &mut World, x: i32, y: i32) -> Entity {
    let e = world.spawn();
    world.positions.insert(e, Position { x, y });
    world.nutritions.insert(e, Nutrition { value: 30.0 });
    world.icons.insert(e, Icon { ch: 'f' });
    world.names.insert(
        e,
        Name {
            value: "Food".to_string(),
        },
    );
    e
}

/// Run all five phases in order.
fn run_full_tick(world: &mut World, tick: Tick) {
    // Phase 1: Environment
    run_temperature(world, tick);
    // Phase 2: Needs
    run_hunger(world, tick);
    // Phase 3: Decisions
    run_decisions(world, tick);
    // Phase 4: Actions
    run_wander(world, tick);
    run_eating(world, tick);
    run_combat(world, tick);
    // Phase 5: Consequences
    run_death(world, tick);
}

/// Set up a standard scenario for determinism testing.
/// Uses the provided world (which already has its seed set).
fn setup_scenario(world: &mut World) {
    wulfaz::loading::load_utility_config(world, "data/utility.ron");
    // Spread creatures across the grid
    for i in 0..6 {
        spawn_creature(world, i * 4, i * 3);
    }
    // Add some food items
    for i in 0..4 {
        spawn_food(world, i * 4, i * 3);
    }
}

/// Run N ticks and return a snapshot of the simulation state.
fn run_and_snapshot(world: &mut World, n: u64) -> WorldSnapshot {
    for i in 0..n {
        let tick = Tick(i);
        world.tick = tick;
        run_full_tick(world, tick);
    }
    WorldSnapshot::capture(world)
}

/// A minimal snapshot of world state for comparison.
/// We capture everything that could differ between runs.
struct WorldSnapshot {
    alive_count: usize,
    /// Sorted entity IDs for deterministic comparison.
    alive_ids: Vec<u64>,
    /// Sorted (entity_id, x, y) for all positioned entities.
    positions: Vec<(u64, i32, i32)>,
    /// Sorted (entity_id, hunger_current) for all hungry entities.
    hungers: Vec<(u64, u32)>, // f32 bits as u32 for exact comparison
    /// Sorted (entity_id, health_current) for all entities with health.
    healths: Vec<(u64, u32)>,
    /// Sorted (entity_id, action_id_ordinal) for all entities with intentions.
    intentions: Vec<(u64, u8)>,
    /// Total event count in the log.
    event_count: usize,
}

impl WorldSnapshot {
    fn capture(world: &World) -> Self {
        let mut alive_ids: Vec<u64> = world.alive.iter().map(|e| e.0).collect();
        alive_ids.sort();

        let mut positions: Vec<(u64, i32, i32)> = world
            .positions
            .iter()
            .map(|(&e, p)| (e.0, p.x, p.y))
            .collect();
        positions.sort_by_key(|&(id, _, _)| id);

        let mut hungers: Vec<(u64, u32)> = world
            .hungers
            .iter()
            .map(|(&e, h)| (e.0, h.current.to_bits()))
            .collect();
        hungers.sort_by_key(|&(id, _)| id);

        let mut healths: Vec<(u64, u32)> = world
            .healths
            .iter()
            .map(|(&e, h)| (e.0, h.current.to_bits()))
            .collect();
        healths.sort_by_key(|&(id, _)| id);

        let mut intentions: Vec<(u64, u8)> = world
            .intentions
            .iter()
            .map(|(&e, i)| (e.0, i.action as u8))
            .collect();
        intentions.sort_by_key(|&(id, _)| id);

        Self {
            alive_count: world.alive.len(),
            alive_ids,
            positions,
            hungers,
            healths,
            intentions,
            event_count: world.events.len(),
        }
    }
}

impl PartialEq for WorldSnapshot {
    fn eq(&self, other: &Self) -> bool {
        self.alive_count == other.alive_count
            && self.alive_ids == other.alive_ids
            && self.positions == other.positions
            && self.hungers == other.hungers
            && self.healths == other.healths
            && self.intentions == other.intentions
            && self.event_count == other.event_count
    }
}

impl std::fmt::Debug for WorldSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldSnapshot")
            .field("alive_count", &self.alive_count)
            .field("alive_ids", &self.alive_ids)
            .field("positions_len", &self.positions.len())
            .field("hungers_len", &self.hungers.len())
            .field("healths_len", &self.healths.len())
            .field("intentions_len", &self.intentions.len())
            .field("event_count", &self.event_count)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Same seed produces identical simulation state
// ---------------------------------------------------------------------------

#[test]
fn same_seed_same_result() {
    let tick_count = 100;

    // Run 1
    let mut world1 = World::new_with_seed(42);
    setup_scenario(&mut world1);
    let snap1 = run_and_snapshot(&mut world1, tick_count);

    // Run 2 — identical seed and setup
    let mut world2 = World::new_with_seed(42);
    setup_scenario(&mut world2);
    let snap2 = run_and_snapshot(&mut world2, tick_count);

    assert_eq!(
        snap1, snap2,
        "two runs with seed 42 diverged after {tick_count} ticks"
    );
}

// ---------------------------------------------------------------------------
// Same seed, verify at every tick (not just final state)
// ---------------------------------------------------------------------------

#[test]
fn same_seed_identical_at_every_tick() {
    let tick_count = 50;

    let mut world1 = World::new_with_seed(42);
    setup_scenario(&mut world1);

    let mut world2 = World::new_with_seed(42);
    setup_scenario(&mut world2);

    for i in 0..tick_count {
        let tick = Tick(i);
        world1.tick = tick;
        world2.tick = tick;

        run_full_tick(&mut world1, tick);
        run_full_tick(&mut world2, tick);

        let snap1 = WorldSnapshot::capture(&world1);
        let snap2 = WorldSnapshot::capture(&world2);
        assert_eq!(snap1, snap2, "worlds diverged at tick {i}");
    }
}

// ---------------------------------------------------------------------------
// Different seeds produce different results
// ---------------------------------------------------------------------------

#[test]
fn different_seeds_different_results() {
    let tick_count = 50;

    let mut world1 = World::new_with_seed(42);
    setup_scenario(&mut world1);
    let snap1 = run_and_snapshot(&mut world1, tick_count);

    let mut world2 = World::new_with_seed(99);
    setup_scenario(&mut world2);
    let snap2 = run_and_snapshot(&mut world2, tick_count);

    // At least one observable field should differ.
    // Positions are the most likely to diverge due to random wander.
    let positions_differ = snap1.positions != snap2.positions;
    let hungers_differ = snap1.hungers != snap2.hungers;
    let healths_differ = snap1.healths != snap2.healths;
    let alive_differs = snap1.alive_ids != snap2.alive_ids;

    assert!(
        positions_differ || hungers_differ || healths_differ || alive_differs,
        "seeds 42 and 99 produced identical state after {tick_count} ticks — \
         randomness is not affecting the simulation",
    );
}

// ---------------------------------------------------------------------------
// Multiple replays all agree
// ---------------------------------------------------------------------------

#[test]
fn multiple_replays_identical() {
    let tick_count = 80;
    let seeds = [42u64, 7, 1337, 0, u64::MAX];

    for &seed in &seeds {
        let mut world_a = World::new_with_seed(seed);
        setup_scenario(&mut world_a);
        let snap_a = run_and_snapshot(&mut world_a, tick_count);

        let mut world_b = World::new_with_seed(seed);
        setup_scenario(&mut world_b);
        let snap_b = run_and_snapshot(&mut world_b, tick_count);

        assert_eq!(
            snap_a, snap_b,
            "replay mismatch with seed {seed} after {tick_count} ticks",
        );
    }
}

// ---------------------------------------------------------------------------
// Deterministic across a longer run with heavy interaction
// ---------------------------------------------------------------------------

#[test]
fn deterministic_with_dense_combat() {
    let tick_count = 150;

    // Set up a scenario with lots of combat: many creatures on one tile
    let setup = |world: &mut World| {
        wulfaz::loading::load_utility_config(world, "data/utility.ron");
        for _ in 0..12 {
            spawn_creature(world, 5, 5);
        }
        for _ in 0..6 {
            spawn_food(world, 5, 5);
        }
    };

    let mut world1 = World::new_with_seed(42);
    setup(&mut world1);
    let snap1 = run_and_snapshot(&mut world1, tick_count);

    let mut world2 = World::new_with_seed(42);
    setup(&mut world2);
    let snap2 = run_and_snapshot(&mut world2, tick_count);

    assert_eq!(
        snap1, snap2,
        "dense combat scenario diverged after {tick_count} ticks",
    );
}
