# Checkpoint

## Active Task
None — ready for next task.

## Last Completed
SCALE-A04 + SCALE-A05: Chunk-aware renderer infrastructure + lazy temperature updates.
- Added Terrain::target_temperature() centralizing temp targets.
- Added per-chunk at_equilibrium flag (cleared by set_terrain, not serialized).
- Added ChunkRange, chunk_at/chunk_at_mut, visible_chunk_range for renderer.
- Refactored run_temperature to iterate by chunk, skip equilibrium chunks.
- Added initialize_temperatures() called after map load — all tiles start at target.
- run_temperature is O(1) no-op in steady state (all chunks at equilibrium).
- 10 new tests, all 164 pass.

## Modified Files
- `src/tile_map.rs` — target_temperature, at_equilibrium, ChunkRange, chunk methods, initialize_temperatures
- `src/systems/temperature.rs` — chunk-aware iteration with equilibrium skip
- `src/main.rs` — initialize_temperatures() call after map load

## Next Action
Pick next task from backlog.
