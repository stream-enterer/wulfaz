# Checkpoint

## Active Task
Vec BuildingRegistry fix complete. Ready for user visual verification + commit.

## Last Completed
Fixed 72% building tiles stuck as Wall due to duplicate Identif keys in HashMap.
- Root cause: BATI.shp has 40,350 records but only 17,155 unique Identif values. HashMap::insert silently overwrote 23,195 buildings' tile data.
- Fix: BuildingRegistry changed from HashMap<BuildingId, BuildingData> to Vec<BuildingData>. BuildingId is now a 1-based sequential index. Added `identif: u32` field to BuildingData. Added `identif_index: HashMap<u32, Vec<BuildingId>>` reverse lookup for A07.
- Result: Floor tiles 2M→7.3M (79% of building tiles), all 9.2M tiles now classified.
- All 319 tests pass, binary tiles regenerated.

## Modified Files
- `src/registry.rs` — HashMap→Vec BuildingRegistry, added identif field, reverse lookup, new tests
- `src/loading_gis.rs` — sequential BuildingId counter, .values()→.iter(), updated save/load/classify
- `.workflow/architecture.md` — updated BuildingRegistry spec

## Next Action
User verifies visual output. Then commit + update backlog if needed.
