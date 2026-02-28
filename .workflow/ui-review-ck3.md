# UI Architecture Review: Learn from CK3

## Philosophy

CK3 has a working UI that ships to millions of players. Every subtle issue — layering,
draw order, tooltip positioning, modal stacking, focus management, scroll behavior,
text overflow, animation timing — has been solved and iterated on across years of patches.

We're not copying CK3's look. We're using it as a **known-good skeleton** to find
antipatterns in our UI framework that are easy to miss in LLM-driven development:
temporal coupling, missing edge cases, draw order bugs, widget composition gaps,
and architectural decisions that seem fine in isolation but break under real use.

The question for every comparison is: **"What problem did CK3 solve that we haven't
encountered yet?"**

## Legal Boundaries (non-negotiable)

**Safe to study and implement:**
- Layout patterns (vbox, hbox, anchoring, expanding) — universal UI concepts
- Widget hierarchy and composition (template/slot/override patterns)
- Window family conventions (sidebar, main tab, floating, modal)
- Layer priority ordering, draw order rules
- Animation timing patterns, easing curves
- Tooltip positioning algorithms, edge-flipping
- Scrollbox patterns, virtual list techniques
- Input flow (focus, capture, modal blocking)

**Never do:**
- Copy .gui file text verbatim into our codebase
- Reproduce CK3's specific visual style (textures, color palette, decorative elements)
- Use CK3 asset files (.dds textures, fonts, sounds)
- Create pixel-identical reproductions of CK3 screens

We already have our own visual identity (dark brown + gold + Libertinus fonts).
We keep it.

---

## Reference Files

| What | Where |
|------|-------|
| CK3 master index | `.workflow/ck3-ui-reference.md` |
| CK3 shared widgets | `.workflow/ck3-gui-shared-index.md` |
| CK3 raw .gui files | `~/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Crusader Kings III/game/gui/` |
| Our UI code | `src/ui/` (mod.rs, widget.rs, draw.rs, input.rs, theme.rs, keybindings.rs, + screen files) |
| Our renderers | `src/main.rs`, `src/font.rs`, `src/panel.rs`, `src/sprite_renderer.rs` |
| Our theme | `src/ui/theme.rs` |

When doing a deep comparison, read the specific CK3 .gui file alongside our
corresponding Rust file. The index tells you which CK3 file maps to which feature.

---

## Methodology

For each area below, follow this process:

1. **Read our code** — understand current implementation, note any TODOs or fragile spots
2. **Read CK3 index** — find the corresponding pattern in `.workflow/ck3-ui-reference.md`
3. **Deep read CK3 file** if the index isn't detailed enough — read the actual .gui file
4. **Compare** — what did CK3 solve that we haven't? What's different and why?
5. **Classify findings** as: bug, antipattern, missing feature, or acceptable difference
6. **Propose fix** — minimal change that addresses the issue without over-engineering

---

## Area 1: Z-Layering & Draw Order

### Our system
- `ZTier` enum: Panel(0), Overlay(1), Modal(2), Tooltip(3)
- Roots sorted by tier, then insertion order within tier
- Draw pass walks tree depth-first, emitting commands to flat DrawList
- DrawList has separate vectors: `panels`, `texts`, `rich_texts`, `sprites`
- Renderer draws ALL panels, then ALL text — **not interleaved by widget**
- Recent fix: split text into map layer (under panels) and UI layer (over panels)
- `FrameLayers` struct tracks vertex ranges for 3-layer render pass

### CK3 system
- 12 named layers from bottom_bottom(1) to debug(50)
- `layer = windows_layer` / `layer = middle` / `layer = events` etc.
- Each widget can set its own layer
- Within a layer, draw order is tree traversal order
- **Key difference**: CK3 doesn't batch all backgrounds then all text. Each widget
  draws its own background + text together, so a panel's background naturally
  occludes text from widgets behind it.

### Questions to investigate
1. Our 3-layer render pass (map text → panels → UI text) means ALL UI text renders
   over ALL panels. Status bar text renders on top of demo panel background.
   **CK3 solves this by drawing each widget atomically.** Do we need per-widget
   draw order, or can we split into more layers (base panels → base text →
   overlay panels → overlay text)?
2. The `ZTier` enum has only 4 tiers. CK3 has 12 layers. Are 4 enough? What
   happens when we need floating windows that layer between Panel and Modal?
3. Our DrawList batches by render type (panels vs text). CK3 batches by widget.
   What's the performance tradeoff of switching to widget-order rendering?
   (Means more pipeline switches between font and panel renderers.)

### What to read
- Our: `src/ui/mod.rs` (draw_node, ZTier), `src/main.rs` (FrameLayers, GpuState::render)
- CK3: `.workflow/ck3-ui-reference.md` "Layer Priority System" section
- CK3 deep: `shared/windows.gui` (layer definitions)

---

## Area 2: Window Families & Standard Frames

### Our system
- No window family abstraction
- Each screen builder (character_panel.rs, outliner.rs, etc.) constructs its own
  Panel + header + content structure ad-hoc
- PanelManager tracks open panels by name, all at ZTier::Panel
- No standard header widget, no standard close button pattern, no standard window
  background pattern

### CK3 system
- 3 window families with shared templates:
  - MainTab: 655×100%, right-anchored, margin_widget wrapper, slide-right animation
  - Sidebar: 610×100%, left-anchored, slide-left animation
  - Floating: fixed size, centered, draggable, spike decoration
- `Window_Background`, `Window_Decoration_Spike`, `Window_Margins` — reusable templates
- `header_pattern` and `widget_header_with_picture` — standard header types
- `buttons_window_control` — standard close/back/pin/minimize button bar

### Questions to investigate
1. Should we create window family builders? E.g., `build_sidebar_window(tree, theme,
   title, closeable) -> WidgetId` that returns a standard frame with header, close
   button, and content area. Screen builders would fill the content area.
2. CK3's `buttons_window_control` has close/back/pin/minimize. We have close only
   (via PanelManager). Do we need back-navigation or pinning?
3. CK3 windows have show/hide animations (slide + fade). Our Animator exists but
   isn't consistently applied to window open/close. How should we standardize this?

### What to read
- Our: `src/ui/character_panel.rs`, `src/ui/outliner.rs` (see how they build frames)
- CK3: `.workflow/ck3-ui-reference.md` "Window Families" section
- CK3 deep: `shared/windows.gui` (templates)

---

## Area 3: Layout System Robustness

### Our system
- Row/Column with gap and CrossAlign (Start/Center/End/Stretch)
- Sizing: Fixed(f32) | Percent(f32) | Fit
- Position: Fixed{x,y} | Percent{x,y}
- Single-pass measure → layout → draw
- Text measurement: **approximate** (char_count × line_height × 0.6)
- No `expand` spacer widget (CK3's key layout primitive)
- No `layoutpolicy_horizontal = expanding` equivalent beyond Percent(100.0)

### CK3 system
- vbox/hbox with `layoutpolicy_horizontal/vertical = expanding`
- `expand = {}` — invisible widget that fills remaining space (flex spacer)
- `set_parent_size_to_minimum = yes` — shrink-wrap parent to content
- `layoutstretchfactor_horizontal` — flex-grow-like proportional sizing
- Corneredtiled 9-slice for all backgrounds (spriteborder = {L T})
- Explicit `minimumsize`, `maximumsize` constraints on any widget

### Questions to investigate
1. We have no `expand` spacer. This is CK3's most-used layout primitive. It enables
   push-to-end, centering, and fill-remaining-space without percent math. Should we
   add an `Expand` widget variant?
2. We have no `layoutstretchfactor`. CK3 uses this for proportional multi-section
   progress bars and flexible column layouts. Worth adding?
3. Our text measurement is approximate. CK3 doesn't measure text on CPU at all —
   its engine handles it. Our layout decisions (wrapping, truncation) are based on
   a 0.6× char-width heuristic. How bad is this in practice? Test with real text.
4. CK3 uses `set_parent_size_to_minimum` extensively for tooltips and dialogs.
   Our `Sizing::Fit` does this, but does it work correctly when children have
   Percent sizing?

### What to read
- Our: `src/ui/mod.rs` (measure_node, layout_node functions)
- CK3: `.workflow/ck3-ui-reference.md` "Standard Sizes & Spacing" section
- CK3 deep: `shared/windows.gui` (expand, layoutpolicy usage)

---

## Area 4: Tooltip System

### Our system
- TooltipContent: Text(String) | Custom(Vec<(Widget, Option<TooltipContent>)>)
- 300ms delay (0ms in 500ms fast-show window after dismiss)
- Position: below-right of cursor, flip left/up at screen edges
- Nesting: stack-based, tooltip widgets can have their own tooltips
- Built as Panel at ZTier::Tooltip
- Dismissed when cursor leaves source or tooltip rect

### CK3 system
- 8 tooltip placement templates (tooltip_es, tooltip_ws, tooltip_ne, tooltip_se, etc.)
- DefaultTooltipText: max_width 450, autoresize, multiline
- DefaultTooltipBackground: bg texture + frame texture + overlay blend
- GlossaryTooltip: special variant with decorative edges, different bg, light-bg format
- Lock indicator: progresspie timer at top-right
- Tooltip margins: 20px horizontal, 12px top, 18px bottom
- Shortcut text at bottom-right

### Questions to investigate
1. CK3 has 8 directional tooltip templates. We have one position algorithm with
   flip logic. Is our algorithm equivalent, or are there edge cases CK3 handles
   that we don't? (E.g., tooltip for widget at exact screen center — which
   direction wins?)
2. CK3 tooltips have a max_width of 450px. Ours is unbounded — tooltip Panel
   sizes to content. Do we need max-width clamping?
3. CK3 has GlossaryTooltip (a styled variant). We have one tooltip style. Is
   this a problem, or does our theming handle it?
4. CK3's tooltip shows keyboard shortcut text at bottom-right. Ours doesn't.
   Should it?
5. Our nested tooltip positioning doesn't account for parent tooltip size.
   Test: hover over a tooltip's content that itself has a tooltip. Does it
   overlap? CK3's nesting uses offset multiplication per level.

### What to read
- Our: `src/ui/mod.rs` (show_tooltip, compute_tooltip_position, update_tooltips)
- CK3: `.workflow/ck3-gui-shared-index.md` section 10 (cooltip.gui)
- CK3 deep: `preload/defaults.gui` (tooltip placement templates)

---

## Area 5: Input & Focus Management

### Our system
- UiState tracks: hovered, focused, pressed, captured, cursor
- Tab cycles focus through focusable widgets (Buttons, ScrollLists)
- Mouse click sets focus
- Modal dim layer blocks clicks (but focus doesn't auto-transfer to modal)
- No ESC handling in ModalStack (caller must wire it)
- Drag: 4px threshold, single captured widget, scrollbar-only currently

### CK3 system
- `filter_mouse = left|right|wheel` — per-widget mouse event filtering
- `focuspolicy = all|click|none` — per-widget focus policy
- `shortcut = "close_window"` / `shortcut = "confirm"` — per-widget keyboard shortcuts
- `movable = yes` — window dragging built-in
- Window controls (close/minimize) have standard keyboard bindings
- VariableSystem for UI state toggles (expand/collapse, tab switching)

### Questions to investigate
1. CK3 has per-widget mouse filtering. We process all mouse events for all widgets.
   Is there a performance or correctness issue with not filtering?
2. CK3 binds keyboard shortcuts directly on widgets (`shortcut = "close_window"`).
   We handle shortcuts in the main loop. Is our approach correct, or does it miss
   cases where the same shortcut should do different things based on focused widget?
3. CK3 windows are draggable via `movable = yes`. Our PanelManager doesn't support
   dragging. Do we need it for floating dialogs?
4. Our modal system requires manual ESC wiring. CK3 binds close_window shortcut
   on the close button, which responds to ESC. Should ModalStack auto-bind ESC?

### What to read
- Our: `src/ui/input.rs` (handle_mouse_input, handle_key_input, handle_scroll)
- CK3: `.workflow/ck3-ui-reference.md` "Cross-Cutting Patterns" section
- CK3 deep: `shared/dialogs.gui` (confirmation_popup — keyboard shortcut pattern)

---

## Area 6: Animation & Transitions

### Our system
- Animator: HashMap<String, Animation> with from/to/duration/easing
- 3 easing functions: Linear, EaseInOut, EaseOut
- Single-value tweening only
- Used for: tooltip fade (150ms), inspector slide (200ms), hover highlight (200ms)
- No state machine, no keyframe tracks
- Manual gc() cleanup

### CK3 system
- State machines: named states with `trigger_on_create`, `next`, `duration`, `delay`
- Multi-property animation: position, alpha, scale, UV offset simultaneously
- Bezier curves: `{0.25, 0.1, 0.25, 1}` (CSS ease), `{0.43, 0, 0.2, 2.2}` (bounce)
- Staggered fade-in: elements enter with increasing delays (0.1s, 0.2s, 0.4s, 0.6s)
- Shimmer/glow sweeps: colordodge texture translation loops (2-8s + 5s delay)
- Screen shake: 3-state loops with px offsets
- Window show/hide always paired with sound events

### Questions to investigate
1. Our Animator can only tween one value at a time. CK3 animates position + alpha
   + scale simultaneously. Do we need multi-property animation?
2. CK3's staggered fade-in (elements appear with increasing delay) is visually
   polished. Can we do this with our current Animator? (Probably yes — start N
   animations with increasing delays. But it's manual.)
3. CK3 pairs every window show/hide with a sound event. We have SoundEvent hooks
   defined in the backlog (UI-503) but not implemented. Priority?
4. CK3 uses state machines for animation (state A → B → C → repeat). Our Animator
   is one-shot. Do we need looping animations?

### What to read
- Our: `src/ui/mod.rs` (Animator, Easing)
- CK3: `.workflow/ck3-ui-reference.md` "Animation Patterns" section
- CK3 deep: `shared/animation.gui`

---

## Area 7: Text & Typography

### Our system
- 2 fonts: Libertinus Mono, Libertinus Serif (bundled OTF)
- FontFamily enum: Mono | Serif
- 3 theme font sizes: header (16), body (12), data/mono (9)
- Rich text: per-span color + font family
- Text measurement: approximate char_count × 0.6 × line_height
- No text formatting system (no #high/#low/#bold markup)

### CK3 system
- 2 typefaces: StandardGameFont (body), TitleFont (decorative)
- 4 font sizes: Tiny(13), Small(15), Medium(18), Big(23)
- Hierarchical contrast: #high (white), #medium (gray, default), #low (dark), #weak
- Semantic text colors: #V (value), #N (negative/red), #P (positive/green), #M (mixed)
- Text formatting DSL: `#high;bold;size:18` inline markup
- Glow levels: none, weak(0.1), medium(0.2), strong(0.4)
- Light-background overrides: maps all named formats to dark-on-light variants

### Questions to investigate
1. CK3's contrast hierarchy (#high/#medium/#low) is a systematic approach to text
   readability. We have text_light and text_dark. Should we adopt a contrast tier
   system instead?
2. CK3 has semantic colors (#P positive, #N negative). We use ad-hoc colors
   (green for health, red for damage). Should we formalize semantic color aliases?
3. CK3's font sizes (13/15/18/23) form a typographic scale. Ours (9/12/16) are
   tighter. Is our scale appropriate for our screen density?
4. The 0.6× char-width heuristic needs testing. How far off is it for Libertinus
   Serif at different sizes? This affects every text-dependent layout decision.

### What to read
- Our: `src/ui/theme.rs` (font defaults), `src/ui/draw.rs` (FontFamily)
- CK3: `.workflow/ck3-ui-reference.md` "Font System" and "Color Palette" sections
- CK3 deep: `preload/textformatting.gui`, `preload/fonts.gui`

---

## Area 8: Scroll & List Patterns

### Our system
- ScrollList widget with virtual scrolling (only visible items laid out)
- Variable-height items via `item_heights: Vec<f32>` (UI-501)
- Scrollbar: vertical, right-aligned, min thumb 20px
- Keyboard nav: Arrow Up/Down, PageUp/Down, Home/End
- Mouse wheel: 40px per delta unit
- Scrollbar drag: tracked by UiState

### CK3 system
- `scrollbox` with blockoverride slots (content, background, margins, empty)
- `fixedgridbox` for virtual/recycled lists (datamodel_reuse_widgets = yes)
- Explicit column/row sizes: `addcolumn = N`, `addrow = N`
- Wrapping via `datamodel_wrap`
- Filter dropdowns, sort controls, search fields integrated into list headers
- `flipdirection = yes` for right-to-left filling

### Questions to investigate
1. CK3's fixedgridbox does widget recycling (`datamodel_reuse_widgets = yes`). Our
   ScrollList creates new widgets each frame. Is this a performance problem? For
   lists of 50-100 items, probably not. For 1000+ character lists, possibly.
2. CK3 scrollbox has a `scrollbox_empty` slot for "no results" text. Our ScrollList
   renders nothing when empty. Should we add an empty-state?
3. CK3 lists have integrated filter/sort controls in headers. Our character_finder
   has a search field but the pattern isn't standardized. Should we create a
   `FilterableList` builder pattern?
4. CK3 uses `fixedgridbox` (grid layout with wrapping) for many lists. We have
   no grid layout widget. Do we need one?

### What to read
- Our: `src/ui/mod.rs` (ScrollList layout), `src/ui/character_finder.rs`
- CK3: `.workflow/ck3-ui-reference.md` "Cross-Cutting Patterns" → "List Pattern"
- CK3 deep: `window_character_finder.gui`

---

## Area 9: Modal & Dialog Patterns

### Our system
- ModalStack: Vec of (dim_layer, content) pairs
- Dim layer: fullscreen semi-transparent Panel at ZTier::Modal
- No standard dialog template
- No click-outside-to-dismiss
- No standard button bar (accept/cancel pattern)
- ESC handling not built into ModalStack

### CK3 system
- `base_dialog`: centered, layer=confirmation, Background_Fade dim, 100%×100%
- `confirmation_popup`: cancel (button_standard) + accept (button_primary), 15px spacer
- `rename_popup`: editbox + optional color picker
- Click-outside-to-dismiss: invisible button behind content as backdrop
- `shortcut = close_window` on cancel, `shortcut = confirm` on accept
- Settings: full-screen dim, left tabs, right scrollable content

### Questions to investigate
1. CK3's click-outside-to-dismiss uses an invisible button as backdrop. Our dim
   layer blocks clicks but doesn't dismiss on click. Should it?
2. CK3 has a standard confirmation dialog (cancel + accept buttons, centered).
   We build each dialog ad-hoc. Should we create `build_confirmation_dialog(
   tree, theme, title, message, accept_text, cancel_text) -> WidgetId`?
3. CK3 binds `shortcut = confirm` to the accept button and `shortcut = close_window`
   to cancel. Our modals don't bind Enter/ESC. This is a usability gap.

### What to read
- Our: `src/ui/mod.rs` (ModalStack)
- CK3: `.workflow/ck3-gui-shared-index.md` section 12 (dialogs.gui)
- CK3 deep: `shared/dialogs.gui`, `interaction_confirmation.gui`

---

## Area 10: Background & Visual Composition

### Our system
- PanelCommand: flat colored rect + 1px border + inner shadow
- Single background per panel, solid color
- No texture support in panel renderer (pure geometric)
- No 9-slice, no overlay blending, no masks
- Shadow: inner shadow effect, fixed direction

### CK3 system
- Background sandwich: base texture (9-slice Corneredtiled) + overlay (blend mode) +
  alpha mask (alphamultiply)
- Status colors: Bad (red 0.7α), Good (green 0.7α), Mixed (yellow)
- Alternating row backgrounds via modulo check
- Vignette effects on buttons
- Multiple stacked modify_texture layers with blend modes

### Questions to investigate
1. Our panels are flat colored rects. CK3 uses textured 9-slice backgrounds.
   We're ASCII-art themed, so flat color may be correct — but should we add
   subtle texture support for visual depth?
2. CK3 uses status colors (green/red/yellow backgrounds) extensively for quick
   visual feedback. We use text color only. Should we add colored panel
   backgrounds for status indication?
3. CK3's alternating row backgrounds improve list readability. Our ScrollList
   items have no visual alternation. Easy win?

### What to read
- Our: `src/ui/draw.rs` (PanelCommand), `src/panel.rs`
- CK3: `.workflow/ck3-gui-shared-index.md` section 2 (backgrounds.gui)

---

## Deliverables

After completing the review, produce:

1. **Bug list**: Issues where our UI is incorrect (draw order, clipping, positioning)
2. **Antipattern list**: Architectural decisions that will cause problems at scale
3. **Missing features ranked by impact**: Things CK3 has that we should add
4. **Proposed changes**: Specific code changes, ordered by priority
5. **Updated backlog entries**: New UI-xxx tasks for `.workflow/backlog.md`

Keep proposals minimal. You must choose the correct and/or architecturally consistent solution over one that is good enough or a hack. You must never make a decision based on preserving backward compatibility. Don't redesign the widget system. Fix the draw order,
add the missing primitives (expand, grid, standard dialogs), and systematize
what's already ad-hoc.

**MANDATORY**: NEVER ADD DEAD CODE SUPPRESSION. ALWAYS REMOVE DEAD CODE
