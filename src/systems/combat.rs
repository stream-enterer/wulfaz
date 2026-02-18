use crate::components::{ActionId, Entity, Tick};
use crate::events::Event;
use crate::systems::fatigue::UNCONSCIOUS_THRESHOLD;
use crate::world::World;
use rand::RngExt;

/// Fatigue gained per attack (flat cost, standing in for encumbrance).
const ATTACK_FATIGUE_COST: f32 = 1.0;

/// Compute fatigue-modified damage: effective_attack - effective_defense, min 1.0.
/// Fatigue degrades stats: -1 defense per 10 fatigue, -1 attack per 20 fatigue.
/// Unconscious defenders (fatigue >= 100) have 0 effective defense.
fn compute_fatigue_damage(world: &World, attacker: Entity, defender: Entity) -> f32 {
    let base_atk = world
        .combat_stats
        .get(&attacker)
        .map(|cs| cs.attack)
        .unwrap_or(0.0);
    let base_def = world
        .combat_stats
        .get(&defender)
        .map(|cs| cs.defense)
        .unwrap_or(0.0);
    let fatigue_a = world
        .fatigues
        .get(&attacker)
        .map(|f| f.current)
        .unwrap_or(0.0);
    let fatigue_d = world
        .fatigues
        .get(&defender)
        .map(|f| f.current)
        .unwrap_or(0.0);

    let eff_atk = (base_atk - fatigue_a / 20.0).max(0.0);
    let eff_def = if fatigue_d >= UNCONSCIOUS_THRESHOLD {
        0.0
    } else {
        (base_def - fatigue_d / 10.0).max(0.0)
    };
    (eff_atk - eff_def).max(1.0)
}

/// Phase 4 (Actions): Combat resolution with fatigue.
///
/// Finds entities with combat_stats, health, and position that share a tile
/// with another combatant. Unconscious entities (fatigue >= 100) cannot attack.
/// Fatigue degrades stats: -1 defense per 10, -1 attack per 20.
/// Each attack costs the attacker ATTACK_FATIGUE_COST fatigue.
/// Damage = effective_attack - effective_defense (min 1.0).
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

        // Unconscious entities cannot attack
        let attacker_fatigue = world
            .fatigues
            .get(&attacker)
            .map(|f| f.current)
            .unwrap_or(0.0);
        if attacker_fatigue >= UNCONSCIOUS_THRESHOLD {
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
            let damage = compute_fatigue_damage(world, attacker, defender);
            attacks.push((attacker, defender, damage));
            found_target = true;
        }

        if !found_target {
            for (j, &(defender, dx, dy, _)) in combatants.iter().enumerate() {
                if i == j {
                    continue;
                }
                if ax == dx && ay == dy {
                    let damage = compute_fatigue_damage(world, attacker, defender);
                    attacks.push((attacker, defender, damage));
                    break; // one attack per attacker per tick
                }
            }
        }
    }

    // Apply attacks
    for (attacker, defender, damage) in attacks {
        // Attacker gains fatigue from attacking
        if let Some(f) = world.fatigues.get_mut(&attacker) {
            f.current += ATTACK_FATIGUE_COST;
        }

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
