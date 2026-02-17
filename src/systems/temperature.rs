use crate::components::Tick;
use crate::tile_map::Terrain;
use crate::world::World;

/// Phase 1 (Environment): Temperature equilibrium system.
///
/// Each tile drifts toward a target temperature determined by its terrain type
/// at a rate of 0.1 degrees per tick. Pure arithmetic — no RNG needed.
/// Uses collect-then-apply mutation pattern.
pub fn run_temperature(world: &mut World, _tick: Tick) {
    let width = world.tiles.width();
    let height = world.tiles.height();

    // Collect temperature changes
    let mut changes: Vec<(usize, usize, f32)> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let terrain = match world.tiles.get_terrain(x, y) {
                Some(t) => t,
                None => continue,
            };
            let current = match world.tiles.get_temperature(x, y) {
                Some(t) => t,
                None => continue,
            };

            let target = match terrain {
                Terrain::Floor => 20.0,    // life support maintains 20°C
                Terrain::Wall => 15.0,     // hull insulation
                Terrain::Vacuum => -270.0, // near absolute zero
            };

            let diff = target - current;
            if diff.abs() < f32::EPSILON {
                continue;
            }

            // 0.1°C/tick = 10°C/sec drift rate (gameplay abstraction)
            let delta = diff.signum() * diff.abs().min(0.1);
            changes.push((x, y, current + delta));
        }
    }

    // Apply changes
    for (x, y, new_temp) in changes {
        world.tiles.set_temperature(x, y, new_temp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Tick;
    use crate::tile_map::TileMap;
    use crate::world::World;

    #[test]
    fn test_vacuum_drifts_toward_target() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Vacuum target is -270.0, default temperature is 20.0
        world.tiles.set_terrain(0, 0, Terrain::Vacuum);
        world.tiles.set_temperature(0, 0, 20.0);

        run_temperature(&mut world, Tick(0));

        let temp = world.tiles.get_temperature(0, 0).unwrap();
        assert!(temp < 20.0, "vacuum tile should cool: got {temp}");
        assert!(
            (temp - 19.9).abs() < f32::EPSILON,
            "should drift by 0.1: got {temp}"
        );
    }

    #[test]
    fn test_floor_stable_at_target() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Floor target is 20.0, default temperature is 20.0
        world.tiles.set_terrain(0, 0, Terrain::Floor);
        world.tiles.set_temperature(0, 0, 20.0);

        run_temperature(&mut world, Tick(0));

        let temp = world.tiles.get_temperature(0, 0).unwrap();
        assert!(
            (temp - 20.0).abs() < f32::EPSILON,
            "floor at target should stay: got {temp}"
        );
    }

    #[test]
    fn test_wall_cools_toward_target() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Wall target is 15.0, start above at 20.0
        world.tiles.set_terrain(0, 0, Terrain::Wall);
        world.tiles.set_temperature(0, 0, 20.0);

        run_temperature(&mut world, Tick(0));

        let temp = world.tiles.get_temperature(0, 0).unwrap();
        assert!(temp < 20.0, "wall tile should cool: got {temp}");
        assert!(
            (temp - 19.9).abs() < f32::EPSILON,
            "should drift by 0.1: got {temp}"
        );
    }
}
