use crate::components::Tick;
use crate::tile_map::CHUNK_SIZE;
use crate::world::World;

/// Phase 1 (Environment): Temperature equilibrium system.
///
/// Each tile drifts toward a target temperature determined by its terrain type
/// at a rate of 0.1 degrees per tick. Pure arithmetic — no RNG needed.
///
/// Iterates by chunk and skips chunks already at equilibrium (O(1) steady state).
/// Uses collect-then-apply mutation pattern.
pub fn run_temperature(world: &mut World, _tick: Tick) {
    let cx_count = world.tiles.chunks_x();
    let cy_count = world.tiles.chunks_y();
    let map_w = world.tiles.width();
    let map_h = world.tiles.height();

    // (cx, cy, lx, ly, new_temp)
    let mut changes: Vec<(usize, usize, usize, usize, f32)> = Vec::new();
    let mut equilibrium_chunks: Vec<(usize, usize)> = Vec::new();

    for cy in 0..cy_count {
        let local_h = if (cy + 1) * CHUNK_SIZE <= map_h {
            CHUNK_SIZE
        } else {
            map_h % CHUNK_SIZE
        };

        for cx in 0..cx_count {
            let chunk = world.tiles.chunk_at(cx, cy);
            if chunk.at_equilibrium {
                continue;
            }

            let local_w = if (cx + 1) * CHUNK_SIZE <= map_w {
                CHUNK_SIZE
            } else {
                map_w % CHUNK_SIZE
            };

            let mut chunk_has_changes = false;

            for ly in 0..local_h {
                for lx in 0..local_w {
                    let terrain = chunk.get_terrain(lx, ly);
                    let current = chunk.get_temperature(lx, ly);
                    let target = terrain.target_temperature();

                    let diff = target - current;
                    if diff.abs() < f32::EPSILON {
                        continue;
                    }

                    // 0.1°C/tick = 10°C/sec drift rate (gameplay abstraction)
                    let delta = diff.signum() * diff.abs().min(0.1);
                    changes.push((cx, cy, lx, ly, current + delta));
                    chunk_has_changes = true;
                }
            }

            if !chunk_has_changes {
                equilibrium_chunks.push((cx, cy));
            }
        }
    }

    // Apply temperature changes
    for (cx, cy, lx, ly, new_temp) in changes {
        world
            .tiles
            .chunk_at_mut(cx, cy)
            .set_temperature(lx, ly, new_temp);
    }

    // Mark chunks that had no changes as at equilibrium
    for (cx, cy) in equilibrium_chunks {
        world.tiles.chunk_at_mut(cx, cy).at_equilibrium = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Tick;
    use crate::tile_map::{Terrain, TileMap};
    use crate::world::World;

    #[test]
    fn test_temperature_drifts_toward_target() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Water target is 10.0, start at 20.0
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
        // Road target is 16.0, default temperature is 16.0
        world.tiles.set_terrain(0, 0, Terrain::Road);
        world.tiles.set_temperature(0, 0, 16.0);

        run_temperature(&mut world, Tick(0));

        let temp = world.tiles.get_temperature(0, 0).unwrap();
        assert!(
            (temp - 16.0).abs() < f32::EPSILON,
            "road at target should stay: got {temp}"
        );
    }

    #[test]
    fn test_temperature_floor_warms() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Floor target is 18.0, start below at 16.0
        world.tiles.set_terrain(0, 0, Terrain::Floor);
        world.tiles.set_temperature(0, 0, 16.0);

        run_temperature(&mut world, Tick(0));

        let temp = world.tiles.get_temperature(0, 0).unwrap();
        assert!(temp > 16.0, "floor tile should warm: got {temp}");
        assert!(
            (temp - 16.1).abs() < f32::EPSILON,
            "should drift by 0.1: got {temp}"
        );
    }

    #[test]
    fn test_temperature_skips_equilibrium_chunks() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(130, 70);
        // Set some varied terrain
        world.tiles.set_terrain(0, 0, Terrain::Water);
        world.tiles.set_terrain(65, 0, Terrain::Floor);

        // Initialize to target → all at equilibrium
        world.tiles.initialize_temperatures();

        // Record temperatures
        let t_water = world.tiles.get_temperature(0, 0).unwrap();
        let t_floor = world.tiles.get_temperature(65, 0).unwrap();

        // run_temperature should be a no-op
        run_temperature(&mut world, Tick(0));

        assert_eq!(world.tiles.get_temperature(0, 0).unwrap(), t_water);
        assert_eq!(world.tiles.get_temperature(65, 0).unwrap(), t_floor);
    }

    #[test]
    fn test_temperature_marks_equilibrium_after_convergence() {
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(1, 1);
        // Road at target = already at equilibrium after one run
        world.tiles.set_terrain(0, 0, Terrain::Road);
        world.tiles.set_temperature(0, 0, 16.0);

        // Chunk starts non-equilibrium (set_terrain clears it)
        assert!(!world.tiles.chunk_at(0, 0).at_equilibrium);

        run_temperature(&mut world, Tick(0));

        // No changes needed → chunk should be marked equilibrium
        assert!(world.tiles.chunk_at(0, 0).at_equilibrium);
    }
}
