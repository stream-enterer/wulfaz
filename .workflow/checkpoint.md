# Checkpoint

## Active Task
None

## Completed
UI-I02 — Map overlay integration — COMPLETE

- 3 overlay color constants added to `Theme`: `overlay_hover`, `overlay_selection`, `overlay_path`
- Hover tile highlight: semi-transparent quad at cursor's map tile position
- Selected entity highlight: gold semi-transparent quad at selected entity's tile
- Wander target highlight: green semi-transparent quad at selected entity's wander target tile
- Overlays render via PanelRenderer before UI panels (correct z-order: overlays under UI, over map background, under map text)
- No border/shadow on overlay quads (pure color tint)
- Bounds-checked: overlays only drawn when tile is within viewport
- 1 new test: `overlay_colors_are_semi_transparent`
- All 296 tests pass (285 unit + 5 determinism + 6 invariant), zero warnings

## Files Modified
- src/ui/theme.rs (3 overlay color fields + defaults + 1 test)
- src/main.rs (overlay quad rendering in RedrawRequested)

## Next Action
Pick next task from backlog. Remaining:
- UI-W05 — Animation system (Tier 5 polish)
- UI-I03 — Keyboard shortcut system (Tier 5 polish)
- UI-DEMO update (Tier 4 milestone)
