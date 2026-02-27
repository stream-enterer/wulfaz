use std::collections::HashMap;

use rand::RngExt;

use crate::components::*;
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

/// Helper to get an f64 value from a child node's first argument.
/// Accepts both float and integer values.
fn child_f64(children: &kdl::KdlDocument, key: &str) -> Option<f64> {
    let val = children.get_arg(key)?;
    val.as_float()
        .or_else(|| val.as_integer().map(|i| i as f64))
}

/// Default body/mind stats for spawning entities. Named archetypes live in
/// `data/archetypes.kdl`; the GIS spawn path looks up the relevant one by name.
pub struct Archetype {
    pub icon: char,
    pub health: f32,
    pub max_hunger: f32,
    pub attack: f32,
    pub defense: f32,
    pub aggression: f32,
    pub gait_profile: GaitProfile,
}

impl Default for Archetype {
    fn default() -> Self {
        Self {
            icon: '?',
            health: 100.0,
            max_hunger: 100.0,
            attack: 10.0,
            defense: 5.0,
            aggression: 0.0,
            gait_profile: GaitProfile::biped(),
        }
    }
}

/// Load all named archetypes from a KDL file.
/// Returns a map from archetype name to its stats.
pub fn load_archetypes(path: &str) -> HashMap<String, Archetype> {
    let mut map = HashMap::new();

    let Some(doc) = parse_kdl_file(path) else {
        return map;
    };

    for node in doc.nodes() {
        if node.name().to_string() != "archetype" {
            continue;
        }

        let Some(name) = node.get(0).and_then(|v| v.as_string()) else {
            continue;
        };

        let Some(children) = node.children() else {
            map.insert(name.to_string(), Archetype::default());
            continue;
        };

        let icon_str = child_str(children, "icon").unwrap_or("?");
        let icon = icon_str.chars().next().unwrap_or('?');
        let health = child_f64(children, "health").unwrap_or(100.0) as f32;
        let max_hunger = child_f64(children, "max_hunger").unwrap_or(100.0) as f32;
        let attack = child_f64(children, "attack").unwrap_or(10.0) as f32;
        let defense = child_f64(children, "defense").unwrap_or(5.0) as f32;
        let aggression = child_f64(children, "aggression").unwrap_or(0.0) as f32;
        let gaits_str = child_str(children, "gaits").unwrap_or("biped");
        let gait_profile = match gaits_str {
            "quadruped" => GaitProfile::quadruped(),
            _ => GaitProfile::biped(),
        };

        map.insert(
            name.to_string(),
            Archetype {
                icon,
                health,
                max_hunger,
                attack,
                defense,
                aggression,
                gait_profile,
            },
        );
    }

    map
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
        Ok(config) => world.mind.utility_config = config,
        Err(e) => {
            log::warn!("failed to parse RON {}: {}, using default config", path, e);
        }
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
            let terrain = if roll < 0.03 {
                Terrain::Water
            } else if roll < 0.06 {
                Terrain::Wall
            } else if roll < 0.12 {
                Terrain::Floor
            } else if roll < 0.14 {
                Terrain::Door
            } else if roll < 0.20 {
                Terrain::Courtyard
            } else if roll < 0.25 {
                Terrain::Garden
            } else if roll < 0.27 {
                Terrain::Bridge
            } else {
                Terrain::Road
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
    fn test_load_archetypes_from_file() {
        let map = load_archetypes("data/archetypes.kdl");
        assert!(map.contains_key("person"), "missing 'person' archetype");
        let person = &map["person"];
        assert_eq!(person.icon, '☻');
        assert_eq!(person.health, 100.0);
        assert_eq!(person.max_hunger, 100.0);
        assert_eq!(person.attack, 10.0);
        assert_eq!(person.defense, 5.0);
        assert_eq!(person.aggression, 0.0);
    }

    #[test]
    fn test_load_archetypes_missing_file() {
        let map = load_archetypes("nonexistent.kdl");
        assert!(map.is_empty());
    }

    #[test]
    fn test_load_terrain_scatters_variety() {
        let mut world = World::new_with_seed(42);
        load_terrain(&mut world, "data/terrain.kdl");
        let w = world.tiles.width();
        let h = world.tiles.height();
        let mut non_road = 0;
        for y in 0..h {
            for x in 0..w {
                if world.tiles.get_terrain(x, y) != Some(Terrain::Road) {
                    non_road += 1;
                }
            }
        }
        assert!(non_road > 0);
    }
}
