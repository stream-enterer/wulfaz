use crate::components::{ActionId, Entity, Tick};
use crate::events::Event;
use crate::world::World;

/// Pickup range: same tile = within 1 meter.
pub fn run_eating(world: &mut World, tick: Tick) {
    // Collect hungry entities and their positions, sorted for determinism
    let mut hungry: Vec<(Entity, i32, i32, f32)> = world
        .mind
        .hungers
        .iter()
        .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
        .filter(|&(&e, h)| {
            if let Some(intention) = world.mind.intentions.get(&e) {
                intention.action == ActionId::Eat
            } else {
                h.current > h.max * 0.5 // legacy fallback
            }
        })
        .filter_map(|(&e, _)| {
            let pos = world.body.positions.get(&e)?;
            Some((e, pos.x, pos.y, 0.0))
        })
        .collect();
    hungry.sort_by_key(|(e, _, _, _)| e.0);

    // Collect food items sorted for determinism
    let mut food_items: Vec<(Entity, f32, i32, i32)> = world
        .mind
        .nutritions
        .iter()
        .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
        .filter_map(|(&e, n)| {
            let pos = world.body.positions.get(&e)?;
            Some((e, n.value, pos.x, pos.y))
        })
        .collect();
    food_items.sort_by_key(|(e, _, _, _)| e.0);

    // Find food items at same positions
    let mut eat_actions: Vec<(Entity, Entity, f32)> = Vec::new(); // (eater, food, nutrition)
    let mut consumed: Vec<Entity> = Vec::new();

    for (eater, ex, ey, _) in &hungry {
        // Prefer intention target if set and valid
        if let Some(target) = world.mind.intentions.get(eater).and_then(|i| i.target)
            && let Some(&(_, nutrition_value, fx, fy)) =
                food_items.iter().find(|(e, _, _, _)| *e == target)
            && !consumed.contains(&target)
            && fx == *ex
            && fy == *ey
            && nutrition_value > 0.0
        {
            eat_actions.push((*eater, target, nutrition_value));
            consumed.push(target);
            continue;
        }
        // Fallback: first food at same position
        for &(food_entity, nutrition_value, fx, fy) in &food_items {
            if consumed.contains(&food_entity) {
                continue;
            }
            if fx == *ex && fy == *ey && nutrition_value > 0.0 {
                eat_actions.push((*eater, food_entity, nutrition_value));
                consumed.push(food_entity);
                break; // one food per eater per tick
            }
        }
    }

    // Apply eating
    for (eater, food, nutrition_value) in eat_actions {
        if let Some(hunger) = world.mind.hungers.get_mut(&eater) {
            hunger.current = (hunger.current - nutrition_value).max(0.0);
        }

        // Push event BEFORE pending_deaths (per ADD-003 rule for lethal events)
        world.events.push(Event::Ate {
            entity: eater,
            food,
            tick,
        });
        world.events.push(Event::Died { entity: food, tick });
        world.pending_deaths.push(food);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::world::World;

    #[test]
    fn test_eating_reduces_hunger() {
        let mut world = World::new_with_seed(42);
        let eater = world.spawn();
        world.body.positions.insert(eater, Position { x: 5, y: 5 });
        world.mind.hungers.insert(
            eater,
            Hunger {
                current: 80.0,
                max: 100.0,
            },
        );

        let food = world.spawn();
        world.body.positions.insert(food, Position { x: 5, y: 5 }); // same position
        world
            .mind
            .nutritions
            .insert(food, Nutrition { value: 30.0 });

        run_eating(&mut world, Tick(0));

        assert_eq!(world.mind.hungers[&eater].current, 50.0);
        assert!(world.pending_deaths.contains(&food));
    }

    #[test]
    fn test_not_hungry_enough_doesnt_eat() {
        let mut world = World::new_with_seed(42);
        let eater = world.spawn();
        world.body.positions.insert(eater, Position { x: 5, y: 5 });
        world.mind.hungers.insert(
            eater,
            Hunger {
                current: 30.0,
                max: 100.0,
            },
        ); // not hungry enough

        let food = world.spawn();
        world.body.positions.insert(food, Position { x: 5, y: 5 });
        world
            .mind
            .nutritions
            .insert(food, Nutrition { value: 30.0 });

        run_eating(&mut world, Tick(0));

        assert_eq!(world.mind.hungers[&eater].current, 30.0); // unchanged
        assert!(!world.pending_deaths.contains(&food));
    }

    #[test]
    fn test_eating_skips_pending_death() {
        let mut world = World::new_with_seed(42);
        let eater = world.spawn();
        world.body.positions.insert(eater, Position { x: 5, y: 5 });
        world.mind.hungers.insert(
            eater,
            Hunger {
                current: 80.0,
                max: 100.0,
            },
        );
        world.pending_deaths.push(eater);

        let food = world.spawn();
        world.body.positions.insert(food, Position { x: 5, y: 5 });
        world
            .mind
            .nutritions
            .insert(food, Nutrition { value: 30.0 });

        run_eating(&mut world, Tick(0));

        assert_eq!(world.mind.hungers[&eater].current, 80.0); // unchanged
    }

    #[test]
    fn test_different_positions_no_eating() {
        let mut world = World::new_with_seed(42);
        let eater = world.spawn();
        world.body.positions.insert(eater, Position { x: 5, y: 5 });
        world.mind.hungers.insert(
            eater,
            Hunger {
                current: 80.0,
                max: 100.0,
            },
        );

        let food = world.spawn();
        world.body.positions.insert(food, Position { x: 10, y: 10 }); // different position
        world
            .mind
            .nutritions
            .insert(food, Nutrition { value: 30.0 });

        run_eating(&mut world, Tick(0));

        assert_eq!(world.mind.hungers[&eater].current, 80.0); // unchanged
    }
}
