use crate::components::{Entity, MoveCooldown, Position, Tick};
use crate::events::Event;
use crate::world::World;
use rand::RngExt;

/// Base ticks between moves. Higher speed reduces this:
/// interval = BASE_MOVE_TICKS / speed.value.
/// At 100 ticks/sec with speed=2, entity moves ~10 times/sec.
const BASE_MOVE_TICKS: u32 = 20;

/// Phase 4 (Actions): Wander/movement system.
///
/// Entities with Position + Speed move on a cooldown timer (like DF).
/// Each tick, cooldowns decrement. When cooldown reaches 0, the entity
/// takes `speed.value` random steps and the cooldown resets.
/// Entities without a MoveCooldown get one auto-assigned (remaining=0
/// so they move immediately on first tick).
/// Final positions are clamped to the tilemap bounds.
/// Uses collect-then-apply mutation pattern.
pub fn run_wander(world: &mut World, tick: Tick) {
    let map_w = world.tiles.width() as i32;
    let map_h = world.tiles.height() as i32;

    // Collect entities that have both position and speed, sorted for determinism
    let mut candidates: Vec<Entity> = world
        .positions
        .keys()
        .filter(|e| world.speeds.contains_key(e))
        .filter(|e| !world.pending_deaths.contains(e))
        .copied()
        .collect();
    candidates.sort_by_key(|e| e.0);

    // Determine which entities move this tick and what their new cooldowns are
    let mut moves: Vec<(Entity, Position)> = Vec::new();
    let mut cooldown_updates: Vec<(Entity, u32)> = Vec::new();

    for e in candidates {
        let remaining = world
            .move_cooldowns
            .get(&e)
            .map(|cd| cd.remaining)
            .unwrap_or(0);

        if remaining > 0 {
            // Still cooling down — decrement
            cooldown_updates.push((e, remaining - 1));
        } else {
            // Ready to move
            if let (Some(pos), Some(speed)) = (world.positions.get(&e), world.speeds.get(&e)) {
                let steps = speed.value;
                let reset = BASE_MOVE_TICKS / steps.max(1);
                let mut x = pos.x;
                let mut y = pos.y;
                for _ in 0..steps {
                    let direction = world.rng.random_range(0..4);
                    let (dx, dy) = match direction {
                        0 => (0, -1), // up
                        1 => (0, 1),  // down
                        2 => (-1, 0), // left
                        _ => (1, 0),  // right
                    };
                    x += dx;
                    y += dy;
                }
                // Clamp to tilemap bounds
                x = x.clamp(0, (map_w - 1).max(0));
                y = y.clamp(0, (map_h - 1).max(0));
                moves.push((e, Position { x, y }));
                cooldown_updates.push((e, reset));
            }
        }
    }

    // Apply cooldown updates
    for (e, remaining) in cooldown_updates {
        world.move_cooldowns.insert(e, MoveCooldown { remaining });
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
    use crate::components::{MoveCooldown, Position, Speed, Tick};
    use crate::world::World;

    #[test]
    fn test_wander_moves_on_first_tick() {
        // No MoveCooldown → entity moves immediately (remaining defaults to 0)
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.speeds.insert(e, Speed { value: 1 });
        let old_pos = world.positions[&e];
        run_wander(&mut world, Tick(0));
        let new_pos = world.positions[&e];
        let dx = (new_pos.x - old_pos.x).abs();
        let dy = (new_pos.y - old_pos.y).abs();
        assert_eq!(dx + dy, 1);
    }

    #[test]
    fn test_wander_respects_cooldown() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.speeds.insert(e, Speed { value: 1 });
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
        assert_eq!(dx + dy, 1);
    }

    #[test]
    fn test_wander_resets_cooldown_after_move() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.speeds.insert(e, Speed { value: 1 });
        // Move on first tick (no cooldown)
        run_wander(&mut world, Tick(0));
        // Cooldown should now be BASE_MOVE_TICKS / 1 = 20
        let cd = world.move_cooldowns[&e].remaining;
        assert_eq!(cd, BASE_MOVE_TICKS);
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
        world.speeds.insert(e, Speed { value: 1 });
        world.pending_deaths.push(e);
        run_wander(&mut world, Tick(0));
        assert_eq!(world.positions[&e].x, 5);
        assert_eq!(world.positions[&e].y, 5);
    }

    #[test]
    fn test_wander_respects_speed_steps() {
        // Higher speed → more steps per move, lower cooldown
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.speeds.insert(e, Speed { value: 3 });
        run_wander(&mut world, Tick(0));
        let new_pos = world.positions[&e];
        let dx = (new_pos.x - 10).abs();
        let dy = (new_pos.y - 10).abs();
        assert!(dx + dy <= 3, "displacement {} exceeds speed 3", dx + dy);
        // Cooldown = BASE_MOVE_TICKS / 3 = 6
        assert_eq!(world.move_cooldowns[&e].remaining, BASE_MOVE_TICKS / 3);
    }

    #[test]
    fn test_wander_clamps_to_map_bounds() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(10, 10);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 0, y: 0 });
        world.speeds.insert(e, Speed { value: 5 });
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
        world1.speeds.insert(e1, Speed { value: 1 });

        let mut world2 = World::new_with_seed(42);
        let e2 = world2.spawn();
        world2.positions.insert(e2, Position { x: 10, y: 10 });
        world2.speeds.insert(e2, Speed { value: 1 });

        // Run several ticks through cooldown cycles
        for t in 0..30 {
            run_wander(&mut world1, Tick(t));
            run_wander(&mut world2, Tick(t));
        }

        assert_eq!(world1.positions[&e1].x, world2.positions[&e2].x);
        assert_eq!(world1.positions[&e1].y, world2.positions[&e2].y);
    }
}
