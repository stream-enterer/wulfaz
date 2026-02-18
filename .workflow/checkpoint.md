# Checkpoint

## Active Task
Paris scale-up architecture planned. Ready to begin Phase A.

## Next Action
SCALE-A01: Expand Terrain enum for city tiles. Then SCALE-A02: chunked TileMap.

## Key Decisions
- 17M tiles, ~1M population, 64×64 chunks
- Three-zone LOD: Active ~4K / Nearby ~50K / Statistical ~950K aggregate
- Terrain: Road, Building, Courtyard, Garden, Water, Bridge, Wall
- HPA* for cross-city pathfinding on chunk boundaries
- District aggregates replace individual entities in statistical zone
- Keep HashMap<Entity, T> for active zone — profile before optimizing
- SIM features can develop in parallel on test map after Phase B

## Reference
`.workflow/architecture.md` — full technical spec
`.workflow/backlog.md` — phased task list with dependencies
