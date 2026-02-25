# Checkpoint

## Active Task
B05 Phase 3 — COMPLETE (4/4 steps done)

## B05 Phase 3 Status
- P3S1 Facade run detection — done (detect_facade_runs with facing metadata)
- P3S2 Door selection heuristic — done (spacing by run length, top-3 runs per building with facing diversity)
- P3S3 Dual-door guarantee — done (force road + courtyard doors on dual-faced buildings)
- P3S4 Door-floor adjacency validation — done (scan all Door tiles for Floor/Garden neighbor)

Metrics after Phase 3:
- avg 3.0 doors/building (down from ~17 in Phase 1)
- 98.5% doors reachable, 98.1% courtyards reachable
- 18 adjacency violations (all from P2 passage carving, not P3 selection)
- 575 dual-door fixups on 575 buildings

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
- src/loading_gis.rs — FacadeRun struct, detect_facade_runs(), select_doors_from_run(), slide_to_valid(), has_interior_adjacency(), dual-door guarantee, adjacency validation, 1 new test (8 total B05 tests)
- .workflow/b05-design.md — Phase 3 step checkboxes marked

## Test Count
224 lib + 5 determinism + 6 invariant = 235 total, all passing

## Next Action
B05 Phase 4 — Edge Cases (all-wall building fix, strict door check, full diagnostic log)
