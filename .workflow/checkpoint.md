# Checkpoint

## Active Task
Standardize spatial scale: 1 tile = 1 meter.

## Modified Files
- `CLAUDE.md` — added Spatial Scale section
- `src/world.rs` — map 64→256, test assertion updated
- `src/systems/wander.rs` — TICKS_PER_METER=100, WANDER_RANGE=30, single-step movement, tests fixed
- `src/systems/decisions.rs` — SENSE_RANGE=30
- `src/tile_map.rs` — MAX_EXPANDED=8192
- `data/creatures.kdl` — Wolf 3→5, Deer 4→6, added m/s comments
- `src/systems/combat.rs` — melee range doc comment
- `src/systems/eating.rs` — pickup range doc comment
- `src/systems/temperature.rs` — drift rate doc comment
- `src/render.rs` — test uses explicit 64×64 tilemap
- `tests/invariants.rs` — test_world() helper with 64×64 tilemap
- `tests/determinism.rs` — test_world() helper, different_seeds tick count 50→500

## Decisions
- TICKS_PER_METER=100 (at 100 ticks/sec, speed 1 = 1 m/s)
- Removed multi-step movement: 1 tile per action, speed only affects cooldown
- Integration tests use 64×64 tilemaps for speed (temperature iterates all tiles)
- different_seeds_different_results needs 500 ticks because higher cooldowns delay combat convergence

## Status
All 250 tests pass. Ready to commit.
