//! One-time GIS preprocessing tool.
//! Reads Paris shapefiles, extracts polygon data, rasterizes tiles, writes binary + RON outputs.
//!
//! Usage: cargo run --bin preprocess [--force] [SOURCE_DATA_DIR] [OUTPUT_DIR]
//! Defaults: SOURCE_DATA from env or "source-data", output dir "data/"

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use wulfaz::loading_gis;

// --- Stamp-file types for incremental builds ---

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct FileFingerprint {
    path: PathBuf,
    size: u64,
    mtime_secs: u64,
    mtime_nanos: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct PreprocessStamp {
    inputs: Vec<FileFingerprint>,
}

fn fingerprint(path: &Path) -> Option<FileFingerprint> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    Some(FileFingerprint {
        path: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
        size: meta.len(),
        mtime_secs: mtime.as_secs(),
        mtime_nanos: mtime.subsec_nanos(),
    })
}

fn collect_input_fingerprints(source_dir: &str) -> Vec<FileFingerprint> {
    let mut fps = Vec::new();

    // Fingerprint the preprocess binary itself
    if let Ok(exe) = std::env::current_exe()
        && let Some(fp) = fingerprint(&exe)
    {
        fps.push(fp);
    }

    // Shapefile sidecar extensions
    let prefixes = [
        "BATI",
        "Vasserot_Ilots",
        "Num_Voies_Vasserot",
        "Vasserot_Hydrographie_WGS84",
    ];
    let extensions = ["shp", "shx", "dbf", "prj", "cpg"];

    for prefix in &prefixes {
        for ext in &extensions {
            let p = PathBuf::from(source_dir).join(format!("{prefix}.{ext}"));
            if p.exists()
                && let Some(fp) = fingerprint(&p)
            {
                fps.push(fp);
            }
        }
    }

    // GeoPackage
    let gpkg = PathBuf::from(source_dir).join("data_extraction_with_population.gpkg");
    if gpkg.exists()
        && let Some(fp) = fingerprint(&gpkg)
    {
        fps.push(fp);
    }

    fps.sort_by(|a, b| a.path.cmp(&b.path));
    fps
}

fn load_stamp(path: &Path) -> Option<PreprocessStamp> {
    let text = std::fs::read_to_string(path).ok()?;
    ron::from_str(&text).ok()
}

fn save_stamp(stamp: &PreprocessStamp, path: &Path) {
    let text = ron::ser::to_string_pretty(stamp, ron::ser::PrettyConfig::default())
        .expect("stamp serialization failed");
    std::fs::write(path, text).expect("failed to write stamp file");
}

fn outputs_exist(output_dir: &str) -> bool {
    ["paris.tiles", "paris.meta.bin", "paris.ron.zst"]
        .iter()
        .all(|name| Path::new(output_dir).join(name).exists())
}

// --- Main ---

fn main() {
    env_logger::init();

    let raw_args: Vec<String> = std::env::args().collect();
    let force = raw_args.iter().any(|a| a == "--force");
    let args: Vec<&String> = raw_args.iter().filter(|a| !a.starts_with("--")).collect();

    let source_dir = if args.len() > 1 {
        args[1].to_string()
    } else {
        std::env::var("SOURCE_DATA").unwrap_or_else(|_| "source-data".into())
    };

    let output_dir = if args.len() > 2 {
        args[2].to_string()
    } else {
        "data".into()
    };

    // --- Incremental build check ---
    let stamp_path = PathBuf::from(&output_dir).join(".preprocess.stamp");
    if !force {
        let current_fps = collect_input_fingerprints(&source_dir);
        if let Some(old_stamp) = load_stamp(&stamp_path)
            && old_stamp.inputs == current_fps
            && outputs_exist(&output_dir)
        {
            println!("Up-to-date — inputs and outputs unchanged. Use --force to rebuild.");
            return;
        }
    }

    let buildings_shp = format!("{source_dir}/BATI.shp");
    let blocks_shp = format!("{source_dir}/Vasserot_Ilots.shp");

    assert!(
        std::path::Path::new(&buildings_shp).exists(),
        "Buildings shapefile not found: {buildings_shp}"
    );
    assert!(
        std::path::Path::new(&blocks_shp).exists(),
        "Blocks shapefile not found: {blocks_shp}"
    );

    println!("Reading shapefiles from: {source_dir}");
    let data = loading_gis::build_from_shapefiles(&buildings_shp, &blocks_shp);

    println!(
        "Extracted: {} buildings, {} blocks, {} quartiers ({}×{} grid)",
        data.buildings.len(),
        data.blocks.len(),
        data.quartier_names.len(),
        data.grid_width,
        data.grid_height,
    );

    // Rasterize: polygons → tile arrays + registries
    println!("Rasterizing...");
    let raster_start = Instant::now();
    let (mut tiles, mut buildings, blocks, quartier_names) = loading_gis::rasterize_paris(&data);
    println!("Rasterized in {:.1}s", raster_start.elapsed().as_secs_f64());

    // A08: Rasterize water (Seine, canals) — must run AFTER rasterize_paris()
    let water_shp = format!("{source_dir}/Vasserot_Hydrographie_WGS84.shp");
    if std::path::Path::new(&water_shp).exists() {
        println!("Rasterizing water...");
        loading_gis::rasterize_water(&water_shp, &mut tiles);
    } else {
        println!("Water shapefile not found: {water_shp} (skipping)");
    }

    // A07: Load addresses + occupants
    let addresses_shp = format!("{source_dir}/Num_Voies_Vasserot.shp");
    let gpkg_path = format!("{source_dir}/data_extraction_with_population.gpkg");

    if std::path::Path::new(&addresses_shp).exists() {
        println!("Loading addresses from {addresses_shp}...");
        loading_gis::load_addresses(&addresses_shp, &mut buildings);
    } else {
        println!("Address shapefile not found: {addresses_shp} (skipping)");
    }

    if std::path::Path::new(&gpkg_path).exists() {
        println!("Loading occupants from {gpkg_path}...");
        loading_gis::load_occupants(&gpkg_path, &mut buildings);
    } else {
        println!("GeoPackage not found: {gpkg_path} (skipping)");
    }

    // Save binary tiles + bincode metadata + debug RON
    let tiles_path = format!("{output_dir}/paris.tiles");
    let meta_bin_path = format!("{output_dir}/paris.meta.bin");
    let meta_ron_path = format!("{output_dir}/paris.meta.ron");
    println!("Saving binary tiles + bincode metadata...");
    loading_gis::save_paris_binary(
        &tiles,
        &buildings,
        &blocks,
        &quartier_names,
        &tiles_path,
        &meta_bin_path,
        &meta_ron_path,
    );

    let tile_size = std::fs::metadata(&tiles_path).map(|m| m.len()).unwrap_or(0);
    let meta_size = std::fs::metadata(&meta_bin_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let meta_ron_size = std::fs::metadata(&meta_ron_path)
        .map(|m| m.len())
        .unwrap_or(0);
    println!(
        "Binary: {:.1}MB tiles + {:.1}MB meta (bincode) + {:.1}MB meta (RON debug)",
        tile_size as f64 / (1024.0 * 1024.0),
        meta_size as f64 / (1024.0 * 1024.0),
        meta_ron_size as f64 / (1024.0 * 1024.0),
    );

    // Save compressed RON (debug/fallback)
    let ron_path = format!("{output_dir}/paris.ron.zst");
    println!("Saving compressed RON to {ron_path} (debug fallback)...");
    let save_start = Instant::now();
    loading_gis::save_paris_ron(&data, &ron_path);
    let ron_size = std::fs::metadata(&ron_path).map(|m| m.len()).unwrap_or(0);
    println!(
        "RON: {:.1}MB in {:.1}s",
        ron_size as f64 / (1024.0 * 1024.0),
        save_start.elapsed().as_secs_f64()
    );

    // Write stamp file on success
    let stamp = PreprocessStamp {
        inputs: collect_input_fingerprints(&source_dir),
    };
    save_stamp(&stamp, &stamp_path);

    println!("Done.");
}
