# Checkpoint

## Active Task
All 12 rasterization pipeline simplifications (S01–S12) are fixed.

## Completed
- **S01** (90dc3e9): Inner ring support via extract_rings + scanline_fill_multi
- **S02+S03+S04** (583c8ff): Extraction diagnostics + pre-filter at actual grid resolution
- **S05+S06+S07+S12** (f4df6de): Three-pass rasterization, BATI=2 carving, expanded garden terms, tile list rebuild
- **S09+S10+S11** (0f52fcd): Majority-vote block assignment, overlap logging

## Pipeline (final)
```
1. Blocks         → Courtyard + block_id + quartier_id
2. BATI=1         → Wall + building_id  (majority-vote block assignment)
3. ALL BATI=2     → Courtyard/Garden, clear building_id, update tile lists
4. classify_walls_floors  (on surviving BATI=1 tiles only)
5. fill_quartier_roads    (BFS quartier_id to Road tiles)
```

## Next Action
Run preprocessor to regenerate binary data, visually verify Halle au Ble courtyard.
