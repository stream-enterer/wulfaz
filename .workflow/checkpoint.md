# Checkpoint

## Active Task
None

## Completed
UI-I03 — Keyboard shortcut system — COMPLETE

- New file: `src/ui/keybindings.rs` — `KeyCombo`, `ModifierFlags`, `Action` enum, `KeyBindings` map with defaults + reverse lookup for labels
- Default bindings: Space=PauseSim, Escape=CloseTopmost, 1-5=SpeedSet
- `KeyBindings::label_for(action)` returns human-readable label (e.g. "Space", "Ctrl+P")
- `StatusBarInfo` struct replaces positional args for `build_status_bar`
- Status bar shows: "PAUSED (Space)" in danger red when paused, "Speed: Nx (N)" with gold highlight when speed > 1
- CloseTopmost priority: tooltips → inspector → exit
- Pause stops tick accumulator; unpause resets frame time to avoid burst
- Speed multiplier applied to tick accumulator: `dt * sim_speed`
- Global keybindings processed before widget focus dispatch in keyboard handler
- 6 keybindings tests + 2 status bar pause/speed tests
- All 537 tests pass (214 lib + 312 main + 5 determinism + 6 invariant), zero warnings

## Files Modified
- src/ui/keybindings.rs (NEW — KeyCombo, ModifierFlags, Action, KeyBindings, key_name, 6 tests)
- src/ui/mod.rs (module wire-up, StatusBarInfo struct, build_status_bar refactored, 2 new tests, 4 existing tests updated)
- src/main.rs (keybindings/paused/sim_speed on App, global keybinding dispatch, pause/speed tick logic)

## Next Action
Pick next task from backlog. Remaining:
- UI-DEMO update (Tier 5 milestone)
