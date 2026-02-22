# Checkpoint

## Active Task
None

## Completed
UI-R01 — Rich text rendering — COMPLETE

- `TextSpan` struct in `draw.rs`: text + color + font_family per span
- `RichTextCommand` in `draw.rs`: spans + position + shared font_size
- `Widget::RichText` variant: holds `Vec<TextSpan>` + font_size
- `DrawList.rich_texts` field for rich text draw commands
- `FontRenderer::prepare_rich_text()`: uses cosmic-text `set_rich_text()` with per-span `Attrs` (color + family), reads `glyph.color_opt` for per-vertex color
- Wired through `measure_node`, `draw_node`, and main.rs render loop
- Demo tree updated: "Population: 1,034,196 souls" mixing Serif body + Mono gold data
- 4 new tests: rich_text_draw_command, rich_text_measure, rich_text_empty_spans, demo_tree_includes_rich_text
- All 250 tests pass, zero warnings

## Files Modified
- src/ui/draw.rs (TextSpan, RichTextCommand, DrawList.rich_texts)
- src/ui/widget.rs (Widget::RichText variant)
- src/ui/mod.rs (measure/draw/re-exports, demo_tree, 4 tests)
- src/font.rs (prepare_rich_text with cosmic-text set_rich_text + color_opt)
- src/main.rs (render loop for rich_texts)

## Next Action
Pick next task from backlog. Unblocked candidates:
- UI-W03 — ScrollList widget (needs W01+W02, both done)
- UI-W04 — Tooltip system (needs W01+W02, both done)
- UI-I01a — Status bar (needs W01+R02, both done)
- UI-I02 — Map overlay (needs P01+P03, both done)
