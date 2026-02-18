//! One-time GIS preprocessing tool.
//! Reads Paris shapefiles, rasterizes onto tile grid, writes binary map file.
//!
//! Usage: cargo run --bin preprocess [PARIS_DATA_DIR] [OUTPUT_PATH]
//! Defaults: PARIS_DATA from env or "../../paris/data", output "data/paris.bin"

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

    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        "data/paris.bin".into()
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
        "Saving to {output_path} ({} buildings, {} blocks, {} quartiers)...",
        data.buildings.buildings.len(),
        data.blocks.blocks.len(),
        data.quartier_names.len(),
    );

    let save_start = Instant::now();
    loading_gis::save_map_data(&data, &output_path);

    let file_size = std::fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);
    println!(
        "Done: {:.1}MB in {:.1}s",
        file_size as f64 / (1024.0 * 1024.0),
        save_start.elapsed().as_secs_f64()
    );
}
