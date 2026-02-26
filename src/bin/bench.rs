//! Entity scalability benchmark.
//!
//! Spawns increasing numbers of creatures on a 256×256 map, runs ticks,
//! and reports per-system and total tick time to find the lag threshold.
//!
//! Usage: cargo run --release --bin bench

use std::collections::HashMap;
use std::time::{Duration, Instant};

use rand::RngExt;

use wulfaz::components::*;
use wulfaz::systems::{
    combat::run_combat, death::run_death, decisions::run_decisions, eating::run_eating,
    fatigue::run_fatigue, hunger::run_hunger, temperature::run_temperature, wander::run_wander,
};
use wulfaz::tile_map::Terrain;
use wulfaz::world::World;

const MAP_SIZE: usize = 256; // 256×256 = 65 536 tiles (meters)
const WARMUP_TICKS: u32 = 50;
const MEASURE_TICKS: u32 = 100;

/// Spawn `n` creatures with full component sets spread across the map.
fn spawn_creatures(world: &mut World, n: usize) {
    let w = world.tiles.width() as i32;
    let h = world.tiles.height() as i32;
    for i in 0..n {
        let e = world.spawn();
        let x = world.rng.random_range(0..w);
        let y = world.rng.random_range(0..h);
        world.body.positions.insert(e, Position { x, y });
        world.body.healths.insert(
            e,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.body.fatigues.insert(e, Fatigue { current: 0.0 });
        world.body.combat_stats.insert(
            e,
            CombatStats {
                attack: 10.0,
                defense: 5.0,
                aggression: 0.3,
            },
        );
        world.body.gait_profiles.insert(e, GaitProfile::biped());
        world.body.current_gaits.insert(e, Gait::Walk);
        world.body.icons.insert(e, Icon { ch: '@' });
        world.body.names.insert(
            e,
            Name {
                value: format!("creature_{}", i),
            },
        );
        world.mind.hungers.insert(
            e,
            Hunger {
                current: 0.0,
                max: 100.0,
            },
        );
        world.mind.action_states.insert(
            e,
            ActionState {
                current_action: None,
                ticks_in_action: 0,
                cooldowns: HashMap::new(),
            },
        );
    }
}

/// Spawn food items (1 food per 4 creatures, scattered).
fn spawn_food(world: &mut World, n: usize) {
    let w = world.tiles.width() as i32;
    let h = world.tiles.height() as i32;
    for _ in 0..n {
        let e = world.spawn();
        let x = world.rng.random_range(0..w);
        let y = world.rng.random_range(0..h);
        world.body.positions.insert(e, Position { x, y });
        world.body.icons.insert(e, Icon { ch: '%' });
        world.body.names.insert(
            e,
            Name {
                value: "food".to_string(),
            },
        );
        world.mind.nutritions.insert(e, Nutrition { value: 30.0 });
    }
}

/// Scatter terrain variety on the map (same distribution as loading.rs).
fn scatter_terrain(world: &mut World) {
    let w = world.tiles.width();
    let h = world.tiles.height();
    for y in 0..h {
        for x in 0..w {
            let roll: f32 = world.rng.random();
            let terrain = if roll < 0.03 {
                Terrain::Water
            } else if roll < 0.06 {
                Terrain::Wall
            } else if roll < 0.12 {
                Terrain::Floor
            } else if roll < 0.14 {
                Terrain::Door
            } else if roll < 0.20 {
                Terrain::Courtyard
            } else if roll < 0.25 {
                Terrain::Garden
            } else if roll < 0.27 {
                Terrain::Bridge
            } else {
                Terrain::Road
            };
            world.tiles.set_terrain(x, y, terrain);
        }
    }
}

struct SystemTimings {
    spatial1: Duration,
    temperature: Duration,
    hunger: Duration,
    fatigue: Duration,
    decisions: Duration,
    wander: Duration,
    spatial2: Duration,
    eating: Duration,
    combat: Duration,
    death: Duration,
}

impl SystemTimings {
    fn total(&self) -> Duration {
        self.spatial1
            + self.temperature
            + self.hunger
            + self.fatigue
            + self.decisions
            + self.wander
            + self.spatial2
            + self.eating
            + self.combat
            + self.death
    }
}

/// Run one tick, timing each system separately.
fn timed_tick(world: &mut World) -> SystemTimings {
    let tick = world.tick;

    let t = Instant::now();
    world.rebuild_spatial_index();
    let spatial1 = t.elapsed();

    let t = Instant::now();
    run_temperature(world, tick);
    let temperature = t.elapsed();

    let t = Instant::now();
    run_hunger(world, tick);
    let hunger = t.elapsed();

    let t = Instant::now();
    run_fatigue(world, tick);
    let fatigue = t.elapsed();

    let t = Instant::now();
    run_decisions(world, tick);
    let decisions = t.elapsed();

    let t = Instant::now();
    run_wander(world, tick);
    let wander = t.elapsed();

    let t = Instant::now();
    world.rebuild_spatial_index();
    let spatial2 = t.elapsed();

    let t = Instant::now();
    run_eating(world, tick);
    let eating = t.elapsed();

    let t = Instant::now();
    run_combat(world, tick);
    let combat = t.elapsed();

    let t = Instant::now();
    run_death(world, tick);
    let death = t.elapsed();

    world.tick = Tick(tick.0 + 1);

    SystemTimings {
        spatial1,
        temperature,
        hunger,
        fatigue,
        decisions,
        wander,
        spatial2,
        eating,
        combat,
        death,
    }
}

fn run_benchmark(entity_count: usize) {
    let mut world = World::new_with_seed(42);
    // Use a larger tile map for the benchmark.
    world.tiles = wulfaz::tile_map::TileMap::new(MAP_SIZE, MAP_SIZE);
    scatter_terrain(&mut world);

    let food_count = entity_count / 4;
    spawn_creatures(&mut world, entity_count);
    spawn_food(&mut world, food_count);

    let total_entities = world.alive.len();

    // Warmup — let decisions/wander settle.
    for _ in 0..WARMUP_TICKS {
        let tick = world.tick;
        world.rebuild_spatial_index();
        run_temperature(&mut world, tick);
        run_hunger(&mut world, tick);
        run_fatigue(&mut world, tick);
        run_decisions(&mut world, tick);
        run_wander(&mut world, tick);
        world.rebuild_spatial_index();
        run_eating(&mut world, tick);
        run_combat(&mut world, tick);
        run_death(&mut world, tick);
        world.tick = Tick(tick.0 + 1);
    }

    let alive_after_warmup = world.alive.len();

    // Measure
    let mut totals = SystemTimings {
        spatial1: Duration::ZERO,
        temperature: Duration::ZERO,
        hunger: Duration::ZERO,
        fatigue: Duration::ZERO,
        decisions: Duration::ZERO,
        wander: Duration::ZERO,
        spatial2: Duration::ZERO,
        eating: Duration::ZERO,
        combat: Duration::ZERO,
        death: Duration::ZERO,
    };

    let wall_start = Instant::now();
    for _ in 0..MEASURE_TICKS {
        let t = timed_tick(&mut world);
        totals.spatial1 += t.spatial1;
        totals.temperature += t.temperature;
        totals.hunger += t.hunger;
        totals.fatigue += t.fatigue;
        totals.decisions += t.decisions;
        totals.wander += t.wander;
        totals.spatial2 += t.spatial2;
        totals.eating += t.eating;
        totals.combat += t.combat;
        totals.death += t.death;
    }
    let wall_elapsed = wall_start.elapsed();
    let alive_after = world.alive.len();

    let n = MEASURE_TICKS;
    let avg_total = totals.total() / n;
    let budget_ms = 10.0; // 100 ticks/sec = 10ms budget
    let avg_ms = avg_total.as_secs_f64() * 1000.0;
    let ok = if avg_ms <= budget_ms { "OK" } else { "OVER" };

    println!(
        "--- {} creatures ({} food, {} total spawned) ---",
        entity_count, food_count, total_entities
    );
    println!(
        "  alive after warmup: {}, after measure: {}",
        alive_after_warmup, alive_after
    );
    println!("  avg tick: {avg_ms:.2}ms  [{ok}]  (budget: {budget_ms}ms = 100 ticks/sec)");
    println!(
        "  wall time for {} ticks: {:.1}ms",
        MEASURE_TICKS,
        wall_elapsed.as_secs_f64() * 1000.0
    );
    println!("  per-system avg (us):");
    println!(
        "    spatial1:    {:>7.0}",
        totals.spatial1.as_micros() as f64 / n as f64
    );
    println!(
        "    temperature: {:>7.0}",
        totals.temperature.as_micros() as f64 / n as f64
    );
    println!(
        "    hunger:      {:>7.0}",
        totals.hunger.as_micros() as f64 / n as f64
    );
    println!(
        "    fatigue:     {:>7.0}",
        totals.fatigue.as_micros() as f64 / n as f64
    );
    println!(
        "    decisions:   {:>7.0}",
        totals.decisions.as_micros() as f64 / n as f64
    );
    println!(
        "    wander:      {:>7.0}",
        totals.wander.as_micros() as f64 / n as f64
    );
    println!(
        "    spatial2:    {:>7.0}",
        totals.spatial2.as_micros() as f64 / n as f64
    );
    println!(
        "    eating:      {:>7.0}",
        totals.eating.as_micros() as f64 / n as f64
    );
    println!(
        "    combat:      {:>7.0}",
        totals.combat.as_micros() as f64 / n as f64
    );
    println!(
        "    death:       {:>7.0}",
        totals.death.as_micros() as f64 / n as f64
    );
    println!();
}

fn main() {
    let counts = [100, 500, 1_000, 2_000, 5_000, 10_000, 20_000, 50_000];

    println!("=== Wulfaz Entity Scalability Benchmark ===");
    println!(
        "Map: {}x{} tiles, warmup: {} ticks, measure: {} ticks",
        MAP_SIZE, MAP_SIZE, WARMUP_TICKS, MEASURE_TICKS
    );
    println!("Budget: 10ms/tick (100 ticks/sec)");
    println!();

    for &count in &counts {
        run_benchmark(count);
    }
}
