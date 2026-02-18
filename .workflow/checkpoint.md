# Checkpoint

## Active Task
Fix BATI field misinterpretation — two-pass rasterization.

## What Changed
- `src/registry.rs`: Fixed bati doc comment, added perimetre/geox/geoy/date_coyec to BuildingData, ilots_vass to BlockData (all serde(default)).
- `src/loading_gis.rs`: Added same fields to ParisBuildingRon/ParisBlockRon. Extracted PERIMETRE/GEOX/GEOY/DATE_COYEC/ILOTS_VASS from shapefiles. Replaced single building loop with two-pass rasterization: Pass 1 (BATI=2 gardens → Garden), Pass 2 (BATI=1 → Wall+building_id). BATI=2 non-gardens and BATI=3 skipped entirely.
- `.workflow/architecture.md`: Fixed BATI description, added new fields to BuildingData/BlockData code blocks, updated Identif note.
- 6 new tests: bati1_rasterized, bati2_courtyard, bati2_garden, bati3_not_rasterized, bati1_overwrites_garden, only_bati1_in_registry.
- Updated existing tests for new struct fields; changed duplicate_identif test to use two BATI=1 buildings.

## Status
All 170 unit tests + 11 integration tests pass. Ready to commit.

## Next Action
Run `cargo run --bin preprocess` to regenerate binary data with corrected rasterization. Verify log output shows ~21K buildings, ~29 gardens, ~19K skipped BATI=2.
