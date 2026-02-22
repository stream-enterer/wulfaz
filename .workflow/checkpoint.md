# Checkpoint

## Active Task
None

## Completed
UI-P02 — Multi-font atlas support — COMPLETE

- `FontFamily` enum: `Mono` (Libertinus Mono), `Serif` (Libertinus Serif) with `family_name()` accessor
- `GlyphCacheKey`: composite `(fontdb::ID, font_size_bits, glyph_id)` per DD-3
- `FontRenderer` loads both bundled fonts into `fontdb::Database` and maps each to a FreeType face via `HashMap<fontdb::ID, freetype::Face>`
- `rasterize_glyph_on_demand()` accepts `font_id`, `font_size_px`, `glyph_id` — dynamically calls `set_char_size` for size-variant rasterization
- `prepare_text_shaped()` parameterized by family name + font size; cosmic-text selects font via `Attrs::family()`
- `prepare_text_with_font()` public API for UI widget text commands
- `TextCommand` and `Widget::Label`/`Widget::Button` carry `font_family: FontFamily`
- `demo_tree()` uses Serif for header/body, Mono for warning (verifies multi-font rendering)
- 7 UI tests (added `draw_list_multi_font`), all 232 tests pass, zero warnings

## Files Modified
- src/font.rs (multi-face loading, GlyphCacheKey, parameterized shaping/rasterization)
- src/ui/draw.rs (FontFamily enum, font_family field on TextCommand)
- src/ui/widget.rs (font_family field on Label and Button)
- src/ui/mod.rs (font_family wiring in draw/measure, multi-font test, demo_tree update)
- src/main.rs (prepare_text_with_font for UI widget commands)

## Next Action
Pick next task from backlog. Tier 2 remaining: UI-R02 (theme), UI-W02 (input routing).
UI-R02 needs UI-P02 (done) + UI-P03 (done). UI-W02 needs UI-W01 (done). Both unblocked.
