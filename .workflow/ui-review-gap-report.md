# UI Review Gap Report

## Area 1: Z-Layering & Draw Order

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | Cross-tier text bleed-through: all UI text rendered over all panels, so status bar text appeared on top of overlay panel backgrounds | DONE | Per-tier render pass in main.rs:207-212 — iterates `for i in 0..Z_TIER_COUNT`, drawing each tier's panels then text |
| 2 | Antipattern: type-batched rendering (all panels → all text) instead of layer-batched | DONE | DrawList still separates by type, but all 4 command types (PanelCommand, TextCommand, RichTextCommand, SpriteCommand) now carry `tier: u8`. Upload loop in main.rs filters by tier |
| 3 | Per-widget draw order investigated, concluded unnecessary — per-tier sufficient | DONE | Within a tier, all panels draw then all text. Widgets in the same tier don't need to occlude each other |
| 4 | 4 tiers sufficient for now; render loop should iterate rather than hardcode tier names | DONE | `Z_TIER_COUNT` constant + `tier_panel_ranges`/`tier_text_ranges` arrays. Adding a tier = increment constant + add enum variant |
| 5 | draw_node needs tier parameter stamped on every command | DONE | `draw_node` takes `tier: u8`, every command construction site includes it |
| 6 | PanelRenderer needed `render_range` and `pending_vertex_count` methods | DONE | panel.rs:223 (`pending_vertex_count`), panel.rs:254 (`render_range`) |
| 7 | Old unused `render()` method on PanelRenderer should be deleted | DONE | Removed, only `render_range` remains |

### Implementation summary
4 files changed: `draw.rs` (tier field on all commands), `mod.rs` (tier parameter in draw_node), `panel.rs` (render_range API, deleted old render), `main.rs` (FrameLayers with per-tier vertex ranges, per-tier render loop).

---

## Area 2: Window Families & Standard Frames

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | No shared frame construction — 8+ screen builders independently build Panel→header→content | DONE | `build_window_frame()` in window.rs:31, `WindowFrame` struct. 6 screens migrated |
| 2 | Manual y-positioning with Position::Fixed instead of layout containers | DONE | Migrated screens use Column layout; no Position::Fixed for internal layout in outliner, character_finder, settings, save_load |
| 3 | Close button inconsistency — only character_panel had one | DONE | All 5 closeable screens now get close button via `closeable: true`. Event_popup deliberately `closeable: false` |
| 4 | No window family sizing abstraction (standard width constants, family-specific anchoring) | PARTIAL | Single generic `build_window_frame(width, height)` built instead of family-specific builders. No shared width constants — each screen defines its own |
| 5 | Window width constants don't use `theme.s()` for ui_scale | NOT STARTED | `theme.s()` exists but grep finds zero call sites in screen builders. All widths are unscaled raw f32 |
| 6 | No window decoration (texture frames) — CK3 uses textured frame strips | SKIPPED | Intentional visual identity choice — flat panels with inner shadows |
| 7 | No window show/hide animations (slide-in/slide-out) | SKIPPED | Classified as polish/future work. Animator exists but not wired to panel open/close |
| 8 | main_menu and loading_screen not migrated to frame builder | NOT STARTED | Proposed as fullscreen family variant. Both still construct frames ad-hoc |
| 9 | character_panel title uses mutation to override Label with RichText | DONE | Pragmatic workaround — mutates frame.title widget post-construction |
| 10 | event_popup applies 5 post-hoc style overrides (gold border, wider padding, etc.) | DONE | Pragmatic workaround — mutates root panel + column gap + title font after construction |
| 11 | ConfirmationDialog builder (bonus — not in original findings) | DONE | `build_confirmation_dialog()` in window.rs:155, `ConfirmationDialog` struct with accept/cancel constants |

### Implementation summary
Created `src/ui/window.rs` (WindowFrame, ConfirmationDialog, 6 tests). Migrated 6 screen builders: outliner, settings, save_load, character_finder, character_panel, event_popup. Not migrated: main_menu, loading_screen (fullscreen). Not modified: map_mode, minimap, opinion_view (not window-type panels).

---

## Area 3: Layout System Robustness

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | No expand spacer widget — CK3's most-used layout primitive | DONE | `Widget::Expand` variant added. Auto-treated as `Sizing::Percent(1.0)` in Row width / Column height. 4 unit tests |
| 2 | Window header close button not right-aligned (bug) | DONE | `Widget::Expand` inserted between title and close_btn in build_window_frame when closeable. All 5 closeable windows fixed |
| 3 | Inspector uses Position::Fixed to right-align close button (antipattern) | DONE | Replaced with Row + [header_label, Expand, close_btn] pattern at mod.rs:3254 |
| 4 | No convention for push-to-end layouts | DONE | Widget::Expand is now the canonical pattern, used in window.rs, confirmation dialog, inspector |
| 5 | No layoutstretchfactor (flex-grow equivalent) | SKIPPED | Existing `Sizing::Percent` system distributes remaining space proportionally. Deliberate decision |
| 6 | 0.6x text measurement heuristic inaccuracy | DONE | Addressed via TextMeasurer trait (see Area 7). All 13 heuristic sites replaced with `tm.measure_text()` |
| 7 | Sizing::Fit behavior with Percent children unclear | SKIPPED | Confirmed correct — Fit parent measures intrinsic content, Percent children have 0 intrinsic size. Same as CK3's set_parent_size_to_minimum |

### Implementation summary
`widget.rs` (Widget::Expand variant), `mod.rs` (5 integration points for Expand, inspector close button fix, 4 tests), `window.rs` (Expand in headers and dialog button rows).

---

## Area 4: Tooltip System

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | No max-width on tooltips — long text overflows as single line | DONE | `tooltip_max_width: 400.0` in Theme, applied as `Constraints::loose()` in show_tooltip(). Text Labels use `wrap: true`. 2 tests |
| 2 | Custom tooltip content uses manual Fixed positioning instead of Column layout | DONE | Replaced with `Widget::Column` wrapper in show_tooltip(). Children inserted into column. Test verifies structure |
| 3 | No keyboard shortcut display in tooltips | NOT STARTED | Deferred as UI-D07 in backlog |
| 4 | Nested tooltip positioning uses cursor-relative offset, not parent-edge-relative | NOT STARTED | Deferred as UI-D08 in backlog |
| 5 | Single positioning algorithm vs CK3's 8-template system | SKIPPED | Acceptable — we only use cursor-following tooltips |
| 6 | Single tooltip style vs CK3's GlossaryTooltip variant | SKIPPED | Acceptable — no glossary system |

### Implementation summary
`theme.rs` (tooltip_max_width field), `input.rs` (show_tooltip rewritten: Constraints, Column wrapper, wrap:true, 3 tests). Backlog: UI-D07 (shortcut display), UI-D08 (nested positioning).

---

## Area 5: Input & Focus Management

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | Global shortcut dispatch blocks widget-contextual shortcuts (ESC always closes before widgets can intercept) | NOT STARTED | Classified as monitoring. Will become a problem when text inputs need ESC. Covered by existing UI-D23 (widget-contextual shortcuts) |
| 2 | No Enter/confirm shortcut for modals | DONE | `Action::ConfirmModal` bound to Enter. Handler in main.rs:578-588 calls `modal_stack.confirm_callback()`, pops modal, dispatches callback |
| 3 | PanelManager not integrated into ESC chain | DONE | `panel_manager.close_topmost()` wired into ESC chain at main.rs:599-604, after tooltips and modals |
| 4 | Focus leaks through modal dim layer (Tab not scoped to active tier) | DONE | `focus_min_tier: ZTier` on UiState. `focusable_widgets_in_tier()` filters by tier. Set to Modal when modals open, Panel otherwise. 3 tests |
| 5 | No click-outside-to-dismiss for modals | DONE | Dim layer has `on_click: DIM_CLICK_ACTION`. dispatch_click matches it and calls `pop_modal_with(false)` |
| 6 | No per-widget focus policy (hardcoded to Button/ScrollList) | NOT STARTED | Deferred as UI-D10 in backlog |
| 7 | No window dragging | NOT STARTED | Deferred as UI-D21 in backlog |

### Implementation summary
`keybindings.rs` (ConfirmModal action + Enter binding), `input.rs` (focus_min_tier, tier-scoped Tab, test), `mod.rs` (focusable_widgets_in_tier, z_tier_of_widget, 2 tests), `modal.rs` (DIM_CLICK_ACTION, confirm_callback), `main.rs` (panel_manager in ESC chain, ConfirmModal handler, modal focus scoping, cleanup_after_modal_pop).

---

## Area 6: Animation & Transitions

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | No EaseIn curve for exit animations | DONE | Easing::EaseIn (cubic t³) in animation.rs:11-12. Test verifies slow-start |
| 2 | No delay support for staggered fade-in | DONE | `delay: Duration` on Anim struct. get() returns `from` during delay. 3 tests |
| 3 | No looping for pulse/glow oscillations | DONE | `looping: bool` on Anim. Ping-pong oscillation in get(). Never GC'd. 3 tests |
| 4 | Panels vanish instantly on close — no hide animation | DONE | PanelManager: close_animated(), close_topmost_animated(), flush_closed(). Demo panel slides out with EaseIn. 2 tests |
| 5 | No cubic bezier curves (CK3 uses CSS-style parameterized curves) | SKIPPED | Existing fixed curves suffice. Deferred as UI-D24 |
| 6 | No multi-property animation (CK3 animates position+alpha+scale simultaneously) | SKIPPED | Multiple named animation keys achieve the same result — convenience, not capability gap |
| 7 | No sound on show/hide | SKIPPED | No audio backend. Pre-existing backlog item UI-503 |
| 8 | No animation state machine / multi-step chaining | SKIPPED | Deferred as UI-D16 |
| 9 | Animator API had too many arguments (Clippy violation) | DONE | Refactored to single `Anim` params struct. Commit 02b33b2 |

### Implementation summary
`animation.rs` (EaseIn, delay, looping, Anim params struct, 7 tests), `panel_manager.rs` (close_animated, flush_closed, ClosingPanel, 2 tests), `theme.rs` (anim_panel_hide_ms), `main.rs` (demo panel EaseIn slide-out, flush_closed each frame).

---

## Area 7: Text & Typography

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | Flat 2-tier text contrast hierarchy (only text_light and text_dark) | DONE | text_dark renamed to text_medium (40 occurrences). New text_low (#A09078) added. Hierarchy: text_light > text_medium > text_low > disabled |
| 2 | No semantic text colors (ad-hoc green/red instead of formalized positive/negative) | DONE | text_positive (green), text_negative (red), text_warning (gold) added to Theme. Adopted by character_panel severity_color(), opinion_view sentiments, outliner alerts |
| 3 | Font size scale (9/12/16) vs CK3's (13/15/18/23) — missing 4th tier | SKIPPED | Current scale works. Deferred as UI-D13 (first instance) |
| 4 | 0.6x char-width heuristic wrong for proportional fonts (bug) | DONE | TextMeasurer trait at draw.rs:28. FontRenderer implements it via cosmic-text Buffer shaping. All 13 heuristic sites in mod.rs replaced |
| 5 | text.len() measures bytes not chars (bug) | DONE | HeuristicMeasurer uses text.chars().count(). Production FontRenderer handles Unicode via shaping |
| 6 | No text measurement API on FontRenderer (layout coupled to renderer) | DONE | TextMeasurer trait decouples layout from renderer. Threaded through layout/measure/draw/tooltip functions. HeuristicMeasurer for tests |
| 7 | No text formatting DSL (CK3's #high;bold;size:18 inline markup) | SKIPPED | Deferred as UI-D11 (first instance). RichText with explicit spans adequate for current screen count |
| 8 | No glow/shadow text effects | SKIPPED | Deferred as UI-D12 (first instance) |
| 9 | UI scaling / accessibility (no way to adjust font sizes or UI density) | DONE | Theme.ui_scale (0.5–2.0), high_contrast flag. Helper methods s(), font_size(), border_width(). Ctrl+=/Ctrl+- keybindings. Settings slider. 5 tests |

### Implementation summary
`draw.rs` (TextMeasurer trait, HeuristicMeasurer), `font.rs` (impl TextMeasurer for FontRenderer, ~30 lines), `mod.rs` (all layout/measure/draw functions take `&mut dyn TextMeasurer`, 13 heuristic sites replaced), `input.rs` (tooltip functions accept measurer), `theme.rs` (text_medium rename, text_low, semantic colors, ui_scale, high_contrast, 7 tests), `keybindings.rs` (ScaleUp/ScaleDown), `settings.rs` (ui_scale slider), `main.rs` (scale handlers, measurer threading), 14 screen builders (text_dark→text_medium rename). ~23 files, ~612 insertions, ~350 deletions.

---

## Area 8: Scroll & List Patterns

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | Scroll offset lost on rebuild (scrolling broken for per-frame-rebuilt lists) | DONE | CharacterFinderInfo and SaveLoadInfo gained scroll_offset fields. PanelManager stores per-panel offsets. Test verifies persistence |
| 2 | No empty state in ScrollList (empty grey box when no results) | DONE | `empty_text: Option<String>` on Widget::ScrollList. Character finder sets "No characters found." Centered grey placeholder text. 2 tests |
| 3 | No alternating row backgrounds (lists lack scanability) | DONE | `scroll_row_alt_alpha: 0.04` in Theme. Odd-indexed rows get [0,0,0, alpha] overlay. 1 new test, 3 updated |
| 4 | No grid layout widget (CK3 fixedgridbox) | SKIPPED | Deferred as UI-D17. No concrete screen requires it yet |
| 5 | No scrollbar track or edge fade | SKIPPED | Acceptable — minimalist aesthetic, thumb alone sufficient |
| 6 | Scroll offset persistence API (no getter to read back offset) | DONE | `WidgetTree::scroll_offset(id)` at mod.rs:2309. Test verifies |
| 7 | No sort controls (CK3 has 15 filter categories + sort toggles) | SKIPPED | Deferred as UI-D18. Low priority until entity counts exceed ~200 |
| 8 | Variable-height ScrollList items (UI-501) | DONE | `item_heights: Vec<f32>` field. 4 helper functions (scroll_item_y/h, scroll_total_height, scroll_first_visible). Layout, draw, scroll, input all updated. 4 tests |
| 9 | Widget recycling (CK3 reuses widgets, we rebuild each frame) | NOT STARTED | Implicitly covered by UI-500 (retained tree optimization). No specific backlog item for recycling alone |

### Implementation summary
`widget.rs` (item_heights, empty_text fields), `mod.rs` (scroll helpers, variable-height layout/draw, alternating rows, empty text, scroll_offset getter, 5 new + 3 updated tests), `input.rs` (variable-height keyboard nav + scrollbar), `panel_manager.rs` (scroll_offsets persistence, 1 test), `character_finder.rs` (scroll_offset field, empty_text), `save_load.rs` (scroll_offset field), `demo.rs` (updated ScrollList construction), `theme.rs` (scroll_row_alt_alpha).

---

## Area 9: Modal & Dialog Patterns

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | No click-outside-to-dismiss | DONE | Dim layer has `on_click: DIM_CLICK_ACTION`. dispatch_click pops modal. Test confirms |
| 2 | No Enter/Return binding for modal accept | DONE | Action::ConfirmModal → Enter. Handler calls confirm_callback(), pops, dispatches |
| 3 | No standard confirmation dialog builder | DONE | `build_confirmation_dialog()` in window.rs:155. Cancel/Accept buttons with callback constants. 2 tests |
| 4 | Modal content not centered (was at default 0,0) | DONE | Position::Center variant added. Layout resolves as centered in parent. ModalStack::push sets it. Test confirms |
| 5 | Stale focus after modal pop (focused widget points to removed widget) | DONE | cleanup_after_modal_pop() checks if focused widget still exists, resets focus_min_tier |
| 6 | Dim layer uses Fixed sizing, doesn't resize with window | DONE | Changed to Sizing::Percent(1.0) for both dimensions. Test asserts |
| 7 | ESC dismisses modal without firing cancel callback (return value ignored) | DONE | CloseTopmost now calls pop_modal_with(false) which dispatches on_dismiss callback |
| 8 | event_choice:* callbacks never dispatched (dead code) | PARTIAL | dispatch_click has catch-all `_ => {}` silently dropping them. Plumbing ready but no game system consumes them. Blocked on simulation features |
| 9 | No dismiss/confirm callbacks on ModalStack::pop (caller can't distinguish intent) | DONE | ModalOptions { on_dismiss, on_confirm }. pop() returns ModalPop with both. pop_modal_with() uses flag to select |
| 10 | No show/hide animation on modals (CK3 fades in 0.25s) | NOT STARTED | Conversation noted as LOW priority. Animator exists but not wired to modals |

### Implementation summary
`modal.rs` (complete rewrite: ModalOptions/ModalPop, DIM_CLICK_ACTION, Percent sizing, Position::Center, confirm_callback, 11 tests), `keybindings.rs` (ConfirmModal + Enter), `mod.rs` (Position::Center variant + layout), `window.rs` (DIALOG_ACCEPT/CANCEL, ConfirmationDialog, build_confirmation_dialog, 2 tests), `main.rs` (cleanup_after_modal_pop, pop_modal_with, dispatch_click, ConfirmModal handler, focus scoping).

---

## Area 10: Background & Visual Composition

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | No texture support in panel renderer (flat rects vs CK3 9-slice) | SKIPPED | Intentional — geometric panels, SpriteRenderer exists for texture needs |
| 2 | No 9-slice, overlay blending, or masks | SKIPPED | Intentional — CK3-specific art style features |
| 3 | No alternating row backgrounds in ScrollList | DONE | Already implemented via scroll_row_alt_alpha. Theme field + draw code |
| 4 | No status-colored panel backgrounds (green/red/yellow tints) | PARTIAL | Theme fields bg_status_good/bad/mixed defined with color values, but no screen builder uses them. Existing backlog item UI-D20 |
| 5 | Button text offset hardcoded at +8.0/+4.0 ignoring node.padding | DONE | Now uses node.padding.left/top. default_padding() returns {4,8,4,8}. Padding overridable per-widget |
| 6 | border_width: 1.0 hardcoded across 6 widget types | DONE | All 6 draw sites use self.control_border_width. Theme provides control_border() with +1 in high_contrast. Default 1.0 |
| 7 | Button text ignores layout-computed padding (double-padding bug when callers override padding) | DONE | Same fix as #5 — draw respects layout-resolved padding |
| 8 | Dropdown/TextInput text offset should also use node.padding | NOT STARTED | Deferred as UI-D19. Low priority unless custom padding or ui_scale != 1.0 |
| 9 | Status backgrounds not used by any screen | NOT STARTED | Deferred as UI-D20. Waiting for dense data views |

### Implementation summary
`mod.rs` (button measure_node returns intrinsic size, button draw uses node.padding, default_padding, control_border_width on WidgetTree, 6 draw sites updated), `theme.rs` (control_border_width, control_border(), bg_status_good/bad/mixed), `main.rs` (wires set_control_border_width from theme).

---

## Ambiguous Items

| Area | Finding | Current Code | What's Unclear |
|------|---------|--------------|----------------|
| 9 | event_choice:* callbacks (F8) | dispatch_click has `_ => {}` catch-all. on_click values set in event_popup.rs but no consumer exists | Whether this should be classified as a UI gap or a simulation gap — the UI plumbing is ready, but the game systems that would respond to these callbacks don't exist yet. Status depends on scope definition |
| 2 | Window family sizing abstraction (F4) | Single generic builder exists | Whether "standard width constants" (SIDEBAR_WIDTH, DIALOG_WIDTH) should be defined centrally vs per-screen is a design choice, not clearly a gap |

---

## Backlog ID Collision (resolved)

UI-D11–D14 were each used twice. The second set has been renumbered to UI-D21–D24.

---

## Summary Statistics

| Status | Count |
|--------|-------|
| DONE | 52 |
| PARTIAL | 3 |
| SKIPPED | 17 |
| NOT STARTED | 10 |
| AMBIGUOUS | 0 (2 flagged above but classified) |

**NOT STARTED breakdown:**
- Already in backlog: UI-D07, UI-D08, UI-D10, UI-D11(2nd), UI-D19, UI-D20
- Implicitly covered: UI-500 (widget recycling via retained tree)
- New entries needed: 3 (see Backlog Additions below)

---

## Backlog Additions

See entries appended to `.workflow/backlog.md` under `## Deferred`.
