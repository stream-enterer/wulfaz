# Checkpoint

## Active Task
None

## Completed
UI-W03 — ScrollList widget — COMPLETE

- `Widget::ScrollList` variant in `widget.rs`: bg/border, item_height, scroll_offset, scrollbar color/width
- Theme scrollbar constants: `scrollbar_width` (6px), `scrollbar_color` (gold 50%), `scroll_item_height` (20px)
- Virtual scrolling in `layout_node`: only visible children get measured/laid out; off-screen items get zero rects
- `layout_scroll_item` helper: positions children in vertical stack, overrides Position/Sizing
- `draw_node` ScrollList arm: background panel + visible children + auto-hiding scrollbar thumb
- Scrollbar thumb size proportional to viewport/content ratio, 20px minimum
- `max_scroll`, `set_scroll_offset`, `scroll_by`, `ensure_visible` public methods on WidgetTree
- ScrollList is focusable (keyboard nav target)
- Input routing: `handle_scroll` bubbles to nearest ScrollList ancestor, applies SCROLL_SPEED (40px/line)
- Keyboard nav: ArrowUp/Down (1 item), PageUp/Down (viewport), Home/End (extremes)
- Scrollbar drag: `ScrollDrag` state in UiState, thumb drag updates scroll_offset proportionally
- All 4 input handlers now take `&mut WidgetTree` (was `&WidgetTree`)
- Demo tree: 100-item ScrollList at (360, 20), 200×160px
- 8 new tests: layout, virtual scrolling, clamping, scrollbar visibility, ensure_visible, focusable, demo
- All 472 tests pass, zero warnings

## Files Modified
- src/ui/widget.rs (ScrollList variant)
- src/ui/theme.rs (scrollbar_width, scrollbar_color, scroll_item_height)
- src/ui/mod.rs (measure/layout/draw/hit_test, scroll helpers, demo tree, 8 tests)
- src/ui/input.rs (mutable tree refs, scroll bubbling, keyboard nav, scrollbar drag)
- src/main.rs (&self.ui_tree → &mut self.ui_tree in 4 call sites)

## Next Action
Pick next task from backlog. Unblocked candidates:
- UI-W04 — Tooltip system (needs W01+W02, both done)
- UI-I01a — Status bar (needs W01+R02, both done)
- UI-I01c — Event log (needs W01+W03+R02, all done now)
- UI-I02 — Map overlay (needs P01+P03, both done)
