# Checkpoint

## Active Task
B05 Phase 4 — COMPLETE (3/3 steps done)

## B05 Phase 4 Status
- P4S1 Small building interior fix — done (3-tier: skip <=4, convert 1 wall for 5-20, relaxed reclassify for 21+)
- P4S2 Strict per-building door check — done (zero tolerance, log failures with id/quartier/superficie/tiles)
- P4S3 Full diagnostic log — done (comprehensive format with all pipeline stages)

## B05 Phase 3 Status
- P3S1 Facade run detection — done
- P3S2 Door selection heuristic — done
- P3S3 Dual-door guarantee — done
- P3S4 Door-floor adjacency validation — done

## B05 Phase 2 Status
- P2S1 BFS carve routing utility — done
- P2S2 Landlocked building passage carving — done
- P2S3 Island courtyard piercing — done
- P2S4 Global connectivity validation — done

## B05 Phase 1 Status
- P1S1 Door candidate detection — done
- P1S2 Garden conversion — done
- P1S3 Per-building door validation — done
- P1S4 Diagnostic log — done

## Modified Files
- src/loading_gis.rs — small building interior fix, strict door check, full diagnostic log, 2 new tests (10 total B05 tests)
- .workflow/b05-design.md — Phase 4 step checkboxes marked

## Test Count
226 lib + 5 determinism + 6 invariant = 237 total, all passing

## Next Action
B05 Final Completion — delete SCALE-B05 from backlog, commit
