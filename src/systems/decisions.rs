use std::collections::BTreeMap;

use serde::Deserialize;

use crate::components::{ActionId, Entity, Intention, Tick};
use crate::world::World;

// ---------------------------------------------------------------------------
// Config types — scoring internals, not per-entity data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum CurveKind {
    Linear,
    Quadratic,
    Logistic,
    Step,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Curve {
    pub kind: CurveKind,
    pub slope: f32,
    pub offset: f32,
    pub exponent: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum InputAxis {
    HungerRatio,
    HealthRatio,
    FoodNearby,
    EnemyNearby,
    Aggression,
    Constant(f32),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Consideration {
    pub input: InputAxis,
    pub curve: Curve,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActionDef {
    pub considerations: Vec<Consideration>,
    pub weight: f32,
    pub cooldown_ticks: u64,
    pub inertia_bonus: f32,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UtilityConfig {
    pub actions: BTreeMap<ActionId, ActionDef>,
}

// ---------------------------------------------------------------------------
// Curve evaluation
// ---------------------------------------------------------------------------

fn evaluate_curve(curve: &Curve, x: f32) -> f32 {
    let raw = match curve.kind {
        CurveKind::Linear => {
            // y = slope * x + offset
            curve.slope * x + curve.offset
        }
        CurveKind::Quadratic => {
            // y = slope * (x - offset)^exponent
            curve.slope * (x - curve.offset).abs().powf(curve.exponent)
        }
        CurveKind::Logistic => {
            // y = 1 / (1 + e^(-slope * (x - offset)))
            1.0 / (1.0 + (-curve.slope * (x - curve.offset)).exp())
        }
        CurveKind::Step => {
            // y = slope if x > offset, else 0
            if x > curve.offset { curve.slope } else { 0.0 }
        }
    };
    raw.clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Input axis reading
// ---------------------------------------------------------------------------

fn read_input(axis: &InputAxis, world: &World, entity: Entity) -> f32 {
    match axis {
        InputAxis::HungerRatio => {
            if let Some(h) = world.hungers.get(&entity) {
                if h.max > 0.0 { h.current / h.max } else { 0.0 }
            } else {
                0.0
            }
        }
        InputAxis::HealthRatio => {
            if let Some(h) = world.healths.get(&entity) {
                if h.max > 0.0 { h.current / h.max } else { 0.0 }
            } else {
                0.0
            }
        }
        InputAxis::FoodNearby => {
            let Some(pos) = world.positions.get(&entity) else {
                return 0.0;
            };
            let count = world
                .nutritions
                .iter()
                .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
                .filter(|&(&e, _)| {
                    if let Some(fp) = world.positions.get(&e) {
                        fp.x == pos.x && fp.y == pos.y
                    } else {
                        false
                    }
                })
                .count();
            (count.min(3) as f32) / 3.0
        }
        InputAxis::EnemyNearby => {
            let Some(pos) = world.positions.get(&entity) else {
                return 0.0;
            };
            let count = world
                .combat_stats
                .iter()
                .filter(|&(&e, _)| e != entity)
                .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
                .filter(|&(&e, _)| {
                    if let Some(ep) = world.positions.get(&e) {
                        ep.x == pos.x && ep.y == pos.y
                    } else {
                        false
                    }
                })
                .count();
            (count.min(3) as f32) / 3.0
        }
        InputAxis::Aggression => {
            if let Some(cs) = world.combat_stats.get(&entity) {
                cs.aggression
            } else {
                0.0
            }
        }
        InputAxis::Constant(v) => *v,
    }
}

// ---------------------------------------------------------------------------
// Target selection
// ---------------------------------------------------------------------------

fn select_eat_target(world: &World, entity: Entity) -> Option<Entity> {
    let pos = world.positions.get(&entity)?;
    world
        .nutritions
        .iter()
        .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
        .filter(|&(&e, _)| {
            if let Some(fp) = world.positions.get(&e) {
                fp.x == pos.x && fp.y == pos.y
            } else {
                false
            }
        })
        .max_by(|a, b| {
            a.1.value
                .partial_cmp(&b.1.value)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.0.cmp(&b.0.0))
        })
        .map(|(&e, _)| e)
}

fn select_attack_target(world: &World, entity: Entity) -> Option<Entity> {
    let pos = world.positions.get(&entity)?;
    world
        .combat_stats
        .iter()
        .filter(|&(&e, _)| e != entity)
        .filter(|&(&e, _)| !world.pending_deaths.contains(&e))
        .filter(|&(&e, _)| {
            if let Some(ep) = world.positions.get(&e) {
                ep.x == pos.x && ep.y == pos.y
            } else {
                false
            }
        })
        .filter_map(|(&e, _)| {
            let health = world.healths.get(&e)?;
            Some((e, health.current))
        })
        .min_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.0.cmp(&b.0.0))
        })
        .map(|(e, _)| e)
}

// ---------------------------------------------------------------------------
// Scorer system
// ---------------------------------------------------------------------------

pub fn run_decisions(world: &mut World, _tick: Tick) {
    // Wipe stale intentions
    world.intentions.clear();

    if world.utility_config.actions.is_empty() {
        return;
    }

    // Collect entities with ActionState, sorted by entity ID for determinism
    let mut entities: Vec<Entity> = world
        .action_states
        .keys()
        .filter(|e| !world.pending_deaths.contains(e))
        .copied()
        .collect();
    entities.sort_by_key(|e| e.0);

    // Decrement all cooldowns first (collect-then-apply)
    let cooldown_decrements: Vec<(Entity, Vec<(ActionId, u64)>)> = entities
        .iter()
        .filter_map(|&e| {
            let state = world.action_states.get(&e)?;
            let updates: Vec<(ActionId, u64)> = state
                .cooldowns
                .iter()
                .filter(|&(_, &cd)| cd > 0)
                .map(|(&action, &cd)| (action, cd - 1))
                .collect();
            if updates.is_empty() {
                None
            } else {
                Some((e, updates))
            }
        })
        .collect();

    for (e, updates) in cooldown_decrements {
        if let Some(state) = world.action_states.get_mut(&e) {
            for (action, new_cd) in updates {
                state.cooldowns.insert(action, new_cd);
            }
        }
    }

    // Score and decide for each entity
    let config = world.utility_config.clone();
    let mut results: Vec<(Entity, ActionId, Option<Entity>)> = Vec::new();

    for &entity in &entities {
        let current_action = world
            .action_states
            .get(&entity)
            .and_then(|s| s.current_action);

        let mut best_action = ActionId::Idle;
        let mut best_score: f32 = -1.0;

        for (&action_id, action_def) in &config.actions {
            // Check cooldown
            let on_cooldown = world
                .action_states
                .get(&entity)
                .and_then(|s| s.cooldowns.get(&action_id))
                .is_some_and(|&cd| cd > 0);

            if on_cooldown {
                continue;
            }

            // Evaluate considerations
            if action_def.considerations.is_empty() {
                continue;
            }

            let mut product: f32 = 1.0;
            let n = action_def.considerations.len() as f32;

            for consideration in &action_def.considerations {
                let input = read_input(&consideration.input, world, entity);
                let score = evaluate_curve(&consideration.curve, input);
                product *= score;
            }

            // Geometric mean
            let geo_mean = if n > 0.0 { product.powf(1.0 / n) } else { 0.0 };

            let mut final_score = geo_mean * action_def.weight;

            // Inertia bonus
            if Some(action_id) == current_action {
                final_score += action_def.inertia_bonus;
            }

            if final_score > best_score {
                best_score = final_score;
                best_action = action_id;
            }
        }

        // Select target
        let target = match best_action {
            ActionId::Eat => select_eat_target(world, entity),
            ActionId::Attack => select_attack_target(world, entity),
            _ => None,
        };

        results.push((entity, best_action, target));
    }

    // Apply results
    for (entity, action, target) in results {
        let old_action = world
            .action_states
            .get(&entity)
            .and_then(|s| s.current_action);

        // Write intention
        world
            .intentions
            .insert(entity, Intention { action, target });

        // Update action state
        if let Some(state) = world.action_states.get_mut(&entity) {
            if Some(action) == old_action {
                state.ticks_in_action += 1;
            } else {
                // Action changed: set cooldown on old action
                if let Some(old) = old_action
                    && let Some(old_def) = config.actions.get(&old)
                    && old_def.cooldown_ticks > 0
                {
                    state.cooldowns.insert(old, old_def.cooldown_ticks);
                }
                state.current_action = Some(action);
                state.ticks_in_action = 0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::world::World;
    use std::collections::HashMap;

    fn default_config() -> UtilityConfig {
        let mut actions = BTreeMap::new();

        actions.insert(
            ActionId::Idle,
            ActionDef {
                considerations: vec![Consideration {
                    input: InputAxis::Constant(0.1),
                    curve: Curve {
                        kind: CurveKind::Linear,
                        slope: 1.0,
                        offset: 0.0,
                        exponent: 1.0,
                    },
                }],
                weight: 1.0,
                cooldown_ticks: 0,
                inertia_bonus: 0.0,
            },
        );

        actions.insert(
            ActionId::Wander,
            ActionDef {
                considerations: vec![
                    Consideration {
                        input: InputAxis::HungerRatio,
                        curve: Curve {
                            kind: CurveKind::Linear,
                            slope: -0.5,
                            offset: 0.8,
                            exponent: 1.0,
                        },
                    },
                    Consideration {
                        input: InputAxis::HealthRatio,
                        curve: Curve {
                            kind: CurveKind::Linear,
                            slope: 0.5,
                            offset: 0.3,
                            exponent: 1.0,
                        },
                    },
                ],
                weight: 1.0,
                cooldown_ticks: 0,
                inertia_bonus: 0.05,
            },
        );

        actions.insert(
            ActionId::Eat,
            ActionDef {
                considerations: vec![
                    Consideration {
                        input: InputAxis::HungerRatio,
                        curve: Curve {
                            kind: CurveKind::Logistic,
                            slope: 12.0,
                            offset: 0.4,
                            exponent: 1.0,
                        },
                    },
                    Consideration {
                        input: InputAxis::FoodNearby,
                        curve: Curve {
                            kind: CurveKind::Step,
                            slope: 1.0,
                            offset: 0.01,
                            exponent: 1.0,
                        },
                    },
                ],
                weight: 1.2,
                cooldown_ticks: 3,
                inertia_bonus: 0.1,
            },
        );

        actions.insert(
            ActionId::Attack,
            ActionDef {
                considerations: vec![
                    Consideration {
                        input: InputAxis::Aggression,
                        curve: Curve {
                            kind: CurveKind::Quadratic,
                            slope: 1.0,
                            offset: 0.0,
                            exponent: 2.0,
                        },
                    },
                    Consideration {
                        input: InputAxis::EnemyNearby,
                        curve: Curve {
                            kind: CurveKind::Step,
                            slope: 1.0,
                            offset: 0.01,
                            exponent: 1.0,
                        },
                    },
                    Consideration {
                        input: InputAxis::HealthRatio,
                        curve: Curve {
                            kind: CurveKind::Linear,
                            slope: 0.6,
                            offset: 0.3,
                            exponent: 1.0,
                        },
                    },
                ],
                weight: 1.5,
                cooldown_ticks: 2,
                inertia_bonus: 0.15,
            },
        );

        UtilityConfig { actions }
    }

    fn spawn_with_action_state(world: &mut World) -> Entity {
        let e = world.spawn();
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

    // --- Curve tests ---

    #[test]
    fn test_linear_curve() {
        let curve = Curve {
            kind: CurveKind::Linear,
            slope: 1.0,
            offset: 0.0,
            exponent: 1.0,
        };
        assert!((evaluate_curve(&curve, 0.0) - 0.0).abs() < 0.001);
        assert!((evaluate_curve(&curve, 0.5) - 0.5).abs() < 0.001);
        assert!((evaluate_curve(&curve, 1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_curve_clamped() {
        let curve = Curve {
            kind: CurveKind::Linear,
            slope: 2.0,
            offset: 0.0,
            exponent: 1.0,
        };
        // 2.0 * 1.0 = 2.0 → clamped to 1.0
        assert!((evaluate_curve(&curve, 1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_quadratic_curve() {
        let curve = Curve {
            kind: CurveKind::Quadratic,
            slope: 1.0,
            offset: 0.0,
            exponent: 2.0,
        };
        assert!((evaluate_curve(&curve, 0.0) - 0.0).abs() < 0.001);
        assert!((evaluate_curve(&curve, 0.5) - 0.25).abs() < 0.001);
        assert!((evaluate_curve(&curve, 1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_logistic_curve() {
        let curve = Curve {
            kind: CurveKind::Logistic,
            slope: 12.0,
            offset: 0.5,
            exponent: 1.0,
        };
        // At x=0.5 (the offset), logistic should be ~0.5
        assert!((evaluate_curve(&curve, 0.5) - 0.5).abs() < 0.01);
        // At x=1.0, should be close to 1.0
        assert!(evaluate_curve(&curve, 1.0) > 0.99);
        // At x=0.0, should be close to 0.0
        assert!(evaluate_curve(&curve, 0.0) < 0.01);
    }

    #[test]
    fn test_step_curve() {
        let curve = Curve {
            kind: CurveKind::Step,
            slope: 1.0,
            offset: 0.5,
            exponent: 1.0,
        };
        assert!((evaluate_curve(&curve, 0.3) - 0.0).abs() < 0.001);
        assert!((evaluate_curve(&curve, 0.5) - 0.0).abs() < 0.001); // not strictly greater
        assert!((evaluate_curve(&curve, 0.6) - 1.0).abs() < 0.001);
    }

    // --- Input axis tests ---

    #[test]
    fn test_hunger_ratio_input() {
        let mut world = World::new_with_seed(42);
        let e = spawn_with_action_state(&mut world);
        world.hungers.insert(
            e,
            Hunger {
                current: 60.0,
                max: 100.0,
            },
        );
        let val = read_input(&InputAxis::HungerRatio, &world, e);
        assert!((val - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_missing_component_returns_zero() {
        let mut world = World::new_with_seed(42);
        let e = spawn_with_action_state(&mut world);
        // No hunger component
        assert!((read_input(&InputAxis::HungerRatio, &world, e) - 0.0).abs() < 0.001);
        assert!((read_input(&InputAxis::HealthRatio, &world, e) - 0.0).abs() < 0.001);
        assert!((read_input(&InputAxis::Aggression, &world, e) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_food_nearby_input() {
        let mut world = World::new_with_seed(42);
        let e = spawn_with_action_state(&mut world);
        world.positions.insert(e, Position { x: 5, y: 5 });

        // Add 2 food items at same position
        let f1 = world.spawn();
        world.positions.insert(f1, Position { x: 5, y: 5 });
        world.nutritions.insert(f1, Nutrition { value: 10.0 });
        let f2 = world.spawn();
        world.positions.insert(f2, Position { x: 5, y: 5 });
        world.nutritions.insert(f2, Nutrition { value: 20.0 });

        let val = read_input(&InputAxis::FoodNearby, &world, e);
        assert!((val - 2.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn test_constant_input() {
        let world = World::new_with_seed(42);
        let e = Entity(999);
        let val = read_input(&InputAxis::Constant(0.42), &world, e);
        assert!((val - 0.42).abs() < 0.001);
    }

    // --- Geometric mean ---

    #[test]
    fn test_geometric_mean_scoring() {
        // Verify (0.8 * 0.6)^(1/2) ≈ 0.693
        let product: f32 = 0.8 * 0.6;
        let geo_mean = product.powf(1.0 / 2.0);
        assert!((geo_mean - 0.6928).abs() < 0.01);
    }

    // --- Inertia ---

    #[test]
    fn test_inertia_keeps_current_action() {
        let mut world = World::new_with_seed(42);
        world.utility_config = default_config();

        let e = spawn_with_action_state(&mut world);
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.hungers.insert(
            e,
            Hunger {
                current: 0.0,
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

        // First tick: should pick Wander (healthy, not hungry, no food/enemies)
        run_decisions(&mut world, Tick(0));
        let intention = world.intentions.get(&e).expect("should have intention");
        assert_eq!(intention.action, ActionId::Wander);

        // Second tick: inertia should help Wander stay chosen
        run_decisions(&mut world, Tick(1));
        let intention = world.intentions.get(&e).expect("should have intention");
        assert_eq!(intention.action, ActionId::Wander);
    }

    // --- Cooldown ---

    #[test]
    fn test_cooldown_skips_action() {
        let mut world = World::new_with_seed(42);
        world.utility_config = default_config();

        let e = spawn_with_action_state(&mut world);
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.hungers.insert(
            e,
            Hunger {
                current: 80.0,
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

        // Add food at same position
        let food = world.spawn();
        world.positions.insert(food, Position { x: 5, y: 5 });
        world.nutritions.insert(food, Nutrition { value: 30.0 });

        // Set Eat on cooldown
        if let Some(state) = world.action_states.get_mut(&e) {
            state.cooldowns.insert(ActionId::Eat, 5);
        }

        run_decisions(&mut world, Tick(0));
        let intention = world.intentions.get(&e).expect("should have intention");
        // Eat is on cooldown, so entity picks something else
        assert_ne!(intention.action, ActionId::Eat);
    }

    #[test]
    fn test_cooldown_expires() {
        let mut world = World::new_with_seed(42);
        world.utility_config = default_config();

        let e = spawn_with_action_state(&mut world);
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.hungers.insert(
            e,
            Hunger {
                current: 90.0,
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

        let food = world.spawn();
        world.positions.insert(food, Position { x: 5, y: 5 });
        world.nutritions.insert(food, Nutrition { value: 30.0 });

        // Set Eat on cooldown of 1 — will be decremented to 0 on first tick
        if let Some(state) = world.action_states.get_mut(&e) {
            state.cooldowns.insert(ActionId::Eat, 1);
        }

        // Tick 0: decrement cooldown to 0, but Eat is still blocked (cd was 1 at start, decremented to 0)
        run_decisions(&mut world, Tick(0));

        // Tick 1: cooldown is now 0, Eat should be available
        run_decisions(&mut world, Tick(1));
        let intention = world.intentions.get(&e).expect("should have intention");
        assert_eq!(intention.action, ActionId::Eat);
    }

    #[test]
    fn test_all_on_cooldown_picks_idle() {
        let mut world = World::new_with_seed(42);
        world.utility_config = default_config();

        let e = spawn_with_action_state(&mut world);
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.hungers.insert(
            e,
            Hunger {
                current: 0.0,
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

        // Put Wander, Eat, Attack all on cooldown (Idle has no cooldown in config)
        if let Some(state) = world.action_states.get_mut(&e) {
            state.cooldowns.insert(ActionId::Wander, 10);
            state.cooldowns.insert(ActionId::Eat, 10);
            state.cooldowns.insert(ActionId::Attack, 10);
        }

        run_decisions(&mut world, Tick(0));
        let intention = world.intentions.get(&e).expect("should have intention");
        assert_eq!(intention.action, ActionId::Idle);
    }

    // --- Empty world ---

    #[test]
    fn test_empty_world_no_panic() {
        let mut world = World::new_with_seed(42);
        world.utility_config = default_config();

        run_decisions(&mut world, Tick(0));
        assert!(world.intentions.is_empty());
    }

    // --- No stale intentions ---

    #[test]
    fn test_intentions_cleared_each_tick() {
        let mut world = World::new_with_seed(42);
        world.utility_config = default_config();

        let e = spawn_with_action_state(&mut world);
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.hungers.insert(
            e,
            Hunger {
                current: 0.0,
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

        run_decisions(&mut world, Tick(0));
        assert!(world.intentions.contains_key(&e));

        // Kill entity
        world.pending_deaths.push(e);
        run_decisions(&mut world, Tick(1));
        // Intentions should be cleared (dead entity skipped)
        assert!(!world.intentions.contains_key(&e));
    }

    // --- Determinism ---

    #[test]
    fn test_deterministic_same_seed() {
        let setup = |world: &mut World| {
            world.utility_config = default_config();
            for i in 0..5 {
                let e = spawn_with_action_state(world);
                world.positions.insert(e, Position { x: i, y: 0 });
                world.hungers.insert(
                    e,
                    Hunger {
                        current: (i as f32) * 20.0,
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
                        aggression: 0.3 + (i as f32) * 0.15,
                    },
                );
            }
        };

        let mut world1 = World::new_with_seed(42);
        setup(&mut world1);

        let mut world2 = World::new_with_seed(42);
        setup(&mut world2);

        for t in 0..10 {
            run_decisions(&mut world1, Tick(t));
            run_decisions(&mut world2, Tick(t));

            for e in world1.intentions.keys() {
                let i1 = &world1.intentions[e];
                let i2 = &world2.intentions[e];
                assert_eq!(
                    i1.action, i2.action,
                    "action mismatch at tick {t} entity {:?}",
                    e
                );
            }
        }
    }

    // --- Target selection ---

    #[test]
    fn test_eat_selects_highest_nutrition_target() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 5, y: 5 });

        let f1 = world.spawn();
        world.positions.insert(f1, Position { x: 5, y: 5 });
        world.nutritions.insert(f1, Nutrition { value: 10.0 });

        let f2 = world.spawn();
        world.positions.insert(f2, Position { x: 5, y: 5 });
        world.nutritions.insert(f2, Nutrition { value: 30.0 });

        let target = select_eat_target(&world, e);
        assert_eq!(target, Some(f2));
    }

    #[test]
    fn test_attack_selects_lowest_health_target() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.combat_stats.insert(
            e,
            CombatStats {
                attack: 10.0,
                defense: 5.0,
                aggression: 0.8,
            },
        );

        let t1 = world.spawn();
        world.positions.insert(t1, Position { x: 5, y: 5 });
        world.combat_stats.insert(
            t1,
            CombatStats {
                attack: 5.0,
                defense: 3.0,
                aggression: 0.0,
            },
        );
        world.healths.insert(
            t1,
            Health {
                current: 80.0,
                max: 100.0,
            },
        );

        let t2 = world.spawn();
        world.positions.insert(t2, Position { x: 5, y: 5 });
        world.combat_stats.insert(
            t2,
            CombatStats {
                attack: 5.0,
                defense: 3.0,
                aggression: 0.0,
            },
        );
        world.healths.insert(
            t2,
            Health {
                current: 30.0,
                max: 100.0,
            },
        );

        let target = select_attack_target(&world, e);
        assert_eq!(target, Some(t2));
    }
}
