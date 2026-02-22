# Checkpoint

## Active Task
None

## Completed
UI-I01a — Status bar — COMPLETE

- `build_status_bar()` in `src/ui/mod.rs`: builds Panel + RichText with tick, population, mode, optional player name
- 3 theme constants in `theme.rs`: `status_bar_bg`, `status_bar_padding_h`, `status_bar_padding_v`
- `node_rect()` method on WidgetTree for reading computed layout rect
- `ui_theme: ui::Theme` stored on App struct for per-frame rebuilds
- Render loop: rebuilds ui_tree every frame (DD-5), status bar height drives map viewport layout
- Removed `render_status()` string rendering from main loop (function kept with `#[allow(dead_code)]`)
- Screen layout: status_bar_h + padding gap + map + hover + events (3 padding gaps, down from 4)
- 4 new tests: structure, turn-based+player, full-width layout, draw output
- All 487 tests pass, zero warnings

## Files Modified
- src/ui/mod.rs (build_status_bar, node_rect, 4 tests)
- src/ui/theme.rs (3 status bar constants)
- src/main.rs (ui_theme on App, status bar rebuild in render loop, layout adjustments)
- src/render.rs (#[allow(dead_code)] on render_status)

## Next Action
Pick next task from backlog. Unblocked candidates:
- UI-I01b — Hover tooltip (needs W01+W04+R02, all done)
- UI-I01c — Event log (needs W01+W03+R02, all done)
- UI-I01d — Entity inspector (needs W01+W02+W03+R01, all done)
- UI-I02 — Map overlay (needs P01+P03, both done)
