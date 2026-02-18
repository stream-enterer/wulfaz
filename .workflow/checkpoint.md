# Checkpoint

## Active Task
Backlog planning updates (no code changes).

## Completed
- S01–S12 rasterization simplifications all fixed (commits 90dc3e9–24f4099)
- Updated A07 spec: extract all 16 SoDUCo years, `occupants_by_year: HashMap<u16, Vec<Occupant>>`, runtime year selection via `world.active_year`
- Added match logging to A07 (per-year summary with top-10 unmatched street names)
- Added design review blockers to B05, B06, B03, C04, C05 (all procedural generation items)
- Updated architecture.md structs to match new A07 multi-year design

## Next Action
Regenerate binary data from preprocessor, visually verify Halle au Ble courtyard. Then begin A07 implementation.
