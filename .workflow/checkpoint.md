# Checkpoint

## Active Task
None

## Completed
UI-P03 — Panel renderer — COMPLETE

- `PanelRenderer` struct with separate wgpu quad pipeline (no texture, uniform-only bind group)
- `PanelVertex`: 64 bytes (position, uv, size_px, bg_color, border_color, border_width, shadow_width)
- All per-panel style baked into vertex attributes — arbitrary panels in one draw call
- `panel.wgsl`: SDF-style fragment shader — gold border stroke, inner shadow falloff, center fill
- sRGB→linear in shader, premultiplied alpha output (matches text pipeline)
- Render order: panels first, then text on top
- Test panel in main.rs: parchment bg, gold border, inner shadow
- Zero warnings, builds clean

## Files Modified
- src/panel.rs (new — PanelVertex, PanelUniforms, PanelRenderer)
- src/panel.wgsl (new — vertex + fragment shader)
- src/main.rs (mod panel, App.panel field, GpuState::render takes both renderers)

## Next Action
Pick next task from backlog. UI-W01 (widget tree) is the last Tier 1 task.
