# Checkpoint

## Active Task
B05 Phase 1 — COMPLETE (4/4 steps done)

## B05 Phase 1 Status
- P1S1 Door candidate detection — done (place_doors in loading_gis.rs, preprocess call site)
- P1S2 Garden conversion — done (parc/jardin Floor→Garden)
- P1S3 Per-building door validation — done (doorless building warnings)
- P1S4 Diagnostic log — done (door count, garden count, doorless count)

## Modified Files
- src/loading_gis.rs — place_doors() function + 3 unit tests
- src/bin/preprocess.rs — place_doors() call site between occupant loading and binary save
- .workflow/b05-design.md — Phase 1 step checkboxes marked

## Test Count
219 lib + 506 tile_map + 5 determinism + 6 invariant = 736 total, all passing

## Next Action
B05 Phase 2 — Connectivity (BFS carving, landlocked buildings, island courtyards)
