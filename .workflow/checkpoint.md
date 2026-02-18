# Checkpoint

## Active Task
SCALE-A01 complete. Ready for SCALE-A02: chunked TileMap.

## Next Action
SCALE-A02: Implement `HashMap<ChunkCoord, Chunk>` with 64×64 chunks. All tile accessors route through chunk lookup.

## Key Decisions
- 30M tiles (6,309 x 4,753), ~1M population, 64×64 chunks, ~7,400 chunks
- Three-zone LOD: Active ~4K / Nearby ~50K / Statistical ~950K aggregate
- Terrain enum: Road, Wall, Floor, Door, Courtyard, Garden, Water, Bridge (implemented)
- Walkability: Wall and Water blocked, all others walkable
- Temperature targets: Water 10°, Bridge 12°, Garden 15°, Wall 15°, Road 16°, Courtyard 16°, Door 17°, Floor 18°
- Default terrain: Road (16°C), replaces old Grass (20°C)
- Render chars: Road=. Wall=# Floor=_ Door=+ Courtyard=, Garden=" Water=~ Bridge==
- Per-tile ID layers: building_id, block_id, quartier_id (all write-once from GIS data)
- Registries: BuildingRegistry, BlockRegistry, StreetRegistry — persist all shapefile metadata

## Modified Files (A01)
- `src/tile_map.rs` — Terrain enum, walkability, default Road
- `src/systems/temperature.rs` — 8 terrain temperature targets
- `src/render.rs` — terrain display characters
- `src/loading.rs` — random scatter for test map
- `src/systems/wander.rs` — test fix (Stone→Wall)
- `data/terrain.kdl` — terrain definitions

## Reference
`.workflow/architecture.md` — full technical spec
`.workflow/backlog.md` — phased task list with dependencies
`~/Development/paris/PROJECT.md` — GIS data reference
