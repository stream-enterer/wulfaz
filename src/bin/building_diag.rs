//! Diagnostic tool for B05 door placement design.
//! Analyzes building topology: wall/floor classification, road adjacency,
//! landlocked buildings, courtyard connectivity, and small-building edge cases.
//!
//! Usage: cargo run --bin building_diag [TILES_PATH] [META_PATH]
//! Defaults: data/paris.tiles data/paris.meta.bin

use std::collections::{HashMap, HashSet, VecDeque};
use wulfaz::loading_gis::load_meta_bincode;
use wulfaz::registry::{BuildingData, BuildingId};
use wulfaz::tile_map::{Terrain, TileMap};

const FOUR_DIRS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tiles_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("data/paris.tiles");
    let meta_path = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or("data/paris.meta.bin");

    // --- Load tiles and metadata ---
    println!("Loading {tiles_path}...");
    let (tiles, tiles_uuid) = TileMap::read_binary(tiles_path).unwrap_or_else(|e| {
        eprintln!("Failed to read {tiles_path}: {e}");
        std::process::exit(1);
    });
    let w = tiles.width();
    let h = tiles.height();
    println!("Grid: {w}x{h} ({} tiles)", w * h);

    println!("Loading {meta_path}...");
    let (metadata, meta_uuid) = load_meta_bincode(meta_path).unwrap_or_else(|e| {
        eprintln!("Failed to read {meta_path}: {e}");
        std::process::exit(1);
    });
    if tiles_uuid != meta_uuid {
        eprintln!("UUID mismatch between tiles and metadata");
        std::process::exit(1);
    }

    // Reconstruct building tile lists (same logic as load_paris_binary)
    let num_buildings = metadata.buildings.len();
    let mut building_tiles: Vec<Vec<(i32, i32)>> = vec![Vec::new(); num_buildings];
    for y in 0..h {
        for x in 0..w {
            if let Some(bid) = tiles.get_building_id(x, y) {
                let idx = bid.0 as usize - 1;
                if idx < num_buildings {
                    building_tiles[idx].push((x as i32, y as i32));
                }
            }
        }
    }

    // Assemble full BuildingData with tiles
    let buildings: Vec<BuildingData> = metadata
        .buildings
        .into_iter()
        .enumerate()
        .map(|(i, mut b)| {
            b.tiles = std::mem::take(&mut building_tiles[i]);
            b
        })
        .collect();

    let total_buildings = buildings.len();
    println!("{total_buildings} buildings loaded\n");

    // === A. BATI Classification ===
    println!("=== A. BATI Classification ===");
    let mut bati_counts: HashMap<u8, usize> = HashMap::new();
    let mut bati_tile_counts: HashMap<u8, usize> = HashMap::new();
    for b in &buildings {
        *bati_counts.entry(b.bati).or_default() += 1;
        *bati_tile_counts.entry(b.bati).or_default() += b.tiles.len();
    }
    for bati in [1, 2, 3] {
        let count = bati_counts.get(&bati).copied().unwrap_or(0);
        let tile_count = bati_tile_counts.get(&bati).copied().unwrap_or(0);
        let label = match bati {
            1 => "built structures",
            2 => "open spaces (courtyards/gardens)",
            3 => "minor features (fixtures)",
            _ => "unknown",
        };
        println!("  BATI={bati}: {count:>6} buildings, {tile_count:>8} tiles  ({label})");
    }
    println!();

    // Filter to BATI=1 only for remaining analysis
    let bati1: Vec<&BuildingData> = buildings.iter().filter(|b| b.bati == 1).collect();
    let bati1_count = bati1.len();
    println!("Analyzing {bati1_count} BATI=1 buildings...\n");

    // === B. Wall/Floor Tile Classification ===
    println!("=== B. Wall/Floor Distribution ===");
    let mut all_wall_buildings = 0usize; // buildings with zero Floor tiles
    let mut has_floor_buildings = 0usize;
    let mut total_wall_tiles = 0usize;
    let mut total_floor_tiles = 0usize;

    // Per-building: classify each tile as wall or floor
    struct BuildingAnalysis {
        id: BuildingId,
        tile_count: usize,
        floor_count: usize,
        road_adjacent_walls: Vec<(i32, i32)>, // wall tiles with Road/Courtyard neighbor
        has_road_access: bool,
        has_courtyard_access: bool,
        superficie: f32,
        quartier: String,
    }

    let mut analyses: Vec<BuildingAnalysis> = Vec::with_capacity(bati1_count);

    for b in &bati1 {
        let mut wall_count = 0usize;
        let mut floor_count = 0usize;
        let mut road_adjacent_walls: Vec<(i32, i32)> = Vec::new();
        let mut court_adjacent_walls: Vec<(i32, i32)> = Vec::new();

        for &(cx, cy) in &b.tiles {
            let terrain = tiles.get_terrain(cx as usize, cy as usize);
            match terrain {
                Some(Terrain::Wall) => {
                    wall_count += 1;
                    // Check cardinal neighbors for Road or Courtyard
                    for &(dx, dy) in &FOUR_DIRS {
                        let nx = cx + dx;
                        let ny = cy + dy;
                        if let Some(nt) = tiles.get_terrain(nx as usize, ny as usize)
                            && (nt == Terrain::Road || nt == Terrain::Courtyard)
                        {
                            road_adjacent_walls.push((cx, cy));
                            if nt == Terrain::Courtyard {
                                court_adjacent_walls.push((cx, cy));
                            }
                            break; // one match is enough to flag this tile
                        }
                    }
                }
                Some(Terrain::Floor) => {
                    floor_count += 1;
                }
                _ => {
                    // Tiles that were overwritten by BATI=2/3 (Courtyard, Garden, Fixture)
                    // still have this building's building_id but different terrain
                }
            }
        }

        let has_road = road_adjacent_walls.iter().any(|&(cx, cy)| {
            FOUR_DIRS.iter().any(|&(dx, dy)| {
                tiles.get_terrain((cx + dx) as usize, (cy + dy) as usize) == Some(Terrain::Road)
            })
        });
        let has_courtyard = !court_adjacent_walls.is_empty();

        if floor_count == 0 {
            all_wall_buildings += 1;
        } else {
            has_floor_buildings += 1;
        }
        total_wall_tiles += wall_count;
        total_floor_tiles += floor_count;

        analyses.push(BuildingAnalysis {
            id: b.id,
            tile_count: b.tiles.len(),
            floor_count,
            road_adjacent_walls,
            has_road_access: has_road,
            has_courtyard_access: has_courtyard,
            superficie: b.superficie,
            quartier: b.quartier.clone(),
        });
    }

    println!("  Buildings with interior (Floor > 0): {has_floor_buildings}");
    println!("  Buildings ALL wall (Floor == 0):     {all_wall_buildings}");
    println!("  Total Wall tiles: {total_wall_tiles}");
    println!("  Total Floor tiles: {total_floor_tiles}");
    println!();

    // Size distribution of all-wall buildings
    println!("  All-wall buildings by tile count:");
    let mut all_wall_by_size: HashMap<usize, usize> = HashMap::new();
    for a in &analyses {
        if a.floor_count == 0 {
            let bucket = match a.tile_count {
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                5..=10 => 5,
                11..=20 => 11,
                _ => 21,
            };
            *all_wall_by_size.entry(bucket).or_default() += 1;
        }
    }
    let mut size_buckets: Vec<(usize, usize)> = all_wall_by_size.into_iter().collect();
    size_buckets.sort();
    for (bucket, count) in &size_buckets {
        let label = match *bucket {
            1 => "    1 tile ",
            2 => "    2 tiles",
            3 => "    3 tiles",
            4 => "    4 tiles",
            5 => "   5-10    ",
            11 => "  11-20    ",
            _ => "  21+      ",
        };
        println!("    {label}: {count}");
    }
    println!();

    // === C. Road Adjacency ===
    println!("=== C. Road/Courtyard Adjacency (BATI=1 with Floor > 0) ===");
    let with_floor: Vec<&BuildingAnalysis> =
        analyses.iter().filter(|a| a.floor_count > 0).collect();
    let with_floor_count = with_floor.len();

    let has_road_count = with_floor.iter().filter(|a| a.has_road_access).count();
    let has_courtyard_only = with_floor
        .iter()
        .filter(|a| !a.has_road_access && a.has_courtyard_access)
        .count();
    let landlocked_count = with_floor
        .iter()
        .filter(|a| a.road_adjacent_walls.is_empty())
        .count();

    println!(
        "  Has Road-adjacent wall:      {has_road_count:>6} ({:.1}%)",
        has_road_count as f64 / with_floor_count as f64 * 100.0
    );
    println!(
        "  Courtyard-only access:        {has_courtyard_only:>6} ({:.1}%)",
        has_courtyard_only as f64 / with_floor_count as f64 * 100.0
    );
    println!(
        "  Landlocked (no Road/Court):   {landlocked_count:>6} ({:.1}%)",
        landlocked_count as f64 / with_floor_count as f64 * 100.0
    );
    println!();

    // Door candidate count distribution (wall tiles adjacent to Road OR Courtyard)
    println!("  Door candidate tiles per building (wall tiles adjacent to Road/Courtyard):");
    let mut door_candidate_dist: HashMap<usize, usize> = HashMap::new();
    for a in &with_floor {
        let bucket = match a.road_adjacent_walls.len() {
            0 => 0,
            1 => 1,
            2 => 2,
            3..=5 => 3,
            6..=10 => 6,
            11..=20 => 11,
            _ => 21,
        };
        *door_candidate_dist.entry(bucket).or_default() += 1;
    }
    let mut door_buckets: Vec<(usize, usize)> = door_candidate_dist.into_iter().collect();
    door_buckets.sort();
    for (bucket, count) in &door_buckets {
        let label = match *bucket {
            0 => "      0    ",
            1 => "      1    ",
            2 => "      2    ",
            3 => "    3-5    ",
            6 => "   6-10    ",
            11 => "  11-20    ",
            _ => "  21+      ",
        };
        println!("    {label}: {count}");
    }
    println!();

    // === D. Landlocked Building Analysis ===
    println!("=== D. Landlocked Building Detail ===");
    let landlocked: Vec<&BuildingAnalysis> = with_floor
        .iter()
        .filter(|a| a.road_adjacent_walls.is_empty())
        .copied()
        .collect();

    if landlocked.is_empty() {
        println!("  No landlocked buildings found!");
    } else {
        // Measure carve depth: BFS from each landlocked building's wall tiles
        // to nearest Road/Courtyard tile, traversing through Wall/Floor tiles
        println!("  Carve depth (min tiles through Wall/Floor to reach Road/Courtyard):");
        let mut depth_dist: HashMap<usize, usize> = HashMap::new();
        let mut unreachable_count = 0usize;
        let max_search = 50; // max BFS depth

        for a in &landlocked {
            let depth = bfs_to_road_or_courtyard(&tiles, &bati1, a.id, max_search);
            if let Some(d) = depth {
                let bucket = match d {
                    1 => 1,
                    2 => 2,
                    3 => 3,
                    4..=5 => 4,
                    6..=10 => 6,
                    _ => 11,
                };
                *depth_dist.entry(bucket).or_default() += 1;
            } else {
                unreachable_count += 1;
            }
        }

        let mut depth_buckets: Vec<(usize, usize)> = depth_dist.into_iter().collect();
        depth_buckets.sort();
        for (bucket, count) in &depth_buckets {
            let label = match *bucket {
                1 => "    1 tile ",
                2 => "    2 tiles",
                3 => "    3 tiles",
                4 => "   4-5     ",
                6 => "  6-10     ",
                _ => "  11+      ",
            };
            println!("    {label}: {count}");
        }
        if unreachable_count > 0 {
            println!("    Unreachable (>{max_search}): {unreachable_count}");
        }
        println!();

        // Landlocked by quartier
        println!("  Landlocked by quartier:");
        let mut by_quartier: HashMap<&str, usize> = HashMap::new();
        for a in &landlocked {
            *by_quartier.entry(&a.quartier).or_default() += 1;
        }
        let mut quartier_list: Vec<(&&str, &usize)> = by_quartier.iter().collect();
        quartier_list.sort_by(|a, b| b.1.cmp(a.1));
        for (q, count) in quartier_list.iter().take(15) {
            println!("    {:<30} {count}", q);
        }
        if quartier_list.len() > 15 {
            println!("    ... and {} more quartiers", quartier_list.len() - 15);
        }
        println!();

        // Size distribution of landlocked buildings
        println!("  Landlocked building sizes (m²):");
        let mut landlocked_sizes: Vec<f32> = landlocked.iter().map(|a| a.superficie).collect();
        landlocked_sizes.sort_by(|a, b| a.total_cmp(b));
        let median = landlocked_sizes[landlocked_sizes.len() / 2];
        let mean = landlocked_sizes.iter().sum::<f32>() / landlocked_sizes.len() as f32;
        let min = landlocked_sizes[0];
        let max = landlocked_sizes[landlocked_sizes.len() - 1];
        println!("    min={min:.0}  median={median:.0}  mean={mean:.0}  max={max:.0} m²");
    }
    println!();

    // === E. Courtyard Connectivity ===
    println!("=== E. Courtyard Connectivity ===");

    // Find all courtyard regions (4-connected components of Courtyard terrain)
    let mut courtyard_visited = vec![false; w * h];
    let mut courtyard_regions: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    for y in 0..h {
        for x in 0..w {
            if tiles.get_terrain(x, y) != Some(Terrain::Courtyard) {
                continue;
            }
            let idx = y * w + x;
            if courtyard_visited[idx] {
                continue;
            }
            let mut region: Vec<(usize, usize)> = Vec::new();
            queue.push_back((x, y));
            courtyard_visited[idx] = true;
            while let Some((cx, cy)) = queue.pop_front() {
                region.push((cx, cy));
                for &(dx, dy) in &FOUR_DIRS {
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let nux = nx as usize;
                    let nuy = ny as usize;
                    let nidx = nuy * w + nux;
                    if !courtyard_visited[nidx]
                        && tiles.get_terrain(nux, nuy) == Some(Terrain::Courtyard)
                    {
                        courtyard_visited[nidx] = true;
                        queue.push_back((nux, nuy));
                    }
                }
            }
            courtyard_regions.push(region);
        }
    }

    // For each courtyard region, check if any tile is adjacent to Road
    let mut connected_to_road = 0usize;
    let mut island_courtyards = 0usize;
    let mut island_courtyard_tiles = 0usize;

    for region in &courtyard_regions {
        let has_road_neighbor = region.iter().any(|&(cx, cy)| {
            FOUR_DIRS.iter().any(|&(dx, dy)| {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 {
                    return false;
                }
                tiles.get_terrain(nx as usize, ny as usize) == Some(Terrain::Road)
            })
        });
        if has_road_neighbor {
            connected_to_road += 1;
        } else {
            island_courtyards += 1;
            island_courtyard_tiles += region.len();
        }
    }

    println!(
        "  Total courtyard regions (4-connected): {}",
        courtyard_regions.len()
    );
    println!("  Connected to Road network:             {connected_to_road}");
    println!(
        "  Island courtyards (no Road neighbor):  {island_courtyards} ({island_courtyard_tiles} tiles)"
    );

    // Size distribution of island courtyards
    if island_courtyards > 0 {
        println!("\n  Island courtyard sizes:");
        let mut island_sizes: Vec<usize> = courtyard_regions
            .iter()
            .filter(|r| {
                !r.iter().any(|&(cx, cy)| {
                    FOUR_DIRS.iter().any(|&(dx, dy)| {
                        let nx = cx as i32 + dx;
                        let ny = cy as i32 + dy;
                        if nx < 0 || ny < 0 {
                            return false;
                        }
                        tiles.get_terrain(nx as usize, ny as usize) == Some(Terrain::Road)
                    })
                })
            })
            .map(|r| r.len())
            .collect();
        island_sizes.sort();

        let mut size_dist: HashMap<usize, usize> = HashMap::new();
        for &sz in &island_sizes {
            let bucket = match sz {
                1..=5 => 1,
                6..=20 => 6,
                21..=50 => 21,
                51..=100 => 51,
                _ => 101,
            };
            *size_dist.entry(bucket).or_default() += 1;
        }
        let mut sd: Vec<(usize, usize)> = size_dist.into_iter().collect();
        sd.sort();
        for (bucket, count) in &sd {
            let label = match *bucket {
                1 => "   1-5 tiles ",
                6 => "  6-20 tiles ",
                21 => " 21-50 tiles ",
                51 => "51-100 tiles ",
                _ => " 101+  tiles ",
            };
            println!("    {label}: {count}");
        }
    }
    println!();

    // === F. Buildings Touching Courtyard (need dual doors) ===
    println!("=== F. Dual-Door Candidates (buildings with both Road and Courtyard access) ===");
    let dual_access = with_floor
        .iter()
        .filter(|a| a.has_road_access && a.has_courtyard_access)
        .count();
    let road_only = with_floor
        .iter()
        .filter(|a| a.has_road_access && !a.has_courtyard_access)
        .count();
    let court_only = with_floor
        .iter()
        .filter(|a| !a.has_road_access && a.has_courtyard_access)
        .count();
    let neither = with_floor
        .iter()
        .filter(|a| !a.has_road_access && !a.has_courtyard_access)
        .count();

    println!("  Road + Courtyard (dual-door):  {dual_access:>6}");
    println!("  Road only:                     {road_only:>6}");
    println!("  Courtyard only:                {court_only:>6}");
    println!("  Neither (landlocked):          {neither:>6}");
    println!();

    // === G. Arcis Neighborhood Focus ===
    println!("=== G. Arcis Neighborhood (B03 target) ===");
    let arcis: Vec<&BuildingAnalysis> = analyses
        .iter()
        .filter(|a| a.quartier == "Arcis" && a.floor_count > 0)
        .collect();
    let arcis_all: Vec<&BuildingAnalysis> =
        analyses.iter().filter(|a| a.quartier == "Arcis").collect();
    let arcis_landlocked = arcis
        .iter()
        .filter(|a| a.road_adjacent_walls.is_empty())
        .count();
    let arcis_all_wall = arcis_all.iter().filter(|a| a.floor_count == 0).count();
    let arcis_dual = arcis
        .iter()
        .filter(|a| a.has_road_access && a.has_courtyard_access)
        .count();

    println!("  Total BATI=1 buildings:   {}", arcis_all.len());
    println!("  With interior (Floor>0):  {}", arcis.len());
    println!("  All-wall (no interior):   {arcis_all_wall}");
    println!("  Landlocked:               {arcis_landlocked}");
    println!("  Dual-door candidates:     {arcis_dual}");

    // Occupant stats for Arcis
    let arcis_buildings: Vec<&BuildingData> = bati1
        .iter()
        .filter(|b| b.quartier == "Arcis")
        .copied()
        .collect();
    let arcis_with_occupants = arcis_buildings
        .iter()
        .filter(|b| !b.occupants_by_year.is_empty())
        .count();
    let arcis_total_occupants: usize = arcis_buildings
        .iter()
        .flat_map(|b| b.occupants_by_year.values())
        .map(|v| v.len())
        .sum();
    println!("  Buildings with occupants: {arcis_with_occupants}");
    println!("  Total occupant records:   {arcis_total_occupants}");
    println!();

    // === H. Summary ===
    println!("=== Summary ===");
    println!("  BATI=1 buildings:            {bati1_count}");
    println!("  With interior:               {has_floor_buildings}");
    println!(
        "  All-wall (no Floor):         {all_wall_buildings} ({:.1}%)",
        all_wall_buildings as f64 / bati1_count as f64 * 100.0
    );
    println!(
        "  Landlocked (with interior):  {landlocked_count} ({:.1}%)",
        landlocked_count as f64 / with_floor_count as f64 * 100.0
    );
    println!("  Island courtyards:           {island_courtyards}");
    println!("  Dual-door candidates:        {dual_access}");
    println!();

    let checks = [
        (
            "All-wall < 10% of BATI=1",
            (all_wall_buildings as f64 / bati1_count as f64) < 0.10,
        ),
        (
            "Landlocked < 20% of interior buildings",
            (landlocked_count as f64 / with_floor_count as f64) < 0.20,
        ),
        ("Island courtyards < 500", island_courtyards < 500),
    ];
    for (name, ok) in &checks {
        let status = if *ok { "PASS" } else { "WARN" };
        println!("  [{status}] {name}");
    }
}

/// BFS from a landlocked building's wall tiles outward through Wall/Floor tiles
/// to find the shortest path to a Road or Courtyard tile.
/// Returns the number of Wall/Floor tiles traversed (the "carve depth").
fn bfs_to_road_or_courtyard(
    tiles: &TileMap,
    _buildings: &[&BuildingData],
    building_id: BuildingId,
    max_depth: usize,
) -> Option<usize> {
    let w = tiles.width() as i32;
    let h = tiles.height() as i32;

    // Collect all tiles of this building as BFS seeds
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    let mut queue: VecDeque<(i32, i32, usize)> = VecDeque::new(); // (x, y, depth)

    // Seed from all wall tiles of the building
    // We need to find the building's tiles by scanning its neighbors
    // Actually, we need to start from the building's wall tiles' neighbors
    // that are outside the building
    let bw = tiles.width();
    let bh = tiles.height();
    for y in 0..bh {
        for x in 0..bw {
            if tiles.get_building_id(x, y) == Some(building_id)
                && tiles.get_terrain(x, y) == Some(Terrain::Wall)
            {
                // Check cardinal neighbors outside this building
                for &(dx, dy) in &FOUR_DIRS {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let nux = nx as usize;
                    let nuy = ny as usize;
                    if tiles.get_building_id(nux, nuy) == Some(building_id) {
                        continue; // same building
                    }
                    let nt = tiles.get_terrain(nux, nuy);
                    // If neighbor is Road or Courtyard, depth is 0 (not actually landlocked)
                    if nt == Some(Terrain::Road) || nt == Some(Terrain::Courtyard) {
                        return Some(0);
                    }
                    // If neighbor is Wall or Floor of another building, it's a carve candidate
                    if (nt == Some(Terrain::Wall) || nt == Some(Terrain::Floor))
                        && visited.insert((nx, ny))
                    {
                        queue.push_back((nx, ny, 1));
                    }
                }
            }
        }
    }

    // BFS outward through Wall/Floor tiles
    while let Some((cx, cy, depth)) = queue.pop_front() {
        if depth > max_depth {
            return None;
        }
        for &(dx, dy) in &FOUR_DIRS {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            if !visited.insert((nx, ny)) {
                continue;
            }
            let nt = tiles.get_terrain(nx as usize, ny as usize);
            if nt == Some(Terrain::Road) || nt == Some(Terrain::Courtyard) {
                return Some(depth);
            }
            if nt == Some(Terrain::Wall) || nt == Some(Terrain::Floor) {
                queue.push_back((nx, ny, depth + 1));
            }
        }
    }
    None
}
