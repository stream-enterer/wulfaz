# Checkpoint

## Active Task
None

## Completed
SCALE-A08 — Seine river + bridge placement — COMPLETE

- ALPAGE Vasserot Hydrography shapefile (87 polygons, 51 in viewport)
- Reprojected NTF Lambert I → WGS84 via ogr2ogr
- Modified: `src/loading_gis.rs` (rasterize_water + count_bridge_crossings), `src/bin/preprocess.rs`
- Results: 1,047,748 water tiles, 35 gap tiles healed, 3,298 bridge tiles (1,406 components)
- Known gap: ~150 tiles east of Pont de Sully (ALPAGE coverage hole)
- 439 tests pass (214 lib + 214 bin + 5 determinism + 6 invariants)

## Next Action
Pick next task from backlog.
