# Checkpoint

## Active Task
TileMap Vec storage + preprocess rasterization complete. Ready for next task.

## Last Completed
HashMap→Vec TileMap + binary tile serialization + preprocess rasterization pipeline.
- TileMap: HashMap<ChunkCoord, Chunk> → Vec<Chunk> with flat indexing (chunks_x field)
- Terrain: #[repr(u8)] with to_u8/from_u8 conversion methods
- Chunk: binary read/write (32KB/chunk: terrain + building_id + block_id + quartier_id)
- TileMap: binary read/write with WULF header (32 bytes)
- Registry types: re-added Clone, Serialize, Deserialize
- ParisMetadataRon: quartier_names + buildings + blocks (tile lists stripped)
- classify_walls_floors/fill_quartier_roads: refactored from &mut World to &mut TileMap
- rasterize_paris: standalone function returning (TileMap, BuildingRegistry, BlockRegistry, Vec<String>)
- save_paris_binary/load_paris_binary: binary tiles + metadata RON
- Preprocess: extracts polygons → rasterizes → saves binary + RON
- Game loading: binary→RON→KDL cascade
- All 317 tests pass, both binaries build in release

## Modified Files
- `src/tile_map.rs` — HashMap→Vec, Clone on Chunk, repr(u8) Terrain, binary read/write
- `src/registry.rs` — added Clone, Serialize, Deserialize to data types
- `src/loading_gis.rs` — rasterize_paris, save/load_paris_binary, refactored classify/BFS
- `src/bin/preprocess.rs` — rasterize + save binary tiles + metadata
- `src/main.rs` — binary→RON→KDL loading cascade
- `.gitignore` — added paris.tiles, paris.meta.ron

## Next Action
Run preprocess to generate binary tiles, verify game loads from binary.
SCALE-A07 (address + occupant loading), SCALE-A08 (Seine + bridges), or SCALE-A05 (lazy tile updates) — all unblocked.
