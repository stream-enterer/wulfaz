# Checkpoint

## Active Task
None

## Completed
UI-W05 — Animation system — COMPLETE

- New file: `src/ui/animation.rs` — `Animator` (HashMap-keyed f32 interpolation), `Easing` enum (Linear, EaseInOut, EaseOut), cubic easing math
- `WidgetTree::set_subtree_opacity()` — walks subtree, multiplies all color alpha channels
- `WidgetTree::set_widget_bg_alpha()` — sets single widget's bg alpha (for hover highlight)
- 4 animation theme constants in `Theme`: tooltip fade (150ms), inspector slide (200ms), hover highlight (200ms, 0.3 alpha)
- 3 concrete animations integrated in main.rs:
  - Hover tooltip fade-in: 150ms EaseOut opacity when hovering a new tile
  - Inspector slide-in: 200ms EaseOut slide from right edge when entity selected
  - Button hover highlight: 200ms EaseOut bg alpha on inspector close button hover/unhover
- Animator tracks `last_hover_tile` and `last_selected_entity` for state change detection
- `Animator::gc()` called each frame to clean up completed animations
- 15 new animation tests + 2 widget tree opacity tests
- All 528 tests pass (214 lib + 303 main + 5 determinism + 6 invariant), zero warnings

## Files Modified
- src/ui/animation.rs (NEW — Animator, Easing, Animation, easing math, 15 tests)
- src/ui/mod.rs (module wire-up, set_subtree_opacity, set_widget_bg_alpha, 2 tests)
- src/ui/theme.rs (4 animation duration/alpha constants)
- src/main.rs (Animator on App, 3 animation integrations in RedrawRequested)

## Next Action
Pick next task from backlog. Remaining:
- UI-I03 — Keyboard shortcut system (Tier 5 polish)
- UI-DEMO update (Tier 5 milestone)
