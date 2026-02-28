# UI-511 Audit Report — `pub(crate)` + Re-export Convention

## Executive Summary

The 33 `#[allow(unused_imports)]` annotations on `pub use` lines in
`src/ui/mod.rs` are **all unnecessary**. Rust's `unused_imports` lint does not
fire on `pub use` items in a library crate — the compiler assumes they may be
consumed by external crates. Only `pub(crate) use` is checked for intra-crate
usage.

**Verified:** stripping all 33 `#[allow(unused_imports)]` and running
`cargo check` produces exactly **1 warning**: the `pub(crate) use
node::WidgetNode;` line (which is genuinely dead).

---

## Phase 1 — Dead Re-export Analysis

### Key Finding: `pub use` Never Warns

The `unused_imports` lint treats `pub use` as public API. It only fires on
`pub(crate) use` when the symbol is unused within the crate. This means:

- All 33 `#[allow(unused_imports)]` on `pub use` lines → **can be removed**
- The 1 `pub(crate) use node::WidgetNode;` → **genuinely dead, should be deleted**

### Liveness Classification

Despite the lint not firing, many re-exports are functionally dead (nothing uses
them through the re-export path). The table below classifies each symbol by
actual usage, which matters for code hygiene even if the compiler doesn't warn.

**Legend:**
- **Live-int** = used via `super::NAME` in non-test ui/ siblings (strongest justification)
- **Live-ext** = NOT used via `super::`, but IS used by main.rs or font.rs
- **Dead** = not used via re-export path anywhere; only accessed via module path or not at all

#### Line 45: `action::{PanelKind, UiAction}`

| Symbol    | Class    | Evidence |
|-----------|----------|----------|
| UiAction  | Live-int | `super::UiAction::*` in modal, map_mode, main_menu, event_popup, sidebar, outliner, character_finder, settings, save_load, context_menu, input |
| PanelKind | Dead     | Accessed only via `super::action::PanelKind` (module path) in panel_manager, context |

#### Line 47: `animation::{Anim, Animator, Easing}`

| Symbol   | Class    | Evidence |
|----------|----------|----------|
| Anim     | Live-ext | `ui::Anim` in main.rs; ui/ siblings use `super::super::animation::Anim` |
| Animator | Live-ext | `ui::Animator::new()` in main.rs; context.rs uses `super::animation::Animator` |
| Easing   | Live-ext | `ui::Easing::*` in main.rs; context.rs uses `super::super::animation::Easing` |

#### Lines 49–51: `character_finder::{CharacterFinderInfo, FinderEntry, FinderSort, build_character_finder, collect_finder_entries}`

| Symbol                | Class | Evidence |
|-----------------------|-------|----------|
| CharacterFinderInfo   | Dead  | Not used outside character_finder.rs |
| FinderEntry           | Dead  | Not used outside character_finder.rs |
| FinderSort            | Dead  | Not used outside character_finder.rs |
| build_character_finder| Dead  | Not called from main.rs yet |
| collect_finder_entries| Dead  | Not called from main.rs yet |

#### Line 53: `character_panel::{CharacterPanelInfo, build_character_panel, collect_character_info}`

| Symbol               | Class | Evidence |
|----------------------|-------|----------|
| CharacterPanelInfo   | Dead  | Not used outside character_panel.rs |
| build_character_panel| Dead  | Not called from main.rs yet |
| collect_character_info| Dead | Not called from main.rs yet |

#### Line 55: `context::{DismissResult, SidebarState, UiContext}`

| Symbol        | Class    | Evidence |
|---------------|----------|----------|
| DismissResult | Live-ext | `ui::DismissResult` in main.rs |
| SidebarState  | Live-ext | `ui::SidebarState` in main.rs |
| UiContext      | Live-ext | `ui::UiContext` in main.rs |

#### Line 57: `context_menu::{ContextMenu, MenuItem}`

| Symbol      | Class | Evidence |
|-------------|-------|----------|
| ContextMenu | Dead  | Not used outside context_menu.rs |
| MenuItem    | Dead  | Not used outside context_menu.rs |

#### Lines 59–62: `draw::{DrawList, FontFamily, HeuristicMeasurer, PanelCommand, RichTextCommand, SpriteCommand, TextCommand, TextMeasurer, TextSpan}`

| Symbol           | Class    | Evidence |
|------------------|----------|----------|
| DrawList         | Live-ext | `ui::DrawList` in main.rs |
| FontFamily       | Live-int | `super::FontFamily` in 11+ builder files; also `crate::ui::FontFamily` in font.rs |
| HeuristicMeasurer| Live-ext | `crate::ui::HeuristicMeasurer` in 5 test blocks within ui/ (dead in non-test cargo check, but used in test builds) |
| PanelCommand     | Dead     | Not used via re-export path anywhere |
| RichTextCommand  | Dead     | Not used via re-export path anywhere |
| SpriteCommand    | Dead     | Not used via re-export path anywhere |
| TextCommand      | Dead     | Not used via re-export path anywhere |
| TextMeasurer     | Live-ext | `crate::ui::TextMeasurer` in font.rs (trait impl) |
| TextSpan         | Dead     | Accessed only via `super::draw::TextSpan` (module path) |

#### Line 64: `entity_inspector::{EntityInspectorInfo, build_entity_inspector, collect_inspector_info}`

| Symbol               | Class    | Evidence |
|----------------------|----------|----------|
| EntityInspectorInfo  | Live-int | `super::EntityInspectorInfo` in sidebar.rs |
| build_entity_inspector| Live-ext | `ui::build_entity_inspector` in main.rs |
| collect_inspector_info| Live-ext | `ui::collect_inspector_info` in main.rs |

#### Line 66: `event_log::{EventLogEntry, build_event_log, collect_event_entries}`

| Symbol              | Class    | Evidence |
|---------------------|----------|----------|
| EventLogEntry       | Dead     | Not used outside event_log.rs |
| build_event_log     | Live-ext | `ui::build_event_log` in main.rs |
| collect_event_entries| Live-ext | `ui::collect_event_entries` in main.rs |

#### Lines 67–68: `event_popup::{EventChoice, NarrativeEvent, build_event_popup}`

| Symbol          | Class | Evidence |
|-----------------|-------|----------|
| EventChoice     | Dead  | Not used outside event_popup.rs |
| NarrativeEvent  | Dead  | Not used outside event_popup.rs |
| build_event_popup| Dead | Not called from main.rs yet |

#### Line 70: `geometry::{Constraints, Edges, Position, Rect, Size, Sizing}`

| Symbol      | Class    | Evidence |
|-------------|----------|----------|
| Constraints | Dead     | Accessed only via `super::geometry::Constraints` (module path) |
| Edges       | Live-int | `super::Edges` in 7+ builder files |
| Position    | Live-int | `super::Position` in 7+ builder files |
| Rect        | Live-int | `super::Rect` in draw.rs |
| Size        | Live-int | `super::Size` in context_menu.rs, sidebar.rs, draw.rs; also `crate::ui::Size` in font.rs |
| Sizing      | Live-int | `super::Sizing` in 12+ builder files |

#### Lines 71–72: `hover_tooltip::{HoverInfo, build_hover_tooltip}`

| Symbol            | Class    | Evidence |
|-------------------|----------|----------|
| HoverInfo         | Live-ext | `ui::HoverInfo` in main.rs |
| build_hover_tooltip| Live-ext | `ui::build_hover_tooltip` in main.rs |

#### Line 74: `input::{MapClick, MouseButton, UiEvent, UiState}`

| Symbol      | Class    | Evidence |
|-------------|----------|----------|
| MapClick    | Dead     | Not used outside input.rs |
| MouseButton | Live-ext | `ui::MouseButton::*` in main.rs |
| UiEvent     | Dead     | Not used outside input.rs |
| UiState     | Live-ext | `ui::UiState::new()` in main.rs |

#### Line 76: `keybindings::{Action, KeyBindings, KeyCombo, ModifierFlags}`

| Symbol       | Class    | Evidence |
|--------------|----------|----------|
| Action       | Live-ext | `ui::Action::*` in main.rs |
| KeyBindings  | Live-ext | `ui::KeyBindings::defaults()` in main.rs |
| KeyCombo     | Live-ext | `ui::KeyCombo` in main.rs |
| ModifierFlags| Live-ext | `ui::ModifierFlags` in main.rs |

#### Line 78: `loading_screen::{LoadingScreenInfo, LoadingStage, build_loading_screen}`

| Symbol           | Class | Evidence |
|------------------|-------|----------|
| LoadingScreenInfo| Dead  | Not called from main.rs yet |
| LoadingStage     | Dead  | Not used outside loading_screen.rs |
| build_loading_screen| Dead | Not called from main.rs yet |

#### Line 80: `main_menu::{AppState, MainMenuInfo, build_main_menu}`

| Symbol       | Class | Evidence |
|--------------|-------|----------|
| AppState     | Dead  | Not used outside main_menu.rs |
| MainMenuInfo | Dead  | Not used outside main_menu.rs |
| build_main_menu | Dead | Not called from main.rs yet |

#### Line 82: `map_mode::{MapMode, MapModeInfo, build_map_mode_selector}`

| Symbol              | Class | Evidence |
|---------------------|-------|----------|
| MapMode             | Dead  | Not used outside map_mode.rs |
| MapModeInfo         | Dead  | Not used outside map_mode.rs |
| build_map_mode_selector | Dead | Not called from main.rs yet |

#### Line 84: `minimap::{MinimapInfo, MinimapTexture, build_minimap, minimap_click_to_world}`

| Symbol               | Class    | Evidence |
|----------------------|----------|----------|
| MinimapInfo          | Live-ext | `ui::MinimapInfo` in main.rs |
| MinimapTexture       | Live-ext | `ui::MinimapTexture::new()` in main.rs |
| build_minimap        | Live-ext | `ui::build_minimap` in main.rs |
| minimap_click_to_world| Live-ext | `ui::minimap_click_to_world` in main.rs |

#### Line 86: `modal::{ModalOptions, ModalPop, ModalStack}`

| Symbol       | Class    | Evidence |
|--------------|----------|----------|
| ModalOptions | Dead     | Used only via `super::modal::ModalOptions` (module path) or `super::super::modal::ModalOptions` in context.rs tests |
| ModalPop     | Dead     | Used only via `super::modal::ModalPop` (module path) in context.rs |
| ModalStack   | Live-ext | `ui::ModalStack::new()` in main.rs |

#### Line 88: `pub(crate) use node::WidgetNode`

| Symbol     | Class | Evidence |
|------------|-------|----------|
| WidgetNode | Dead  | All usages go through `super::node::WidgetNode` (module path). **Only re-export that produces a compiler warning.** |

#### Line 90: `node::{UiPerfMetrics, ZTier}`

| Symbol       | Class    | Evidence |
|--------------|----------|----------|
| UiPerfMetrics| Live-ext | `ui::UiPerfMetrics` in main.rs |
| ZTier        | Live-int | `super::ZTier` in modal.rs, context_menu.rs, input.rs |

#### Line 92: `notification::{NotificationManager, NotificationPriority}`

| Symbol               | Class | Evidence |
|----------------------|-------|----------|
| NotificationManager  | Dead  | Not used outside notification.rs |
| NotificationPriority | Dead  | Not used outside notification.rs |

#### Lines 94–98: `opinion_view::*`, `outliner::*`

| Symbol           | Class | Evidence |
|------------------|-------|----------|
| OpinionModifier  | Dead  | Not called from main.rs yet |
| OpinionViewInfo  | Dead  | Not called from main.rs yet |
| Sentiment        | Dead  | Not called from main.rs yet |
| build_opinion_view| Dead | Not called from main.rs yet |
| ActiveEvent      | Dead  | Not called from main.rs yet |
| AlertEntry       | Dead  | Not called from main.rs yet |
| AlertPriority    | Dead  | Not called from main.rs yet |
| OutlinerInfo     | Dead  | Not called from main.rs yet |
| PinnedCharacter  | Dead  | Not called from main.rs yet |
| build_outliner   | Dead  | Not called from main.rs yet |

#### Line 100: `panel_manager::PanelManager`

| Symbol       | Class    | Evidence |
|--------------|----------|----------|
| PanelManager | Live-ext | `ui::PanelManager::new()` in main.rs |

#### Line 102: `pause_overlay::build_pause_overlay`

| Symbol             | Class    | Evidence |
|--------------------|----------|----------|
| build_pause_overlay| Live-ext | `ui::build_pause_overlay` in main.rs |

#### Line 104: `save_load::{SaveFileEntry, SaveLoadInfo, build_save_load_screen}`

| Symbol              | Class | Evidence |
|---------------------|-------|----------|
| SaveFileEntry       | Dead  | Not called from main.rs yet |
| SaveLoadInfo        | Dead  | Not called from main.rs yet |
| build_save_load_screen | Dead | Not called from main.rs yet |

#### Line 106: `settings::{SettingsInfo, build_settings_screen}`

| Symbol              | Class | Evidence |
|---------------------|-------|----------|
| SettingsInfo        | Dead  | Not called from main.rs yet |
| build_settings_screen| Dead | Not called from main.rs yet |

#### Lines 108–111: `sidebar::{MAIN_TAB_WIDTH, SIDEBAR_MARGIN, SidebarInfo, TAB_COUNT, build_placeholder_view, build_showcase_view, build_tab_strip}`

| Symbol              | Class    | Evidence |
|---------------------|----------|----------|
| MAIN_TAB_WIDTH      | Live-ext | `ui::MAIN_TAB_WIDTH` in main.rs |
| SIDEBAR_MARGIN      | Live-ext | `ui::SIDEBAR_MARGIN` in main.rs |
| SidebarInfo         | Live-ext | `ui::SidebarInfo` in main.rs |
| TAB_COUNT           | Dead     | Not used outside sidebar.rs |
| build_placeholder_view| Live-ext | `ui::build_placeholder_view` in main.rs |
| build_showcase_view | Live-ext | `ui::build_showcase_view` in main.rs |
| build_tab_strip     | Live-ext | `ui::build_tab_strip` in main.rs |

#### Line 113: `sprite::{SpriteAtlas, SpriteRect}`

| Symbol      | Class | Evidence |
|-------------|-------|----------|
| SpriteAtlas | Dead  | Not used outside sprite.rs via re-export |
| SpriteRect  | Dead  | Not used outside sprite.rs via re-export |

#### Lines 115–117: `status_bar::*`, `theme::Theme`, `tree::WidgetTree`

| Symbol        | Class    | Evidence |
|---------------|----------|----------|
| StatusBarInfo | Live-ext | `ui::StatusBarInfo` in main.rs |
| build_status_bar | Live-ext | `ui::build_status_bar` in main.rs |
| Theme         | Live-ext | `ui::Theme::default()` in main.rs (no `#[allow]` currently) |
| WidgetTree    | Live-int | `super::WidgetTree` in 15+ builder files (no `#[allow]` currently) |

#### Line 119: `widget::{CrossAlign, TooltipContent, Widget}`

| Symbol         | Class    | Evidence |
|----------------|----------|----------|
| CrossAlign     | Dead     | Accessed only via `super::widget::CrossAlign` (module path) |
| TooltipContent | Dead     | Accessed only via `super::widget::TooltipContent` (module path) |
| Widget         | Live-int | `super::Widget` in 13+ builder files |

#### Lines 121–122: `window::{ConfirmationDialog, WindowFrame, build_confirmation_dialog, build_window_frame}`

| Symbol                  | Class | Evidence |
|-------------------------|-------|----------|
| ConfirmationDialog      | Dead  | Not used outside window.rs |
| WindowFrame             | Dead  | Not used outside window.rs via re-export |
| build_confirmation_dialog| Dead | Not called from main.rs yet |
| build_window_frame      | Dead  | Accessed only via `super::window::build_window_frame` (module path) |

### Summary Counts

| Category  | Count | Description |
|-----------|-------|-------------|
| Live-int  | 12    | Used via `super::NAME` in non-test ui/ code — naturally no warning |
| Live-ext  | 36    | Used by main.rs or font.rs — `pub use` never warns anyway |
| Dead      | 57    | Not used via re-export path; only via module path or not at all |
| **Total** | **105** | Across all re-export lines (including WidgetNode) |

### `#[allow(unused_imports)]` Disposition

| Action | Count | Detail |
|--------|-------|--------|
| Remove `#[allow]` (was never needed) | 33 | All `pub use` lines — the lint never fires on `pub use` in a lib crate |
| Delete dead `pub(crate) use` | 1 | `node::WidgetNode` — the only re-export that actually warns |
| Delete dead `pub use` lines | — | Optional hygiene; the compiler doesn't care |

---

## Phase 2 — Grouping Analysis

### Current Organization

Re-exports are ordered alphabetically by source module. Each `pub use` block
has a comment citing a ticket number, e.g. `// Public API: used by main.rs for
character finder (UI-402)`.

### Proposed Concern Groups (after removing 57 dead symbols)

| Group | Symbols | Count |
|-------|---------|-------|
| **Core tree** | WidgetTree, Widget | 2 |
| **Geometry** | Edges, Position, Rect, Size, Sizing | 5 |
| **Drawing** | DrawList, FontFamily, HeuristicMeasurer, TextMeasurer | 4 |
| **Node metadata** | ZTier, UiPerfMetrics | 2 |
| **Input** | MouseButton, UiState | 2 |
| **Actions** | UiAction | 1 |
| **Animation** | Anim, Animator, Easing | 3 |
| **Infrastructure** | Theme, KeyBindings, Action, KeyCombo, ModifierFlags, UiContext, DismissResult, SidebarState, PanelManager, ModalStack | 10 |
| **Builders** | EntityInspectorInfo, build_entity_inspector, collect_inspector_info, build_event_log, collect_event_entries, HoverInfo, build_hover_tooltip, MinimapInfo, MinimapTexture, build_minimap, minimap_click_to_world, build_pause_overlay, MAIN_TAB_WIDTH, SIDEBAR_MARGIN, SidebarInfo, build_placeholder_view, build_showcase_view, build_tab_strip, StatusBarInfo, build_status_bar | 20 |

9 groups, 49 symbols. Only "Actions" has <2 items (UiAction alone — could
merge into Core or Infrastructure).

### Assessment

Concern grouping would be a mild improvement over alphabetical-by-module, but
the real readability win comes from deleting the 57 dead symbols, which cuts
the re-export block from ~80 lines to ~30 lines. At 30 lines, grouping vs
alphabetical barely matters.

---

## Phase 3 — Module Merge Analysis

### Candidate Pairs

| Candidate | Lines A | Lines B | Combined | Same Concern? | Re-export Savings | Recommendation |
|-----------|---------|---------|----------|---------------|-------------------|----------------|
| node.rs + widget.rs | 82 | 207 | 289 | Partial (data types, but node is arena metadata vs widget is variant enum) | 1 line | **Weak.** node.rs is arena infrastructure (WidgetNode, ZTier, UiPerfMetrics); widget.rs is the Widget enum. Different audiences. |
| geometry.rs + draw.rs | 146 | 171 | 317 | No (spatial math vs rendering primitives) | 1 line | **No.** Clearly different concerns. |
| action.rs → context.rs | 66 | 248 | 314 | Yes (context already imports action) | 1 line | **Plausible.** UiAction/PanelKind are tightly coupled with UiContext dispatch. But action.rs is also imported by many builders for `set_on_click`. Keeping it separate gives it a clear identity. |
| tree_tooltip.rs + tree_hit_test.rs | 96 | 181 | 277 | Yes (tree operations) | 0 lines | **No savings.** Neither has re-exports. Both are tree operations but tooltip and hit-test are distinct concerns. |
| test_helpers.rs | 9 | — | — | N/A | 0 lines | **Yes, inline it.** 9 lines (1 function) could move to any module's test block or into a shared test cfg block. |

### Assessment

Module merges yield negligible re-export savings (0–1 line each). The primary
benefit would be reducing module count, but the project's one-concern-per-file
convention is well-established and the modules are clear. The only
recommendation is inlining `test_helpers.rs` (9 lines).

---

## Phase 4 — Final Verdict

### Findings

1. **The `#[allow(unused_imports)]` problem is a phantom.** The lint never fires
   on `pub use` in a lib crate. All 33 annotations were defensive/prophylactic
   and can be removed unconditionally.

2. **57 of 105 re-exported symbols are functionally dead** — not accessed through
   the re-export path by any code. Most are builders not yet wired into main.rs,
   or types only accessed via direct module paths. Deleting them would cut the
   re-export block from ~80 lines to ~30 lines.

3. **`pub(crate) use node::WidgetNode;` is the only re-export that produces a
   real warning.** It should be deleted; all consumers use `super::node::WidgetNode`.

4. **Grouping re-exports by concern** is a minor readability improvement. After
   dead-symbol cleanup, the block is small enough that alphabetical order works fine.

5. **Module merges** yield no meaningful re-export savings. `test_helpers.rs`
   (9 lines) is the only candidate worth inlining.

### Recommendation: Keep Convention, Clean Up

**Keep** the `pub(crate)` modules + `pub use` re-export convention. It works
correctly — the lint issue was a misunderstanding. Apply cleanup:

| Action | Impact |
|--------|--------|
| Remove all 33 `#[allow(unused_imports)]` annotations | Eliminates noise; annotations were never needed |
| Delete `pub(crate) use node::WidgetNode;` | Fixes the only real compiler warning |
| Delete 57 dead `pub use` symbols | Reduces re-export block from ~80 to ~30 lines |
| Remove ticket-number comments on re-exports | They reference internal planning, not code rationale |
| Inline `test_helpers.rs` (9 lines) | Reduces module count by 1 |

**Do not amend CLAUDE.md.** The convention in CLAUDE.md
("`mod.rs` is module declarations + re-exports only") is sound. The issue was
over-broad re-exporting and unnecessary `#[allow]` annotations, not a
convention problem.

### Surviving Re-exports After Cleanup (30 lines)

```rust
// Core tree types
pub use tree::WidgetTree;
pub use widget::Widget;

// Geometry
pub use geometry::{Edges, Position, Rect, Size, Sizing};

// Drawing
pub use draw::{DrawList, FontFamily, HeuristicMeasurer, TextMeasurer};

// Node metadata
pub use node::{UiPerfMetrics, ZTier};

// Actions & input
pub use action::UiAction;
pub use input::{MouseButton, UiState};

// Animation
pub use animation::{Anim, Animator, Easing};

// Infrastructure
pub use context::{DismissResult, SidebarState, UiContext};
pub use keybindings::{Action, KeyBindings, KeyCombo, ModifierFlags};
pub use modal::ModalStack;
pub use panel_manager::PanelManager;
pub use theme::Theme;

// Builder APIs (consumed by main.rs)
pub use entity_inspector::{EntityInspectorInfo, build_entity_inspector, collect_inspector_info};
pub use event_log::{build_event_log, collect_event_entries};
pub use hover_tooltip::{HoverInfo, build_hover_tooltip};
pub use minimap::{MinimapInfo, MinimapTexture, build_minimap, minimap_click_to_world};
pub use pause_overlay::build_pause_overlay;
pub use sidebar::{
    MAIN_TAB_WIDTH, SIDEBAR_MARGIN, SidebarInfo,
    build_placeholder_view, build_showcase_view, build_tab_strip,
};
pub use status_bar::{StatusBarInfo, build_status_bar};
```
