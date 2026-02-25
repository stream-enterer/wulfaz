# UI Architecture Migration Plan

> Three independent migrations implementing decisions from `ui-architecture-patterns.md`.
> Each phase is self-contained — can be done in any order.
> Reference: `.workflow/ui-architecture-patterns.md` Pass 5 (Proving Ground).

## How to Use This Document

Each step has:
- **What:** Exactly what changes.
- **Where:** File paths and line ranges (as of commit beaf1f6).
- **Checkpoint:** Verifiable condition that proves the step is complete.

Steps within a phase are sequential. Phases are independent.

---

## Phase 1 — UiAction Enum Migration (D5 + D7 + D13)

Goal: Replace stringly-typed callback dispatch with compiler-verified enums.
Eliminates 18 silent dead callbacks. Removes runtime string parsing for payloads.
Migration is compiler-guided: change a type, fix every error.

Decisions implemented: D5-B (enum callbacks), D7-B (return Option\<UiAction\>), D13-B (PanelKind enum).

### P1S1 — Define UiAction and PanelKind enums

Create `src/ui/action.rs`. Define two enums covering the full current callback vocabulary.

**UiAction enum — complete variant list** (derived from all 25 `set_on_click` call sites across 13 files):

```rust
/// Every UI interaction the app can dispatch. Exhaustive match enforces handling.
#[derive(Debug, Clone)]
pub enum UiAction {
    // Inspector (src/ui/mod.rs — build_entity_inspector)
    InspectorClose,

    // Sidebar (src/ui/sidebar.rs)
    SelectTab(usize),

    // Modal (src/ui/modal.rs)
    ModalDismiss,

    // Dialog (src/ui/window.rs — build_confirmation_dialog)
    DialogAccept,
    DialogCancel,

    // Main menu (src/ui/main_menu.rs)
    MenuNewGame,
    MenuContinue,
    MenuLoad,
    MenuSettings,
    MenuQuit,

    // Outliner (src/ui/outliner.rs)
    OutlinerSelectCharacter(Entity),
    OutlinerSelectEvent(String),

    // Character finder (src/ui/character_finder.rs)
    FinderSort,
    FinderSelect(Entity),

    // Settings (src/ui/settings.rs)
    SettingsUiScale,
    SettingsWindowMode,

    // Save/Load (src/ui/save_load.rs)
    SaveLoadSave,
    SaveLoadLoad,
    SaveLoadSelect(String),

    // Map mode (src/ui/map_mode.rs)
    MapModeChange,
    MapModeSpeed,

    // Event popup (src/ui/event_popup.rs) — data-driven, callback from KDL
    EventChoice(String),

    // Context menu (src/ui/context_menu.rs) — data-driven, action from MenuItem
    ContextAction(String),
}
```

**PanelKind enum:**

```rust
/// Known panel types. Used as keys in PanelManager and scroll offset maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelKind {
    Sidebar,
    Outliner,
    CharacterPanel,
    CharacterFinder,
    OpinionView,
    Settings,
    SaveLoad,
    MapMode,
    EventPopup,
}
```

Register the module in `src/ui/mod.rs`:
```rust
pub(crate) mod action;
pub use action::{UiAction, PanelKind};
```

- Checkpoint: `cargo check` succeeds. New file exists. Enums are importable from `ui::UiAction` and `ui::PanelKind`.

### P1S2 — Change WidgetNode.on_click from String to UiAction

In `src/ui/mod.rs`, WidgetNode struct (line 264):
- Change `pub on_click: Option<String>` → `pub on_click: Option<UiAction>`.

In `src/ui/mod.rs`, `set_on_click` method (~line 534):
- Change signature from `pub fn set_on_click(&mut self, id: WidgetId, key: impl Into<String>)` → `pub fn set_on_click(&mut self, id: WidgetId, action: UiAction)`.
- Change body from `node.on_click = Some(key.into())` → `node.on_click = Some(action)`.

- Checkpoint: `cargo check` fails with errors at every `set_on_click` call site (25 sites across 13 files) and every place that reads `on_click` (input.rs). This is expected — the compiler is now guiding the migration.

### P1S3 — Update all set_on_click call sites

Fix each compiler error. Convert string literals to enum variants. For each file:

| File | Current String | New Variant |
|------|---------------|-------------|
| `src/ui/mod.rs` | `"inspector::close"` | `UiAction::InspectorClose` |
| `src/ui/sidebar.rs` | `format!("sidebar::tab::{i}")` | `UiAction::SelectTab(i)` |
| `src/ui/modal.rs` | `MODAL_DISMISS` | `UiAction::ModalDismiss` |
| `src/ui/window.rs` | `DIALOG_ACCEPT` | `UiAction::DialogAccept` |
| `src/ui/window.rs` | `DIALOG_CANCEL` | `UiAction::DialogCancel` |
| `src/ui/main_menu.rs` | `"menu::new_game"` etc (5) | `UiAction::MenuNewGame` etc |
| `src/ui/outliner.rs` | `format!("outliner::character:{}", id)` | `UiAction::OutlinerSelectCharacter(entity)` |
| `src/ui/outliner.rs` | `format!("outliner::event:{}", cb)` | `UiAction::OutlinerSelectEvent(cb)` |
| `src/ui/character_finder.rs` | `"finder::sort"` | `UiAction::FinderSort` |
| `src/ui/character_finder.rs` | `format!("finder::select:{}", id)` | `UiAction::FinderSelect(entity)` |
| `src/ui/settings.rs` | `"settings::ui_scale"` | `UiAction::SettingsUiScale` |
| `src/ui/settings.rs` | `"settings::window_mode"` | `UiAction::SettingsWindowMode` |
| `src/ui/save_load.rs` | `"save_load::save"` | `UiAction::SaveLoadSave` |
| `src/ui/save_load.rs` | `"save_load::load"` | `UiAction::SaveLoadLoad` |
| `src/ui/save_load.rs` | `format!("save_load::select:{}", name)` | `UiAction::SaveLoadSelect(name)` |
| `src/ui/map_mode.rs` | `"map_mode::change"` | `UiAction::MapModeChange` |
| `src/ui/map_mode.rs` | `"map_mode::speed"` | `UiAction::MapModeSpeed` |
| `src/ui/event_popup.rs` | `format!("event_choice:{}", cb)` | `UiAction::EventChoice(cb)` |
| `src/ui/context_menu.rs` | `item.action.clone()` | `UiAction::ContextAction(item.action.clone())` |

Also remove the string constants `MODAL_DISMISS`, `DIALOG_ACCEPT`, `DIALOG_CANCEL` from wherever they are defined (no longer needed — the enum variant IS the constant).

- Checkpoint: All `set_on_click` call sites compile. No string callback literals remain in `src/ui/` builder files. `grep -r 'set_on_click.*"' src/ui/` returns zero hits (excluding comments and tests).

### P1S4 — Update UiState click_event to carry UiAction

In `src/ui/input.rs`, UiState struct (~line 110):
- Change `pub click_event: Option<(WidgetId, String)>` → `pub click_event: Option<(WidgetId, UiAction)>`.

In `src/ui/input.rs`, wherever `click_event` is set (in `handle_mouse_input`):
- The code currently clones the string from `node.on_click`. Now it clones the `UiAction`. `UiAction` must derive `Clone` (already specified in P1S1).

Update `poll_click()` return type:
- From `Option<(WidgetId, String)>` → `Option<(WidgetId, UiAction)>`.

- Checkpoint: `src/ui/input.rs` compiles. `poll_click` returns typed actions.

### P1S5 — Rewrite dispatch_click to exhaustive enum match

In `src/main.rs`, `App::dispatch_click` (~line 494):
- Change signature from `fn dispatch_click(&mut self, action: &str)` → `fn dispatch_click(&mut self, action: UiAction)`.
- Replace the string `match` + `starts_with` + `parse` logic with an exhaustive `match action { ... }`.
- Remove the `_ =>` catch-all arm. Every variant must be handled explicitly (even if the handler is `=> {}` for not-yet-implemented actions). The compiler will enforce this.
- Remove payload parsing code (`starts_with("sidebar::tab::")`, `.parse::<u64>()`). Payloads are now typed fields on the enum variant.

Also update the call site in `main.rs` that calls `dispatch_click`:
- Currently: `if let Some((_wid, action)) = self.ui_state.poll_click() { self.dispatch_click(&action); }`
- Now: `if let Some((_wid, action)) = self.ui.input.poll_click() { self.dispatch_click(action); }` (pass by value, not reference — `UiAction` is moved, not borrowed).

- Checkpoint: `cargo check` succeeds. `dispatch_click` has no `_ =>` arm. All 22+ variants are explicitly matched. No `starts_with` or `.parse` calls remain in dispatch.

### P1S6 — Migrate PanelManager to PanelKind keys

In `src/ui/panel_manager.rs`:
- Change `panels: HashMap<String, PanelEntry>` → `panels: HashMap<PanelKind, PanelEntry>`.
- Change `draw_order: Vec<String>` → `draw_order: Vec<PanelKind>`.
- Change `scroll_offsets: HashMap<String, f32>` → `HashMap<PanelKind, f32>` (temporary — this moves to UiContext in Phase 2).
- Change `closing: Vec<ClosingPanel>` — if `ClosingPanel` contains a name string, change to `PanelKind`.
- Update all method signatures: `open(name: &str, ...)` → `open(kind: PanelKind, ...)`, etc.

In `src/main.rs`, update all PanelManager call sites:
- `panel_manager.open("sidebar", ...)` → `panel_manager.open(PanelKind::Sidebar, ...)`, etc.

- Checkpoint: `cargo check` succeeds. No string literals remain in PanelManager interactions. `grep -r 'panel_manager.*"' src/main.rs` returns zero hits.

### P1S7 — Update tests

Update any tests that construct `WidgetNode` manually or call `set_on_click` with strings. Tests in `src/ui/mod.rs` (lines 3800-8597) and any tests in builder files.

- Checkpoint: `cargo test` passes. Zero warnings about unused imports of removed string constants.

### P1 Verification

- `cargo build` succeeds.
- `cargo test` passes.
- `grep -rn 'dispatch_click.*&str\|on_click.*String\|"sidebar::tab\|"inspector::close\|"modal::dismiss\|"menu::' src/` returns zero hits (all strings replaced with enum variants).
- Running the game: clicking UI buttons dispatches actions correctly. No silent dead buttons.

---

## Phase 2 — UiContext Consolidation (D1 + D15)

Goal: Bundle all UI state into one struct with pub sub-fields, mirroring World's pattern.
Eliminates ad-hoc state on App. Decouples scroll persistence from PanelManager.

Decisions implemented: D1-A (single UiContext + split borrows), D15-B (scroll on UiContext).

### P2S1 — Define UiContext struct

Create `src/ui/context.rs`. Define the struct with pub fields for split borrows:

```rust
use crate::ui::{UiState, Animator, ModalStack, PanelManager, WidgetId};
use std::collections::HashMap;

/// All persistent UI state. Mirrors World's role for the simulation layer.
/// Pub fields enable Rust's field-level split borrowing.
pub struct UiContext {
    /// Input state: hover, focus, press, captured, scroll drag, tooltips.
    pub input: UiState,
    /// Active animations keyed by string name.
    pub animator: Animator,
    /// Modal dialog stack with dim layers and focus scoping.
    pub modals: ModalStack,
    /// Open panel tracking, draw order, animated close.
    pub panels: PanelManager,
    /// Scroll offsets for all scrollable widgets, keyed by stable name.
    /// Decoupled from PanelManager — covers sidebar, modals, and inline scrollables.
    pub scroll: HashMap<String, f32>,
    /// Sidebar-specific persistent state.
    pub sidebar: SidebarState,
}

/// Sidebar persistent state (was ad-hoc fields on App).
pub struct SidebarState {
    pub active_tab: Option<usize>,
    pub scroll_offset: f32,
    pub scroll_view_id: Option<WidgetId>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            active_tab: None,
            scroll_offset: 0.0,
            scroll_view_id: None,
        }
    }
}
```

Register the module in `src/ui/mod.rs`:
```rust
pub(crate) mod context;
pub use context::{UiContext, SidebarState};
```

- Checkpoint: `cargo check` succeeds. `UiContext` is importable from `ui::UiContext`.

### P2S2 — Move scroll_offsets out of PanelManager

In `src/ui/panel_manager.rs`:
- Remove the `scroll_offsets: HashMap<PanelKind, f32>` field (or `HashMap<String, f32>` if Phase 1 hasn't run yet).
- Remove `save_scroll`, `restore_scroll`, and any scroll-related methods.
- Any code in PanelManager that reads/writes scroll offsets now takes `&mut HashMap<String, f32>` as a parameter, or callers handle scroll externally.

- Checkpoint: `cargo check` fails at scroll call sites in `main.rs`. PanelManager no longer owns scroll data.

### P2S3 — Replace App's UI fields with single UiContext

In `src/main.rs`, App struct (~lines 354-376):

Remove these fields:
```rust
    ui_state: ui::UiState,
    // ui_tree stays on App — it is ephemeral, not persistent state
    // ui_theme stays on App — it is immutable configuration, not state
    animator: ui::Animator,
    modal_stack: ui::ModalStack,
    panel_manager: ui::PanelManager,
    sidebar_active_tab: Option<usize>,
    sidebar_scroll_offset: f32,
    sidebar_scroll_view_id: Option<ui::WidgetId>,
```

Add:
```rust
    ui: ui::UiContext,
    // ui_tree: ui::WidgetTree — stays as-is (ephemeral, rebuilt every frame)
    // ui_theme: ui::Theme — stays as-is (immutable config)
```

Note: `ui_tree` and `ui_theme` remain directly on App. The tree is ephemeral (rebuilt every frame) — it is not persistent state. The theme is immutable configuration. Only persistent mutable UI state goes into UiContext.

In App::new() (or equivalent constructor), initialize:
```rust
    ui: ui::UiContext {
        input: ui::UiState::new(),
        animator: ui::Animator::new(),
        modals: ui::ModalStack::new(),
        panels: ui::PanelManager::new(),
        scroll: HashMap::new(),
        sidebar: ui::SidebarState::new(),
    },
```

- Checkpoint: `cargo check` fails with ~125+ errors in `main.rs` at every `self.ui_state`, `self.animator`, `self.modal_stack`, `self.panel_manager`, `self.sidebar_active_tab`, `self.sidebar_scroll_offset`, `self.sidebar_scroll_view_id` access. This is expected and compiler-guided.

### P2S4 — Rewrite all App field accesses

Mechanical find-and-replace in `src/main.rs` (~105 `self.ui_state` references, ~21 sidebar references, plus animator/modal/panel references):

| Old | New |
|-----|-----|
| `self.ui_state` | `self.ui.input` |
| `self.animator` | `self.ui.animator` |
| `self.modal_stack` | `self.ui.modals` |
| `self.panel_manager` | `self.ui.panels` |
| `self.sidebar_active_tab` | `self.ui.sidebar.active_tab` |
| `self.sidebar_scroll_offset` | `self.ui.sidebar.scroll_offset` |
| `self.sidebar_scroll_view_id` | `self.ui.sidebar.scroll_view_id` |

For scroll offset save/restore sites (currently calling PanelManager methods):
- Replace `self.panel_manager.save_scroll(name, offset)` → `self.ui.scroll.insert(name.to_string(), offset)`.
- Replace `self.panel_manager.restore_scroll(name)` → `self.ui.scroll.get(name).copied().unwrap_or(0.0)`.

- Checkpoint: `cargo check` succeeds. No direct references to `self.ui_state`, `self.animator`, `self.modal_stack`, `self.panel_manager`, `self.sidebar_active_tab`, `self.sidebar_scroll_offset`, or `self.sidebar_scroll_view_id` remain in `src/main.rs`.

### P2S5 — Update builder function signatures

Builder functions in `src/ui/*.rs` that currently take individual state pieces (`&UiState`, `&Animator`, etc.) may need signature updates. There are two valid approaches:

**Approach A (pass sub-fields):** Builders take the specific sub-fields they need. E.g., `build_sidebar(tree, &theme, &mut ui.input, &ui.scroll, &ui.sidebar)`. This avoids exclusive `&mut UiContext` borrows.

**Approach B (pass &UiContext or &mut UiContext):** Builders take the full context. Simpler signatures but prevents concurrent sub-field borrows.

**Use Approach A** for builders that are called while other parts of UiContext are borrowed. Use Approach B only for top-level orchestration in main.rs.

In practice, most builders only need `&Theme` (immutable, on App not UiContext) and `&mut WidgetTree` (ephemeral, on App not UiContext), plus a data struct. The UiContext sub-fields they need are typically just scroll offsets (`&ui.scroll`) and sidebar state (`&ui.sidebar`). These are already separate pub fields, so split borrows work naturally.

- Checkpoint: `cargo check` succeeds. All builder signatures are consistent.

### P2S6 — Update tests

Any test that constructs an App or uses UI state fields directly needs updating to use UiContext.

- Checkpoint: `cargo test` passes.

### P2 Verification

- `cargo build` succeeds.
- `cargo test` passes.
- `grep -n 'self\.ui_state\|self\.animator\b\|self\.modal_stack\|self\.panel_manager\|self\.sidebar_active_tab\|self\.sidebar_scroll_offset\|self\.sidebar_scroll_view_id' src/main.rs` returns zero hits.
- All persistent UI state lives on `self.ui.*`. No ad-hoc UI state fields on App.
- Running the game: sidebar tabs, scroll positions, modal dialogs, and animations all work as before.

---

## Phase 3 — mod.rs Decomposition (D3 + D16)

Goal: Split `src/ui/mod.rs` (8,597 lines) into ~15 focused files.
Each file has one concern. Mirrors sim layer's one-system-per-file pattern.
Also removes vestigial dirty tracking (dead code in rebuild-every-frame model).

Decisions implemented: D3-C (extract types + split methods), D4-A (keep closed enum), D16-A (one operation per file).

### P3S1 — Remove vestigial dirty tracking

In `src/ui/mod.rs`:

Remove from WidgetNode (line 273):
- `pub dirty: bool,`

Remove `mark_dirty` method (~line 549-562).

Remove all `dirty: true` assignments in `insert`, `insert_root`, `insert_root_with_tier`, `set_*` methods.

Remove `node.dirty = false` in layout pass (~line 865).

Update any tests that assert on `dirty` (test `dirty_propagation` at line 3874 — delete this test entirely, it tests dead functionality).

**Why:** The dirty flag is set on every insert (line 366) and cleared unconditionally in layout (line 865). Since the tree is rebuilt from scratch every frame, dirty tracking does no useful work. It creates false signals that the tree is "partially retained" when it is fully ephemeral.

- Checkpoint: `cargo check` succeeds. `grep -n 'dirty' src/ui/mod.rs` returns zero hits (excluding comments). `cargo test` passes (with `dirty_propagation` test deleted).

### P3S2 — Extract geometry types to geometry.rs

Create `src/ui/geometry.rs`. Move these types from `src/ui/mod.rs`:
- `Size` (lines 117-121)
- `Rect` (lines 123-153) + `contains`, `intersect` methods
- `Constraints` (lines 156-190) + `tight`, `loose`, `clamp` methods
- `Edges` (lines 192-223) + `all`, `horizontal`, `vertical` methods
- `Position` enum (lines 231-247) + Default impl
- `Sizing` enum (lines 249-257)

In `src/ui/mod.rs`:
- Add `mod geometry;`
- Add re-exports: `pub use geometry::{Size, Rect, Constraints, Edges, Position, Sizing};`
- Remove the moved type definitions.

Update imports in all files that use these types. They are re-exported from `ui::`, so external imports (`use crate::ui::Rect`) should continue to work. Internal imports within `src/ui/` may need updating from `super::Rect` to `super::geometry::Rect` or just `super::Rect` (if re-exported).

- Checkpoint: `cargo check` succeeds. `src/ui/geometry.rs` exists with all 6 types. `src/ui/mod.rs` is ~140 lines shorter.

### P3S3 — Extract WidgetNode and ZTier to node.rs

Create `src/ui/node.rs`. Move from `src/ui/mod.rs`:
- `WidgetNode` struct (lines 264-286) + UiPerfMetrics (lines 88-110)
- `ZTier` enum (lines 297-307)

In `src/ui/mod.rs`:
- Add `mod node;`
- Re-export: `pub use node::{WidgetNode, ZTier, UiPerfMetrics};`

- Checkpoint: `cargo check` succeeds. `src/ui/node.rs` exists. `src/ui/mod.rs` is ~65 lines shorter.

### P3S4 — Extract WidgetTree core to tree.rs

Create `src/ui/tree.rs`. Move the WidgetTree struct definition and core arena operations:
- `WidgetTree` struct (lines 314-322)
- `new()` (~line 325)
- `set_scroll_row_alt_alpha`, `set_control_border_width` (~lines 335-343)
- `insert_root`, `insert_root_with_tier`, `insert` (~lines 345-406)
- `default_padding` (~lines 407-430)
- `remove`, `collect_subtree` (~lines 432-468)
- `get`, `get_mut`, `node_rect` (~lines 469-482)
- `set_position`, `set_sizing`, `set_padding`, `set_margin`, `set_tooltip`, `set_constraints`, `set_on_click`, `set_clip_rect` (~lines 484-547)
- `roots`, `roots_draw_order`, `z_tier`, `z_tier_of_widget`, `set_z_tier` (~lines 565-640)
- `widget_count` (~line 350)

In `src/ui/mod.rs`:
- Add `mod tree;`
- Re-export: `pub use tree::WidgetTree;`
- Also add `pub(crate) use node::WidgetNode;` if not already exported.

The `WidgetTree` struct's `arena` field is private. Other `impl WidgetTree` blocks in sibling files within the `ui` module can still access it because they are in the same module. No visibility changes needed.

- Checkpoint: `cargo check` succeeds. `src/ui/tree.rs` contains the WidgetTree struct and all core arena methods (~290 lines).

### P3S5 — Extract tooltip helpers to tree_tooltip.rs

Create `src/ui/tree_tooltip.rs`. Move tooltip-related `impl WidgetTree` methods:
- `insert_tooltip_chrome` (~lines 644-667)
- `position_tooltip` (~lines 669-699)
- Any internal tooltip positioning helpers

```rust
// src/ui/tree_tooltip.rs
use super::tree::WidgetTree;
// ... imports ...

impl WidgetTree {
    pub fn insert_tooltip_chrome(&mut self, ...) -> ... { ... }
    pub fn position_tooltip(&mut self, ...) { ... }
}
```

Register in `src/ui/mod.rs`: `mod tree_tooltip;`

- Checkpoint: `cargo check` succeeds. `src/ui/tree_tooltip.rs` exists (~90 lines).

### P3S6 — Extract hit testing to tree_hit_test.rs

Create `src/ui/tree_hit_test.rs`. Move:
- `hit_test` (~line 701)
- `hit_test_node` (~line 711)
- `focusable_widgets` (~line 733)
- `focusable_widgets_in_tier` (~line 744)
- `collect_focusable` (~line 757)

Register in `src/ui/mod.rs`: `mod tree_hit_test;`

- Checkpoint: `cargo check` succeeds. `src/ui/tree_hit_test.rs` exists (~75 lines).

### P3S7 — Extract layout to tree_layout.rs

Create `src/ui/tree_layout.rs`. Move the layout pass — the largest single concern:
- `layout` (~line 776)
- `layout_node` (~line 792)
- `layout_node_children` (~line 875)
- `merge_clips` (~line 1365)
- `layout_scroll_item` (~line 1374)
- `child_extent` (~line 1405)
- `measure_node` (~line 1421)
- `measure_node_constrained` (~line 1431)
- All layout helper functions in the range ~776-1810.

Register in `src/ui/mod.rs`: `mod tree_layout;`

- Checkpoint: `cargo check` succeeds. `src/ui/tree_layout.rs` exists (~1,035 lines). This is the largest concern file, comparable to `systems/decisions.rs` (1,057 lines).

### P3S8 — Extract draw to tree_draw.rs

Create `src/ui/tree_draw.rs`. Move draw command generation:
- `draw` (~line 1812)
- `draw_with_measurer` (~line 1817)
- `draw_node` (~line 1833)
- All draw helper functions in the range ~1812-2555.

Register in `src/ui/mod.rs`: `mod tree_draw;`

- Checkpoint: `cargo check` succeeds. `src/ui/tree_draw.rs` exists (~750 lines).

### P3S9 — Extract scroll helpers to tree_scroll.rs

Create `src/ui/tree_scroll.rs`. Move:
- `scroll_item_y` (~line 2569)
- `scroll_item_h` (~line 2582)
- `scroll_total_height` (~line 2591)
- `scroll_first_visible` (~line 2604)
- `max_scroll` (~line 2627)
- `set_scroll_offset` (~line 2659)
- `scroll_offset` (~line 2674)
- `scroll_by` (~line 2686)
- `ensure_visible` (~line 2700)

Register in `src/ui/mod.rs`: `mod tree_scroll;`

- Checkpoint: `cargo check` succeeds. `src/ui/tree_scroll.rs` exists (~180 lines).

### P3S10 — Extract animation helpers to tree_anim.rs

Create `src/ui/tree_anim.rs`. Move:
- `set_subtree_opacity` (~line 2742)
- `apply_opacity` (~line 2753)
- `set_widget_bg_alpha` (~line 2855)

Register in `src/ui/mod.rs`: `mod tree_anim;`

- Checkpoint: `cargo check` succeeds. `src/ui/tree_anim.rs` exists (~130 lines).

### P3S11 — Extract free-standing builders to own files

Five builder functions currently live in mod.rs. Each becomes its own file, consistent with every other builder already having its own file (sidebar.rs, character_panel.rs, outliner.rs, etc.).

| Builder | Current Location | New File | ~Lines |
|---------|-----------------|----------|--------|
| `build_pause_overlay` | mod.rs ~line 2964 | `src/ui/pause_overlay.rs` | 30 |
| `build_status_bar` + `StatusBarInfo` | mod.rs ~line 2947 | `src/ui/status_bar.rs` | 200 |
| `build_hover_tooltip` + `HoverInfo` | mod.rs ~line 3096 | `src/ui/hover_tooltip.rs` | 170 |
| `build_event_log` + `EventLogEntry` | mod.rs ~line 3260 | `src/ui/event_log.rs` | 250 |
| `build_entity_inspector` + `EntityInspectorInfo` | mod.rs ~line 3470 | `src/ui/entity_inspector.rs` | 275 |

Each file also gets its associated info struct (e.g., `StatusBarInfo` moves with `build_status_bar`).

Register each in `src/ui/mod.rs` and add appropriate re-exports.

Update import sites in `src/main.rs` — these builders are called during the UI build phase.

- Checkpoint: `cargo check` succeeds. 5 new files exist. Each contains one builder function and its associated data struct. No builder functions remain in mod.rs.

### P3S12 — Split tests by concern

The test module in mod.rs (lines 3800-8597, ~4,800 lines) should split so that each test file accompanies the code it tests:

| Test Group | Target File | Approx Tests |
|------------|-------------|--------------|
| Tree insert/remove, z-tier | `tree.rs` | ~8 tests |
| Layout (row, column, constraints, clip, percent, expand, wrap) | `tree_layout.rs` | ~40 tests |
| Draw commands | `tree_draw.rs` | ~5 tests |
| Scroll (offset, clamping, visible, variable height) | `tree_scroll.rs` | ~12 tests |
| Opacity, bg alpha | `tree_anim.rs` | ~2 tests |
| Hit test, focusable widgets | `tree_hit_test.rs` | ~4 tests |
| Status bar | `status_bar.rs` | ~8 tests |
| Hover tooltip | `hover_tooltip.rs` | ~8 tests |
| Event log | `event_log.rs` | ~6 tests |
| Entity inspector | `entity_inspector.rs` | ~8 tests |
| Widget-specific (dropdown, checkbox, slider, etc.) | `tree_draw.rs` or keep in a `tests.rs` | ~15 tests |

Tests go into a `#[cfg(test)] mod tests { ... }` block at the bottom of each target file.

Shared test helpers (e.g., `fn screen() -> Size`, `fn spawn_full_entity`) go into a `src/ui/test_helpers.rs` file (with `#[cfg(test)]` on the module).

- Checkpoint: `cargo test` passes. No test module remains in `src/ui/mod.rs`.

### P3S13 — Verify mod.rs is a thin re-export hub

After all extractions, `src/ui/mod.rs` should contain only:
- Module declarations (`mod geometry; mod node; mod tree; ...`)
- Re-exports (`pub use geometry::Rect; pub use tree::WidgetTree; ...`)
- The `WidgetId` newtype key definition (from slotmap — `new_key_type! { pub struct WidgetId; }`)

Target: ~100-120 lines. No function bodies. No struct definitions beyond WidgetId.

- Checkpoint: `wc -l src/ui/mod.rs` returns ≤150 lines. No `fn ` appears in the file outside of slotmap macro expansion.

### P3 Verification

- `cargo build` succeeds.
- `cargo test` passes.
- `wc -l src/ui/mod.rs` ≤ 150 lines (down from 8,597).
- Every `src/ui/tree_*.rs` file contains exactly one concern (one `impl WidgetTree` block with related methods).
- Every builder file contains one `build_*` function and its associated data struct.
- `tokei src/ui/` shows no file exceeding ~1,100 lines (tree_layout.rs is the largest).
- `grep -n 'dirty' src/ui/` returns zero hits in non-test, non-comment lines.

---

## Phase Ordering Recommendations

The phases are independent but have a natural ordering that minimizes wasted work:

1. **Phase 1 first** (UiAction enum) — smallest diff, highest impact. Every subsequent phase benefits from typed callbacks. If Phase 2 runs after Phase 1, the scroll key type can use `PanelKind` from the start instead of strings.

2. **Phase 2 second** (UiContext) — medium diff. Uses `PanelKind` from Phase 1 for scroll keys. All main.rs field accesses change, so doing this before Phase 3 avoids changing the same lines twice.

3. **Phase 3 last** (mod.rs split) — largest diff but purely structural (no semantic changes). Easier to review after Phases 1-2 have stabilized the types.

If phases run in parallel (e.g., on separate branches), merge Phase 1 first, then Phase 2, then Phase 3, resolving conflicts at each merge.

---

## Post-Migration: CLAUDE.md Update

After all three phases are complete, add a UI architecture section to `CLAUDE.md` codifying the new patterns. Candidate rules:

- **UiContext** is the single struct for persistent UI state. Analogous to World. Sub-fields are pub for split borrows.
- **WidgetTree** is ephemeral — rebuilt every frame. Not persistent state.
- **Builders** are free functions: `fn build_*(tree: &mut WidgetTree, ...) → WidgetId`.
- **Callbacks** use `UiAction` enum. No string callbacks. Dynamic data-driven actions use `UiAction::EventChoice(String)` / `UiAction::ContextAction(String)`.
- **Panels** keyed by `PanelKind` enum. No string panel names.
- **One operation per file** for WidgetTree: `tree_layout.rs`, `tree_draw.rs`, etc.
- **One builder per file** for UI panels/screens.
- **No dirty tracking** on the ephemeral tree. State persistence is external (on UiContext).

These rules parallel the simulation layer rules, establishing consistency across the engine.
