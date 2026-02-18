use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use shapefile::dbase::FieldValue;

use crate::registry::{
    BlockData, BlockId, BlockRegistry, BuildingData, BuildingRegistry, estimate_floor_count,
};
use crate::tile_map::{Terrain, TileMap};
use crate::world::World;

// --- RON data types (decoupled from runtime types) ---

#[derive(Serialize, Deserialize)]
pub struct ParisBuildingRon {
    pub identif: u32,
    pub quartier: String,
    pub superficie: f32,
    pub bati: u8,
    pub nom_bati: Option<String>,
    pub num_ilot: String,
    #[serde(default)]
    pub perimetre: f32,
    #[serde(default)]
    pub geox: f64,
    #[serde(default)]
    pub geoy: f64,
    #[serde(default)]
    pub date_coyec: Option<String>,
    pub polygon: Vec<(f64, f64)>,
}

#[derive(Serialize, Deserialize)]
pub struct ParisBlockRon {
    pub id_ilots: String,
    pub quartier: String,
    pub aire: f32,
    #[serde(default)]
    pub ilots_vass: String,
    pub polygon: Vec<(f64, f64)>,
}

#[derive(Serialize, Deserialize)]
pub struct ParisMapRon {
    pub grid_width: usize,
    pub grid_height: usize,
    pub buildings: Vec<ParisBuildingRon>,
    pub blocks: Vec<ParisBlockRon>,
    pub quartier_names: Vec<String>,
}

/// Metadata saved alongside the binary tile file.
/// Contains registry data that doesn't belong in the flat tile arrays.
#[derive(Serialize, Deserialize)]
pub struct ParisMetadataRon {
    pub quartier_names: Vec<String>,
    /// BuildingData with tiles field left empty (reconstructed from binary on load).
    pub buildings: Vec<BuildingData>,
    pub blocks: Vec<BlockData>,
}

// --- RON serialization ---

/// Write ParisMapRon to a RON file. Used by preprocess binary.
#[allow(dead_code)]
pub fn save_paris_ron(data: &ParisMapRon, path: &str) {
    let pretty = ron::ser::PrettyConfig::default();
    let ron_str = ron::ser::to_string_pretty(data, pretty)
        .unwrap_or_else(|e| panic!("Failed to serialize RON: {e}"));
    let mut file =
        std::fs::File::create(path).unwrap_or_else(|e| panic!("Failed to create {path}: {e}"));
    file.write_all(ron_str.as_bytes())
        .unwrap_or_else(|e| panic!("Failed to write {path}: {e}"));
}

/// Read ParisMapRon from a RON file.
pub fn load_paris_ron(path: &str) -> ParisMapRon {
    let ron_str =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
    ron::from_str(&ron_str).unwrap_or_else(|e| panic!("Failed to parse RON from {path}: {e}"))
}

// --- Coordinate conversion (used by preprocess pipeline + tests) ---
// At lat 48.857°, 1° longitude ≈ 73,490 m, 1° latitude ≈ 111,320 m.
#[allow(dead_code)]
const LAT_CENTER: f64 = 48.857;
#[allow(dead_code)]
const M_PER_DEG_LON: f64 = 111_320.0 * 0.6579; // cos(48.857°) ≈ 0.6579
#[allow(dead_code)]
const M_PER_DEG_LAT: f64 = 111_320.0;
#[allow(dead_code)]
const PAD: f64 = 30.0; // meters padding on all sides

#[allow(dead_code)]
const VIEW_MIN_LON: f64 = 2.298_146_8;
#[allow(dead_code)]
const VIEW_MAX_LON: f64 = 2.384_218_3;
#[allow(dead_code)]
const VIEW_MIN_LAT: f64 = 48.841_093_9;
#[allow(dead_code)]
const VIEW_MAX_LAT: f64 = 48.883_751_7;

#[allow(dead_code)]
fn lonlat_to_tile(lon: f64, lat: f64) -> (f64, f64) {
    let x = (lon - VIEW_MIN_LON) * M_PER_DEG_LON + PAD;
    let y = (VIEW_MAX_LAT - lat) * M_PER_DEG_LAT + PAD;
    (x, y)
}

#[allow(dead_code)]
fn compute_grid_size() -> (usize, usize) {
    let w = ((VIEW_MAX_LON - VIEW_MIN_LON) * M_PER_DEG_LON).ceil() as usize + PAD as usize * 2;
    let h = ((VIEW_MAX_LAT - VIEW_MIN_LAT) * M_PER_DEG_LAT).ceil() as usize + PAD as usize * 2;
    (w, h)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn extract_outer_ring(shape: &shapefile::Polygon) -> Vec<(f64, f64)> {
    // The first ring in the polygon is the outer ring.
    if let Some(ring) = shape.rings().first() {
        ring.points().iter().map(|p| (p.x, p.y)).collect()
    } else {
        Vec::new()
    }
}

#[allow(dead_code)]
fn get_string_field(record: &shapefile::dbase::Record, name: &str) -> String {
    match record.get(name) {
        Some(FieldValue::Character(Some(s))) => s.trim().to_string(),
        Some(FieldValue::Memo(s)) => s.trim().to_string(),
        _ => String::new(),
    }
}

#[allow(dead_code)]
fn get_numeric_field(record: &shapefile::dbase::Record, name: &str) -> f64 {
    match record.get(name) {
        Some(FieldValue::Numeric(Some(v))) => *v,
        Some(FieldValue::Float(Some(v))) => *v as f64,
        Some(FieldValue::Double(v)) => *v,
        Some(FieldValue::Integer(v)) => *v as f64,
        _ => 0.0,
    }
}

#[allow(dead_code)]
fn get_integer_field(record: &shapefile::dbase::Record, name: &str) -> i32 {
    match record.get(name) {
        Some(FieldValue::Integer(v)) => *v,
        Some(FieldValue::Numeric(Some(v))) => *v as i32,
        _ => 0,
    }
}

// --- Preprocess pipeline: extract RON from shapefiles ---

#[allow(dead_code)]
fn extract_blocks_from_shapefile(blocks_shp: &str) -> (Vec<ParisBlockRon>, Vec<String>) {
    let mut reader = shapefile::Reader::from_path(blocks_shp)
        .unwrap_or_else(|e| panic!("Failed to open {blocks_shp}: {e}"));

    let mut blocks = Vec::new();
    let mut quartier_set: Vec<String> = Vec::new();

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
        // Skip blocks with no rasterizable area.
        if scanline_fill(&ring, 10000, 10000).is_empty() {
            continue;
        }

        let id_ilots = get_string_field(&record, "ID_ILOTS");
        let quartier = get_string_field(&record, "QUARTIER");
        let aire = get_numeric_field(&record, "AIRE") as f32;
        let ilots_vass = get_string_field(&record, "ILOTS_VASS");

        // Track quartier names.
        if !quartier_set.contains(&quartier) {
            quartier_set.push(quartier.clone());
        }

        blocks.push(ParisBlockRon {
            id_ilots,
            quartier,
            aire,
            ilots_vass,
            polygon: ring,
        });
    }

    log::info!(
        "  Extracted {} blocks, {} quartiers",
        blocks.len(),
        quartier_set.len()
    );
    (blocks, quartier_set)
}

#[allow(dead_code)]
fn extract_buildings_from_shapefile(buildings_shp: &str) -> Vec<ParisBuildingRon> {
    let mut reader = shapefile::Reader::from_path(buildings_shp)
        .unwrap_or_else(|e| panic!("Failed to open {buildings_shp}: {e}"));

    let mut buildings = Vec::new();

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
        if scanline_fill(&ring, 10000, 10000).is_empty() {
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
        let perimetre = get_numeric_field(&record, "PERIMETRE") as f32;
        let geox = get_numeric_field(&record, "GEOX");
        let geoy = get_numeric_field(&record, "GEOY");
        let date_raw = get_string_field(&record, "DATE_COYEC");
        let date_coyec = if date_raw.is_empty() {
            None
        } else {
            Some(date_raw)
        };

        buildings.push(ParisBuildingRon {
            identif,
            quartier,
            superficie,
            bati,
            nom_bati,
            num_ilot,
            perimetre,
            geox,
            geoy,
            date_coyec,
            polygon: ring,
        });
    }

    log::info!("  Extracted {} buildings", buildings.len());
    buildings
}

/// Build ParisMapRon from shapefiles. Used by the preprocess binary.
#[allow(dead_code)]
pub fn build_from_shapefiles(buildings_shp: &str, blocks_shp: &str) -> ParisMapRon {
    let total_start = Instant::now();

    let (grid_w, grid_h) = compute_grid_size();
    log::info!(
        "GIS grid: {}×{} tiles ({} chunks)",
        grid_w,
        grid_h,
        (grid_w.div_ceil(64)) * (grid_h.div_ceil(64))
    );

    let block_start = Instant::now();
    let (blocks, quartier_names) = extract_blocks_from_shapefile(blocks_shp);
    log::info!(
        "Blocks extracted in {:.1}s",
        block_start.elapsed().as_secs_f64()
    );

    let bldg_start = Instant::now();
    let buildings = extract_buildings_from_shapefile(buildings_shp);
    log::info!(
        "Buildings extracted in {:.1}s",
        bldg_start.elapsed().as_secs_f64()
    );

    log::info!(
        "Extraction complete in {:.1}s: {} blocks, {} buildings, {} quartiers",
        total_start.elapsed().as_secs_f64(),
        blocks.len(),
        buildings.len(),
        quartier_names.len(),
    );

    ParisMapRon {
        grid_width: grid_w,
        grid_height: grid_h,
        buildings,
        blocks,
        quartier_names,
    }
}

// --- Rasterization: RON polygons → TileMap + registries ---

/// Rasterize all polygons and run classification/BFS, returning standalone products.
/// Used by both the preprocess binary (save to binary) and the game (fallback path).
#[allow(dead_code)]
pub fn rasterize_paris(
    data: &ParisMapRon,
) -> (TileMap, BuildingRegistry, BlockRegistry, Vec<String>) {
    let total_start = Instant::now();

    let grid_w = data.grid_width;
    let grid_h = data.grid_height;
    let mut tiles = TileMap::new(grid_w, grid_h);
    let mut buildings = BuildingRegistry::new();
    let mut blocks = BlockRegistry::new();

    // Build quartier name→id map (1-based).
    let mut quartier_map: HashMap<String, u8> = HashMap::new();
    for (i, name) in data.quartier_names.iter().enumerate() {
        quartier_map.insert(name.clone(), (i + 1) as u8);
    }

    // --- Blocks ---
    let block_start = Instant::now();
    let mut next_block_id: u16 = 1;
    let mut total_block_tiles = 0usize;

    for block_ron in &data.blocks {
        let cells = scanline_fill(&block_ron.polygon, grid_w, grid_h);
        if cells.is_empty() {
            continue;
        }

        let quartier_id = quartier_map.get(&block_ron.quartier).copied().unwrap_or(0);
        let block_id = BlockId(next_block_id);
        next_block_id += 1;

        for &(cx, cy) in &cells {
            let ux = cx as usize;
            let uy = cy as usize;
            tiles.set_terrain(ux, uy, Terrain::Courtyard);
            tiles.set_block_id(ux, uy, block_id);
            tiles.set_quartier_id(ux, uy, quartier_id);
        }
        total_block_tiles += cells.len();

        blocks.insert(BlockData {
            id: block_id,
            id_ilots: block_ron.id_ilots.clone(),
            quartier: block_ron.quartier.clone(),
            aire: block_ron.aire,
            ilots_vass: block_ron.ilots_vass.clone(),
            buildings: Vec::new(),
        });
    }
    log::info!(
        "  {} blocks, {} block tiles in {:.1}s",
        blocks.blocks.len(),
        total_block_tiles,
        block_start.elapsed().as_secs_f64(),
    );

    // --- Pass 1: BATI=2 named gardens → Garden terrain ---
    // Only polygons with "jardin" or "parc" in nom_bati need rasterization.
    // Non-garden BATI=2 and all BATI=3: tiles already Courtyard from block pass.
    let garden_start = Instant::now();
    let mut garden_tile_count = 0usize;
    let mut garden_polygon_count = 0usize;
    let mut skipped_bati2 = 0usize;
    let mut skipped_bati3 = 0usize;

    for bldg_ron in &data.buildings {
        match bldg_ron.bati {
            2 => {
                let is_garden = bldg_ron.nom_bati.as_ref().is_some_and(|name| {
                    let lower = name.to_lowercase();
                    lower.contains("jardin") || lower.contains("parc")
                });
                if !is_garden {
                    skipped_bati2 += 1;
                    continue;
                }
                let cells = scanline_fill(&bldg_ron.polygon, grid_w, grid_h);
                for &(cx, cy) in &cells {
                    tiles.set_terrain(cx as usize, cy as usize, Terrain::Garden);
                }
                garden_tile_count += cells.len();
                garden_polygon_count += 1;
            }
            3 => {
                skipped_bati3 += 1;
            }
            _ => {} // BATI=1 handled in pass 2
        }
    }
    log::info!(
        "  {} garden polygons ({} tiles), {} BATI=2 courtyards skipped, \
         {} BATI=3 features skipped in {:.1}s",
        garden_polygon_count,
        garden_tile_count,
        skipped_bati2,
        skipped_bati3,
        garden_start.elapsed().as_secs_f64(),
    );

    // --- Pass 2: BATI=1 buildings → Wall + building_id + registry ---
    // Overwrites Garden if a building polygon overlaps (building wins).
    let bldg_start = Instant::now();
    let mut total_building_tiles = 0usize;

    for bldg_ron in &data.buildings {
        if bldg_ron.bati != 1 {
            continue;
        }

        let cells = scanline_fill(&bldg_ron.polygon, grid_w, grid_h);
        if cells.is_empty() {
            continue;
        }

        let building_id = buildings.next_id();
        let floor_count = estimate_floor_count(bldg_ron.superficie);

        // Determine which block this building sits in.
        let mut block_for_building: Option<BlockId> = None;
        for &(cx, cy) in &cells {
            if let Some(bid) = tiles.get_block_id(cx as usize, cy as usize) {
                block_for_building = Some(bid);
                break;
            }
        }

        let mut tile_list = Vec::with_capacity(cells.len());
        for &(cx, cy) in &cells {
            let ux = cx as usize;
            let uy = cy as usize;
            tiles.set_terrain(ux, uy, Terrain::Wall);
            tiles.set_building_id(ux, uy, building_id);
            tile_list.push((cx, cy));
        }
        total_building_tiles += cells.len();

        if let Some(bid) = block_for_building
            && let Some(block) = blocks.blocks.get_mut(&bid)
        {
            block.buildings.push(building_id);
        }

        buildings.insert(BuildingData {
            id: building_id,
            identif: bldg_ron.identif,
            quartier: bldg_ron.quartier.clone(),
            superficie: bldg_ron.superficie,
            bati: bldg_ron.bati,
            nom_bati: bldg_ron.nom_bati.clone(),
            num_ilot: bldg_ron.num_ilot.clone(),
            perimetre: bldg_ron.perimetre,
            geox: bldg_ron.geox,
            geoy: bldg_ron.geoy,
            date_coyec: bldg_ron.date_coyec.clone(),
            floor_count,
            tiles: tile_list,
            addresses: Vec::new(),
            occupants: Vec::new(),
        });
    }
    log::info!(
        "  {} buildings (from {} unique identifs), {} building tiles in {:.1}s",
        buildings.len(),
        buildings.identif_index.len(),
        total_building_tiles,
        bldg_start.elapsed().as_secs_f64(),
    );

    // --- Wall/floor classification ---
    let class_start = Instant::now();
    classify_walls_floors(&mut tiles, &buildings);
    log::info!(
        "  Wall/floor classification in {:.1}s",
        class_start.elapsed().as_secs_f64()
    );

    // --- Quartier BFS ---
    let bfs_start = Instant::now();
    fill_quartier_roads(&mut tiles, grid_w, grid_h);
    log::info!(
        "  Quartier BFS in {:.1}s",
        bfs_start.elapsed().as_secs_f64()
    );

    log::info!(
        "Rasterization complete in {:.1}s",
        total_start.elapsed().as_secs_f64(),
    );

    (tiles, buildings, blocks, data.quartier_names.clone())
}

// --- Game-side loader: reconstruct TileMap from RON polygons ---

/// Reconstruct the full TileMap, registries, and quartier data from RON polygons.
pub fn apply_paris_ron(world: &mut World, data: ParisMapRon) {
    let (tiles, buildings, blocks, quartier_names) = rasterize_paris(&data);
    world.tiles = tiles;
    world.buildings = buildings;
    world.blocks = blocks;
    world.quartier_names = quartier_names;
}

// --- Binary save/load ---

/// Save rasterized tile data + metadata for fast game loading.
/// Tile arrays go to `tiles_path` (binary), registry data to `meta_path` (RON).
/// BuildingData.tiles is stripped from metadata (reconstructed from binary on load).
#[allow(dead_code)]
pub fn save_paris_binary(
    tiles: &TileMap,
    buildings: &BuildingRegistry,
    blocks: &BlockRegistry,
    quartier_names: &[String],
    tiles_path: &str,
    meta_path: &str,
) {
    // Write binary tiles
    let tile_start = Instant::now();
    tiles
        .write_binary(tiles_path)
        .unwrap_or_else(|e| panic!("Failed to write {tiles_path}: {e}"));
    let tile_size = std::fs::metadata(tiles_path).map(|m| m.len()).unwrap_or(0);
    log::info!(
        "  Binary tiles: {:.1}MB in {:.1}s",
        tile_size as f64 / (1024.0 * 1024.0),
        tile_start.elapsed().as_secs_f64()
    );

    // Write metadata RON (strip tile lists from BuildingData)
    let meta_start = Instant::now();
    let meta_buildings: Vec<BuildingData> = buildings
        .buildings
        .iter()
        .map(|b| {
            let mut b = b.clone();
            b.tiles = Vec::new(); // strip — reconstructed from binary on load
            b
        })
        .collect();
    // Vec is already in insertion order (sequential by BuildingId)

    let mut meta_blocks: Vec<BlockData> = blocks.blocks.values().cloned().collect();
    meta_blocks.sort_by_key(|b| b.id.0);

    let metadata = ParisMetadataRon {
        quartier_names: quartier_names.to_vec(),
        buildings: meta_buildings,
        blocks: meta_blocks,
    };

    let pretty = ron::ser::PrettyConfig::default();
    let ron_str = ron::ser::to_string_pretty(&metadata, pretty)
        .unwrap_or_else(|e| panic!("Failed to serialize metadata RON: {e}"));
    let mut file = std::fs::File::create(meta_path)
        .unwrap_or_else(|e| panic!("Failed to create {meta_path}: {e}"));
    file.write_all(ron_str.as_bytes())
        .unwrap_or_else(|e| panic!("Failed to write {meta_path}: {e}"));

    let meta_size = std::fs::metadata(meta_path).map(|m| m.len()).unwrap_or(0);
    log::info!(
        "  Metadata RON: {:.1}MB in {:.1}s",
        meta_size as f64 / (1024.0 * 1024.0),
        meta_start.elapsed().as_secs_f64()
    );
}

/// Load pre-rasterized binary tiles + metadata into World.
/// Reconstructs BuildingData.tiles by scanning the tile array.
pub fn load_paris_binary(world: &mut World, tiles_path: &str, meta_path: &str) {
    let total_start = Instant::now();

    // Load binary tiles
    let tile_start = Instant::now();
    world.tiles = TileMap::read_binary(tiles_path)
        .unwrap_or_else(|e| panic!("Failed to read {tiles_path}: {e}"));
    log::info!(
        "  Binary tiles loaded in {:.1}s ({}×{})",
        tile_start.elapsed().as_secs_f64(),
        world.tiles.width(),
        world.tiles.height()
    );

    // Load metadata RON
    let meta_start = Instant::now();
    let ron_str = std::fs::read_to_string(meta_path)
        .unwrap_or_else(|e| panic!("Failed to read {meta_path}: {e}"));
    let metadata: ParisMetadataRon =
        ron::from_str(&ron_str).unwrap_or_else(|e| panic!("Failed to parse {meta_path}: {e}"));
    log::info!(
        "  Metadata loaded in {:.1}s: {} buildings, {} blocks, {} quartiers",
        meta_start.elapsed().as_secs_f64(),
        metadata.buildings.len(),
        metadata.blocks.len(),
        metadata.quartier_names.len()
    );

    world.quartier_names = metadata.quartier_names;

    // Reconstruct building tile lists by scanning the tile array.
    // BuildingId is 1-based; allocate per-building tile vecs indexed by id-1.
    let scan_start = Instant::now();
    let num_buildings = metadata.buildings.len();
    let mut building_tiles: Vec<Vec<(i32, i32)>> = vec![Vec::new(); num_buildings];
    let w = world.tiles.width();
    let h = world.tiles.height();
    for y in 0..h {
        for x in 0..w {
            if let Some(bid) = world.tiles.get_building_id(x, y) {
                let idx = bid.0 as usize - 1;
                if idx < num_buildings {
                    building_tiles[idx].push((x as i32, y as i32));
                }
            }
        }
    }
    log::info!(
        "  Tile scan in {:.1}s: {} buildings",
        scan_start.elapsed().as_secs_f64(),
        num_buildings
    );

    // Populate registry (Vec-backed, preserves insertion order)
    for (i, mut bdata) in metadata.buildings.into_iter().enumerate() {
        bdata.tiles = std::mem::take(&mut building_tiles[i]);
        world.buildings.insert(bdata);
    }
    for bdata in metadata.blocks {
        world.blocks.insert(bdata);
    }

    log::info!(
        "Paris binary loaded in {:.1}s",
        total_start.elapsed().as_secs_f64()
    );
}

/// Classify building tiles into Wall vs Floor.
/// A tile is Wall if any cardinal neighbor is not in the same building's
/// original polygon. Uses each building's own tile set (not the global tile
/// map's building_id) so overlapping polygons at party walls don't create
/// false thick walls.
fn classify_walls_floors(tiles: &mut TileMap, buildings: &BuildingRegistry) {
    let mut wall_count = 0usize;
    let mut floor_count = 0usize;

    for bdata in &buildings.buildings {
        let tile_set: HashSet<(i32, i32)> = bdata.tiles.iter().copied().collect();

        for &(cx, cy) in &bdata.tiles {
            let mut is_edge = false;
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                if !tile_set.contains(&(cx + dx, cy + dy)) {
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
            tiles.set_terrain(cx as usize, cy as usize, terrain);
        }
    }

    log::info!("  {} wall tiles, {} floor tiles", wall_count, floor_count);
}

/// Multi-source BFS to assign quartier_id to road tiles.
/// Expands from all tiles that already have quartier_id != 0.
fn fill_quartier_roads(tiles: &mut TileMap, grid_w: usize, grid_h: usize) {
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    let mut assigned = 0usize;

    // Seed: all tiles with quartier_id already set.
    for y in 0..grid_h {
        for x in 0..grid_w {
            if let Some(qid) = tiles.get_quartier_id(x, y)
                && qid != 0
            {
                queue.push_back((x, y));
            }
        }
    }

    while let Some((x, y)) = queue.pop_front() {
        let qid = tiles.get_quartier_id(x, y).unwrap_or(0);
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
            if let Some(nqid) = tiles.get_quartier_id(nux, nuy)
                && nqid == 0
            {
                tiles.set_quartier_id(nux, nuy, qid);
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
            assert!((0..20).contains(&x));
            assert!((0..20).contains(&y));
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
        // Create a 5x5 building block — expect 16 wall + 9 floor
        let mut tiles = TileMap::new(10, 10);
        let mut buildings = BuildingRegistry::new();

        let bid = buildings.next_id(); // BuildingId(1)
        let mut tile_list = Vec::new();
        for y in 2..7 {
            for x in 2..7 {
                tiles.set_terrain(x, y, Terrain::Wall);
                tiles.set_building_id(x, y, bid);
                tile_list.push((x as i32, y as i32));
            }
        }

        buildings.insert(BuildingData {
            id: bid,
            identif: 42,
            quartier: "Test".into(),
            superficie: 100.0,
            bati: 1,
            nom_bati: None,
            num_ilot: "T1".into(),
            perimetre: 0.0,
            geox: 0.0,
            geoy: 0.0,
            date_coyec: None,
            floor_count: 3,
            tiles: tile_list,
            addresses: Vec::new(),
            occupants: Vec::new(),
        });

        classify_walls_floors(&mut tiles, &buildings);

        // Count walls and floors
        let mut walls = 0;
        let mut floors = 0;
        for y in 2..7 {
            for x in 2..7 {
                match tiles.get_terrain(x, y) {
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
            assert!((0..10).contains(&x), "x={x} out of bounds");
            assert!((0..10).contains(&y), "y={y} out of bounds");
        }
        // Should fill approximately 5x5 = 25 tiles (the in-bounds quarter)
        assert!(
            tiles.len() >= 20 && tiles.len() <= 30,
            "len={}",
            tiles.len()
        );
    }

    #[test]
    fn test_paris_ron_roundtrip() {
        let data = ParisMapRon {
            grid_width: 100,
            grid_height: 80,
            buildings: vec![ParisBuildingRon {
                identif: 42,
                quartier: "Arcis".into(),
                superficie: 120.0,
                bati: 1,
                nom_bati: Some("Mairie".into()),
                num_ilot: "860IL74".into(),
                perimetre: 44.0,
                geox: 601234.5,
                geoy: 128456.7,
                date_coyec: Some("1830".into()),
                polygon: vec![(10.0, 10.0), (20.0, 10.0), (20.0, 20.0), (10.0, 20.0)],
            }],
            blocks: vec![ParisBlockRon {
                id_ilots: "860IL74".into(),
                quartier: "Arcis".into(),
                aire: 5000.0,
                ilots_vass: "74".into(),
                polygon: vec![(5.0, 5.0), (25.0, 5.0), (25.0, 25.0), (5.0, 25.0)],
            }],
            quartier_names: vec!["Arcis".into(), "Marais".into()],
        };

        let ron_str = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default())
            .expect("serialize");
        let back: ParisMapRon = ron::from_str(&ron_str).expect("deserialize");

        assert_eq!(back.grid_width, 100);
        assert_eq!(back.grid_height, 80);
        assert_eq!(back.buildings.len(), 1);
        assert_eq!(back.buildings[0].identif, 42);
        assert_eq!(back.buildings[0].nom_bati, Some("Mairie".into()));
        assert_eq!(back.blocks.len(), 1);
        assert_eq!(back.buildings[0].perimetre, 44.0);
        assert_eq!(back.buildings[0].geox, 601234.5);
        assert_eq!(back.buildings[0].geoy, 128456.7);
        assert_eq!(back.buildings[0].date_coyec, Some("1830".into()));
        assert_eq!(back.blocks[0].id_ilots, "860IL74");
        assert_eq!(back.blocks[0].ilots_vass, "74");
        assert_eq!(back.quartier_names, vec!["Arcis", "Marais"]);
    }

    /// Helper: build a minimal ParisMapRon for rasterization tests.
    /// Block covers 5..25 x 5..25, building/garden polygons placed inside.
    fn make_test_map(bldg_entries: Vec<ParisBuildingRon>) -> ParisMapRon {
        ParisMapRon {
            grid_width: 30,
            grid_height: 30,
            buildings: bldg_entries,
            blocks: vec![ParisBlockRon {
                id_ilots: "BLK1".into(),
                quartier: "TestQ".into(),
                aire: 400.0,
                ilots_vass: "1".into(),
                polygon: vec![
                    (5.0, 5.0),
                    (25.0, 5.0),
                    (25.0, 25.0),
                    (5.0, 25.0),
                    (5.0, 5.0),
                ],
            }],
            quartier_names: vec!["TestQ".into()],
        }
    }

    fn make_bldg(bati: u8, nom_bati: Option<&str>, poly: Vec<(f64, f64)>) -> ParisBuildingRon {
        ParisBuildingRon {
            identif: 1,
            quartier: "TestQ".into(),
            superficie: 100.0,
            bati,
            nom_bati: nom_bati.map(|s| s.to_string()),
            num_ilot: "BLK1".into(),
            perimetre: 0.0,
            geox: 0.0,
            geoy: 0.0,
            date_coyec: None,
            polygon: poly,
        }
    }

    #[test]
    fn test_bati1_rasterized_as_building() {
        let bldg = make_bldg(
            1,
            None,
            vec![
                (10.0, 10.0),
                (16.0, 10.0),
                (16.0, 16.0),
                (10.0, 16.0),
                (10.0, 10.0),
            ],
        );
        let data = make_test_map(vec![bldg]);
        let (tiles, buildings, _blocks, _) = rasterize_paris(&data);

        // Building tiles should be Wall or Floor, with building_id != 0
        assert!(!buildings.is_empty(), "registry should have 1 building");
        assert_eq!(buildings.len(), 1);
        let bd = &buildings.buildings[0];
        assert!(!bd.tiles.is_empty());
        for &(x, y) in &bd.tiles {
            let t = tiles.get_terrain(x as usize, y as usize).unwrap();
            assert!(
                t == Terrain::Wall || t == Terrain::Floor,
                "expected Wall/Floor at ({x},{y}), got {t:?}"
            );
            assert!(tiles.get_building_id(x as usize, y as usize).is_some());
        }
    }

    #[test]
    fn test_bati2_courtyard_not_rasterized() {
        let bldg = make_bldg(
            2,
            None,
            vec![
                (10.0, 10.0),
                (16.0, 10.0),
                (16.0, 16.0),
                (10.0, 16.0),
                (10.0, 10.0),
            ],
        );
        let data = make_test_map(vec![bldg]);
        let (tiles, buildings, _blocks, _) = rasterize_paris(&data);

        // BATI=2 with no garden name: tiles stay Courtyard, no building_id
        assert!(buildings.is_empty(), "registry should be empty");
        let t = tiles.get_terrain(13, 13).unwrap();
        assert_eq!(t, Terrain::Courtyard);
        assert!(tiles.get_building_id(13, 13).is_none());
    }

    #[test]
    fn test_bati2_garden_detected() {
        let bldg = make_bldg(
            2,
            Some("Parc ou jardin"),
            vec![
                (10.0, 10.0),
                (16.0, 10.0),
                (16.0, 16.0),
                (10.0, 16.0),
                (10.0, 10.0),
            ],
        );
        let data = make_test_map(vec![bldg]);
        let (tiles, buildings, _blocks, _) = rasterize_paris(&data);

        // BATI=2 with garden name: tiles become Garden, no building_id
        assert!(buildings.is_empty(), "registry should be empty");
        let t = tiles.get_terrain(13, 13).unwrap();
        assert_eq!(t, Terrain::Garden);
        assert!(tiles.get_building_id(13, 13).is_none());
    }

    #[test]
    fn test_bati3_not_rasterized() {
        let bldg = make_bldg(
            3,
            Some("Fontaine"),
            vec![
                (10.0, 10.0),
                (16.0, 10.0),
                (16.0, 16.0),
                (10.0, 16.0),
                (10.0, 10.0),
            ],
        );
        let data = make_test_map(vec![bldg]);
        let (tiles, buildings, _blocks, _) = rasterize_paris(&data);

        // BATI=3: tiles stay Courtyard, no building_id
        assert!(buildings.is_empty(), "registry should be empty");
        let t = tiles.get_terrain(13, 13).unwrap();
        assert_eq!(t, Terrain::Courtyard);
        assert!(tiles.get_building_id(13, 13).is_none());
    }

    #[test]
    fn test_bati1_overwrites_garden() {
        // Garden covers 10..20 x 10..20, building covers 12..18 x 12..18
        let garden = make_bldg(
            2,
            Some("Jardin public"),
            vec![
                (10.0, 10.0),
                (20.0, 10.0),
                (20.0, 20.0),
                (10.0, 20.0),
                (10.0, 10.0),
            ],
        );
        let building = make_bldg(
            1,
            None,
            vec![
                (12.0, 12.0),
                (18.0, 12.0),
                (18.0, 18.0),
                (12.0, 18.0),
                (12.0, 12.0),
            ],
        );
        let data = make_test_map(vec![garden, building]);
        let (tiles, buildings, _blocks, _) = rasterize_paris(&data);

        // Building wins in overlap area
        assert_eq!(buildings.len(), 1);
        let bd = &buildings.buildings[0];
        for &(x, y) in &bd.tiles {
            let t = tiles.get_terrain(x as usize, y as usize).unwrap();
            assert!(
                t == Terrain::Wall || t == Terrain::Floor,
                "building tile ({x},{y}) should be Wall/Floor, got {t:?}"
            );
        }

        // Non-overlapping garden tiles stay Garden
        let t_corner = tiles.get_terrain(10, 10).unwrap();
        assert_eq!(
            t_corner,
            Terrain::Garden,
            "non-overlapping garden tile should stay Garden"
        );
    }

    #[test]
    fn test_only_bati1_in_registry() {
        let b1 = make_bldg(
            1,
            None,
            vec![
                (7.0, 7.0),
                (12.0, 7.0),
                (12.0, 12.0),
                (7.0, 12.0),
                (7.0, 7.0),
            ],
        );
        let b2_court = make_bldg(
            2,
            None,
            vec![
                (13.0, 7.0),
                (18.0, 7.0),
                (18.0, 12.0),
                (13.0, 12.0),
                (13.0, 7.0),
            ],
        );
        let b2_garden = make_bldg(
            2,
            Some("Jardin"),
            vec![
                (7.0, 13.0),
                (12.0, 13.0),
                (12.0, 18.0),
                (7.0, 18.0),
                (7.0, 13.0),
            ],
        );
        let b3 = make_bldg(
            3,
            Some("Fontaine"),
            vec![
                (13.0, 13.0),
                (18.0, 13.0),
                (18.0, 18.0),
                (13.0, 18.0),
                (13.0, 13.0),
            ],
        );
        let data = make_test_map(vec![b1, b2_court, b2_garden, b3]);
        let (_tiles, buildings, _blocks, _) = rasterize_paris(&data);

        // Only BATI=1 entries in registry
        assert_eq!(buildings.len(), 1, "only BATI=1 should be in registry");
        assert_eq!(buildings.buildings[0].bati, 1);
    }
}
