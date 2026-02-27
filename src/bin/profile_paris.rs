//! Profile per-system tick cost on the real Paris map.
//!
//! Loads paris.tiles + paris.meta.bin, spawns GIS entities for Arcis,
//! runs 200 ticks, and reports per-system average timings.
//!
//! Usage: cargo run --release --bin profile_paris

use std::time::Instant;

use wulfaz::components::Tick;
use wulfaz::loading;
use wulfaz::loading_gis;
use wulfaz::systems::{
    combat::run_combat, death::run_death, decisions::run_decisions, eating::run_eating,
    fatigue::run_fatigue, hunger::run_hunger, temperature::run_temperature, wander::run_wander,
};
use wulfaz::world::World;

const WARMUP_TICKS: u32 = 20;
const MEASURE_TICKS: u32 = 200;

struct Timings {
    spatial1: u128,
    temperature: u128,
    hunger: u128,
    fatigue: u128,
    decisions: u128,
    wander: u128,
    spatial2: u128,
    eating: u128,
    combat: u128,
    death: u128,
}

impl Timings {
    fn zero() -> Self {
        Self {
            spatial1: 0,
            temperature: 0,
            hunger: 0,
            fatigue: 0,
            decisions: 0,
            wander: 0,
            spatial2: 0,
            eating: 0,
            combat: 0,
            death: 0,
        }
    }
    fn total(&self) -> u128 {
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

fn timed_tick(world: &mut World) -> Timings {
    let tick = world.tick;
    let mut t = Timings::zero();

    macro_rules! measure {
        ($field:ident, $body:expr) => {{
            let start = Instant::now();
            $body;
            t.$field = start.elapsed().as_micros();
        }};
    }

    measure!(spatial1, world.rebuild_spatial_index());
    measure!(temperature, run_temperature(world, tick));
    measure!(hunger, run_hunger(world, tick));
    measure!(fatigue, run_fatigue(world, tick));
    measure!(decisions, run_decisions(world, tick));
    measure!(wander, run_wander(world, tick));
    measure!(spatial2, world.rebuild_spatial_index());
    measure!(eating, run_eating(world, tick));
    measure!(combat, run_combat(world, tick));
    measure!(death, run_death(world, tick));

    world.tick = Tick(tick.0 + 1);
    t
}

fn main() {
    let mut world = World::new_with_seed(42);

    // Load real Paris map
    let paris_tiles = std::path::Path::new("data/paris.tiles");
    let paris_meta = std::path::Path::new("data/paris.meta.bin");
    let paris_ron = std::path::Path::new("data/paris.ron.zst");

    if paris_tiles.exists() && paris_meta.exists() {
        loading_gis::load_paris_binary(
            &mut world,
            paris_tiles.to_str().expect("non-UTF8 path"),
            paris_meta.to_str().expect("non-UTF8 path"),
        );
    } else if paris_ron.exists() {
        let data = loading_gis::load_paris_ron(paris_ron.to_str().expect("non-UTF8 path"));
        loading_gis::apply_paris_ron(&mut world, data);
    } else {
        eprintln!("ERROR: No Paris map data found. Need data/paris.tiles + data/paris.meta.bin");
        std::process::exit(1);
    }

    world.tiles.initialize_temperatures();
    loading::load_utility_config(&mut world, "data/utility.ron");
    let archetypes = loading::load_archetypes("data/archetypes.kdl");
    let person = archetypes
        .get("person")
        .expect("data/archetypes.kdl must define a 'person' archetype");
    loading_gis::spawn_gis_entities(&mut world, "Arcis", person);

    let entity_count = world.alive.len();
    let map_w = world.tiles.width();
    let map_h = world.tiles.height();
    println!(
        "Map: {}x{} ({:.1}M tiles)",
        map_w,
        map_h,
        (map_w * map_h) as f64 / 1_000_000.0
    );
    println!("Entities: {}", entity_count);
    println!();

    // Warmup
    print!("Warming up ({} ticks)...", WARMUP_TICKS);
    for _ in 0..WARMUP_TICKS {
        let _ = timed_tick(&mut world);
    }
    println!(" done");

    // Measure
    println!("Measuring {} ticks...", MEASURE_TICKS);
    let mut sum = Timings::zero();
    let mut max_total: u128 = 0;

    // Track per-tick wander for spotting A* spikes
    let mut wander_max: u128 = 0;
    let mut wander_spikes = 0u32; // ticks where wander > 1000us

    for i in 0..MEASURE_TICKS {
        let t = timed_tick(&mut world);
        let tick_total = t.total();

        if tick_total > max_total {
            max_total = tick_total;
        }
        if t.wander > wander_max {
            wander_max = t.wander;
        }
        if t.wander > 1000 {
            wander_spikes += 1;
            if wander_spikes <= 5 {
                println!(
                    "  tick {}: wander spike {}us (total {}us)",
                    i, t.wander, tick_total
                );
            }
        }

        sum.spatial1 += t.spatial1;
        sum.temperature += t.temperature;
        sum.hunger += t.hunger;
        sum.fatigue += t.fatigue;
        sum.decisions += t.decisions;
        sum.wander += t.wander;
        sum.spatial2 += t.spatial2;
        sum.eating += t.eating;
        sum.combat += t.combat;
        sum.death += t.death;
    }

    let n = MEASURE_TICKS as u128;
    println!();
    println!(
        "=== Per-system averages ({}x{}, {} entities) ===",
        map_w, map_h, entity_count
    );
    println!("  spatial1:    {:>8}us", sum.spatial1 / n);
    println!("  temperature: {:>8}us", sum.temperature / n);
    println!("  hunger:      {:>8}us", sum.hunger / n);
    println!("  fatigue:     {:>8}us", sum.fatigue / n);
    println!("  decisions:   {:>8}us", sum.decisions / n);
    println!(
        "  wander:      {:>8}us  (max: {}us, spikes>1ms: {}/{})",
        sum.wander / n,
        wander_max,
        wander_spikes,
        MEASURE_TICKS
    );
    println!("  spatial2:    {:>8}us", sum.spatial2 / n);
    println!("  eating:      {:>8}us", sum.eating / n);
    println!("  combat:      {:>8}us", sum.combat / n);
    println!("  death:       {:>8}us", sum.death / n);
    println!("  ─────────────────────");
    let avg_total = sum.total() / n;
    let ok = if avg_total <= 10_000 { "OK" } else { "OVER" };
    println!(
        "  TOTAL:       {:>8}us  [{ok}]  (max: {}us)",
        avg_total, max_total
    );
}
