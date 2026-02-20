//! Diagnostic tool for water rasterization and bridge detection.
//! Loads paris.tiles binary and produces a quality report.
//!
//! Usage: cargo run --bin water_diag [TILES_PATH]
//! Default: data/paris.tiles

use std::collections::{HashSet, VecDeque};
use wulfaz::tile_map::{Terrain, TileMap};

/// 8-connected neighbor offsets.
const EIGHT_DIRS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// Convert tile coordinate to longitude.
fn tile_to_lon(x: usize) -> f64 {
    (x as f64 - 30.0) / 73254.28 + 2.2981468
}

/// Convert tile coordinate to latitude.
fn tile_to_lat(y: usize) -> f64 {
    48.8837517 - (y as f64 - 30.0) / 111320.0
}

/// Historical bridge reference (1810-1836, Vasserot survey period).
#[allow(dead_code)]
struct BridgeRef {
    name: &'static str,
    lon: f64,
    lat: f64,
    est_width_m: u32,
    notes: &'static str,
}

fn bridge_refs() -> Vec<BridgeRef> {
    vec![
        BridgeRef {
            name: "Pont des Invalides",
            lon: 2.3131,
            lat: 48.8632,
            est_width_m: 12,
            notes: "1829 suspension",
        },
        BridgeRef {
            name: "Pont de la Concorde",
            lon: 2.3212,
            lat: 48.8625,
            est_width_m: 15,
            notes: "1791",
        },
        BridgeRef {
            name: "Pont Royal",
            lon: 2.3258,
            lat: 48.8612,
            est_width_m: 15,
            notes: "1689",
        },
        BridgeRef {
            name: "Pont du Carrousel",
            lon: 2.3310,
            lat: 48.8598,
            est_width_m: 12,
            notes: "1834 borderline",
        },
        BridgeRef {
            name: "Pont des Arts",
            lon: 2.3373,
            lat: 48.8580,
            est_width_m: 10,
            notes: "1804 pedestrian",
        },
        BridgeRef {
            name: "Pont Neuf (west arm)",
            lon: 2.3412,
            lat: 48.8568,
            est_width_m: 22,
            notes: "1607",
        },
        BridgeRef {
            name: "Pont Neuf (east arm)",
            lon: 2.3432,
            lat: 48.8560,
            est_width_m: 15,
            notes: "1607",
        },
        BridgeRef {
            name: "Pont Saint-Michel",
            lon: 2.3455,
            lat: 48.8535,
            est_width_m: 12,
            notes: "medieval south",
        },
        BridgeRef {
            name: "Petit Pont",
            lon: 2.3470,
            lat: 48.8525,
            est_width_m: 10,
            notes: "ancient south",
        },
        BridgeRef {
            name: "Pont au Double",
            lon: 2.3487,
            lat: 48.8525,
            est_width_m: 8,
            notes: "1634 south",
        },
        BridgeRef {
            name: "Pont au Change",
            lon: 2.3472,
            lat: 48.8558,
            est_width_m: 15,
            notes: "medieval north",
        },
        BridgeRef {
            name: "Pont Notre-Dame",
            lon: 2.3487,
            lat: 48.8555,
            est_width_m: 12,
            notes: "medieval north",
        },
        BridgeRef {
            name: "Pont de la Cite",
            lon: 2.3495,
            lat: 48.8553,
            est_width_m: 10,
            notes: "demolished 1858",
        },
        BridgeRef {
            name: "Pont d'Arcole",
            lon: 2.3510,
            lat: 48.8545,
            est_width_m: 4,
            notes: "1828 footbridge",
        },
        BridgeRef {
            name: "Pont de l'Archeveche",
            lon: 2.3510,
            lat: 48.8510,
            est_width_m: 10,
            notes: "1828 south",
        },
        BridgeRef {
            name: "Pont Saint-Louis",
            lon: 2.3535,
            lat: 48.8530,
            est_width_m: 8,
            notes: "between islands",
        },
        BridgeRef {
            name: "Pont de la Tournelle",
            lon: 2.3545,
            lat: 48.8505,
            est_width_m: 12,
            notes: "1656 south",
        },
        BridgeRef {
            name: "Pont Marie",
            lon: 2.3570,
            lat: 48.8520,
            est_width_m: 12,
            notes: "1635 north",
        },
        BridgeRef {
            name: "Pont Louis-Philippe",
            lon: 2.3555,
            lat: 48.8542,
            est_width_m: 10,
            notes: "1834 borderline",
        },
        BridgeRef {
            name: "Pont d'Austerlitz",
            lon: 2.3650,
            lat: 48.8478,
            est_width_m: 18,
            notes: "1807",
        },
    ]
}

/// Convert lon/lat to tile coordinates.
fn lon_to_tile(lon: f64) -> f64 {
    (lon - 2.2981468) * 73254.28 + 30.0
}

fn lat_to_tile(lat: f64) -> f64 {
    (48.8837517 - lat) * 111320.0 + 30.0
}

struct ComponentInfo {
    id: usize,
    tiles: Vec<(usize, usize)>,
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
    center_x: usize,
    center_y: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tiles_path = if args.len() > 1 {
        &args[1]
    } else {
        "data/paris.tiles"
    };

    println!("Loading {tiles_path}...");
    let tiles = TileMap::read_binary(tiles_path).unwrap_or_else(|e| {
        eprintln!("Failed to read {tiles_path}: {e}");
        std::process::exit(1);
    });
    let grid_w = tiles.width();
    let grid_h = tiles.height();
    println!("Grid: {}x{} ({} tiles)\n", grid_w, grid_h, grid_w * grid_h);

    // --- A. Terrain Census ---
    println!("=== A. Terrain Census ===");
    let mut counts = [0u64; 9];
    for y in 0..grid_h {
        for x in 0..grid_w {
            if let Some(t) = tiles.get_terrain(x, y) {
                counts[t.to_u8() as usize] += 1;
            }
        }
    }
    let names = [
        "Road",
        "Wall",
        "Floor",
        "Door",
        "Courtyard",
        "Garden",
        "Water",
        "Bridge",
        "Fixture",
    ];
    for (i, name) in names.iter().enumerate() {
        if counts[i] > 0 {
            println!("  {:<12} {:>10}", name, counts[i]);
        }
    }
    println!();

    // --- B. Bridge Component Analysis ---
    println!("=== B. Bridge Component Analysis ===");
    let mut bridge_tiles_list: Vec<(usize, usize)> = Vec::new();
    for y in 0..grid_h {
        for x in 0..grid_w {
            if tiles.get_terrain(x, y) == Some(Terrain::Bridge) {
                bridge_tiles_list.push((x, y));
            }
        }
    }

    let bridge_set: HashSet<(usize, usize)> = bridge_tiles_list.iter().copied().collect();
    let mut visited = vec![false; grid_w * grid_h];
    let mut components: Vec<ComponentInfo> = Vec::new();
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    let mut comp_id = 0usize;

    for &(x, y) in &bridge_tiles_list {
        let idx = y * grid_w + x;
        if visited[idx] {
            continue;
        }
        comp_id += 1;
        let mut comp_tiles: Vec<(usize, usize)> = Vec::new();
        queue.push_back((x, y));
        visited[idx] = true;

        while let Some((cx, cy)) = queue.pop_front() {
            comp_tiles.push((cx, cy));
            for &(dx, dy) in &EIGHT_DIRS {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || ny < 0 || nx >= grid_w as i32 || ny >= grid_h as i32 {
                    continue;
                }
                let nux = nx as usize;
                let nuy = ny as usize;
                let nidx = nuy * grid_w + nux;
                if bridge_set.contains(&(nux, nuy)) && !visited[nidx] {
                    visited[nidx] = true;
                    queue.push_back((nux, nuy));
                }
            }
        }

        let mut min_x = usize::MAX;
        let mut max_x = 0;
        let mut min_y = usize::MAX;
        let mut max_y = 0;
        for &(tx, ty) in &comp_tiles {
            min_x = min_x.min(tx);
            max_x = max_x.max(tx);
            min_y = min_y.min(ty);
            max_y = max_y.max(ty);
        }
        let center_x = (min_x + max_x) / 2;
        let center_y = (min_y + max_y) / 2;

        components.push(ComponentInfo {
            id: comp_id,
            tiles: comp_tiles,
            min_x,
            max_x,
            min_y,
            max_y,
            center_x,
            center_y,
        });
    }

    components.sort_by(|a, b| b.tiles.len().cmp(&a.tiles.len()));

    println!(
        "  {} Bridge tiles in {} components (8-connected)\n",
        bridge_tiles_list.len(),
        components.len()
    );
    println!(
        "  {:>4}  {:>6}  {:>24}  {:>12}  {:>8}",
        "ID", "Tiles", "Bbox", "Center (lon,lat)", "W/L ratio"
    );

    for c in &components {
        let lon = tile_to_lon(c.center_x);
        let lat = tile_to_lat(c.center_y);
        let w = c.max_x - c.min_x + 1;
        let h = c.max_y - c.min_y + 1;
        let longest = w.max(h) as f64;
        let ratio = c.tiles.len() as f64 / longest;
        println!(
            "  {:>4}  {:>6}  ({:>4},{:>4})-({:>4},{:>4})  ({:.4},{:.4})  {:>8.1}",
            c.id,
            c.tiles.len(),
            c.min_x,
            c.min_y,
            c.max_x,
            c.max_y,
            lon,
            lat,
            ratio,
        );
    }
    println!();

    // --- C. Historical Bridge Matching ---
    println!("=== C. Historical Bridge Matching ===");
    let refs = bridge_refs();
    let mut matched = 0usize;
    let max_match_dist = 100.0f64;

    for br in &refs {
        let expected_x = lon_to_tile(br.lon);
        let expected_y = lat_to_tile(br.lat);

        let mut best_dist = f64::MAX;
        let mut best_comp: Option<&ComponentInfo> = None;

        for c in &components {
            let dx = c.center_x as f64 - expected_x;
            let dy = c.center_y as f64 - expected_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best_comp = Some(c);
            }
        }

        let status = if best_dist <= max_match_dist {
            matched += 1;
            "MATCH"
        } else {
            "MISS "
        };

        if let Some(c) = best_comp {
            println!(
                "  {} {:30} dist={:>5.0}  comp#{:<4} ({:>5} tiles)  [{}]",
                status,
                br.name,
                best_dist,
                c.id,
                c.tiles.len(),
                br.notes,
            );
        } else {
            println!(
                "  {}  {:30} — no components found  [{}]",
                status, br.name, br.notes,
            );
        }
    }
    println!(
        "\n  Matched: {}/{} ({:.0}%)\n",
        matched,
        refs.len(),
        matched as f64 / refs.len() as f64 * 100.0
    );

    // --- D. Water Integrity Checks ---
    println!("=== D. Water Integrity Checks ===");

    // D1: Orphan Road tiles (Road with all 4 cardinal neighbors Water)
    let mut orphan_road = 0u64;
    for y in 1..grid_h.saturating_sub(1) {
        for x in 1..grid_w.saturating_sub(1) {
            if tiles.get_terrain(x, y) != Some(Terrain::Road) {
                continue;
            }
            let all_water = [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)]
                .iter()
                .all(|&(dx, dy)| {
                    tiles.get_terrain((x as i32 + dx) as usize, (y as i32 + dy) as usize)
                        == Some(Terrain::Water)
                });
            if all_water {
                orphan_road += 1;
            }
        }
    }
    println!(
        "  Orphan Road tiles (4 cardinal Water neighbors): {}",
        orphan_road
    );

    // D2: Water tiles with non-zero building_id
    let mut water_with_building = 0u64;
    for y in 0..grid_h {
        for x in 0..grid_w {
            if tiles.get_terrain(x, y) == Some(Terrain::Water)
                && tiles.get_building_id(x, y).is_some()
            {
                water_with_building += 1;
            }
        }
    }
    println!(
        "  Water tiles with building_id: {} {}",
        water_with_building,
        if water_with_building == 0 {
            "(OK)"
        } else {
            "(FAIL)"
        },
    );

    // D3: Island integrity
    // Ile de la Cite: ~x=3400-3900, y=3300-3600
    // Ile Saint-Louis: ~x=3900-4400, y=3350-3700
    let cite_count = count_non_water(&tiles, 3400, 3900, 3300, 3600);
    let stlouis_count = count_non_water(&tiles, 3900, 4400, 3350, 3700);
    println!(
        "  Ile de la Cite non-Water tiles: {} {}",
        cite_count,
        if cite_count >= 5000 { "(OK)" } else { "(LOW)" },
    );
    println!(
        "  Ile Saint-Louis non-Water tiles: {} {}",
        stlouis_count,
        if stlouis_count >= 5000 {
            "(OK)"
        } else {
            "(LOW)"
        },
    );

    // D4: River continuity (y=3000-4000)
    println!("\n  River continuity (y=3100-3800):");
    let mut no_water_rows = 0u32;
    let mut short_run_rows = 0u32;
    for y in 3100..3800.min(grid_h) {
        let mut max_run = 0u32;
        let mut current_run = 0u32;
        for x in 0..grid_w {
            if tiles.get_terrain(x, y) == Some(Terrain::Water) {
                current_run += 1;
                max_run = max_run.max(current_run);
            } else {
                current_run = 0;
            }
        }
        if max_run == 0 {
            no_water_rows += 1;
        } else if max_run < 50 {
            short_run_rows += 1;
        }
    }
    println!(
        "    Rows with no Water: {} {}",
        no_water_rows,
        if no_water_rows == 0 { "(OK)" } else { "(FAIL)" }
    );
    println!(
        "    Rows with longest run < 50: {} {}",
        short_run_rows,
        if short_run_rows < 10 {
            "(OK)"
        } else {
            "(WARN)"
        },
    );

    // --- Summary ---
    println!("\n=== Quality Summary ===");
    let water_ok = counts[6] >= 900_000;
    let comp_ok = components.len() >= 5 && components.len() <= 50;
    let match_ok = matched >= 8;
    let orphan_ok = orphan_road < 200;
    let building_ok = water_with_building == 0;
    let cite_ok = cite_count >= 2000;
    let stlouis_ok = stlouis_count >= 2000;
    let continuity_ok = no_water_rows == 0;

    let checks = [
        ("Water coverage > 900K", water_ok),
        ("Bridge components 5-50", comp_ok),
        ("Historical matches >= 8/20", match_ok),
        ("Orphan Road < 200", orphan_ok),
        ("No Water+building_id", building_ok),
        ("Ile de la Cite integrity", cite_ok),
        ("Ile Saint-Louis integrity", stlouis_ok),
        ("River continuity", continuity_ok),
    ];

    let mut pass = 0;
    for (name, ok) in &checks {
        let status = if *ok { "PASS" } else { "FAIL" };
        println!("  [{}] {}", status, name);
        if *ok {
            pass += 1;
        }
    }
    println!("\n  {}/{} checks passed", pass, checks.len());
}

fn count_non_water(tiles: &TileMap, x0: usize, x1: usize, y0: usize, y1: usize) -> u64 {
    let mut count = 0u64;
    let max_x = x1.min(tiles.width());
    let max_y = y1.min(tiles.height());
    for y in y0..max_y {
        for x in x0..max_x {
            if tiles.get_terrain(x, y) != Some(Terrain::Water) {
                count += 1;
            }
        }
    }
    count
}
