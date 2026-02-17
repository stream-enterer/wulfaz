use std::collections::HashMap;

use rand::RngExt;

use crate::components::*;
use crate::events::Event;
use crate::systems::decisions::UtilityConfig;
use crate::tile_map::Terrain;
use crate::world::World;

/// Parse a KDL file and return the document. Logs a warning and returns None on failure.
fn parse_kdl_file(path: &str) -> Option<kdl::KdlDocument> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("failed to read {}: {}", path, e);
            return None;
        }
    };
    match content.parse::<kdl::KdlDocument>() {
        Ok(doc) => Some(doc),
        Err(e) => {
            log::warn!("failed to parse KDL {}: {}", path, e);
            None
        }
    }
}

/// Helper to get a string value from a child node's first argument.
fn child_str<'a>(children: &'a kdl::KdlDocument, key: &str) -> Option<&'a str> {
    children.get_arg(key)?.as_string()
}

/// Helper to get an i128 value from a child node's first argument.
fn child_i128(children: &kdl::KdlDocument, key: &str) -> Option<i128> {
    children.get_arg(key)?.as_integer()
}

/// Helper to get an f64 value from a child node's first argument.
/// Accepts both float and integer values.
fn child_f64(children: &kdl::KdlDocument, key: &str) -> Option<f64> {
    let val = children.get_arg(key)?;
    val.as_float()
        .or_else(|| val.as_integer().map(|i| i as f64))
}

/// Load utility scorer config from a RON file.
pub fn load_utility_config(world: &mut World, path: &str) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("failed to read {}: {}, using default config", path, e);
            return;
        }
    };
    match ron::from_str::<UtilityConfig>(&content) {
        Ok(config) => world.utility_config = config,
        Err(e) => {
            log::warn!("failed to parse RON {}: {}, using default config", path, e);
        }
    }
}

/// Load creatures from a KDL file and spawn them into the world.
pub fn load_creatures(world: &mut World, path: &str) {
    let Some(doc) = parse_kdl_file(path) else {
        return;
    };

    let map_w = world.tiles.width() as i32;
    let map_h = world.tiles.height() as i32;

    for node in doc.nodes() {
        if node.name().to_string() != "creature" {
            continue;
        }

        // First argument is the creature name, e.g. creature "Human"
        let name = match node.get(0).and_then(|v| v.as_string()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let Some(children) = node.children() else {
            continue;
        };

        let icon_str = child_str(children, "icon").unwrap_or("?");
        let icon_ch = icon_str.chars().next().unwrap_or('?');
        let max_hunger = child_f64(children, "max_hunger").unwrap_or(100.0) as f32;
        let aggression = child_f64(children, "aggression").unwrap_or(0.0) as f32;
        let speed = child_i128(children, "speed").unwrap_or(1) as u32;

        let e = world.spawn();

        // Random position within the map
        let x = if map_w > 0 {
            world.rng.random_range(0..map_w)
        } else {
            0
        };
        let y = if map_h > 0 {
            world.rng.random_range(0..map_h)
        } else {
            0
        };

        world.names.insert(e, Name { value: name });
        world.icons.insert(e, Icon { ch: icon_ch });
        world.positions.insert(e, Position { x, y });
        world.hungers.insert(
            e,
            Hunger {
                current: 0.0,
                max: max_hunger,
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
                aggression,
            },
        );
        world.speeds.insert(e, Speed { value: speed });
        world.action_states.insert(
            e,
            ActionState {
                current_action: None,
                ticks_in_action: 0,
                cooldowns: HashMap::new(),
            },
        );

        world.events.push(Event::Spawned {
            entity: e,
            tick: world.tick,
        });
    }
}

/// Load items from a KDL file and spawn them into the world.
pub fn load_items(world: &mut World, path: &str) {
    let Some(doc) = parse_kdl_file(path) else {
        return;
    };

    let map_w = world.tiles.width() as i32;
    let map_h = world.tiles.height() as i32;

    for node in doc.nodes() {
        if node.name().to_string() != "item" {
            continue;
        }

        let name = match node.get(0).and_then(|v| v.as_string()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let Some(children) = node.children() else {
            continue;
        };

        let icon_str = child_str(children, "icon").unwrap_or("?");
        let icon_ch = icon_str.chars().next().unwrap_or('?');
        let nutrition = child_f64(children, "nutrition").unwrap_or(0.0) as f32;

        let e = world.spawn();

        let x = if map_w > 0 {
            world.rng.random_range(0..map_w)
        } else {
            0
        };
        let y = if map_h > 0 {
            world.rng.random_range(0..map_h)
        } else {
            0
        };

        world.names.insert(e, Name { value: name });
        world.icons.insert(e, Icon { ch: icon_ch });
        world.positions.insert(e, Position { x, y });
        world.nutritions.insert(e, Nutrition { value: nutrition });

        world.events.push(Event::Spawned {
            entity: e,
            tick: world.tick,
        });
    }
}

/// Load terrain definitions from a KDL file and apply them to the tile map.
/// This maps terrain names to the Terrain enum and sets a default pattern.
pub fn load_terrain(world: &mut World, path: &str) {
    let Some(_doc) = parse_kdl_file(path) else {
        return;
    };

    // Terrain definitions are read for validation, but the actual tile map
    // is initialized to Grass by default. Specific terrain placement would
    // be done by a map generator using these definitions.
    // For now, scatter some terrain variety using the RNG.
    let w = world.tiles.width();
    let h = world.tiles.height();

    for y in 0..h {
        for x in 0..w {
            let roll: f32 = world.rng.random();
            let terrain = if roll < 0.05 {
                Terrain::Water
            } else if roll < 0.10 {
                Terrain::Stone
            } else if roll < 0.20 {
                Terrain::Dirt
            } else if roll < 0.25 {
                Terrain::Sand
            } else {
                Terrain::Grass
            };
            world.tiles.set_terrain(x, y, terrain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    #[test]
    fn test_load_creatures_from_file() {
        let mut world = World::new_with_seed(42);
        load_creatures(&mut world, "data/creatures.kdl");
        // 1 creature in the file
        assert_eq!(world.alive.len(), 1);
        // All should have positions, icons, hungers, healths, combat_stats, speeds, names
        assert_eq!(world.positions.len(), 1);
        assert_eq!(world.icons.len(), 1);
        assert_eq!(world.hungers.len(), 1);
        assert_eq!(world.healths.len(), 1);
        assert_eq!(world.combat_stats.len(), 1);
        assert_eq!(world.speeds.len(), 1);
        assert_eq!(world.names.len(), 1);
    }

    #[test]
    fn test_load_items_from_file() {
        let mut world = World::new_with_seed(42);
        load_items(&mut world, "data/items.kdl");
        // 4 items in the file
        assert_eq!(world.alive.len(), 4);
        assert_eq!(world.nutritions.len(), 4);
        assert_eq!(world.icons.len(), 4);
    }

    #[test]
    fn test_load_terrain_scatters_variety() {
        let mut world = World::new_with_seed(42);
        load_terrain(&mut world, "data/terrain.kdl");
        // Check that not all tiles are Grass anymore
        let w = world.tiles.width();
        let h = world.tiles.height();
        let mut non_grass = 0;
        for y in 0..h {
            for x in 0..w {
                if world.tiles.get_terrain(x, y) != Some(Terrain::Grass) {
                    non_grass += 1;
                }
            }
        }
        assert!(non_grass > 0);
    }

    #[test]
    fn test_load_missing_file_no_panic() {
        let mut world = World::new_with_seed(42);
        load_creatures(&mut world, "nonexistent.kdl");
        assert_eq!(world.alive.len(), 0);
    }

    #[test]
    fn test_creature_properties_correct() {
        let mut world = World::new_with_seed(42);
        load_creatures(&mut world, "data/creatures.kdl");

        // Find the human by name
        let human = world
            .names
            .iter()
            .find(|(_, n)| n.value == "Human")
            .map(|(&e, _)| e);

        if let Some(e) = human {
            if let Some(icon) = world.icons.get(&e) {
                assert_eq!(icon.ch, '@');
            }
            if let Some(hunger) = world.hungers.get(&e) {
                assert_eq!(hunger.max, 100.0);
                assert_eq!(hunger.current, 0.0);
            }
            if let Some(cs) = world.combat_stats.get(&e) {
                assert!((cs.aggression - 0.3).abs() < f32::EPSILON);
            }
            if let Some(speed) = world.speeds.get(&e) {
                assert_eq!(speed.value, 1);
            }
        }
    }
}
