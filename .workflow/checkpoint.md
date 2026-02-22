# Checkpoint

## Active Task
UI-100 + UI-101 — Row and Column auto-layout containers — COMPLETE

## Completed
- Added `CrossAlign` enum (Start, Center, End, Stretch) to `widget.rs`
- Added `Widget::Row { gap, align }` and `Widget::Column { gap, align }` variants
- Implemented `measure_node()` for both: Row sums widths + gaps, Column sums heights + gaps
- Implemented `layout_node()` for both: two-pass layout (measure then position)
  - Flex behavior: Percent-sized children split remaining space after fixed/fit children
  - CrossAlign positioning: Start/Center/End/Stretch for cross-axis
- Updated `draw_node()` and `apply_opacity()` with no-op arms (transparent containers)
- 7 new tests: gap spacing, cross-align center, percent splitting, no draw commands

## Files Modified
- src/ui/widget.rs (CrossAlign enum, Row/Column variants)
- src/ui/mod.rs (measure_node, layout_node, draw_node, apply_opacity, tests)

## Next Action
UI-102 through UI-108 remaining in Phase UI-1 backlog
