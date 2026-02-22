# Checkpoint

## Active Task
None

## Completed
UI-I01b — Hover tooltip — COMPLETE

- `build_hover_tooltip()` in `src/ui/mod.rs`: builds tooltip-styled Panel with rich text for terrain, quartier, address/building, occupants, entities
- `HoverInfo` struct in `src/ui/mod.rs`: decoupled data carrier (extracted from World in main.rs, consumed by UI builder)
- `collect_hover_info()` in `src/main.rs`: extracts structured data from World for the hovered tile
- Tooltip styled like W04 (tooltip_bg_color, border, shadow, padding from Theme)
- Positioned near cursor with edge-flipping via `UiState::compute_tooltip_position` (made pub(crate))
- Occupant display capped at 5 with "+N more" overflow
- Entity display: icon (gold) + name for alive entities on the tile
- Hover line removed from screen layout — map viewport gained 1 line of height
- Tree re-laid out after tooltip insertion (2 layout passes per frame)
- `render_hover_info` marked `#[allow(dead_code)]` (kept as reference)
- 6 new tests: terrain-only, full building, occupant truncation, entity display, draw output, screen positioning
- All 268 tests pass, zero warnings

## Files Modified
- src/ui/mod.rs (HoverInfo, build_hover_tooltip, 6 tests)
- src/ui/input.rs (pub(crate) compute_tooltip_position)
- src/main.rs (collect_hover_info, tooltip integration, layout adjustments)
- src/render.rs (#[allow(dead_code)] on render_hover_info)

## Next Action
Pick next task from backlog. Unblocked candidates:
- UI-I01c — Event log (needs W01+W03+R02, all done)
- UI-I01d — Entity inspector (needs W01+W02+W03+R01, all done)
- UI-I02 — Map overlay (needs P01+P03, both done)
