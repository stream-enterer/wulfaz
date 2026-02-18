use std::collections::BTreeMap;

use crate::components::Name;
use crate::events::Event;
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
        for (&entity, pos) in &world.positions {
            if !world.alive.contains(&entity) {
                continue;
            }
            let is_creature = world.combat_stats.contains_key(&entity);
            if (pass == 0) == is_creature {
                continue; // pass 0: items only; pass 1: creatures only
            }
            if let Some(icon) = world.icons.get(&entity) {
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

    if let Some(player) = world.player
        && let Some(name) = world.names.get(&player)
    {
        status.push_str(&format!(" | @{}", name.value));
    }

    status
}

/// Render recent significant events as a multi-line string.
///
/// Over-fetches from the ring buffer and filters to significant events only
/// (Spawned, Died, Ate, Attacked). Moved and HungerChanged are skipped.
/// Uses a friendlier format without tick brackets.
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

    // Over-fetch to find enough significant events
    let events = world.events.recent(count * 10);
    let mut lines: Vec<String> = Vec::new();

    for event in events {
        let line = match event {
            Event::Spawned { entity, .. } => {
                format!("{} spawned", resolve(entity))
            }
            Event::Died { entity, .. } => {
                format!("{} died", resolve(entity))
            }
            Event::Ate { entity, food, .. } => {
                format!("{} ate {}", resolve(entity), resolve(food))
            }
            Event::Attacked {
                attacker,
                defender,
                damage,
                ..
            } => {
                format!(
                    "{} attacks {} ({:.0} dmg)",
                    resolve(attacker),
                    resolve(defender),
                    damage
                )
            }
            // Skip noisy events
            Event::Moved { .. } | Event::HungerChanged { .. } => continue,
        };
        lines.push(line);
        if lines.len() >= count {
            break;
        }
    }

    lines.join("\n")
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
        world.tiles = crate::tile_map::TileMap::new(8, 1);
        world.tiles.set_terrain(0, 0, Terrain::Road);
        world.tiles.set_terrain(1, 0, Terrain::Wall);
        world.tiles.set_terrain(2, 0, Terrain::Floor);
        world.tiles.set_terrain(3, 0, Terrain::Door);
        world.tiles.set_terrain(4, 0, Terrain::Courtyard);
        world.tiles.set_terrain(5, 0, Terrain::Garden);
        world.tiles.set_terrain(6, 0, Terrain::Water);
        world.tiles.set_terrain(7, 0, Terrain::Bridge);

        let output = render_full(&world);
        assert_eq!(output, ".#_+,\"~=");
    }

    #[test]
    fn entity_icon_overlays_terrain() {
        let mut world = World::new_with_seed(42);
        world.tiles = crate::tile_map::TileMap::new(5, 5);

        let e = world.spawn();
        world.positions.insert(e, Position { x: 2, y: 3 });
        world.icons.insert(e, Icon { ch: 'g' });

        let output = render_full(&world);
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
        world.positions.insert(e, Position { x: 2, y: 2 });
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
        world.positions.insert(e, Position { x: 10, y: 10 });
        world.icons.insert(e, Icon { ch: 'X' });

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
        world.positions.insert(e, Position { x: -1, y: -3 });
        world.icons.insert(e, Icon { ch: 'N' });

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
        world.positions.insert(item, Position { x: 2, y: 2 });
        world.icons.insert(item, Icon { ch: '/' });

        // Creature (has combat_stats) — drawn on top
        let creature = world.spawn();
        world.positions.insert(creature, Position { x: 2, y: 2 });
        world.icons.insert(creature, Icon { ch: 'g' });
        world.combat_stats.insert(
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
        world.positions.insert(item, Position { x: 2, y: 2 });
        world.icons.insert(item, Icon { ch: '/' });

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
        world.positions.insert(e, Position { x: 6, y: 6 });
        world.icons.insert(e, Icon { ch: 'g' });

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
        assert_eq!(output, "Goblin attacks Troll (12 dmg)");
    }

    #[test]
    fn recent_events_empty_log() {
        let world = World::new_with_seed(42);
        let output = render_recent_events(&world, 5);
        assert!(output.is_empty());
    }
}
