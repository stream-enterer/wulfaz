use crate::components::{DecisionCooldown, Entity, ForageTarget, Tick};
use crate::world::World;

/// Hunger threshold above which a creature seeks food (0–100 scale).
const HUNGER_THRESHOLD: f32 = 70.0;

/// Ticks between forage re-evaluations.
const DECISION_COOLDOWN_RESET: u32 = 10;

/// Phase 3 system: hungry creatures decide to walk toward the nearest visible
/// food item. Writes a `ForageTarget` intention. Respects `decision_cooldown`
/// to avoid re-evaluating every tick.
pub fn run_forage_decision(world: &mut World, _tick: Tick) {
    // Drive off hungers table — only entities with hunger can forage.
    let mut entities: Vec<Entity> = world
        .mind
        .hungers
        .keys()
        .filter(|e| !world.pending_deaths.contains(e))
        .filter(|e| world.player != Some(**e))
        .copied()
        .collect();
    entities.sort_by_key(|e| e.0);

    // Collect cooldown decrements and decisions, then apply.
    struct Decision {
        entity: Entity,
        target: Option<ForageTarget>,
        new_cooldown: Option<u32>,
    }

    let mut decisions: Vec<Decision> = Vec::new();

    for &entity in &entities {
        // Decrement cooldown. Skip if still on cooldown after decrement.
        if let Some(cd) = world.mind.decision_cooldowns.get(&entity)
            && cd.remaining > 1
        {
            decisions.push(Decision {
                entity,
                target: None,
                new_cooldown: Some(cd.remaining - 1),
            });
            continue;
            // remaining is 0 or 1 → falls through to re-evaluate
        }

        // Check hunger threshold.
        let Some(hunger) = world.mind.hungers.get(&entity) else {
            continue;
        };
        if hunger.current < HUNGER_THRESHOLD {
            // Not hungry enough — clear any stale forage target.
            decisions.push(Decision {
                entity,
                target: None,
                new_cooldown: Some(DECISION_COOLDOWN_RESET),
            });
            continue;
        }

        // If we already have a forage target that is alive and we have a
        // cached path to it, keep the existing intention.
        if let Some(existing) = world.mind.forage_targets.get(&entity) {
            let target_alive =
                world.alive.contains(&existing.0) && !world.pending_deaths.contains(&existing.0);
            let has_cached_path = world.mind.cached_paths.contains_key(&entity);
            if target_alive && has_cached_path {
                decisions.push(Decision {
                    entity,
                    target: Some(*existing),
                    new_cooldown: Some(DECISION_COOLDOWN_RESET),
                });
                continue;
            }
        }

        // Find nearest visible food item.
        let Some(pos) = world.body.positions.get(&entity) else {
            continue;
        };
        let Some(pr) = world.mind.perception_ranges.get(&entity) else {
            continue;
        };

        let best_food = world
            .entities_in_range(pos.x, pos.y, pr.range)
            .filter(|&e| e != entity)
            .filter(|e| !world.pending_deaths.contains(e))
            .filter_map(|e| {
                let n = world.mind.nutritions.get(&e)?;
                let fp = world.body.positions.get(&e)?;
                let dist = (fp.x - pos.x).abs().max((fp.y - pos.y).abs());
                Some((e, dist, n.value))
            })
            .min_by(|a, b| {
                // Nearest first, then highest nutrition, then lowest entity ID.
                a.1.cmp(&b.1)
                    .then_with(|| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal))
                    .then_with(|| a.0.0.cmp(&b.0.0))
            })
            .map(|(e, _, _)| e);

        let target = best_food.map(ForageTarget);

        decisions.push(Decision {
            entity,
            target,
            new_cooldown: Some(DECISION_COOLDOWN_RESET),
        });
    }

    // Apply all decisions.
    for d in decisions {
        if let Some(cd) = d.new_cooldown {
            world
                .mind
                .decision_cooldowns
                .insert(d.entity, DecisionCooldown { remaining: cd });
        }
        match d.target {
            Some(ft) => {
                world.mind.forage_targets.insert(d.entity, ft);
            }
            None => {
                world.mind.forage_targets.remove(&d.entity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::world::World;

    fn spawn_creature(world: &mut World, x: i32, y: i32, hunger: f32) -> Entity {
        let e = world.spawn();
        world.body.positions.insert(e, Position { x, y });
        world.mind.hungers.insert(
            e,
            Hunger {
                current: hunger,
                max: 100.0,
            },
        );
        world
            .mind
            .perception_ranges
            .insert(e, PerceptionRange { range: 20 });
        e
    }

    fn spawn_food(world: &mut World, x: i32, y: i32, nutrition: f32) -> Entity {
        let e = world.spawn();
        world.body.positions.insert(e, Position { x, y });
        world
            .mind
            .nutritions
            .insert(e, Nutrition { value: nutrition });
        e
    }

    #[test]
    fn hungry_creature_targets_nearest_food() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 80.0);
        let _far_food = spawn_food(&mut world, 15, 15, 50.0);
        let near_food = spawn_food(&mut world, 6, 5, 10.0);

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        let ft = world
            .mind
            .forage_targets
            .get(&creature)
            .expect("should have forage target");
        assert_eq!(ft.0, near_food);
    }

    #[test]
    fn not_hungry_creature_does_not_forage() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 30.0);
        spawn_food(&mut world, 6, 5, 10.0);

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        assert!(!world.mind.forage_targets.contains_key(&creature));
    }

    #[test]
    fn food_outside_perception_ignored() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 80.0);
        // Place food beyond perception range (20 tiles).
        spawn_food(&mut world, 50, 50, 10.0);

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        assert!(!world.mind.forage_targets.contains_key(&creature));
    }

    #[test]
    fn keeps_existing_target_with_cached_path() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 80.0);
        let food1 = spawn_food(&mut world, 10, 5, 10.0);
        // Closer food added later — should be ignored because existing target is cached.
        let _food2 = spawn_food(&mut world, 6, 5, 20.0);

        // Pre-set forage target and cached path to food1.
        world
            .mind
            .forage_targets
            .insert(creature, ForageTarget(food1));
        world.mind.cached_paths.insert(
            creature,
            CachedPath {
                steps: vec![(6, 5), (7, 5)],
                goal: (10, 5),
            },
        );

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        let ft = world
            .mind
            .forage_targets
            .get(&creature)
            .expect("should keep target");
        assert_eq!(ft.0, food1);
    }

    #[test]
    fn re_evaluates_when_target_dead() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 80.0);
        let food1 = spawn_food(&mut world, 10, 5, 10.0);
        let food2 = spawn_food(&mut world, 7, 5, 20.0);

        // Pre-set forage target to food1 with cached path.
        world
            .mind
            .forage_targets
            .insert(creature, ForageTarget(food1));
        world.mind.cached_paths.insert(
            creature,
            CachedPath {
                steps: vec![(6, 5)],
                goal: (10, 5),
            },
        );

        // Kill food1.
        world.pending_deaths.insert(food1);

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        let ft = world
            .mind
            .forage_targets
            .get(&creature)
            .expect("should pick new target");
        assert_eq!(ft.0, food2);
    }

    #[test]
    fn decision_cooldown_prevents_re_evaluation() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 80.0);
        spawn_food(&mut world, 6, 5, 10.0);

        // Set cooldown to 5 — should not evaluate this tick.
        world
            .mind
            .decision_cooldowns
            .insert(creature, DecisionCooldown { remaining: 5 });

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        // No forage target set (cooldown blocked evaluation).
        assert!(!world.mind.forage_targets.contains_key(&creature));
        // Cooldown decremented.
        let cd = world.mind.decision_cooldowns.get(&creature).unwrap();
        assert_eq!(cd.remaining, 4);
    }

    #[test]
    fn cooldown_expires_allows_evaluation() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 80.0);
        let food = spawn_food(&mut world, 6, 5, 10.0);

        // Cooldown at 1 — will expire, allowing evaluation this tick.
        world
            .mind
            .decision_cooldowns
            .insert(creature, DecisionCooldown { remaining: 1 });

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        let ft = world
            .mind
            .forage_targets
            .get(&creature)
            .expect("should forage");
        assert_eq!(ft.0, food);
        // Cooldown reset.
        let cd = world.mind.decision_cooldowns.get(&creature).unwrap();
        assert_eq!(cd.remaining, DECISION_COOLDOWN_RESET);
    }

    #[test]
    fn skips_pending_deaths() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 80.0);
        spawn_food(&mut world, 6, 5, 10.0);

        world.pending_deaths.insert(creature);

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        assert!(!world.mind.forage_targets.contains_key(&creature));
    }

    #[test]
    fn empty_world_no_panic() {
        let mut world = World::new_with_seed(42);
        run_forage_decision(&mut world, Tick(0));
    }

    #[test]
    fn prefers_higher_nutrition_at_same_distance() {
        let mut world = World::new_with_seed(42);
        let creature = spawn_creature(&mut world, 5, 5, 80.0);
        let _low = spawn_food(&mut world, 6, 5, 5.0);
        let high = spawn_food(&mut world, 4, 5, 50.0);

        world.rebuild_spatial_index();
        run_forage_decision(&mut world, Tick(0));

        let ft = world
            .mind
            .forage_targets
            .get(&creature)
            .expect("should forage");
        assert_eq!(ft.0, high);
    }
}
