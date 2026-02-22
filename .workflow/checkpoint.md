# Checkpoint

## Active Task
None

## Completed
SCALE-B01 — Spatial index — COMPLETE

- Added `smallvec = "1"` dependency
- New field on World: `spatial_index: HashMap<(i32,i32), SmallVec<[Entity; 4]>>`
- `rebuild_spatial_index()` — clears and rebuilds from `body.positions`, filters by `alive`
- `entities_at(x, y)` — O(1) lookup returning `&[Entity]`
- Called at start of every tick in `run_one_tick()`, before Phase 1
- 3 new tests: rebuild indexes positions, excludes dead entities, clears on rebuild
- All 329 tests pass (318 lib + 5 determinism + 6 invariant), zero warnings (except expected dead_code for entities_at until B02)

## Files Modified
- Cargo.toml (smallvec dep)
- src/world.rs (spatial_index field, rebuild method, query method, 3 tests)
- src/main.rs (rebuild call at tick start)

## Next Action
SCALE-B02 — Convert spatial queries in combat, eating, decisions to use spatial index.
