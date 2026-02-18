# Checkpoint

## Active Task
Fix rasterization pipeline simplifications S01–S12.

## Completed
- **S01** (90dc3e9): Inner ring support. `extract_rings` + `scanline_fill_multi`
  with even-odd hole exclusion. RON structs, extraction, rasterization all wired.

## Remaining (S02–S12)
See audit in previous checkpoint (f3983f8) for full details per simplification.

Correct final pipeline ordering:
```
1. Blocks         → Courtyard + block_id + quartier_id
2. BATI=1         → Wall + building_id  (overwrites Courtyard)
3. BATI=2         → Courtyard/Garden + clear building_id  (carves holes)
4. classify_walls_floors  (on surviving BATI=1 tiles only)
5. fill_quartier_roads    (BFS quartier_id to Road tiles)
```

## Next Action
S02–S04 extraction diagnostics, then S05 pass reorder (the critical fix).
