use crate::components::{
    ActionId, CachedPath, Entity, Gait, MoveCooldown, Position, Tick, WanderTarget,
};
use crate::events::Event;
use crate::tile_map::{find_path, is_diagonal_step};
use crate::world::World;
use rand::RngExt;

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

    // Collect entities that have both position and gait profile, sorted for determinism.
    // Skip the player entity — player movement is handled directly in main.rs.
    let mut candidates: Vec<Entity> = world
        .body
        .positions
        .keys()
        .filter(|e| world.player != Some(**e))
        .filter(|e| world.body.gait_profiles.contains_key(e))
        .filter(|e| !world.pending_deaths.contains(e))
        .copied()
        .collect();
    candidates.sort_by_key(|e| e.0);

    // Determine which entities move this tick and what their new cooldowns are
    let mut moves: Vec<(Entity, Position)> = Vec::new();
    let mut cooldown_updates: Vec<(Entity, u32)> = Vec::new();
    let mut wander_target_updates: Vec<(Entity, Option<WanderTarget>)> = Vec::new();

    enum PathUpdate {
        Remove,
        Advance,             // bump next_step by 1
        Replace(CachedPath), // fresh path from A*
    }
    let mut cached_path_updates: Vec<(Entity, PathUpdate)> = Vec::new();

    for e in candidates {
        let remaining = world
            .body
            .move_cooldowns
            .get(&e)
            .map(|cd| cd.remaining)
            .unwrap_or(0);

        if remaining > 0 {
            // Still cooling down — decrement
            cooldown_updates.push((e, remaining - 1));
            continue;
        }

        let Some(pos) = world.body.positions.get(&e) else {
            continue;
        };
        let Some(profile) = world.body.gait_profiles.get(&e) else {
            continue;
        };
        let gait = world
            .body
            .current_gaits
            .get(&e)
            .copied()
            .unwrap_or(Gait::Walk);
        let base_cooldown = profile.cooldown(gait);

        let intention = world.mind.intentions.get(&e);
        let action = intention.map(|i| i.action);

        // Exhaustive match on ActionId to determine movement mode.
        // Idle: stop and reassess — clear stale movement state.
        // Eat/Attack: track target entity position.
        // Wander/None: pathfind to random destination.
        let is_tracking = match action {
            Some(ActionId::Idle) => {
                cooldown_updates.push((e, base_cooldown));
                wander_target_updates.push((e, None));
                cached_path_updates.push((e, PathUpdate::Remove));
                continue;
            }
            Some(ActionId::Eat) | Some(ActionId::Attack) => true,
            Some(ActionId::Wander) | None => false,
        };

        // Determine goal position
        let goal: Option<(i32, i32)> = if is_tracking {
            // Pathfind to target entity's position (moving target)
            intention
                .and_then(|i| i.target)
                .and_then(|t| world.body.positions.get(&t))
                .map(|p| (p.x, p.y))
        } else {
            // Wander or no intention: use cached wander target or pick new
            let at_goal = world
                .mind
                .wander_targets
                .get(&e)
                .is_some_and(|wt| wt.goal_x == pos.x && wt.goal_y == pos.y);

            if !at_goal {
                world
                    .mind
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
        };

        let Some((gx, gy)) = goal else {
            // No goal — fallback random step
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
            let x = (pos.x + dx).clamp(0, (map_w - 1).max(0));
            let y = (pos.y + dy).clamp(0, (map_h - 1).max(0));
            if !world.tiles.is_walkable(x as usize, y as usize) {
                cooldown_updates.push((e, base_cooldown));
                continue;
            }
            let is_diag = is_diagonal_step((pos.x, pos.y), (x, y));
            if is_diag && !world.tiles.diagonal_clear(pos.x, pos.y, x, y) {
                cooldown_updates.push((e, base_cooldown));
                continue;
            }
            let reset = if is_diag {
                base_cooldown * DIAGONAL_FACTOR / 100
            } else {
                base_cooldown
            };
            moves.push((e, Position { x, y }));
            cooldown_updates.push((e, reset));
            wander_target_updates.push((e, None));
            cached_path_updates.push((e, PathUpdate::Remove));
            continue;
        };

        // For tracking intentions (Eat/Attack), always invalidate cached path
        // since the target may have moved. For wander, try to reuse cached path.
        let cached_step = if !is_tracking {
            world
                .mind
                .cached_paths
                .get(&e)
                .filter(|cp| cp.goal == (gx, gy) && cp.next_step < cp.steps.len())
                .map(|cp| cp.steps[cp.next_step])
        } else {
            None
        };

        if let Some(dest) = cached_step {
            // Validate cached step: reject if it crosses a diagonal wall seam.
            let is_diag = is_diagonal_step((pos.x, pos.y), dest);
            if is_diag && !world.tiles.diagonal_clear(pos.x, pos.y, dest.0, dest.1) {
                // Stale cache contains illegal diagonal — invalidate and re-path next tick.
                cooldown_updates.push((e, base_cooldown));
                cached_path_updates.push((e, PathUpdate::Remove));
                continue;
            }
            // Use cached path: advance index
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

            // Check if path will be exhausted after this step
            let cp = &world.mind.cached_paths[&e];
            if cp.next_step + 1 >= cp.steps.len() {
                // Path exhausted — clear target and cached path
                wander_target_updates.push((e, None));
                cached_path_updates.push((e, PathUpdate::Remove));
            } else {
                wander_target_updates.push((
                    e,
                    Some(WanderTarget {
                        goal_x: gx,
                        goal_y: gy,
                    }),
                ));
                cached_path_updates.push((e, PathUpdate::Advance));
            }
        } else if let Some(path) = find_path(
            &world.tiles,
            (pos.x, pos.y),
            (gx, gy),
            &mut world.path_workspace,
        ) {
            // Compute fresh A* path using pooled workspace
            if path.is_empty() {
                // Already at goal
                cooldown_updates.push((e, base_cooldown));
                if !is_tracking {
                    wander_target_updates.push((e, None));
                }
                cached_path_updates.push((e, PathUpdate::Remove));
            } else {
                let dest = path[0];
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

                if !is_tracking {
                    if path.len() <= 1 {
                        // Will arrive this step — clear target
                        wander_target_updates.push((e, None));
                        cached_path_updates.push((e, PathUpdate::Remove));
                    } else {
                        wander_target_updates.push((
                            e,
                            Some(WanderTarget {
                                goal_x: gx,
                                goal_y: gy,
                            }),
                        ));
                        // Cache path with next_step=1 (step 0 already consumed)
                        cached_path_updates.push((
                            e,
                            PathUpdate::Replace(CachedPath {
                                steps: path,
                                goal: (gx, gy),
                                next_step: 1,
                            }),
                        ));
                    }
                } else {
                    // Tracking: don't cache (target moves), invalidate any stale cache
                    cached_path_updates.push((e, PathUpdate::Remove));
                }
            }
        } else {
            // A* failed — fallback random step
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
            let x = (pos.x + dx).clamp(0, (map_w - 1).max(0));
            let y = (pos.y + dy).clamp(0, (map_h - 1).max(0));
            if !world.tiles.is_walkable(x as usize, y as usize) {
                cooldown_updates.push((e, base_cooldown));
                wander_target_updates.push((e, None));
                cached_path_updates.push((e, PathUpdate::Remove));
                continue;
            }
            let is_diag = is_diagonal_step((pos.x, pos.y), (x, y));
            if is_diag && !world.tiles.diagonal_clear(pos.x, pos.y, x, y) {
                cooldown_updates.push((e, base_cooldown));
                wander_target_updates.push((e, None));
                cached_path_updates.push((e, PathUpdate::Remove));
                continue;
            }
            let reset = if is_diag {
                base_cooldown * DIAGONAL_FACTOR / 100
            } else {
                base_cooldown
            };
            moves.push((e, Position { x, y }));
            cooldown_updates.push((e, reset));
            wander_target_updates.push((e, None));
            cached_path_updates.push((e, PathUpdate::Remove));
        }
    }

    // Apply cooldown updates
    for (e, remaining) in cooldown_updates {
        world
            .body
            .move_cooldowns
            .insert(e, MoveCooldown { remaining });
    }

    // Apply wander target updates
    for (e, target) in wander_target_updates {
        if let Some(wt) = target {
            world.mind.wander_targets.insert(e, wt);
        } else {
            world.mind.wander_targets.remove(&e);
        }
    }

    // Apply cached path updates
    for (e, update) in cached_path_updates {
        match update {
            PathUpdate::Remove => {
                world.mind.cached_paths.remove(&e);
            }
            PathUpdate::Advance => {
                if let Some(cp) = world.mind.cached_paths.get_mut(&e) {
                    cp.next_step += 1;
                }
            }
            PathUpdate::Replace(cp) => {
                world.mind.cached_paths.insert(e, cp);
            }
        }
    }

    // Apply moves
    for (e, new_pos) in moves {
        if let Some(pos) = world.body.positions.get_mut(&e) {
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
        world.body.positions.insert(e, Position { x: 10, y: 10 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());
        let old_pos = world.body.positions[&e];
        run_wander(&mut world, Tick(0));
        let new_pos = world.body.positions[&e];
        let dx = (new_pos.x - old_pos.x).abs();
        let dy = (new_pos.y - old_pos.y).abs();
        // Entity moved exactly one step (Chebyshev distance 1)
        assert_eq!(dx.max(dy), 1);
    }

    #[test]
    fn test_wander_respects_cooldown() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 10, y: 10 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());
        world
            .body
            .move_cooldowns
            .insert(e, MoveCooldown { remaining: 3 });

        // Ticks 0-2: still cooling down, no movement
        for t in 0..3 {
            run_wander(&mut world, Tick(t));
            assert_eq!(world.body.positions[&e].x, 10);
            assert_eq!(world.body.positions[&e].y, 10);
        }

        // Tick 3: cooldown reached 0, entity moves
        run_wander(&mut world, Tick(3));
        let pos = world.body.positions[&e];
        let dx = (pos.x - 10).abs();
        let dy = (pos.y - 10).abs();
        assert_eq!(dx.max(dy), 1);
    }

    #[test]
    fn test_wander_resets_cooldown_after_move() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 10, y: 10 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());
        // Move on first tick (no cooldown)
        run_wander(&mut world, Tick(0));
        // Walk cooldown: cardinal=9, diagonal=9*141/100=12
        let cd = world.body.move_cooldowns[&e].remaining;
        assert!(
            cd == 9 || cd == 12,
            "cooldown {cd} should be 9 (cardinal) or 12 (diagonal)"
        );
    }

    #[test]
    fn test_wander_skips_entities_without_speed() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 5, y: 5 });
        run_wander(&mut world, Tick(0));
        assert_eq!(world.body.positions[&e].x, 5);
        assert_eq!(world.body.positions[&e].y, 5);
    }

    #[test]
    fn test_wander_skips_pending_death() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 5, y: 5 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());
        world.pending_deaths.insert(e);
        run_wander(&mut world, Tick(0));
        assert_eq!(world.body.positions[&e].x, 5);
        assert_eq!(world.body.positions[&e].y, 5);
    }

    #[test]
    fn test_wander_sprint_gait_cooldown() {
        // Sprint gait → shorter cooldown than Walk, still 1 tile per action
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 10, y: 10 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());
        world.body.current_gaits.insert(e, Gait::Sprint);
        run_wander(&mut world, Tick(0));
        let new_pos = world.body.positions[&e];
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
        let cd = world.body.move_cooldowns[&e].remaining;
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
        world.body.positions.insert(e, Position { x: 0, y: 0 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());
        world.body.current_gaits.insert(e, Gait::Sprint);
        // Run many ticks — entity must never leave bounds
        for t in 0..200 {
            run_wander(&mut world, Tick(t));
            let pos = &world.body.positions[&e];
            assert!(pos.x >= 0 && pos.x < 10, "x={} out of bounds", pos.x);
            assert!(pos.y >= 0 && pos.y < 10, "y={} out of bounds", pos.y);
        }
    }

    #[test]
    fn test_wander_deterministic_with_seed() {
        let mut world1 = World::new_with_seed(42);
        let e1 = world1.spawn();
        world1.body.positions.insert(e1, Position { x: 10, y: 10 });
        world1.body.gait_profiles.insert(e1, GaitProfile::biped());

        let mut world2 = World::new_with_seed(42);
        let e2 = world2.spawn();
        world2.body.positions.insert(e2, Position { x: 10, y: 10 });
        world2.body.gait_profiles.insert(e2, GaitProfile::biped());

        // Run several ticks through cooldown cycles
        for t in 0..30 {
            run_wander(&mut world1, Tick(t));
            run_wander(&mut world2, Tick(t));
        }

        assert_eq!(world1.body.positions[&e1].x, world2.body.positions[&e2].x);
        assert_eq!(world1.body.positions[&e1].y, world2.body.positions[&e2].y);
    }

    // --- A* pathfinding tests ---

    #[test]
    fn test_pathfind_to_eat_target() {
        use crate::components::{Intention, Nutrition};

        let mut world = World::new_with_seed(42);
        let creature = world.spawn();
        world
            .body
            .positions
            .insert(creature, Position { x: 5, y: 5 });
        world
            .body
            .gait_profiles
            .insert(creature, GaitProfile::biped());

        let food = world.spawn();
        world.body.positions.insert(food, Position { x: 8, y: 5 });
        world
            .mind
            .nutritions
            .insert(food, Nutrition { value: 30.0 });

        // Set Eat intention targeting the food
        world.mind.intentions.insert(
            creature,
            Intention {
                action: ActionId::Eat,
                target: Some(food),
            },
        );

        run_wander(&mut world, Tick(0));

        // Should move toward food (east)
        let pos = world.body.positions[&creature];
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
            world.tiles.set_terrain(5, y, Terrain::Wall);
        }

        let creature = world.spawn();
        world
            .body
            .positions
            .insert(creature, Position { x: 4, y: 5 });
        world
            .body
            .gait_profiles
            .insert(creature, GaitProfile::biped());

        let target = world.spawn();
        world.body.positions.insert(target, Position { x: 6, y: 5 });

        world.mind.intentions.insert(
            creature,
            Intention {
                action: ActionId::Attack,
                target: Some(target),
            },
        );
        world.body.combat_stats.insert(
            target,
            crate::components::CombatStats {
                attack: 5.0,
                defense: 3.0,
                aggression: 0.0,
            },
        );

        run_wander(&mut world, Tick(0));

        let pos = world.body.positions[&creature];
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
        world.body.positions.insert(e, Position { x: 5, y: 5 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());

        world.mind.intentions.insert(
            e,
            Intention {
                action: ActionId::Idle,
                target: None,
            },
        );

        run_wander(&mut world, Tick(0));
        assert_eq!(world.body.positions[&e].x, 5);
        assert_eq!(world.body.positions[&e].y, 5);
    }

    #[test]
    fn test_idle_clears_cached_movement_state() {
        use crate::components::{CachedPath, Intention, WanderTarget};

        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 5, y: 5 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());

        // Pre-populate stale movement state
        world.mind.wander_targets.insert(
            e,
            WanderTarget {
                goal_x: 10,
                goal_y: 10,
            },
        );
        world.mind.cached_paths.insert(
            e,
            CachedPath {
                steps: vec![(6, 5), (7, 5)],
                goal: (10, 10),
                next_step: 0,
            },
        );

        world.mind.intentions.insert(
            e,
            Intention {
                action: ActionId::Idle,
                target: None,
            },
        );

        run_wander(&mut world, Tick(0));

        // Position unchanged
        assert_eq!(world.body.positions[&e].x, 5);
        assert_eq!(world.body.positions[&e].y, 5);
        // Stale movement state cleared
        assert!(
            !world.mind.wander_targets.contains_key(&e),
            "wander_targets should be cleared on Idle"
        );
        assert!(
            !world.mind.cached_paths.contains_key(&e),
            "cached_paths should be cleared on Idle"
        );
    }

    #[test]
    fn test_pathfind_arrives_at_target() {
        use crate::components::{Intention, Nutrition};

        let mut world = World::new_with_seed(42);
        let creature = world.spawn();
        world
            .body
            .positions
            .insert(creature, Position { x: 5, y: 5 });
        world
            .body
            .gait_profiles
            .insert(creature, GaitProfile::biped());

        let food = world.spawn();
        world.body.positions.insert(food, Position { x: 7, y: 5 });
        world
            .mind
            .nutritions
            .insert(food, Nutrition { value: 30.0 });

        // Run enough ticks to cover distance 2 (cooldown=10 per step at speed 1)
        for t in 0..250 {
            world.mind.intentions.insert(
                creature,
                Intention {
                    action: ActionId::Eat,
                    target: Some(food),
                },
            );
            run_wander(&mut world, Tick(t));
        }

        let pos = world.body.positions[&creature];
        assert_eq!(
            (pos.x, pos.y),
            (7, 5),
            "entity should arrive at food position"
        );
    }

    #[test]
    fn test_random_fallback_blocks_wall_cardinal() {
        use crate::tile_map::Terrain;

        // Surround entity with walls on all 8 sides — it must not move.
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);
        for dx in -1..=1_i32 {
            for dy in -1..=1_i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                world
                    .tiles
                    .set_terrain((2 + dx) as usize, (2 + dy) as usize, Terrain::Wall);
            }
        }
        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 2, y: 2 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());

        // Run many ticks — entity must stay put.
        for t in 0..50 {
            run_wander(&mut world, Tick(t));
            let pos = &world.body.positions[&e];
            assert_eq!(
                (pos.x, pos.y),
                (2, 2),
                "entity moved into a wall on tick {t}"
            );
        }
    }

    #[test]
    fn test_random_fallback_blocks_diagonal_wall_seam() {
        use crate::tile_map::Terrain;

        // Create a diagonal wall seam that only blocks diagonal movement.
        // . # .
        // . E .
        // . # .
        // Entity at (1,1). Walls at (1,0) and (1,2) block N and S.
        // Now also block diagonals via seam:
        //   # . .
        //   . E .
        //   . . #
        // Walls at (0,0) and (2,2). Diagonal NW and SE are seam-blocked.
        // Also wall at (2,0) blocks E of row 0, making NE shoulder blocked.
        // And wall at (0,2) blocks W of row 2, making SW shoulder blocked.
        // This leaves only E and W as valid moves.
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);
        world.tiles.set_terrain(0, 0, Terrain::Wall);
        world.tiles.set_terrain(1, 0, Terrain::Wall);
        world.tiles.set_terrain(2, 0, Terrain::Wall);
        world.tiles.set_terrain(0, 2, Terrain::Wall);
        world.tiles.set_terrain(1, 2, Terrain::Wall);
        world.tiles.set_terrain(2, 2, Terrain::Wall);

        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 1, y: 1 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());

        // Run many ticks. Entity should never end up on a wall tile.
        for t in 0..100 {
            run_wander(&mut world, Tick(t));
            let pos = &world.body.positions[&e];
            assert!(
                world.tiles.is_walkable(pos.x as usize, pos.y as usize),
                "entity at ({},{}) is on non-walkable tile on tick {t}",
                pos.x,
                pos.y,
            );
        }
    }

    #[test]
    fn test_cached_path_diagonal_seam_invalidated() {
        use crate::components::{CachedPath, Intention, WanderTarget};
        use crate::tile_map::Terrain;

        // Inject a stale cached path that includes a diagonal wall squeeze.
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(10, 10);
        // Wall seam at (3,3) and (4,4)
        world.tiles.set_terrain(3, 3, Terrain::Wall);
        world.tiles.set_terrain(4, 4, Terrain::Wall);

        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 3, y: 4 });
        world.body.gait_profiles.insert(e, GaitProfile::biped());

        // Intent: wander to (5,3)
        world.mind.intentions.insert(
            e,
            Intention {
                action: ActionId::Wander,
                target: None,
            },
        );
        world.mind.wander_targets.insert(
            e,
            WanderTarget {
                goal_x: 5,
                goal_y: 3,
            },
        );

        // Inject stale cache with illegal diagonal step (3,4)->(4,3)
        // Shoulder tiles: (4,4)=Wall and (3,3)=Wall — blocked.
        world.mind.cached_paths.insert(
            e,
            CachedPath {
                steps: vec![(4, 3), (5, 3)],
                goal: (5, 3),
                next_step: 0,
            },
        );

        run_wander(&mut world, Tick(0));

        let pos = &world.body.positions[&e];
        // Entity must NOT have moved to (4,3) through the wall seam.
        assert_ne!(
            (pos.x, pos.y),
            (4, 3),
            "entity squeezed through diagonal wall seam via stale cache"
        );
        // The stale cache should have been invalidated.
        assert!(
            !world.mind.cached_paths.contains_key(&e),
            "stale cached path should have been removed"
        );
    }
}
