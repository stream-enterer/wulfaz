# Checkpoint

## Active Task
None

## Completed
UI-P01 — Per-vertex color attribute — COMPLETE

- Added `color: [f32; 4]` to `TextVertex` (16 → 32 bytes/vertex)
- Pipeline vertex layout: new attribute at location 2 (Float32x4, offset 16)
- `text.wgsl`: per-vertex color passed through vs_main → fs_main, replaces uniform fg_color in compositing
- `build_vertices()` and `prepare_text_shaped()` accept color parameter
- `prepare_text()` and `prepare_map()` forward color to internals
- All callers in main.rs pass `[FG_SRGB[0], FG_SRGB[1], FG_SRGB[2], 1.0]`
- Uniform fg_color remains in buffer (no bind group change) but unused by shader
- Zero warnings, builds clean

## Files Modified
- src/font.rs (TextVertex, pipeline layout, build_vertices, prepare_text_shaped, prepare_text, prepare_map)
- src/text.wgsl (VertexInput, VertexOutput, vs_main, fs_main)
- src/main.rs (callers pass fg4 color)

## Next Action
Pick next task from backlog.
