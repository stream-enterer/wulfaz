use rand::RngExt;
use crate::components::{Entity, Position, Tick};
use crate::world::World;

/// Phase 4 (Actions): Wander/movement system.
///
/// Entities with both a Position and a Speed component move 1 tile in a random
/// cardinal direction each tick. Entities in pending_deaths are skipped.
/// Uses collect-then-apply mutation pattern.
pub fn run_wander(world: &mut World, tick: Tick) {
    let _ = tick; // available for future use

    // Collect entities that have both position and speed, sorted for determinism
    let mut movers: Vec<Entity> = world.positions.keys()
        .filter(|e| world.speeds.contains_key(e))
        .filter(|e| !world.pending_deaths.contains(e))
        .copied()
        .collect();
    movers.sort_by_key(|e| e.0);

    // Generate random moves
    let moves: Vec<(Entity, Position)> = movers.into_iter()
        .filter_map(|e| {
            let pos = world.positions.get(&e)?;
            let direction = world.rng.random_range(0..4);
            let (dx, dy) = match direction {
                0 => (0, -1),  // up
                1 => (0, 1),   // down
                2 => (-1, 0),  // left
                _ => (1, 0),   // right
            };
            Some((e, Position { x: pos.x + dx, y: pos.y + dy }))
        })
        .collect();

    // Apply moves
    for (e, new_pos) in moves {
        if let Some(pos) = world.positions.get_mut(&e) {
            *pos = new_pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Position, Speed, Tick};
    use crate::world::World;

    #[test]
    fn test_wander_moves_entity() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.speeds.insert(e, Speed { value: 1 });
        let old_pos = world.positions[&e];
        run_wander(&mut world, Tick(0));
        let new_pos = world.positions[&e];
        // Should have moved by exactly 1 in one axis
        let dx = (new_pos.x - old_pos.x).abs();
        let dy = (new_pos.y - old_pos.y).abs();
        assert_eq!(dx + dy, 1);
    }

    #[test]
    fn test_wander_skips_entities_without_speed() {
        let mut world = World::new_with_seed(42);
        let e = world.spawn();
        world.positions.insert(e, Position { x: 5, y: 5 });
        // No speed component
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
    fn test_wander_deterministic_with_seed() {
        let mut world1 = World::new_with_seed(42);
        let e1 = world1.spawn();
        world1.positions.insert(e1, Position { x: 10, y: 10 });
        world1.speeds.insert(e1, Speed { value: 1 });
        run_wander(&mut world1, Tick(0));

        let mut world2 = World::new_with_seed(42);
        let e2 = world2.spawn();
        world2.positions.insert(e2, Position { x: 10, y: 10 });
        world2.speeds.insert(e2, Speed { value: 1 });
        run_wander(&mut world2, Tick(0));

        assert_eq!(world1.positions[&e1].x, world2.positions[&e2].x);
        assert_eq!(world1.positions[&e1].y, world2.positions[&e2].y);
    }
}
