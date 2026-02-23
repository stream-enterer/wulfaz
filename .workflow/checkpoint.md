# Checkpoint

## Active Task
Phase UI-5 — COMPLETE (3/3 tasks done)

## Phase UI-5 Status
- UI-505 Performance profiling — done (UiPerfMetrics, timing instrumentation, status bar display)
- UI-501 Variable-height ScrollList — done (helper functions, layout/draw/scroll updates, backward compat)
- UI-504 UI scaling / accessibility — done (Theme.s()/font_size(), ScaleUp/ScaleDown keybindings)

## Modified Files
- src/ui/mod.rs — UiPerfMetrics, widget_count(), scroll helpers, perf in StatusBarInfo
- src/ui/widget.rs — item_heights field on ScrollList
- src/ui/theme.rs — ui_scale, high_contrast, s(), font_size(), border_width(), text_alpha()
- src/ui/keybindings.rs — ScaleUp/ScaleDown actions, Ctrl+=/Ctrl+- bindings
- src/ui/input.rs — variable-height scroll nav + scrollbar drag
- src/ui/demo.rs, character_finder.rs, save_load.rs — item_heights: Vec::new()
- src/main.rs — timing instrumentation, ScaleUp/ScaleDown handlers

## Test Count
456 unit + 5 determinism + 6 invariant = 467 total, all passing

## Next Action
Delete Phase UI-5 from backlog. Start next phase.
