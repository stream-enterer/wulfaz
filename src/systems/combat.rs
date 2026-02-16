use crate::components::{ActionId, Entity, Tick};
use crate::events::Event;
use crate::world::World;
use rand::RngExt;

/// Phase 4 (Actions): Combat resolution.
///
/// Finds entities with combat_stats, health, and position that share a tile
/// with another combatant. Aggressive entities (aggression > 0.5) attack if
/// an RNG check passes. Damage = attacker.attack - defender.defense (min 1.0).
/// Defender health is reduced; if it drops to 0 or below, a death event is
/// pushed and the defender is added to pending_deaths.
pub fn run_combat(world: &mut World, tick: Tick) {
    // Collect combatants with position, health, and combat_stats, sorted for determinism
    let mut combatants: Vec<(Entity, i32, i32, f32)> = world
        .combat_stats
        .iter()
        .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
        .filter_map(|(&e, cs)| {
            let pos = world.positions.get(&e)?;
            let _health = world.healths.get(&e)?;
            Some((e, pos.x, pos.y, cs.aggression))
        })
        .collect();
    combatants.sort_by_key(|(e, _, _, _)| e.0);

    // Find attack pairs: aggressive entity attacks another at same position
    let mut attacks: Vec<(Entity, Entity, f32)> = Vec::new(); // (attacker, defender, damage)

    for i in 0..combatants.len() {
        let (attacker, ax, ay, aggression) = combatants[i];

        // Gate on intention if present, else legacy fallback
        if let Some(intention) = world.intentions.get(&attacker) {
            if intention.action != ActionId::Attack {
                continue;
            }
        } else if aggression <= 0.5 {
            continue;
        }

        // RNG check: aggression is probability of attacking
        let roll: f32 = world.rng.random();
        if roll > aggression {
            continue;
        }

        // Prefer intention target if set and valid (same tile, alive)
        let preferred_target = world.intentions.get(&attacker).and_then(|i| i.target);

        let mut found_target = false;
        if let Some(target) = preferred_target
            && let Some(&(defender, dx, dy, _)) =
                combatants.iter().find(|(e, _, _, _)| *e == target)
            && defender != attacker
            && ax == dx
            && ay == dy
        {
            let atk = world
                .combat_stats
                .get(&attacker)
                .map(|cs| cs.attack)
                .unwrap_or(0.0);
            let def = world
                .combat_stats
                .get(&defender)
                .map(|cs| cs.defense)
                .unwrap_or(0.0);
            let damage = (atk - def).max(1.0);
            attacks.push((attacker, defender, damage));
            found_target = true;
        }

        if !found_target {
            for (j, &(defender, dx, dy, _)) in combatants.iter().enumerate() {
                if i == j {
                    continue;
                }
                if ax == dx && ay == dy {
                    let atk = world
                        .combat_stats
                        .get(&attacker)
                        .map(|cs| cs.attack)
                        .unwrap_or(0.0);
                    let def = world
                        .combat_stats
                        .get(&defender)
                        .map(|cs| cs.defense)
                        .unwrap_or(0.0);
                    let damage = (atk - def).max(1.0);
                    attacks.push((attacker, defender, damage));
                    break; // one attack per attacker per tick
                }
            }
        }
    }

    // Apply attacks
    for (attacker, defender, damage) in attacks {
        if let Some(health) = world.healths.get_mut(&defender) {
            health.current -= damage;
            health.current = health.current.clamp(0.0, health.max);

            world.events.push(Event::Attacked {
                attacker,
                defender,
                damage,
                tick,
            });

            if health.current <= 0.0 {
                // Lethal event: push AFTER the decision, BEFORE pending_deaths
                world.events.push(Event::Died {
                    entity: defender,
                    tick,
                });
                world.pending_deaths.push(defender);
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
    fn test_combat_damages_defender() {
        let mut world = World::new_with_seed(42);

        let attacker = world.spawn();
        world.positions.insert(attacker, Position { x: 5, y: 5 });
        world.healths.insert(
            attacker,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            attacker,
            CombatStats {
                attack: 15.0,
                defense: 5.0,
                aggression: 1.0,
            },
        );

        let defender = world.spawn();
        world.positions.insert(defender, Position { x: 5, y: 5 }); // same position
        world.healths.insert(
            defender,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            defender,
            CombatStats {
                attack: 5.0,
                defense: 3.0,
                aggression: 0.0,
            },
        );

        run_combat(&mut world, Tick(0));

        // Attacker has aggression 1.0 so always attacks. Damage = 15-3 = 12
        assert!(world.healths[&defender].current < 100.0);
    }

    #[test]
    fn test_combat_kills_defender() {
        let mut world = World::new_with_seed(42);

        let attacker = world.spawn();
        world.positions.insert(attacker, Position { x: 5, y: 5 });
        world.healths.insert(
            attacker,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            attacker,
            CombatStats {
                attack: 50.0,
                defense: 5.0,
                aggression: 1.0,
            },
        );

        let defender = world.spawn();
        world.positions.insert(defender, Position { x: 5, y: 5 });
        world.healths.insert(
            defender,
            Health {
                current: 5.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            defender,
            CombatStats {
                attack: 5.0,
                defense: 3.0,
                aggression: 0.0,
            },
        );

        run_combat(&mut world, Tick(0));

        assert!(world.pending_deaths.contains(&defender));
    }

    #[test]
    fn test_combat_clamps_health_to_zero() {
        let mut world = World::new_with_seed(42);

        let attacker = world.spawn();
        world.positions.insert(attacker, Position { x: 5, y: 5 });
        world.healths.insert(
            attacker,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            attacker,
            CombatStats {
                attack: 200.0,
                defense: 5.0,
                aggression: 1.0,
            },
        );

        let defender = world.spawn();
        world.positions.insert(defender, Position { x: 5, y: 5 });
        world.healths.insert(
            defender,
            Health {
                current: 3.0,
                max: 50.0,
            },
        );
        world.combat_stats.insert(
            defender,
            CombatStats {
                attack: 5.0,
                defense: 1.0,
                aggression: 0.0,
            },
        );

        run_combat(&mut world, Tick(0));

        // Health should be clamped to 0.0, not negative
        assert_eq!(world.healths[&defender].current, 0.0);
    }

    #[test]
    fn test_combat_skips_pending_death() {
        let mut world = World::new_with_seed(42);

        let attacker = world.spawn();
        world.positions.insert(attacker, Position { x: 5, y: 5 });
        world.healths.insert(
            attacker,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            attacker,
            CombatStats {
                attack: 15.0,
                defense: 5.0,
                aggression: 1.0,
            },
        );
        world.pending_deaths.push(attacker); // already dying

        let defender = world.spawn();
        world.positions.insert(defender, Position { x: 5, y: 5 });
        world.healths.insert(
            defender,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            defender,
            CombatStats {
                attack: 5.0,
                defense: 3.0,
                aggression: 0.0,
            },
        );

        run_combat(&mut world, Tick(0));

        assert_eq!(world.healths[&defender].current, 100.0); // undamaged
    }

    #[test]
    fn test_combat_different_positions_no_fight() {
        let mut world = World::new_with_seed(42);

        let attacker = world.spawn();
        world.positions.insert(attacker, Position { x: 5, y: 5 });
        world.healths.insert(
            attacker,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            attacker,
            CombatStats {
                attack: 15.0,
                defense: 5.0,
                aggression: 1.0,
            },
        );

        let defender = world.spawn();
        world.positions.insert(defender, Position { x: 10, y: 10 }); // different position
        world.healths.insert(
            defender,
            Health {
                current: 100.0,
                max: 100.0,
            },
        );
        world.combat_stats.insert(
            defender,
            CombatStats {
                attack: 5.0,
                defense: 3.0,
                aggression: 0.0,
            },
        );

        run_combat(&mut world, Tick(0));

        assert_eq!(world.healths[&defender].current, 100.0); // undamaged
    }
}
