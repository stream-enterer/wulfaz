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
                Terrain::Water => 10.0,
                Terrain::Stone => 15.0,
                Terrain::Grass | Terrain::Dirt => 20.0,
                Terrain::Sand => 22.0,
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
    fn test_temperature_drifts_toward_target() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Water target is 10.0, default temperature is 20.0
        world.tiles.set_terrain(0, 0, Terrain::Water);
        world.tiles.set_temperature(0, 0, 20.0);

        run_temperature(&mut world, Tick(0));

        let temp = world.tiles.get_temperature(0, 0).unwrap();
        assert!(temp < 20.0, "water tile should cool: got {temp}");
        assert!(
            (temp - 19.9).abs() < f32::EPSILON,
            "should drift by 0.1: got {temp}"
        );
    }

    #[test]
    fn test_temperature_stable_at_target() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Grass target is 20.0, default temperature is 20.0
        world.tiles.set_terrain(0, 0, Terrain::Grass);
        world.tiles.set_temperature(0, 0, 20.0);

        run_temperature(&mut world, Tick(0));

        let temp = world.tiles.get_temperature(0, 0).unwrap();
        assert!(
            (temp - 20.0).abs() < f32::EPSILON,
            "grass at target should stay: got {temp}"
        );
    }

    #[test]
    fn test_temperature_sand_warms() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Sand target is 22.0, start below at 20.0
        world.tiles.set_terrain(0, 0, Terrain::Sand);
        world.tiles.set_temperature(0, 0, 20.0);

        run_temperature(&mut world, Tick(0));

        let temp = world.tiles.get_temperature(0, 0).unwrap();
        assert!(temp > 20.0, "sand tile should warm: got {temp}");
        assert!(
            (temp - 20.1).abs() < f32::EPSILON,
            "should drift by 0.1: got {temp}"
        );
    }
}
