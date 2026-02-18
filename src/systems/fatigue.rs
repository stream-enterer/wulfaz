use crate::components::{Entity, Tick};
use crate::events::Event;
use crate::world::World;
use rand::RngExt;

/// Natural fatigue recovery per tick when below unconscious threshold.
const RECOVERY_RATE: f32 = 0.2; // 20/sec at 100 tps
/// Faster recovery per tick when at or above unconscious threshold.
const FAST_RECOVERY_RATE: f32 = 1.0; // 100/sec at 100 tps
/// Fatigue threshold at which a unit falls unconscious.
pub const UNCONSCIOUS_THRESHOLD: f32 = 100.0;
/// Fatigue threshold above which excess converts to HP damage.
const HP_DAMAGE_THRESHOLD: f32 = 200.0;

/// Phase 2 (Needs): Natural fatigue recovery and excess fatigue HP damage.
///
/// Reduces fatigue by RECOVERY_RATE per tick (FAST_RECOVERY_RATE if >= 100).
/// If fatigue exceeds 200, converts excess to HP damage: 1 per 50 excess,
/// with remainder having a (remainder*2)% chance of +1 more. Skips pending deaths.
pub fn run_fatigue(world: &mut World, tick: Tick) {
    // Collect recovery updates (can't borrow world.rng while iterating fatigues)
    let updates: Vec<(Entity, f32)> = world
        .fatigues
        .iter()
        .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
        .map(|(&e, f)| {
            let recovery = if f.current >= UNCONSCIOUS_THRESHOLD {
                FAST_RECOVERY_RATE
            } else {
                RECOVERY_RATE
            };
            let new_fatigue = (f.current - recovery).max(0.0);
            (e, new_fatigue)
        })
        .collect();

    // Apply recovery and check for HP damage from excess fatigue
    for (e, new_fatigue) in updates {
        if let Some(f) = world.fatigues.get_mut(&e) {
            f.current = new_fatigue;
        }

        // HP damage from excess fatigue (> 200)
        if new_fatigue > HP_DAMAGE_THRESHOLD {
            let excess = new_fatigue - HP_DAMAGE_THRESHOLD;
            let guaranteed = (excess / 50.0).floor();
            let remainder = excess % 50.0;
            let chance = remainder * 2.0 / 100.0;
            let roll: f32 = world.rng.random();
            let bonus = if roll < chance { 1.0 } else { 0.0 };
            let total_damage = guaranteed + bonus;

            if total_damage > 0.0
                && let Some(health) = world.healths.get_mut(&e)
            {
                health.current = (health.current - total_damage).max(0.0);
                if health.current <= 0.0 {
                    world.events.push(Event::Died { entity: e, tick });
                    world.pending_deaths.push(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::world::World;

    #[test]
    fn test_fatigue_recovers_naturally() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.fatigues.insert(e, Fatigue { current: 10.0 });

        run_fatigue(&mut world, Tick(0));
        assert!((world.fatigues[&e].current - 9.8).abs() < 0.001);
    }

    #[test]
    fn test_fatigue_clamps_to_zero() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.fatigues.insert(e, Fatigue { current: 0.1 });

        run_fatigue(&mut world, Tick(0));
        assert_eq!(world.fatigues[&e].current, 0.0);
    }

    #[test]
    fn test_fast_recovery_when_unconscious() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.fatigues.insert(e, Fatigue { current: 105.0 });

        run_fatigue(&mut world, Tick(0));
        assert!((world.fatigues[&e].current - 104.0).abs() < 0.001);
    }

    #[test]
    fn test_excess_fatigue_damages_hp() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        // 260 fatigue → excess 60 → guaranteed 1 HP damage (60/50 = 1.2 → floor 1)
        world.fatigues.insert(e, Fatigue { current: 260.0 });
        world.healths.insert(
            e,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );

        run_fatigue(&mut world, Tick(0));
        // Should have taken at least 1 HP damage
        assert!(world.healths[&e].current < 100.0);
    }

    #[test]
    fn test_excess_fatigue_can_kill() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        // Very high fatigue with very low health
        world.fatigues.insert(e, Fatigue { current: 500.0 });
        world.healths.insert(
            e,
            Health {
                current: 1.0,
                max: 100.0,
            },
        );

        run_fatigue(&mut world, Tick(0));
        assert!(world.pending_deaths.contains(&e));
    }

    #[test]
    fn test_no_hp_damage_below_threshold() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.fatigues.insert(e, Fatigue { current: 150.0 });
        world.healths.insert(
            e,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );

        run_fatigue(&mut world, Tick(0));
        assert_eq!(world.healths[&e].current, 100.0);
    }

    #[test]
    fn test_skips_pending_death() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.fatigues.insert(e, Fatigue { current: 50.0 });
        world.pending_deaths.push(e);

        run_fatigue(&mut world, Tick(0));
        assert_eq!(world.fatigues[&e].current, 50.0);
    }

    #[test]
    fn test_zero_fatigue_stays_zero() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.fatigues.insert(e, Fatigue { current: 0.0 });

        run_fatigue(&mut world, Tick(0));
        assert_eq!(world.fatigues[&e].current, 0.0);
    }
}
