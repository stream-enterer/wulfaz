# Checkpoint

## Active Task
B05 Phase 2 — COMPLETE (4/4 steps done)

## B05 Phase 2 Status
- P2S1 BFS carve routing utility — done (bfs_to_walkable in loading_gis.rs)
- P2S2 Landlocked building passage carving — done (carve through walls to reach walkable terrain)
- P2S3 Island courtyard piercing — done (4-connected BFS regions, pierce perimeter buildings)
- P2S4 Global connectivity validation — done (BFS from Road, count reachable doors/courtyards)

## B05 Phase 1 Status
- P1S1 Door candidate detection — done
- P1S2 Garden conversion — done
- P1S3 Per-building door validation — done
- P1S4 Diagnostic log — done

## Modified Files
- src/loading_gis.rs — bfs_to_walkable(), landlocked carving, courtyard piercing, connectivity BFS, 4 new tests (7 total B05 tests)
- .workflow/b05-design.md — Phase 2 step checkboxes marked

## Test Count
511 lib + 5 determinism + 6 invariant = 522 total, all passing

## Next Action
B05 Phase 3 — Door Quality (facade runs, spacing heuristic, dual-door guarantee)
