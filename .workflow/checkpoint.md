# Checkpoint

## Active Task
None

## Completed
SCALE-B02 — Convert spatial queries — COMPLETE

- Added `entities_in_range(cx, cy, range)` method on World (iterator over Chebyshev square)
- Added second `rebuild_spatial_index()` call in `run_one_tick()` after `run_wander` (movement invalidates index)
- **combat.rs**: Target search uses `entities_at(ax, ay)` instead of scanning full combatants vec
- **eating.rs**: Food search uses `entities_at(ex, ey)` instead of collecting all food items
- **decisions.rs**: FoodNearby/EnemyNearby sensing uses `entities_in_range` instead of full table scan
- **decisions.rs**: `select_eat_target`/`select_attack_target` use `entities_in_range(SENSE_RANGE)` — now bounded to sense range instead of unbounded
- Tests updated: all spatial-dependent tests call `rebuild_spatial_index()` before system/query
- All 548 tests pass (319 lib + 218 integration + 5 determinism + 6 invariant), zero warnings

## Files Modified
- src/world.rs (entities_in_range method, removed dead_code allow, 1 new test)
- src/main.rs (second rebuild_spatial_index call after movement)
- src/systems/combat.rs (spatial index target lookup, 3 tests updated)
- src/systems/eating.rs (spatial index food lookup, 1 test updated)
- src/systems/decisions.rs (4 spatial query conversions, 6 tests updated)

## Next Action
SCALE-B04 — Increase A* node limit to 32K.
