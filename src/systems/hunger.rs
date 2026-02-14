use crate::components::Tick;
use crate::events::Event;
use crate::world::World;

/// Phase 2 (Needs): Hunger increases over time.
///
/// Every living entity with a Hunger component gets hungrier by 1.0 per tick,
/// clamped to hunger.max. Entities in pending_deaths are skipped.
pub fn run_hunger(world: &mut World, tick: Tick) {
    let mut changes: Vec<(crate::components::Entity, f32, f32)> = world
        .hungers
        .iter()
        .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
        .map(|(&e, h)| {
            let new_val = (h.current + 1.0).min(h.max);
            (e, h.current, new_val)
        })
        .collect();
    changes.sort_by_key(|(e, _, _)| e.0);

    for (e, old, new_val) in changes {
        if let Some(h) = world.hungers.get_mut(&e) {
            h.current = new_val;
            world.events.push(Event::HungerChanged {
                entity: e,
                old,
                new_val,
                tick,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Hunger, Tick};
    use crate::world::World;

    #[test]
    fn test_hunger_increases() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.hungers.insert(
            e,
            Hunger {
                current: 0.0,
                max: 100.0,
            },
        );
        run_hunger(&mut world, Tick(0));
        assert!(world.hungers[&e].current > 0.0);
    }

    #[test]
    fn test_hunger_clamped_at_max() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.hungers.insert(
            e,
            Hunger {
                current: 99.5,
                max: 100.0,
            },
        );
        run_hunger(&mut world, Tick(0));
        assert_eq!(world.hungers[&e].current, 100.0);
    }

    #[test]
    fn test_hunger_skips_pending_death() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.hungers.insert(
            e,
            Hunger {
                current: 50.0,
                max: 100.0,
            },
        );
        world.pending_deaths.push(e);
        run_hunger(&mut world, Tick(0));
        assert_eq!(world.hungers[&e].current, 50.0); // unchanged
    }
}
