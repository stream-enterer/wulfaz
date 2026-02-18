# Checkpoint

## Active Task
A07 — Address + Occupant Loading (All Years) — COMPLETE

## Completed
- Added `rusqlite = { version = "0.33", features = ["bundled"] }` to Cargo.toml
- `Occupant` now has `naics: String` field
- `BuildingData.occupants` → `occupants_by_year: HashMap<u16, Vec<Occupant>>`
- Added `StreetId`, `StreetData`, `StreetRegistry` types with `build_from_buildings()`
- Added `streets: StreetRegistry` and `active_year: u16` to World
- Updated all `occupants: Vec::new()` → `occupants_by_year: HashMap::new()` in loading_gis.rs + registry.rs
- Implemented `normalize_street_name()` with accent folding, abbreviation expansion, prefix stripping
- Implemented `load_addresses()` — reads Vasserot address shapefile, matches via Identif
- Implemented `load_occupants()` — reads SoDUCo GeoPackage via rusqlite, fuzzy street name + house number matching
- Updated `preprocess.rs` to call load_addresses + load_occupants (skips gracefully if files missing)
- Updated `load_paris_binary()` to reconstruct StreetRegistry and set active_year=1845
- 8 new tests: normalize patterns, StreetRegistry build, BuildingData RON roundtrip
- All 189 tests pass (178 lib + 5 determinism + 6 invariants)

## Next Action
Run `cargo run --bin preprocess` with actual Paris data to verify match rates.
