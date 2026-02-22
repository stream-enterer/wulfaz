# Checkpoint

## Active Task
None

## Completed
UI-W04 — Tooltip system — COMPLETE

- `TooltipContent` enum in `widget.rs`: Text (simple label) and Custom (widget list with nested tooltips)
- Theme tooltip constants: delay 300ms, fast window 500ms, offset 8px, nesting offset 4px, padding 8px, darker parchment bg, gold border 1px, shadow 4px
- `tooltip: Option<TooltipContent>` field on WidgetNode, `set_tooltip()` on WidgetTree
- `measure_node()` made public for tooltip size estimation
- `TooltipEntry`/`TooltipPending` types in `input.rs`
- `tooltip_stack`, `tooltip_pending`, `tooltip_last_dismiss` fields on UiState
- `update_tooltips()`: hover delay, fast-show window, show/dismiss lifecycle
- `show_tooltip()`: builds Panel root from TooltipContent, vertical stacking, edge-flip positioning
- `dismiss_all_tooltips()`, `tooltip_count()` public API
- `find_tooltip_ancestor()`: walks parent chain to find widget with tooltip content
- `compute_tooltip_position()`: below-right of cursor, flips if clipping screen, nesting offset
- Nested tooltips: Custom content children can have their own tooltips → stacking
- Recursive dismissal: top of stack popped when cursor leaves both tooltip rect and source rect
- Demo tree: button at (580,20) with 3-level nested tooltip chain (level 1 → level 2 → level 3)
- 11 new tests: delay, show after delay, dismiss on leave, stays on source, stays inside tooltip, fast show window, edge flip (right/bottom), nested chain, dismiss all, demo tree tooltip
- All 483 tests pass, zero warnings

## Files Modified
- src/ui/widget.rs (TooltipContent enum)
- src/ui/theme.rs (10 tooltip constants)
- src/ui/mod.rs (WidgetNode tooltip field, set_tooltip, pub measure_node, demo tooltip button, updated demo tests)
- src/ui/input.rs (TooltipEntry, TooltipPending, UiState tooltip fields, update/show/dismiss/position methods, 11 tests)

## Next Action
Pick next task from backlog. Unblocked candidates:
- UI-I01a — Status bar (needs W01+R02, both done)
- UI-I01b — Hover tooltip (needs W01+W04+R02, all done now)
- UI-I01c — Event log (needs W01+W03+R02, all done)
- UI-I02 — Map overlay (needs P01+P03, both done)
