# Checkpoint

## Active Task
None

## Completed
UI-DEMO — Widget showcase — COMPLETE

- New file: `src/ui/demo.rs` — `build_demo()` function, `DemoLiveData` struct, 3 tests
- F11 toggles demo panel, `--ui-demo` CLI flag activates at startup
- `ToggleDemo` action added to keybindings (F11 default)
- Demo panel (400px, left side) showcases all 5 UI tiers:
  - Tier 1: Typography (header/body/data/warning/disabled)
  - Tier 3: Rich text (mixed fonts/colors), ScrollList (50 items), 3-level tooltip chain
  - Tier 4: Live entity data (tick, population, first entity stats with severity colors)
  - Tier 5: Buttons with keybinding labels (Pause/Speed/Close), animated slide-in
- Slide-in animation on open, Esc closes demo (added to CloseTopmost priority)
- Old `demo_tree()` in mod.rs removed, 4 tests updated to use new `demo::build_demo()`
- All 540 tests pass (214 lib + 315 main + 5 determinism + 6 invariant), zero warnings

## Files Modified
- src/ui/demo.rs (NEW — build_demo, DemoLiveData, 3 tests)
- src/ui/mod.rs (module wire-up, removed old demo_tree, 3 tests updated)
- src/ui/keybindings.rs (ToggleDemo action, F11 default binding)
- src/ui/input.rs (1 test updated)
- src/main.rs (show_demo field, ToggleDemo handler, demo building in render, --ui-demo flag)

## Next Action
Pick next task from backlog. All UI tasks complete. Remaining:
- Phase A (SCALE-A09), Phase B, Phase C, Phase D, SIM-*, GROW-*
