use std::collections::{BTreeMap, HashMap};

use crate::components::Name;
use crate::events::Event;
use crate::tile_map::Terrain;
use crate::world::World;

/// Convert a terrain tile to its display character.
fn terrain_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Grass => '.',
        Terrain::Water => '~',
        Terrain::Stone => '#',
        Terrain::Dirt => ',',
        Terrain::Sand => ':',
    }
}

/// Render the simulation world as a text grid string.
///
/// Places terrain characters as background, then overlays entity icons at
/// their positions. Only alive entities are rendered. Entities with positions
/// outside the tile grid or missing an icon are skipped.
///
/// This function is READ-ONLY and does not modify world state.
pub fn render_world_to_string(world: &World) -> String {
    let width = world.tiles.width();
    let height = world.tiles.height();

    if width == 0 || height == 0 {
        return String::new();
    }

    // Build the terrain grid.
    let mut grid: Vec<Vec<char>> = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = Vec::with_capacity(width);
        for x in 0..width {
            if let Some(terrain) = world.tiles.get_terrain(x, y) {
                row.push(terrain_char(terrain));
            } else {
                row.push(' ');
            }
        }
        grid.push(row);
    }

    // Build a position-to-icon map for alive entities, so last-inserted wins.
    let mut entity_icons: HashMap<(usize, usize), char> = HashMap::new();
    for (&entity, pos) in &world.positions {
        if !world.alive.contains(&entity) {
            continue;
        }
        if let Some(icon) = world.icons.get(&entity) {
            // Convert i32 position to usize grid coordinates, skip if negative
            // or out of bounds.
            if pos.x >= 0 && pos.y >= 0 {
                let ux = pos.x as usize;
                let uy = pos.y as usize;
                if ux < width && uy < height {
                    entity_icons.insert((ux, uy), icon.ch);
                }
            }
        }
    }

    // Overlay entity icons onto the grid.
    for ((x, y), ch) in &entity_icons {
        grid[*y][*x] = *ch;
    }

    // Build the final string. Each row is one line, no trailing newline.
    let mut result = String::with_capacity((width + 1) * height);
    for (i, row) in grid.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        for &ch in row {
            result.push(ch);
        }
    }

    result
}

/// Render a status line showing current tick, entity count, and name breakdown.
///
/// Format: "Tick: N | Entities: M | Name1:count Name2:count"
///
/// This function is READ-ONLY and does not modify world state.
pub fn render_status(world: &World) -> String {
    let mut status = format!("Tick: {} | Entities: {}", world.tick.0, world.alive.len());

    let mut name_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for (&entity, Name { value }) in &world.names {
        if world.alive.contains(&entity) {
            *name_counts.entry(value.as_str()).or_insert(0) += 1;
        }
    }
    if !name_counts.is_empty() {
        status.push_str(" | ");
        let parts: Vec<String> = name_counts
            .iter()
            .map(|(name, count)| format!("{name}:{count}"))
            .collect();
        status.push_str(&parts.join(" "));
    }

    status
}

/// Render recent events as a multi-line string.
///
/// Uses `world.events.recent(count)` to get the most recent events, then
/// formats each one. Entity names are resolved via `world.names`, with a
/// fallback of `"E{id}"` for despawned entities.
///
/// This function is READ-ONLY and does not modify world state.
pub fn render_recent_events(world: &World, count: usize) -> String {
    if world.events.is_empty() {
        return String::new();
    }

    let resolve = |e: &crate::components::Entity| -> String {
        world
            .names
            .get(e)
            .map(|n| n.value.clone())
            .unwrap_or_else(|| format!("E{}", e.0))
    };

    let events = world.events.recent(count);
    let mut lines: Vec<String> = Vec::with_capacity(events.len());

    for event in events {
        let line = match event {
            Event::Spawned { entity, tick } => {
                format!("[{}] {} spawned", tick.0, resolve(entity))
            }
            Event::Died { entity, tick } => {
                format!("[{}] {} died", tick.0, resolve(entity))
            }
            Event::Moved { entity, x, y, tick } => {
                format!("[{}] {} moved to ({},{})", tick.0, resolve(entity), x, y)
            }
            Event::Ate { entity, food, tick } => {
                format!("[{}] {} ate {}", tick.0, resolve(entity), resolve(food))
            }
            Event::Attacked {
                attacker,
                defender,
                damage,
                tick,
            } => {
                format!(
                    "[{}] {} attacked {} for {:.0} dmg",
                    tick.0,
                    resolve(attacker),
                    resolve(defender),
                    damage
                )
            }
            Event::HungerChanged {
                entity,
                old,
                new_val,
                tick,
            } => {
                format!(
                    "[{}] {} hunger {:.0}->{:.0}",
                    tick.0,
                    resolve(entity),
                    old,
                    new_val
                )
            }
        };
        lines.push(line);
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::tile_map::Terrain;
    use crate::world::World;

    #[test]
    fn empty_world_renders_all_grass() {
        let world = World::new_with_seed(42);
        let output = render_world_to_string(&world);
        // Default TileMap is 64x64, all Grass.
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 64);
        for line in &lines {
            assert_eq!(line.len(), 64);
            assert!(line.chars().all(|c| c == '.'));
        }
    }

    #[test]
    fn terrain_types_render_correctly() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 1);
        world.tiles.set_terrain(0, 0, Terrain::Grass);
        world.tiles.set_terrain(1, 0, Terrain::Water);
        world.tiles.set_terrain(2, 0, Terrain::Stone);
        world.tiles.set_terrain(3, 0, Terrain::Dirt);
        world.tiles.set_terrain(4, 0, Terrain::Sand);

        let output = render_world_to_string(&world);
        assert_eq!(output, ".~#,:");
    }

    #[test]
    fn entity_icon_overlays_terrain() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.positions.insert(e, Position { x: 2, y: 3 });
        world.icons.insert(e, Icon { ch: 'g' });

        let output = render_world_to_string(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[3].chars().nth(2), Some('g'));
        // Other cells remain terrain.
        assert_eq!(lines[3].chars().nth(0), Some('.'));
    }

    #[test]
    fn dead_entities_not_rendered() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.positions.insert(e, Position { x: 1, y: 1 });
        world.icons.insert(e, Icon { ch: 'T' });
        world.alive.remove(&e); // Entity is dead.

        let output = render_world_to_string(&world);
        let lines: Vec<&str> = output.lines().collect();
        // Should show terrain, not the entity icon.
        assert_eq!(lines[1].chars().nth(1), Some('.'));
    }

    #[test]
    fn entity_without_icon_not_rendered() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.positions.insert(e, Position { x: 2, y: 2 });
        // No icon inserted.

        let output = render_world_to_string(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[2].chars().nth(2), Some('.'));
    }

    #[test]
    fn entity_outside_bounds_not_rendered() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.icons.insert(e, Icon { ch: 'X' });

        // Should not panic, entity is just off-grid.
        let output = render_world_to_string(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn entity_at_negative_position_not_rendered() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.positions.insert(e, Position { x: -1, y: -3 });
        world.icons.insert(e, Icon { ch: 'N' });

        // Should not panic.
        let output = render_world_to_string(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn zero_size_map_returns_empty_string() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(0, 0);

        let output = render_world_to_string(&world);
        assert!(output.is_empty());
    }

    #[test]
    fn multiple_entities_on_same_tile() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e1 = world.spawn();
        world.positions.insert(e1, Position { x: 2, y: 2 });
        world.icons.insert(e1, Icon { ch: 'a' });

        let e2 = world.spawn();
        world.positions.insert(e2, Position { x: 2, y: 2 });
        world.icons.insert(e2, Icon { ch: 'b' });

        let output = render_world_to_string(&world);
        let lines: Vec<&str> = output.lines().collect();
        let ch = lines[2].chars().nth(2).unwrap();
        // One of the two entities will be displayed (HashMap iteration order).
        assert!(ch == 'a' || ch == 'b');
    }

    #[test]
    fn status_format() {
        let mut world = World::new_with_seed(42);
        world.tick = Tick(42);
        let _e1 = world.spawn();
        let _e2 = world.spawn();
        let _e3 = world.spawn();

        let status = render_status(&world);
        // No names assigned, so no name breakdown appended
        assert_eq!(status, "Tick: 42 | Entities: 3");
    }

    #[test]
    fn status_empty_world() {
        let world = World::new_with_seed(42);
        let status = render_status(&world);
        assert_eq!(status, "Tick: 0 | Entities: 0");
    }

    #[test]
    fn status_shows_entity_names() {
        let mut world = World::new_with_seed(42);
        world.tick = Tick(5);
        let e1 = world.spawn();
        world.names.insert(
            e1,
            Name {
                value: "Goblin".to_string(),
            },
        );
        let e2 = world.spawn();
        world.names.insert(
            e2,
            Name {
                value: "Goblin".to_string(),
            },
        );
        let e3 = world.spawn();
        world.names.insert(
            e3,
            Name {
                value: "Troll".to_string(),
            },
        );

        let status = render_status(&world);
        assert_eq!(status, "Tick: 5 | Entities: 3 | Goblin:2 Troll:1");
    }

    #[test]
    fn recent_events_renders_attack() {
        let mut world = World::new_with_seed(42);
        let a = world.spawn();
        world.names.insert(
            a,
            Name {
                value: "Goblin".to_string(),
            },
        );
        let d = world.spawn();
        world.names.insert(
            d,
            Name {
                value: "Troll".to_string(),
            },
        );

        world.events.push(crate::events::Event::Attacked {
            attacker: a,
            defender: d,
            damage: 12.0,
            tick: Tick(3),
        });

        let output = render_recent_events(&world, 5);
        assert_eq!(output, "[3] Goblin attacked Troll for 12 dmg");
    }

    #[test]
    fn recent_events_empty_log() {
        let world = World::new_with_seed(42);
        let output = render_recent_events(&world, 5);
        assert!(output.is_empty());
    }
}
