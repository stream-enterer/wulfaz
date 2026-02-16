# Checkpoint

## Active Task
None — A* pathfinding implementation complete.

## Completed
- A* pathfinding added to TileMap (8-directional, uniform cost, Chebyshev heuristic)
- Wander system rewritten as unified movement: Eat/Attack pathfind to target, Wander pathfinds to random destination, random walk fallback
- Decision system updated: target selection finds nearest on map (not same-tile), FoodNearby/EnemyNearby sense within Chebyshev distance 20
- WanderTarget component added for caching wander destinations
- 8-directional movement (added in prior change, kept)

## Modified Files
- `src/tile_map.rs` — Terrain::is_walkable(), TileMap::is_walkable(), TileMap::find_path()
- `src/components.rs` — WanderTarget struct
- `src/world.rs` — wander_targets table, despawn, validate
- `src/systems/wander.rs` — unified A* movement system
- `src/systems/decisions.rs` — distance-based target selection + sense range

## Decisions
- A* lives on TileMap (not World) to avoid borrow conflicts with world.rng
- SENSE_RANGE = 20 Chebyshev for FoodNearby/EnemyNearby
- WANDER_RANGE = 10 for random destination picking
- MAX_EXPANDED = 2048 nodes for A* search limit
- Target selection: nearest first, then nutrition/health, then entity ID tiebreak
- Eating/combat systems unchanged — still gate on co-location

## Next Action
All tests pass (255 total). Ready for commit.
