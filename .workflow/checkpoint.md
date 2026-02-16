# Checkpoint

## Active Task
Accurate diagonal movement cost (√2).

## Modified Files
- `src/tile_map.rs` — octile heuristic, CARDINAL_COST/DIAGONAL_COST constants, direction-dependent step cost, is_diagonal_step helper, 2 new tests
- `src/systems/wander.rs` — TICKS_PER_DIAGONAL constant, per-branch cooldown (cardinal vs diagonal), updated 2 test assertions
- `CLAUDE.md` — updated Spatial Scale section (diagonal cost documented)

## Decisions
- Fixed-point integer costs (100/141) to avoid float comparison in BinaryHeap
- Cooldown moved from shared calculation into each movement branch (A* path, random fallback, idle)
- Tests accept either cardinal or diagonal cooldown where random direction is seed-dependent

## Status
All 259 tests pass. Ready to commit.
