//! Property-based cross-system invariant tests.
//!
//! These are integration tests that exercise multiple systems together and
//! verify structural invariants hold across ticks. They require the crate to
//! expose its internals as a library. Add a `src/lib.rs` that re-exports the
//! modules (components, world, systems, events, etc.) to make these compile:
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

use wulfaz::components::*;
use wulfaz::systems::combat::run_combat;
use wulfaz::systems::death::run_death;
use wulfaz::systems::eating::run_eating;
use wulfaz::systems::hunger::run_hunger;
use wulfaz::systems::temperature::run_temperature;
use wulfaz::systems::wander::run_wander;
use wulfaz::tile_map::TileMap;
use wulfaz::world::{World, validate_world};

/// Create a test world with a 64×64 tilemap (matches default)
/// to keep temperature iteration fast in tests.
fn test_world(seed: u64) -> World {
    let mut world = World::new_with_seed(seed);
    world.tiles = TileMap::new(64, 64);
    world
}

/// Spawn a creature with a full set of property table entries.
/// Placed at the given grid position.
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
    e
}

/// Spawn a food item at the given grid position.
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

/// Run all five phases in order, matching the main loop phase contract.
fn run_full_tick(world: &mut World, tick: Tick) {
    // Phase 1: Environment
    run_temperature(world, tick);
    // Phase 2: Needs
    run_hunger(world, tick);
    // Phase 3: Decisions (no systems yet)
    // Phase 4: Actions
    run_wander(world, tick);
    run_eating(world, tick);
    run_combat(world, tick);
    // Phase 5: Consequences — run_death ALWAYS last
    run_death(world, tick);
}

/// Run N ticks, updating world.tick each time.
fn run_n_ticks(world: &mut World, n: u64) {
    for i in 0..n {
        let tick = Tick(i);
        world.tick = tick;
        run_full_tick(world, tick);
    }
}

// ---------------------------------------------------------------------------
// Invariant: no zombie entities after a full tick
// ---------------------------------------------------------------------------

#[test]
fn no_zombie_entities_after_full_tick() {
    let mut world = test_world(42);

    // Spawn several creatures spread out so they don't immediately kill each other
    for i in 0..10 {
        spawn_creature(&mut world, i * 3, i * 3);
    }

    // Run 100 ticks
    run_n_ticks(&mut world, 100);

    // validate_world checks every property table key is in alive
    validate_world(&world);
}

// ---------------------------------------------------------------------------
// Invariant: entity count conservation (spawned - died = alive)
// ---------------------------------------------------------------------------

#[test]
fn entity_count_conservation() {
    let mut world = test_world(42);

    let initial_count = 8;
    for i in 0..initial_count {
        spawn_creature(&mut world, i * 5, i * 5);
    }

    // Track how many we started with
    let spawned = world.alive.len();
    assert_eq!(spawned, initial_count as usize);

    // Run 50 ticks
    run_n_ticks(&mut world, 50);

    // Count died events
    let died_count = world
        .events
        .iter()
        .filter(|e| matches!(e, wulfaz::events::Event::Died { .. }))
        .count();

    // alive = spawned - died (no new spawns happen during these ticks)
    assert_eq!(
        world.alive.len(),
        spawned - died_count,
        "alive ({}) should equal spawned ({}) minus died ({})",
        world.alive.len(),
        spawned,
        died_count,
    );
}

// ---------------------------------------------------------------------------
// Invariant: pending_deaths is empty after run_death
// ---------------------------------------------------------------------------

#[test]
fn pending_deaths_empty_after_death_phase() {
    let mut world = test_world(42);

    // Spawn creatures at same position to provoke combat deaths
    for _ in 0..6 {
        spawn_creature(&mut world, 5, 5);
    }

    for i in 0..50u64 {
        let tick = Tick(i);
        world.tick = tick;

        // Run all phases
        run_hunger(&mut world, tick);
        run_wander(&mut world, tick);
        run_eating(&mut world, tick);
        run_combat(&mut world, tick);

        // Before death phase, pending_deaths may be non-empty
        // After death phase, it MUST be empty
        run_death(&mut world, tick);
        assert!(
            world.pending_deaths.is_empty(),
            "pending_deaths not empty after run_death on tick {}",
            i,
        );
    }
}

// ---------------------------------------------------------------------------
// Invariant: all property table keys are in alive set (validate_world)
// ---------------------------------------------------------------------------

#[test]
fn all_property_table_keys_in_alive_set() {
    let mut world = test_world(42);

    // Mix of creatures and food items
    for i in 0..5 {
        spawn_creature(&mut world, i * 2, 0);
        spawn_food(&mut world, i * 2, 0);
    }

    // Run 100 ticks — deaths will happen from combat
    for i in 0..100u64 {
        let tick = Tick(i);
        world.tick = tick;
        run_full_tick(&mut world, tick);

        // Check invariant EVERY tick
        validate_world(&world);
    }
}

// ---------------------------------------------------------------------------
// Invariant: no zombie entities with many interacting creatures
// ---------------------------------------------------------------------------

#[test]
fn no_zombies_with_dense_population() {
    let mut world = test_world(123);

    // Dense cluster: many creatures on same tile = lots of combat
    for _ in 0..20 {
        spawn_creature(&mut world, 10, 10);
    }
    // Scatter some food around
    for i in 0..10 {
        spawn_food(&mut world, 8 + i, 10);
    }

    run_n_ticks(&mut world, 200);

    // World must still be structurally valid
    validate_world(&world);
    assert!(world.pending_deaths.is_empty());
}

// ---------------------------------------------------------------------------
// Invariant: dead entities are fully cleaned up from all tables
// ---------------------------------------------------------------------------

#[test]
fn dead_entities_removed_from_all_tables() {
    let mut world = test_world(42);

    let entities: Vec<Entity> = (0..10).map(|i| spawn_creature(&mut world, i, 0)).collect();

    // Kill half of them manually
    for &e in &entities[..5] {
        world.pending_deaths.push(e);
    }

    run_death(&mut world, Tick(0));

    for &e in &entities[..5] {
        assert!(!world.alive.contains(&e), "entity should not be alive");
        assert!(
            !world.positions.contains_key(&e),
            "entity should not be in positions"
        );
        assert!(
            !world.hungers.contains_key(&e),
            "entity should not be in hungers"
        );
        assert!(
            !world.healths.contains_key(&e),
            "entity should not be in healths"
        );
        assert!(
            !world.combat_stats.contains_key(&e),
            "entity should not be in combat_stats"
        );
        assert!(
            !world.speeds.contains_key(&e),
            "entity should not be in speeds"
        );
        assert!(
            !world.icons.contains_key(&e),
            "entity should not be in icons"
        );
        assert!(
            !world.names.contains_key(&e),
            "entity should not be in names"
        );
    }

    // The other half should still be alive
    for &e in &entities[5..] {
        assert!(world.alive.contains(&e), "entity should still be alive");
        assert!(
            world.positions.contains_key(&e),
            "entity should still be in positions"
        );
    }

    validate_world(&world);
}
