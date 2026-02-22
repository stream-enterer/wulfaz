# Checkpoint

## Active Task
None

## Completed
UI-W01 — Widget tree core + layout model — COMPLETE

- `WidgetTree`: slotmap arena with parent/child tracking, dirty flags
- `WidgetId`: slotmap key for O(1) widget lookup
- `Widget` flat enum: `Panel`, `Label`, `Button` (DD-1)
- Box model layout: `Position` (Fixed/Percent), `Sizing` (Fixed/Percent/Fit), `Edges` (padding/margin)
- `measure()` → intrinsic size, `layout()` → positioned rects, `draw()` → DrawList
- `DrawList`: `Vec<PanelCommand>` + `Vec<TextCommand>`, consumed by PanelRenderer + FontRenderer
- `demo_tree()`: Tier 1 showcase — parchment panel + 3 colored labels (gold/white/red)
- Integrated into main.rs render pipeline (replaces hardcoded test panel)
- 6 unit tests: insert/remove, dirty propagation, fixed layout, percent sizing, draw output
- All 231 tests pass, zero errors

## Files Modified
- Cargo.toml (added slotmap = "1")
- src/ui/mod.rs (new — WidgetTree, layout, draw, demo_tree, tests)
- src/ui/widget.rs (new — Widget enum)
- src/ui/draw.rs (new — DrawList, PanelCommand, TextCommand)
- src/main.rs (mod ui, DrawList integration in render flow)

## Next Action
Pick next task from backlog. Tier 1 is now complete (P01 + P03 + W01).
Tier 2 candidates: UI-P02 (multi-font atlas), UI-R02 (theme), UI-W02 (input routing).
