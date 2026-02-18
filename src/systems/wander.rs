use crate::components::{ActionId, Entity, Gait, MoveCooldown, Position, Tick, WanderTarget};
use crate::events::Event;
use crate::tile_map::is_diagonal_step;
use crate::world::World;
use rand::RngExt;

/// How far (Chebyshev) a fleeing entity runs from its threat (30 meters).
const FLEE_RANGE: i32 = 30;

/// √2 multiplier as fixed-point (141/100) for diagonal movement cost.
const DIAGONAL_FACTOR: u32 = 141;

/// How far (Chebyshev) a wandering entity picks random destinations (30 meters).
const WANDER_RANGE: i32 = 30;

/// Phase 4 (Actions): Unified movement system.
///
/// Handles all entity movement via A* pathfinding:
/// - Eat/Attack intention with target: pathfind toward target position.
/// - Wander intention (or no intention): pathfind to a random destination.
/// - Idle intention: skip movement.
///
/// Falls back to random 8-directional steps if no path is found.
/// Cooldown timer gates movement speed (like DF).
pub fn run_wander(world: &mut World, tick: Tick) {
    let map_w = world.tiles.width() as i32;
    let map_h = world.tiles.height() as i32;

    // Collect entities that have both position and gait profile, sorted for determinism
    let mut candidates: Vec<Entity> = world
        .positions
        .keys()
        .filter(|e| world.gait_profiles.contains_key(e))
        .filter(|e| !world.pending_deaths.contains(e))
        .copied()
        .collect();
    candidates.sort_by_key(|e| e.0);

    // Determine which entities move this tick and what their new cooldowns are
    let mut moves: Vec<(Entity, Position)> = Vec::new();
    let mut cooldown_updates: Vec<(Entity, u32)> = Vec::new();
    let mut wander_target_updates: Vec<(Entity, Option<WanderTarget>)> = Vec::new();

    for e in candidates {
        let remaining = world
            .move_cooldowns
            .get(&e)
            .map(|cd| cd.remaining)
            .unwrap_or(0);

        if remaining > 0 {
            // Still cooling down — decrement
            cooldown_updates.push((e, remaining - 1));
            continue;
        }

        let Some(pos) = world.positions.get(&e) else {
            continue;
        };
        let Some(profile) = world.gait_profiles.get(&e) else {
            continue;
        };
        let gait = world.current_gaits.get(&e).copied().unwrap_or(Gait::Walk);
        let base_cooldown = profile.cooldown(gait);

        let intention = world.intentions.get(&e);
        let action = intention.map(|i| i.action);

        // Idle: skip movement but set cooldown (walk rate, no actual movement)
        if action == Some(ActionId::Idle) {
            cooldown_updates.push((e, base_cooldown));
            continue;
        }

        // Determine goal position
        let goal: Option<(i32, i32)> = match action {
            Some(ActionId::Eat) | Some(ActionId::Attack) | Some(ActionId::Charge) => {
                // Pathfind to target entity's position
                intention
                    .and_then(|i| i.target)
                    .and_then(|t| world.positions.get(&t))
                    .map(|p| (p.x, p.y))
            }
            Some(ActionId::Flee) => {
                // Flee: compute point in opposite direction from threat
                intention
                    .and_then(|i| i.target)
                    .and_then(|t| world.positions.get(&t))
                    .map(|threat_pos| {
                        let dx = pos.x - threat_pos.x;
                        let dy = pos.y - threat_pos.y;
                        if dx == 0 && dy == 0 {
                            // Same tile as threat: pick random direction
                            let dir = world.rng.random_range(0..8);
                            let (rdx, rdy) = match dir {
                                0 => (0, -1),
                                1 => (1, -1),
                                2 => (1, 0),
                                3 => (1, 1),
                                4 => (0, 1),
                                5 => (-1, 1),
                                6 => (-1, 0),
                                _ => (-1, -1),
                            };
                            let gx = (pos.x + rdx * FLEE_RANGE).clamp(0, (map_w - 1).max(0));
                            let gy = (pos.y + rdy * FLEE_RANGE).clamp(0, (map_h - 1).max(0));
                            (gx, gy)
                        } else {
                            // Normalize direction and scale by FLEE_RANGE
                            let len = ((dx * dx + dy * dy) as f32).sqrt();
                            let ndx = (dx as f32 / len * FLEE_RANGE as f32) as i32;
                            let ndy = (dy as f32 / len * FLEE_RANGE as f32) as i32;
                            let gx = (pos.x + ndx).clamp(0, (map_w - 1).max(0));
                            let gy = (pos.y + ndy).clamp(0, (map_h - 1).max(0));
                            (gx, gy)
                        }
                    })
            }
            _ => {
                // Wander or no intention: use cached wander target or pick new
                let at_goal = world
                    .wander_targets
                    .get(&e)
                    .is_some_and(|wt| wt.goal_x == pos.x && wt.goal_y == pos.y);

                if !at_goal {
                    world
                        .wander_targets
                        .get(&e)
                        .map(|wt| (wt.goal_x, wt.goal_y))
                } else {
                    None
                }
                .or_else(|| {
                    // Pick new random walkable destination
                    for _ in 0..5 {
                        let dx = world.rng.random_range(-WANDER_RANGE..=WANDER_RANGE);
                        let dy = world.rng.random_range(-WANDER_RANGE..=WANDER_RANGE);
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let gx = (pos.x + dx).clamp(0, (map_w - 1).max(0));
                        let gy = (pos.y + dy).clamp(0, (map_h - 1).max(0));
                        if gx == pos.x && gy == pos.y {
                            continue;
                        }
                        if world.tiles.is_walkable(gx as usize, gy as usize) {
                            return Some((gx, gy));
                        }
                    }
                    None
                })
            }
        };

        // Try A* pathfinding
        if let Some((gx, gy)) = goal
            && let Some(path) = world.tiles.find_path((pos.x, pos.y), (gx, gy))
        {
            let step_count = 1.min(path.len());
            if step_count > 0 {
                let dest = path[step_count - 1];
                let is_diag = is_diagonal_step((pos.x, pos.y), dest);
                let reset = if is_diag {
                    base_cooldown * DIAGONAL_FACTOR / 100
                } else {
                    base_cooldown
                };
                moves.push((
                    e,
                    Position {
                        x: dest.0,
                        y: dest.1,
                    },
                ));
                cooldown_updates.push((e, reset));
            } else {
                cooldown_updates.push((e, base_cooldown));
            }

            // Update wander target for Wander/no-intention entities
            if action != Some(ActionId::Eat)
                && action != Some(ActionId::Attack)
                && action != Some(ActionId::Charge)
                && action != Some(ActionId::Flee)
            {
                if step_count >= path.len() {
                    // Arrived (or will arrive) — clear target
                    wander_target_updates.push((e, None));
                } else {
                    wander_target_updates.push((
                        e,
                        Some(WanderTarget {
                            goal_x: gx,
                            goal_y: gy,
                        }),
                    ));
                }
            }
            continue;
        }

        // Fallback: single random 8-directional step
        let direction = world.rng.random_range(0..8);
        let (dx, dy) = match direction {
            0 => (0, -1),  // N
            1 => (1, -1),  // NE
            2 => (1, 0),   // E
            3 => (1, 1),   // SE
            4 => (0, 1),   // S
            5 => (-1, 1),  // SW
            6 => (-1, 0),  // W
            _ => (-1, -1), // NW
        };
        // Clamp to tilemap bounds
        let x = (pos.x + dx).clamp(0, (map_w - 1).max(0));
        let y = (pos.y + dy).clamp(0, (map_h - 1).max(0));
        let is_diag = is_diagonal_step((pos.x, pos.y), (x, y));
        let reset = if is_diag {
            base_cooldown * DIAGONAL_FACTOR / 100
        } else {
            base_cooldown
        };
        moves.push((e, Position { x, y }));
        cooldown_updates.push((e, reset));
        wander_target_updates.push((e, None));
    }

    // Apply cooldown updates
    for (e, remaining) in cooldown_updates {
        world.move_cooldowns.insert(e, MoveCooldown { remaining });
    }

    // Apply wander target updates
    for (e, target) in wander_target_updates {
        if let Some(wt) = target {
            world.wander_targets.insert(e, wt);
        } else {
            world.wander_targets.remove(&e);
        }
    }

    // Apply moves
    for (e, new_pos) in moves {
        if let Some(pos) = world.positions.get_mut(&e) {
            *pos = new_pos;
            world.events.push(Event::Moved {
                entity: e,
                x: new_pos.x,
                y: new_pos.y,
                tick,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Gait, GaitProfile, MoveCooldown, Position, Tick};
    use crate::world::World;

    #[test]
    fn test_wander_moves_on_first_tick() {
        // No MoveCooldown → entity moves immediately (remaining defaults to 0)
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.gait_profiles.insert(e, GaitProfile::biped());
        let old_pos = world.positions[&e];
        run_wander(&mut world, Tick(0));
        let new_pos = world.positions[&e];
        let dx = (new_pos.x - old_pos.x).abs();
        let dy = (new_pos.y - old_pos.y).abs();
        // Entity moved exactly one step (Chebyshev distance 1)
        assert_eq!(dx.max(dy), 1);
    }

    #[test]
    fn test_wander_respects_cooldown() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.gait_profiles.insert(e, GaitProfile::biped());
        world
            .move_cooldowns
            .insert(e, MoveCooldown { remaining: 3 });

        // Ticks 0-2: still cooling down, no movement
        for t in 0..3 {
            run_wander(&mut world, Tick(t));
            assert_eq!(world.positions[&e].x, 10);
            assert_eq!(world.positions[&e].y, 10);
        }

        // Tick 3: cooldown reached 0, entity moves
        run_wander(&mut world, Tick(3));
        let pos = world.positions[&e];
        let dx = (pos.x - 10).abs();
        let dy = (pos.y - 10).abs();
        assert_eq!(dx.max(dy), 1);
    }

    #[test]
    fn test_wander_resets_cooldown_after_move() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.gait_profiles.insert(e, GaitProfile::biped());
        // Move on first tick (no cooldown)
        run_wander(&mut world, Tick(0));
        // Walk cooldown: cardinal=9, diagonal=9*141/100=12
        let cd = world.move_cooldowns[&e].remaining;
        assert!(
            cd == 9 || cd == 12,
            "cooldown {cd} should be 9 (cardinal) or 12 (diagonal)"
        );
    }

    #[test]
    fn test_wander_skips_entities_without_speed() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 5, y: 5 });
        run_wander(&mut world, Tick(0));
        assert_eq!(world.positions[&e].x, 5);
        assert_eq!(world.positions[&e].y, 5);
    }

    #[test]
    fn test_wander_skips_pending_death() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.gait_profiles.insert(e, GaitProfile::biped());
        world.pending_deaths.push(e);
        run_wander(&mut world, Tick(0));
        assert_eq!(world.positions[&e].x, 5);
        assert_eq!(world.positions[&e].y, 5);
    }

    #[test]
    fn test_wander_sprint_gait_cooldown() {
        // Sprint gait → shorter cooldown than Walk, still 1 tile per action
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.gait_profiles.insert(e, GaitProfile::biped());
        world.current_gaits.insert(e, Gait::Sprint);
        run_wander(&mut world, Tick(0));
        let new_pos = world.positions[&e];
        let dx = (new_pos.x - 10).abs();
        let dy = (new_pos.y - 10).abs();
        // 1 tile per action: Chebyshev distance exactly 1
        assert_eq!(
            dx.max(dy),
            1,
            "displacement ({},{}) should be exactly 1",
            dx,
            dy
        );
        // Sprint cooldown: cardinal=3, diagonal=3*141/100=4
        let cd = world.move_cooldowns[&e].remaining;
        assert!(
            cd == 3 || cd == 4,
            "cooldown {cd} should be 3 (cardinal) or 4 (diagonal)"
        );
    }

    #[test]
    fn test_wander_clamps_to_map_bounds() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(10, 10);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 0, y: 0 });
        world.gait_profiles.insert(e, GaitProfile::biped());
        world.current_gaits.insert(e, Gait::Sprint);
        // Run many ticks — entity must never leave bounds
        for t in 0..200 {
            run_wander(&mut world, Tick(t));
            let pos = &world.positions[&e];
            assert!(pos.x >= 0 && pos.x < 10, "x={} out of bounds", pos.x);
            assert!(pos.y >= 0 && pos.y < 10, "y={} out of bounds", pos.y);
        }
    }

    #[test]
    fn test_wander_deterministic_with_seed() {
        let mut world1 = World::new_with_seed(42);
        let e1 = world1.spawn();
        world1.positions.insert(e1, Position { x: 10, y: 10 });
        world1.gait_profiles.insert(e1, GaitProfile::biped());

        let mut world2 = World::new_with_seed(42);
        let e2 = world2.spawn();
        world2.positions.insert(e2, Position { x: 10, y: 10 });
        world2.gait_profiles.insert(e2, GaitProfile::biped());

        // Run several ticks through cooldown cycles
        for t in 0..30 {
            run_wander(&mut world1, Tick(t));
            run_wander(&mut world2, Tick(t));
        }

        assert_eq!(world1.positions[&e1].x, world2.positions[&e2].x);
        assert_eq!(world1.positions[&e1].y, world2.positions[&e2].y);
    }

    // --- A* pathfinding tests ---

    #[test]
    fn test_pathfind_to_eat_target() {
        use crate::components::{Intention, Nutrition};

        let mut world = World::new_with_seed(42);
        let creature = world.spawn();
        world.positions.insert(creature, Position { x: 5, y: 5 });
        world.gait_profiles.insert(creature, GaitProfile::biped());

        let food = world.spawn();
        world.positions.insert(food, Position { x: 8, y: 5 });
        world.nutritions.insert(food, Nutrition { value: 30.0 });

        // Set Eat intention targeting the food
        world.intentions.insert(
            creature,
            Intention {
                action: ActionId::Eat,
                target: Some(food),
            },
        );

        run_wander(&mut world, Tick(0));

        // Should move toward food (east)
        let pos = world.positions[&creature];
        assert!(
            pos.x > 5,
            "entity should move toward food at x=8, got x={}",
            pos.x
        );
    }

    #[test]
    fn test_pathfind_around_obstacle() {
        use crate::components::Intention;
        use crate::tile_map::Terrain;

        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(10, 10);

        // Wall blocking direct path
        for y in 3..=7 {
            world.tiles.set_terrain(5, y, Terrain::Stone);
        }

        let creature = world.spawn();
        world.positions.insert(creature, Position { x: 4, y: 5 });
        world.gait_profiles.insert(creature, GaitProfile::biped());

        let target = world.spawn();
        world.positions.insert(target, Position { x: 6, y: 5 });

        world.intentions.insert(
            creature,
            Intention {
                action: ActionId::Attack,
                target: Some(target),
            },
        );
        world.combat_stats.insert(
            target,
            crate::components::CombatStats {
                attack: 5.0,
                defense: 3.0,
                aggression: 0.0,
            },
        );

        run_wander(&mut world, Tick(0));

        let pos = world.positions[&creature];
        // Should NOT be on the wall
        assert_ne!(pos.x, 5, "entity should path around the wall");
        // Should have moved from starting position
        assert!(pos.x != 4 || pos.y != 5, "entity should have moved");
    }

    #[test]
    fn test_idle_skips_movement() {
        use crate::components::Intention;

        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 5, y: 5 });
        world.gait_profiles.insert(e, GaitProfile::biped());

        world.intentions.insert(
            e,
            Intention {
                action: ActionId::Idle,
                target: None,
            },
        );

        run_wander(&mut world, Tick(0));
        assert_eq!(world.positions[&e].x, 5);
        assert_eq!(world.positions[&e].y, 5);
    }

    #[test]
    fn test_pathfind_arrives_at_target() {
        use crate::components::{Intention, Nutrition};

        let mut world = World::new_with_seed(42);
        let creature = world.spawn();
        world.positions.insert(creature, Position { x: 5, y: 5 });
        world.gait_profiles.insert(creature, GaitProfile::biped());

        let food = world.spawn();
        world.positions.insert(food, Position { x: 7, y: 5 });
        world.nutritions.insert(food, Nutrition { value: 30.0 });

        // Run enough ticks to cover distance 2 (cooldown=10 per step at speed 1)
        for t in 0..250 {
            world.intentions.insert(
                creature,
                Intention {
                    action: ActionId::Eat,
                    target: Some(food),
                },
            );
            run_wander(&mut world, Tick(t));
        }

        let pos = world.positions[&creature];
        assert_eq!(
            (pos.x, pos.y),
            (7, 5),
            "entity should arrive at food position"
        );
    }
}
