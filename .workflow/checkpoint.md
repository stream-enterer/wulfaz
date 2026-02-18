# Checkpoint

## Active Task
RON intermediate format complete. Ready for next task.

## Last Completed
Replaced binary blob (bincode 467MB) with RON intermediate format (~5-7MB).
- Preprocess extracts polygon vertices + metadata → `data/paris.ron`
- Game reconstructs TileMap from polygons at startup (rasterize, classify, BFS)
- Removed: bincode, serde-big-array deps; Serialize/Deserialize from runtime types
- All 311 tests pass, both binaries build

## Modified Files
- `Cargo.toml` — removed bincode, serde-big-array
- `src/tile_map.rs` — stripped Serialize/Deserialize from Terrain, ChunkCoord, Chunk, TileMap
- `src/components.rs` — stripped Serialize/Deserialize from Tick
- `src/registry.rs` — stripped Serialize/Deserialize from all types
- `src/loading_gis.rs` — rewritten: RON types, extract/apply split, roundtrip test
- `src/bin/preprocess.rs` — outputs RON instead of bincode
- `src/main.rs` — loads .ron instead of .bin
- `.gitignore` — paris.bin → paris.ron
- Deleted `data/paris.bin` (467MB)

## Next Action
SCALE-A07 (address + occupant loading), SCALE-A08 (Seine + bridges), or SCALE-A05 (lazy tile updates) — all unblocked.
