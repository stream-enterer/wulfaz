# Checkpoint

## Active Task
None

## Completed
UI-I01c — Event log panel — COMPLETE

- `EventLogEntry` enum in `src/ui/mod.rs`: Spawned/Died/Ate/Attacked variants (decoupled data carrier)
- `build_event_log()` in `src/ui/mod.rs`: builds ScrollList with RichText children, auto-scrolls to bottom
- `collect_event_entries()` in `src/ui/mod.rs`: extracts significant events from EventLog + names table
- Filters to significant events (Spawned/Died/Ate/Attacked), skips Moved/HungerChanged
- Themed colors: text_light for names, danger for death/damage, gold for food, disabled for verbs
- Chrome panel at bottom of screen, full width, fixed height (5 visible items × scroll_item_height)
- Auto-scroll to newest events (computes max offset at build time)
- Replaces string-based `render_recent_events()` and `font.prepare_text()` calls
- `render_recent_events` and `font.prepare_text` marked `#[allow(dead_code)]`
- 8 new tests: empty, spawned, died, ate, attacked, auto-scroll, draw output, sizing
- All 501 tests pass, zero warnings

## Files Modified
- src/ui/mod.rs (EventLogEntry, build_event_log, collect_event_entries, 8 tests)
- src/main.rs (event log integration, removed string-based event rendering)
- src/render.rs (#[allow(dead_code)] on render_recent_events)
- src/font.rs (#[allow(dead_code)] on prepare_text)

## Next Action
Pick next task from backlog. Unblocked candidates:
- UI-I01d — Entity inspector (needs W01+W02+W03+R01, all done)
- UI-I02 — Map overlay (needs P01+P03, both done)
