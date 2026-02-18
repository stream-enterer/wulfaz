use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use shapefile::dbase::FieldValue;

use crate::registry::{BlockData, BlockId, BuildingData, BuildingId, estimate_floor_count};
use crate::tile_map::{Terrain, TileMap};
use crate::world::World;

// --- Coordinate conversion constants ---
// At lat 48.857°, 1° longitude ≈ 73,490 m, 1° latitude ≈ 111,320 m.
#[allow(dead_code)]
const LAT_CENTER: f64 = 48.857;
const M_PER_DEG_LON: f64 = 111_320.0 * 0.6579; // cos(48.857°) ≈ 0.6579
const M_PER_DEG_LAT: f64 = 111_320.0;
const PAD: f64 = 30.0; // meters padding on all sides

// Viewport bounds: outermost building vertices.
const VIEW_MIN_LON: f64 = 2.298_146_8;
const VIEW_MAX_LON: f64 = 2.384_218_3;
const VIEW_MIN_LAT: f64 = 48.841_093_9;
const VIEW_MAX_LAT: f64 = 48.883_751_7;

/// Convert lon/lat to tile coordinates (meters from top-left origin).
fn lonlat_to_tile(lon: f64, lat: f64) -> (f64, f64) {
    let x = (lon - VIEW_MIN_LON) * M_PER_DEG_LON + PAD;
    let y = (VIEW_MAX_LAT - lat) * M_PER_DEG_LAT + PAD;
    (x, y)
}

/// Compute grid dimensions from the viewport bounds + padding.
fn compute_grid_size() -> (usize, usize) {
    let w = ((VIEW_MAX_LON - VIEW_MIN_LON) * M_PER_DEG_LON).ceil() as usize + PAD as usize * 2;
    let h = ((VIEW_MAX_LAT - VIEW_MIN_LAT) * M_PER_DEG_LAT).ceil() as usize + PAD as usize * 2;
    (w, h)
}

/// Convert a shapefile polygon ring (lon/lat points) to tile-space coordinates.
/// Only converts the outer ring (first part).
fn polygon_to_meters(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    points
        .iter()
        .map(|&(lon, lat)| lonlat_to_tile(lon, lat))
        .collect()
}

/// Scanline polygon rasterization using even-odd fill rule.
/// Returns all tile coordinates (x, y) inside the polygon.
pub fn scanline_fill(ring: &[(f64, f64)], width: usize, height: usize) -> Vec<(i32, i32)> {
    let mut filled = Vec::new();
    if ring.len() < 3 {
        return filled;
    }

    let ys: Vec<f64> = ring.iter().map(|p| p.1).collect();
    let min_row = (ys.iter().cloned().fold(f64::INFINITY, f64::min).floor() as i32).max(0);
    let max_row =
        (ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max).ceil() as i32).min(height as i32 - 1);

    let n = ring.len();

    for row in min_row..=max_row {
        let y = row as f64 + 0.5;
        let mut intersections = Vec::new();
        let mut j = n - 1;
        for i in 0..n {
            let yi = ring[i].1;
            let yj = ring[j].1;
            if (yi > y) != (yj > y) {
                let xi = ring[i].0;
                let xj = ring[j].0;
                let x_int = xi + (y - yi) / (yj - yi) * (xj - xi);
                intersections.push(x_int);
            }
            j = i;
        }
        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut k = 0;
        while k + 1 < intersections.len() {
            let x_start = (intersections[k] - 0.5).ceil().max(0.0) as i32;
            let x_end = (intersections[k + 1] - 0.5)
                .floor()
                .min((width as i32 - 1) as f64) as i32;
            for col in x_start..=x_end {
                filled.push((col, row));
            }
            k += 2;
        }
    }

    filled
}

/// Check if a polygon's bounding box overlaps the viewport.
fn bbox_overlaps(points: &[(f64, f64)]) -> bool {
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    for &(lon, lat) in points {
        if lon < min_lon {
            min_lon = lon;
        }
        if lon > max_lon {
            max_lon = lon;
        }
        if lat < min_lat {
            min_lat = lat;
        }
        if lat > max_lat {
            max_lat = lat;
        }
    }
    !(max_lon < VIEW_MIN_LON
        || min_lon > VIEW_MAX_LON
        || max_lat < VIEW_MIN_LAT
        || min_lat > VIEW_MAX_LAT)
}

/// Extract the outer ring points from a shapefile polygon shape.
/// Returns lon/lat pairs for the first ring only.
fn extract_outer_ring(shape: &shapefile::Polygon) -> Vec<(f64, f64)> {
    // The first ring in the polygon is the outer ring.
    if let Some(ring) = shape.rings().first() {
        ring.points().iter().map(|p| (p.x, p.y)).collect()
    } else {
        Vec::new()
    }
}

/// Helper: extract a string field from a dbase record, returning empty string on missing/null.
fn get_string_field(record: &shapefile::dbase::Record, name: &str) -> String {
    match record.get(name) {
        Some(FieldValue::Character(Some(s))) => s.trim().to_string(),
        Some(FieldValue::Memo(s)) => s.trim().to_string(),
        _ => String::new(),
    }
}

/// Helper: extract a numeric field as f64 from a dbase record.
fn get_numeric_field(record: &shapefile::dbase::Record, name: &str) -> f64 {
    match record.get(name) {
        Some(FieldValue::Numeric(Some(v))) => *v,
        Some(FieldValue::Float(Some(v))) => *v as f64,
        Some(FieldValue::Double(v)) => *v,
        Some(FieldValue::Integer(v)) => *v as f64,
        _ => 0.0,
    }
}

/// Helper: extract an integer field from a dbase record.
fn get_integer_field(record: &shapefile::dbase::Record, name: &str) -> i32 {
    match record.get(name) {
        Some(FieldValue::Integer(v)) => *v,
        Some(FieldValue::Numeric(Some(v))) => *v as i32,
        _ => 0,
    }
}

/// Main entry point: load GIS shapefiles and populate world terrain + registries.
pub fn load_gis(world: &mut World, buildings_shp: &str, blocks_shp: &str) {
    let total_start = Instant::now();

    // Compute grid dimensions and create TileMap.
    let (grid_w, grid_h) = compute_grid_size();
    log::info!(
        "GIS grid: {}×{} tiles ({} chunks)",
        grid_w,
        grid_h,
        (grid_w.div_ceil(64)) * (grid_h.div_ceil(64))
    );
    world.tiles = TileMap::new(grid_w, grid_h);

    // --- Phase 1: Load blocks ---
    let block_start = Instant::now();
    load_blocks(world, blocks_shp, grid_w, grid_h);
    log::info!(
        "Blocks loaded in {:.1}s",
        block_start.elapsed().as_secs_f64()
    );

    // --- Phase 2: Load buildings ---
    let bldg_start = Instant::now();
    load_buildings(world, buildings_shp, grid_w, grid_h);
    log::info!(
        "Buildings loaded in {:.1}s",
        bldg_start.elapsed().as_secs_f64()
    );

    // --- Phase 3: Classify wall/floor ---
    let class_start = Instant::now();
    classify_walls_floors(world);
    log::info!(
        "Wall/floor classification in {:.1}s",
        class_start.elapsed().as_secs_f64()
    );

    // --- Phase 4: Quartier BFS for road tiles ---
    let bfs_start = Instant::now();
    fill_quartier_roads(world, grid_w, grid_h);
    log::info!("Quartier BFS in {:.1}s", bfs_start.elapsed().as_secs_f64());

    log::info!(
        "GIS loading complete in {:.1}s: {} blocks, {} buildings, {} quartiers",
        total_start.elapsed().as_secs_f64(),
        world.blocks.blocks.len(),
        world.buildings.buildings.len(),
        world.quartier_names.len(),
    );
}

/// Load block polygons from Vasserot_Ilots.shp.
/// Marks tiles as Courtyard, sets block_id and quartier_id.
fn load_blocks(world: &mut World, blocks_shp: &str, grid_w: usize, grid_h: usize) {
    let mut reader = shapefile::Reader::from_path(blocks_shp)
        .unwrap_or_else(|e| panic!("Failed to open {blocks_shp}: {e}"));

    let mut quartier_map: HashMap<String, u8> = HashMap::new();
    let mut next_block_id: u16 = 1;
    let mut total_block_tiles = 0usize;

    for result in reader.iter_shapes_and_records() {
        let (shape, record) = result.unwrap_or_else(|e| panic!("Error reading block record: {e}"));

        let polygon = match shape {
            shapefile::Shape::Polygon(p) => p,
            _ => continue,
        };

        let outer = extract_outer_ring(&polygon);
        if outer.is_empty() || !bbox_overlaps(&outer) {
            continue;
        }

        let ring = polygon_to_meters(&outer);
        let cells = scanline_fill(&ring, grid_w, grid_h);
        if cells.is_empty() {
            continue;
        }

        let id_ilots = get_string_field(&record, "ID_ILOTS");
        let quartier = get_string_field(&record, "QUARTIER");
        let aire = get_numeric_field(&record, "AIRE") as f32;

        // Assign quartier_id (1-based).
        let next_qid = quartier_map.len() as u8 + 1;
        let quartier_id = *quartier_map.entry(quartier.clone()).or_insert(next_qid);

        let block_id = BlockId(next_block_id);
        next_block_id += 1;

        for &(cx, cy) in &cells {
            let ux = cx as usize;
            let uy = cy as usize;
            world.tiles.set_terrain(ux, uy, Terrain::Courtyard);
            world.tiles.set_block_id(ux, uy, block_id);
            world.tiles.set_quartier_id(ux, uy, quartier_id);
        }
        total_block_tiles += cells.len();

        world.blocks.insert(BlockData {
            id: block_id,
            id_ilots,
            quartier,
            aire,
            buildings: Vec::new(),
        });
    }

    // Build quartier_names from the map (indexed by quartier_id - 1).
    let mut names = vec![String::new(); quartier_map.len()];
    for (name, &id) in &quartier_map {
        names[(id - 1) as usize] = name.clone();
    }
    world.quartier_names = names;

    log::info!(
        "  {} blocks, {} block tiles, {} quartiers",
        world.blocks.blocks.len(),
        total_block_tiles,
        world.quartier_names.len(),
    );
}

/// Load building polygons from BATI.shp.
/// Marks tiles with building_id, overwrites terrain temporarily (classified later).
fn load_buildings(world: &mut World, buildings_shp: &str, grid_w: usize, grid_h: usize) {
    let mut reader = shapefile::Reader::from_path(buildings_shp)
        .unwrap_or_else(|e| panic!("Failed to open {buildings_shp}: {e}"));

    let mut total_building_tiles = 0usize;

    for result in reader.iter_shapes_and_records() {
        let (shape, record) =
            result.unwrap_or_else(|e| panic!("Error reading building record: {e}"));

        let polygon = match shape {
            shapefile::Shape::Polygon(p) => p,
            _ => continue,
        };

        let outer = extract_outer_ring(&polygon);
        if outer.is_empty() || !bbox_overlaps(&outer) {
            continue;
        }

        let ring = polygon_to_meters(&outer);
        let cells = scanline_fill(&ring, grid_w, grid_h);
        if cells.is_empty() {
            continue;
        }

        let identif = get_integer_field(&record, "Identif") as u32;
        let quartier = get_string_field(&record, "QUARTIER");
        let superficie = get_numeric_field(&record, "SUPERFICIE") as f32;
        let bati = get_integer_field(&record, "BATI") as u8;
        let nom_bati_raw = get_string_field(&record, "Nom_Bati");
        let nom_bati = if nom_bati_raw.is_empty() {
            None
        } else {
            Some(nom_bati_raw)
        };
        let num_ilot = get_string_field(&record, "NUM_ILOT");

        let building_id = BuildingId(identif);
        let floor_count = estimate_floor_count(superficie);

        // Determine which block this building sits in (from block_id already set on tiles).
        let mut block_for_building: Option<BlockId> = None;
        for &(cx, cy) in &cells {
            if let Some(bid) = world.tiles.get_block_id(cx as usize, cy as usize) {
                block_for_building = Some(bid);
                break;
            }
        }

        // Mark tiles: set building_id, set terrain to Wall (temporary — classified later).
        let mut tile_list = Vec::with_capacity(cells.len());
        for &(cx, cy) in &cells {
            let ux = cx as usize;
            let uy = cy as usize;
            world.tiles.set_terrain(ux, uy, Terrain::Wall);
            world.tiles.set_building_id(ux, uy, building_id);

            // Inherit quartier from block if not already set.
            if world.tiles.get_quartier_id(ux, uy) == Some(0) {
                // Look up quartier from the building record itself.
                // (This shouldn't normally happen since blocks are loaded first.)
            }

            tile_list.push((cx, cy));
        }
        total_building_tiles += cells.len();

        // Add building to its block's buildings list.
        if let Some(bid) = block_for_building
            && let Some(block) = world.blocks.blocks.get_mut(&bid)
        {
            block.buildings.push(building_id);
        }

        world.buildings.insert(BuildingData {
            id: building_id,
            quartier,
            superficie,
            bati,
            nom_bati,
            num_ilot,
            floor_count,
            tiles: tile_list,
            addresses: Vec::new(),
            occupants: Vec::new(),
        });
    }

    log::info!(
        "  {} buildings, {} building tiles",
        world.buildings.buildings.len(),
        total_building_tiles,
    );
}

/// Classify building tiles into Wall vs Floor.
/// A tile is Wall if any cardinal neighbor is not in the same building.
/// Otherwise it's Floor.
fn classify_walls_floors(world: &mut World) {
    let mut wall_count = 0usize;
    let mut floor_count = 0usize;
    let grid_w = world.tiles.width() as i32;
    let grid_h = world.tiles.height() as i32;

    // Collect all (tile, building_id) pairs, then classify.
    let all_tiles: Vec<(i32, i32, BuildingId)> = world
        .buildings
        .buildings
        .values()
        .flat_map(|b| b.tiles.iter().map(move |&(x, y)| (x, y, b.id)))
        .collect();

    for (cx, cy, bid) in all_tiles {
        let mut is_edge = false;
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || nx >= grid_w || ny >= grid_h {
                is_edge = true;
                break;
            }
            let neighbor_bid = world.tiles.get_building_id(nx as usize, ny as usize);
            if neighbor_bid != Some(bid) {
                is_edge = true;
                break;
            }
        }

        let terrain = if is_edge {
            wall_count += 1;
            Terrain::Wall
        } else {
            floor_count += 1;
            Terrain::Floor
        };
        world.tiles.set_terrain(cx as usize, cy as usize, terrain);
    }

    log::info!("  {} wall tiles, {} floor tiles", wall_count, floor_count);
}

/// Multi-source BFS to assign quartier_id to road tiles.
/// Expands from all tiles that already have quartier_id != 0.
fn fill_quartier_roads(world: &mut World, grid_w: usize, grid_h: usize) {
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    let mut assigned = 0usize;

    // Seed: all tiles with quartier_id already set.
    for y in 0..grid_h {
        for x in 0..grid_w {
            if let Some(qid) = world.tiles.get_quartier_id(x, y)
                && qid != 0
            {
                queue.push_back((x, y));
            }
        }
    }

    while let Some((x, y)) = queue.pop_front() {
        let qid = world.tiles.get_quartier_id(x, y).unwrap_or(0);
        if qid == 0 {
            continue;
        }

        for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= grid_w as i32 || ny >= grid_h as i32 {
                continue;
            }
            let nux = nx as usize;
            let nuy = ny as usize;
            if let Some(nqid) = world.tiles.get_quartier_id(nux, nuy)
                && nqid == 0
            {
                world.tiles.set_quartier_id(nux, nuy, qid);
                assigned += 1;
                queue.push_back((nux, nuy));
            }
        }
    }

    log::info!("  {} road tiles assigned quartier via BFS", assigned);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanline_fill_triangle() {
        // Triangle with vertices at (5,0), (0,10), (10,10)
        let ring = vec![(5.0, 0.0), (0.0, 10.0), (10.0, 10.0), (5.0, 0.0)];
        let tiles = scanline_fill(&ring, 20, 20);
        assert!(!tiles.is_empty());
        // All tiles should be within bounds
        for &(x, y) in &tiles {
            assert!(x >= 0 && x < 20);
            assert!(y >= 0 && y < 20);
        }
        // Row 5 (middle) should have tiles — triangle widens going down
        let row5: Vec<i32> = tiles.iter().filter(|t| t.1 == 5).map(|t| t.0).collect();
        assert!(!row5.is_empty());
        // Row 9 (near base) should be wider than row 2 (near apex)
        let row9: Vec<i32> = tiles.iter().filter(|t| t.1 == 9).map(|t| t.0).collect();
        let row2: Vec<i32> = tiles.iter().filter(|t| t.1 == 2).map(|t| t.0).collect();
        assert!(
            row9.len() > row2.len(),
            "row9={} should be wider than row2={}",
            row9.len(),
            row2.len()
        );
    }

    #[test]
    fn test_scanline_fill_square() {
        // 10x10 square from (0,0) to (10,10)
        let ring = vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ];
        let tiles = scanline_fill(&ring, 20, 20);
        // Should fill approximately 100 tiles (10x10)
        assert!(
            tiles.len() >= 90,
            "Expected ~100 tiles, got {}",
            tiles.len()
        );
        assert!(
            tiles.len() <= 110,
            "Expected ~100 tiles, got {}",
            tiles.len()
        );
    }

    #[test]
    fn test_coordinate_conversion() {
        // VIEW_MIN_LON, VIEW_MAX_LAT should map near (PAD, PAD)
        let (x, y) = lonlat_to_tile(VIEW_MIN_LON, VIEW_MAX_LAT);
        assert!((x - PAD).abs() < 1.0, "x={x} expected near {PAD}");
        assert!((y - PAD).abs() < 1.0, "y={y} expected near {PAD}");

        // VIEW_MAX_LON, VIEW_MIN_LAT should map near (grid_w - PAD, grid_h - PAD)
        let (grid_w, grid_h) = compute_grid_size();
        let (x2, y2) = lonlat_to_tile(VIEW_MAX_LON, VIEW_MIN_LAT);
        assert!(
            (x2 - (grid_w as f64 - PAD)).abs() < 2.0,
            "x2={x2} expected near {}",
            grid_w as f64 - PAD
        );
        assert!(
            (y2 - (grid_h as f64 - PAD)).abs() < 2.0,
            "y2={y2} expected near {}",
            grid_h as f64 - PAD
        );
    }

    #[test]
    fn test_wall_floor_classification() {
        // Create a 5x5 building block — expect 12 wall + 9 floor
        // (actually for a 5x5 grid: perimeter = 16 wall, interior = 9 floor)
        let mut world = World::new_with_seed(42);
        world.tiles = TileMap::new(10, 10);

        let bid = BuildingId(1);
        let mut tile_list = Vec::new();
        for y in 2..7 {
            for x in 2..7 {
                world.tiles.set_terrain(x, y, Terrain::Wall);
                world.tiles.set_building_id(x, y, bid);
                tile_list.push((x as i32, y as i32));
            }
        }

        world.buildings.insert(BuildingData {
            id: bid,
            quartier: "Test".into(),
            superficie: 100.0,
            bati: 1,
            nom_bati: None,
            num_ilot: "T1".into(),
            floor_count: 3,
            tiles: tile_list,
            addresses: Vec::new(),
            occupants: Vec::new(),
        });

        classify_walls_floors(&mut world);

        // Count walls and floors
        let mut walls = 0;
        let mut floors = 0;
        for y in 2..7 {
            for x in 2..7 {
                match world.tiles.get_terrain(x, y) {
                    Some(Terrain::Wall) => walls += 1,
                    Some(Terrain::Floor) => floors += 1,
                    other => panic!("unexpected terrain at ({x},{y}): {other:?}"),
                }
            }
        }
        assert_eq!(walls, 16, "5x5 building perimeter = 16 walls");
        assert_eq!(floors, 9, "5x5 building interior = 9 floors");
    }

    #[test]
    fn test_grid_size_matches_python() {
        let (w, h) = compute_grid_size();
        // Python reference: grid_w ≈ 6369, grid_h ≈ 4810 (with PAD=2)
        // Our PAD=30, so grid is wider by 56 on each axis.
        // Expected: ~6309+60 = ~6369, ~4753+60 = ~4813
        assert!(w > 6000 && w < 7000, "grid_w={w}");
        assert!(h > 4500 && h < 5500, "grid_h={h}");
    }

    #[test]
    fn test_scanline_fill_degenerate() {
        // Less than 3 points — no fill
        let ring = vec![(0.0, 0.0), (1.0, 1.0)];
        assert!(scanline_fill(&ring, 10, 10).is_empty());

        // Empty ring
        assert!(scanline_fill(&[], 10, 10).is_empty());
    }

    #[test]
    fn test_scanline_fill_out_of_bounds_clamped() {
        // Polygon partially outside grid
        let ring = vec![
            (-5.0, -5.0),
            (5.0, -5.0),
            (5.0, 5.0),
            (-5.0, 5.0),
            (-5.0, -5.0),
        ];
        let tiles = scanline_fill(&ring, 10, 10);
        // Should only produce tiles within [0, 10)
        for &(x, y) in &tiles {
            assert!(x >= 0 && x < 10, "x={x} out of bounds");
            assert!(y >= 0 && y < 10, "y={y} out of bounds");
        }
        // Should fill approximately 5x5 = 25 tiles (the in-bounds quarter)
        assert!(
            tiles.len() >= 20 && tiles.len() <= 30,
            "len={}",
            tiles.len()
        );
    }
}
