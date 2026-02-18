# Checkpoint

## Active Task
Paris scale-up architecture planned. Ready to begin Phase A.

## Next Action
SCALE-A01: Expand Terrain enum for city tiles. Then SCALE-A02: chunked TileMap.

## Key Decisions
- 30M tiles (6,309 x 4,753), ~1M population, 64×64 chunks, ~7,400 chunks
- Three-zone LOD: Active ~4K / Nearby ~50K / Statistical ~950K aggregate
- Terrain: Road, Wall, Floor, Door, Courtyard, Garden, Water, Bridge
- Per-tile ID layers: building_id, block_id, quartier_id (all write-once from GIS data)
- Registries: BuildingRegistry, BlockRegistry, StreetRegistry — persist all shapefile metadata
- Phase A = raw geometry + data loading only. No procedural generation (doors, passages, furniture, gardens deferred to Phase B)
- Seine + bridges are hardcoded polygon data, rasterized same as blocks (A08)
- HPA* for cross-city pathfinding on chunk boundaries
- District aggregates replace individual entities in statistical zone
- Keep HashMap<Entity, T> for active zone — profile before optimizing
- SIM features can develop in parallel on test map after Phase B

## Reference
`.workflow/architecture.md` — full technical spec (data structures, Chunk definition, registries)
`.workflow/backlog.md` — phased task list with dependencies
`~/Development/paris/PROJECT.md` — GIS data reference (schemas, queries, coordinate conversion)
`~/Development/paris/render_city.py` — working Python rasterization reference
