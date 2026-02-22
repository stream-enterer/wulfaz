# Checkpoint

## Active Task
None

## Completed
SCALE-B04 — Increase A* node limit to 32K — COMPLETE

- Changed `MAX_EXPANDED` from 8192 to 32,768 in `TileMap::find_path()`
- Stopgap for larger-map pathing until HPA* (SCALE-D03)
- All 548 tests pass, zero warnings

## Files Modified
- src/tile_map.rs (line 650: MAX_EXPANDED 8192 → 32_768)

## Next Action
SCALE-B05 — Door placement + passage carving (BLOCKED: design review required)
