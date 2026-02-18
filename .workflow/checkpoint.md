# Checkpoint

## Active Task
SCALE-A03 complete. Ready for next task.

## Next Action
SCALE-A07 (address + occupant loading), SCALE-A08 (Seine + bridges), or SCALE-A05 (lazy tile updates) — all unblocked.

## Key Decisions (A03)
- `src/registry.rs`: BuildingId(u32), BlockId(u16), BuildingData, BlockData, BuildingRegistry, BlockRegistry
- Per-chunk ID arrays: building_id [u32; 4096], block_id [u16; 4096], quartier_id [u8; 4096] — 28KB/chunk overhead
- Scanline rasterization ported from Python render_city.py: even-odd fill rule, y+0.5 ray casting
- Loading order: blocks → buildings → wall/floor classification → quartier BFS
- Wall = any cardinal neighbor not in same building; Floor = all neighbors in same building
- Quartier assignment: multi-source BFS from block/building tiles fills road tiles
- Floor count estimation: <50m²→2, 50-150→3, 150-400→4, >400→5
- GIS loading conditional on PARIS_DATA env var; falls back to KDL terrain
- Coordinate conversion: LAT_CENTER=48.857, M_PER_DEG_LON≈73490, M_PER_DEG_LAT=111320, PAD=30m
- Grid size: ~6369×4813 tiles, ~99×76 chunks

## Modified Files (A03)
- `Cargo.toml` — added `shapefile = "0.7"`
- `src/registry.rs` — NEW: ID types + registry structs
- `src/loading_gis.rs` — NEW: full GIS loading pipeline with scanline rasterizer
- `src/tile_map.rs` — added building_id, block_id, quartier_id arrays to Chunk + accessors
- `src/world.rs` — added BuildingRegistry, BlockRegistry, quartier_names fields
- `src/main.rs` — conditional GIS loading, mod declarations
- `src/lib.rs` — added mod registry, mod loading_gis

## Reference
`.workflow/backlog.md` — phased task list with dependencies
`~/Development/paris/PROJECT.md` — GIS data reference
