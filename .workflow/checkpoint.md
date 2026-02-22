# Checkpoint

## Active Task
None

## Completed
UI-W02 — Input routing + hit testing — COMPLETE

- `Rect::contains(px, py)` for point-in-rect tests
- `WidgetTree::hit_test(x, y)` — back-to-front traversal returns topmost widget
- `WidgetTree::focusable_widgets()` — depth-first collection of Buttons for Tab cycling
- `UiState` struct: hovered, focused, pressed, captured, drag tracking
- `handle_cursor_moved` / `handle_mouse_input` / `handle_key_input` / `handle_scroll`
- Mouse capture holds during drag (threshold 4px), released on mouse-up
- Tab cycles focus through focusable widgets (Buttons)
- Click on Button sets focus; click outside clears focus
- UI events consumed before game input (keyboard, mouse, scroll)
- Persistent `ui_tree` + `ui_state` on App (not rebuilt each frame)
- 11 new tests, all 246 tests pass, zero warnings

## Files Modified
- src/ui/input.rs (new — UiEvent, MouseButton, UiState, 11 tests)
- src/ui/mod.rs (Rect::contains, hit_test, focusable_widgets, mod input)
- src/main.rs (UiState + ui_tree on App, event routing)

## Next Action
Pick next task from backlog. Unblocked candidates:
- UI-R01 — Rich text rendering (needs P01+P02, both done)
- UI-I01a — Status bar (needs W01+R02, both done)
- UI-I02 — Map overlay (needs P01+P03, both done)
Tier 3 tasks W03/W04 now unblocked (need W01+W02, both done).
