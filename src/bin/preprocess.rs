//! One-time GIS preprocessing tool.
//! Reads Paris shapefiles, extracts polygon data, rasterizes tiles, writes binary + RON outputs.
//!
//! Usage: cargo run --bin preprocess [PARIS_DATA_DIR] [OUTPUT_DIR]
//! Defaults: PARIS_DATA from env or "../../paris/data", output dir "data/"

use std::time::Instant;

use wulfaz::loading_gis;

fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    let paris_data = if args.len() > 1 {
        args[1].clone()
    } else {
        std::env::var("PARIS_DATA").unwrap_or_else(|_| "../../paris/data".into())
    };

    let output_dir = if args.len() > 2 {
        args[2].clone()
    } else {
        "data".into()
    };

    let buildings_shp = format!("{paris_data}/buildings/BATI.shp");
    let blocks_shp = format!("{paris_data}/plots/Vasserot_Ilots.shp");

    assert!(
        std::path::Path::new(&buildings_shp).exists(),
        "Buildings shapefile not found: {buildings_shp}"
    );
    assert!(
        std::path::Path::new(&blocks_shp).exists(),
        "Blocks shapefile not found: {blocks_shp}"
    );

    println!("Reading shapefiles from: {paris_data}");
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
    let (tiles, mut buildings, blocks, quartier_names) = loading_gis::rasterize_paris(&data);
    println!("Rasterized in {:.1}s", raster_start.elapsed().as_secs_f64());

    // A07: Load addresses + occupants
    let addresses_shp = format!("{paris_data}/addresses/Num_Voies_Vasserot.shp");
    let gpkg_path = format!("{paris_data}/soduco/data/data_extraction_with_population.gpkg");

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

    // Save binary tiles + metadata RON
    let tiles_path = format!("{output_dir}/paris.tiles");
    let meta_path = format!("{output_dir}/paris.meta.ron");
    println!("Saving binary tiles to {tiles_path}...");
    loading_gis::save_paris_binary(
        &tiles,
        &buildings,
        &blocks,
        &quartier_names,
        &tiles_path,
        &meta_path,
    );

    let tile_size = std::fs::metadata(&tiles_path).map(|m| m.len()).unwrap_or(0);
    let meta_size = std::fs::metadata(&meta_path).map(|m| m.len()).unwrap_or(0);
    println!(
        "Binary: {:.1}MB tiles + {:.1}MB metadata",
        tile_size as f64 / (1024.0 * 1024.0),
        meta_size as f64 / (1024.0 * 1024.0),
    );

    // Save RON (debug/fallback)
    let ron_path = format!("{output_dir}/paris.ron");
    println!("Saving RON to {ron_path} (debug fallback)...");
    let save_start = Instant::now();
    loading_gis::save_paris_ron(&data, &ron_path);
    let ron_size = std::fs::metadata(&ron_path).map(|m| m.len()).unwrap_or(0);
    println!(
        "RON: {:.1}MB in {:.1}s",
        ron_size as f64 / (1024.0 * 1024.0),
        save_start.elapsed().as_secs_f64()
    );

    println!("Done.");
}
