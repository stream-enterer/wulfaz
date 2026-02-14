use std::collections::HashMap;

use crate::components::*;
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

/// Render a status line showing current tick and entity count.
///
/// Format: "Tick: N | Entities: M"
///
/// This function is READ-ONLY and does not modify world state.
pub fn render_status(world: &World) -> String {
    format!(
        "Tick: {} | Entities: {}",
        world.tick.0,
        world.alive.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(status, "Tick: 42 | Entities: 3");
    }

    #[test]
    fn status_empty_world() {
        let world = World::new_with_seed(42);
        let status = render_status(&world);
        assert_eq!(status, "Tick: 0 | Entities: 0");
    }
}
