use crate::tile_map::Terrain;
use crate::world::World;

/// Convert a terrain tile to its display character.
fn terrain_char(terrain: Terrain) -> char {
    match terrain {
        Terrain::Road => '.',
        Terrain::Wall => '#',
        Terrain::Floor => '_',
        Terrain::Door => '+',
        Terrain::Courtyard => ',',
        Terrain::Garden => '"',
        Terrain::Water => '~',
        Terrain::Bridge => '=',
        Terrain::Fixture => 'o',
    }
}

/// Render a viewport of the simulation world as a text grid string.
///
/// `cam_x` and `cam_y` are the world coordinates of the top-left visible tile.
/// `viewport_cols` and `viewport_rows` define the size of the visible area.
/// World coordinates outside the tilemap render as spaces. Entity icons are
/// overlaid if they fall within the viewport rectangle. Only alive entities
/// are rendered.
///
/// This function is READ-ONLY and does not modify world state.
pub fn render_world_to_string(
    world: &World,
    cam_x: i32,
    cam_y: i32,
    viewport_cols: usize,
    viewport_rows: usize,
) -> String {
    if viewport_cols == 0 || viewport_rows == 0 {
        return String::new();
    }

    let map_w = world.tiles.width();
    let map_h = world.tiles.height();

    // Build the terrain grid for the viewport.
    let mut grid: Vec<Vec<char>> = Vec::with_capacity(viewport_rows);
    for vy in 0..viewport_rows {
        let mut row = Vec::with_capacity(viewport_cols);
        for vx in 0..viewport_cols {
            let wx = cam_x + vx as i32;
            let wy = cam_y + vy as i32;
            if wx >= 0 && wy >= 0 {
                let ux = wx as usize;
                let uy = wy as usize;
                if ux < map_w && uy < map_h {
                    if let Some(terrain) = world.tiles.get_terrain(ux, uy) {
                        row.push(terrain_char(terrain));
                    } else {
                        row.push(' ');
                    }
                } else {
                    row.push(' ');
                }
            } else {
                row.push(' ');
            }
        }
        grid.push(row);
    }

    // Overlay alive entity icons in two passes: items first, creatures on top.
    // An entity is a "creature" if it has combat_stats; otherwise it's an item.
    // This ensures creatures are always visible when sharing a tile with items.
    for pass in 0..2 {
        for (&entity, pos) in &world.body.positions {
            if !world.alive.contains(&entity) {
                continue;
            }
            let is_creature = world.body.combat_stats.contains_key(&entity);
            if (pass == 0) == is_creature {
                continue; // pass 0: items only; pass 1: creatures only
            }
            if let Some(icon) = world.body.icons.get(&entity) {
                let vx = pos.x - cam_x;
                let vy = pos.y - cam_y;
                if vx >= 0 && vy >= 0 {
                    let vxu = vx as usize;
                    let vyu = vy as usize;
                    if vxu < viewport_cols && vyu < viewport_rows {
                        grid[vyu][vxu] = icon.ch;
                    }
                }
            }
        }
    }

    // Build the final string. Each row is one line, no trailing newline.
    let mut result = String::with_capacity((viewport_cols + 1) * viewport_rows);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::tile_map::Terrain;
    use crate::world::World;

    /// Helper: render full map (backward-compat wrapper for tests).
    fn render_full(world: &World) -> String {
        let w = world.tiles.width();
        let h = world.tiles.height();
        render_world_to_string(world, 0, 0, w, h)
    }

    #[test]
    fn empty_world_renders_all_road() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(64, 64);
        let output = render_full(&world);
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
        world.tiles = crate::tile_map::TileMap::new(9, 1);
        world.tiles.set_terrain(0, 0, Terrain::Road);
        world.tiles.set_terrain(1, 0, Terrain::Wall);
        world.tiles.set_terrain(2, 0, Terrain::Floor);
        world.tiles.set_terrain(3, 0, Terrain::Door);
        world.tiles.set_terrain(4, 0, Terrain::Courtyard);
        world.tiles.set_terrain(5, 0, Terrain::Garden);
        world.tiles.set_terrain(6, 0, Terrain::Water);
        world.tiles.set_terrain(7, 0, Terrain::Bridge);
        world.tiles.set_terrain(8, 0, Terrain::Fixture);

        let output = render_full(&world);
        assert_eq!(output, ".#_+,\"~=o");
    }

    #[test]
    fn entity_icon_overlays_terrain() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 2, y: 3 });
        world.body.icons.insert(e, Icon { ch: 'g' });

        let output = render_full(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[3].chars().nth(2), Some('g'));
        // Other cells remain terrain.
        assert_eq!(lines[3].chars().next(), Some('.'));
    }

    #[test]
    fn dead_entities_not_rendered() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 1, y: 1 });
        world.body.icons.insert(e, Icon { ch: 'T' });
        world.alive.remove(&e); // Entity is dead.

        let output = render_full(&world);
        let lines: Vec<&str> = output.lines().collect();
        // Should show terrain, not the entity icon.
        assert_eq!(lines[1].chars().nth(1), Some('.'));
    }

    #[test]
    fn entity_without_icon_not_rendered() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 2, y: 2 });
        // No icon inserted.

        let output = render_full(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[2].chars().nth(2), Some('.'));
    }

    #[test]
    fn entity_outside_bounds_not_rendered() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 10, y: 10 });
        world.body.icons.insert(e, Icon { ch: 'X' });

        // Should not panic, entity is just off-grid.
        let output = render_full(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn entity_at_negative_position_not_rendered() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.body.positions.insert(e, Position { x: -1, y: -3 });
        world.body.icons.insert(e, Icon { ch: 'N' });

        // Should not panic.
        let output = render_full(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn zero_size_viewport_returns_empty_string() {
        let world = World::new_with_seed(42);
        let output = render_world_to_string(&world, 0, 0, 0, 0);
        assert!(output.is_empty());
    }

    #[test]
    fn creature_renders_on_top_of_item() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        // Item (no combat_stats) — drawn first
        let item = world.spawn();
        world.body.positions.insert(item, Position { x: 2, y: 2 });
        world.body.icons.insert(item, Icon { ch: '/' });

        // Creature (has combat_stats) — drawn on top
        let creature = world.spawn();
        world
            .body
            .positions
            .insert(creature, Position { x: 2, y: 2 });
        world.body.icons.insert(creature, Icon { ch: 'g' });
        world.body.combat_stats.insert(
            creature,
            CombatStats {
                attack: 10.0,
                defense: 5.0,
                aggression: 0.8,
            },
        );

        let output = render_full(&world);
        let lines: Vec<&str> = output.lines().collect();
        // Creature always wins
        assert_eq!(lines[2].chars().nth(2), Some('g'));
    }

    #[test]
    fn item_visible_when_no_creature() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let item = world.spawn();
        world.body.positions.insert(item, Position { x: 2, y: 2 });
        world.body.icons.insert(item, Icon { ch: '/' });

        let output = render_full(&world);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines[2].chars().nth(2), Some('/'));
    }

    #[test]
    fn viewport_camera_offset() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(10, 10);
        world.tiles.set_terrain(5, 5, Terrain::Water);

        let e = world.spawn();
        world.body.positions.insert(e, Position { x: 6, y: 6 });
        world.body.icons.insert(e, Icon { ch: 'g' });

        // Camera at (3,3), viewport 5x5 => shows world (3..8, 3..8)
        let output = render_world_to_string(&world, 3, 3, 5, 5);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 5);
        // Water at world (5,5) => viewport (2,2)
        assert_eq!(lines[2].chars().nth(2), Some('~'));
        // Entity at world (6,6) => viewport (3,3)
        assert_eq!(lines[3].chars().nth(3), Some('g'));
    }

    #[test]
    fn viewport_beyond_map_shows_spaces() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(3, 3);

        // Camera at (1,1), viewport 4x4 => world coords (1..5, 1..5)
        // Map is 3x3, so cols 3+ and rows 3+ are out of bounds
        let output = render_world_to_string(&world, 1, 1, 4, 4);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 4);
        // Row 0: world y=1, x=1..4 => two in-bounds road, two spaces
        assert_eq!(lines[0], "..  ");
        // Row 2: world y=3, all out of bounds
        assert_eq!(lines[2], "    ");
        assert_eq!(lines[3], "    ");
    }
}
