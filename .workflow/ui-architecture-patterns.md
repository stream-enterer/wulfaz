# UI Architecture Pattern Decisions

> Decision map for Wulfaz UI layer architectural patterns.
> Generated via concentric planning: 4 passes, each sweeping all decisions.

## Constraints (Non-Negotiable)

From the simulation layer, carried forward for consistency:

- **Explicit over clever.** No magic, no registries, no indirection.
- **One concern per file.** Each file has one job.
- **State lives in known places.** Not scattered across modules.
- **Simple over flexible.** No over-engineering. No hypothetical future needs.
- **No traits as primary decomposition.** Modules are plain functions/structs, not polymorphic. Exception: a trait at a hardware boundary (e.g., `TextMeasurer` decoupling layout from font renderer) is acceptable. Traits as the decomposition axis between UI modules (Layout trait, Drawable trait) are not.

---

## Pass 1 — Enumerate

Every UI architectural decision point as a one-liner.

1. **State ownership** — Where does all UI state live? One struct? Several? On App?
2. **Widget tree lifecycle** — Rebuild every frame, or retain and diff?
3. **mod.rs decomposition** — How to split the 7.2k-line ui/mod.rs.
4. **Widget type set** — Closed enum, open trait, or something else?
5. **Callback dispatch** — How do widget interactions reach app logic?
6. **Layout model** — How widgets get sized and positioned.
7. **Input routing** — How raw events reach the right widget.
8. **Builder pattern** — How UI trees get constructed each frame.
9. **Theme/styling** — How visual properties are configured.
10. **Animation ownership** — Where animation state lives and who applies it.
11. **Draw command pipeline** — Intermediate representation between tree and GPU.
12. **Z-ordering** — How layering (panels, modals, tooltips) is controlled.
13. **Panel lifecycle** — How named panels are opened, closed, and tracked.
14. **Modal system** — How modals block input and stack.
15. **Scroll management** — How scroll state persists across frame rebuilds.
16. **File-to-concern mapping** — What constitutes "one concern" for UI files.

**Gap check: "What decision points are missing from this list?"**

- Accessibility (keyboard navigation, focus order) — already handled by UiState focus cycling. Not a separate decision.
- Text measurement / font integration — already handled by TextMeasurer trait in draw.rs. Stable, not a decision point.
- Tooltip system — subsumed by Z-ordering (#12), state ownership (#1), and animation (#10).
- Context menus — subsumed by panel lifecycle (#13) and Z-ordering (#12).

No missing decisions. Proceeding.

---

## Pass 2 — Shape

For each decision: what are the available patterns, what does each consume/produce, and what depends on what?

### D1. State Ownership

**Current:** Split across 6 locations on App — `UiState`, `Animator`, `ModalStack`, `PanelManager`, plus ad-hoc fields (`sidebar_active_tab`, `sidebar_scroll_offset`).

**Available patterns:**

- **A. Single UiContext struct with split borrows.** All UI state in one struct, passed by `&mut` to all UI functions. Analogous to how World is the single struct for simulation. To solve Rust borrow conflicts, UiContext exposes a `fn split(&mut self) -> (&Theme, &mut InputState, &mut ScrollMap, ...)` method or uses pub fields for field-level borrowing. Compile-time enforcement: if new state is needed, it must go in UiContext.
  - Inputs: all current state fields
  - Outputs: one organizational home, split borrows for usage
  - Depends on: D3 (mod.rs decomposition), D16 (file mapping)

- **B. Keep split, but formalize the set.** Current approach, but name and document the exact set of UI state structs. Each struct owns one domain (input, animation, panels, modals). App holds them all. Only advantage over A: lower migration cost. Disadvantage: enforcement is social (code review), not compile-time. Risk: ad-hoc fields creep back onto App.
  - Inputs: existing structs
  - Outputs: documented, bounded set of state locations
  - Depends on: D16 (what is "one concern")

- **C. State on the tree itself.** Widget tree nodes carry persistent state (scroll offsets, open/closed). No separate state structs.
  - Inputs: widget tree
  - Outputs: stateful tree
  - Depends on: D2 (tree lifecycle — only works if tree is retained)

**Excluded by constraints:**
- Global state / singletons — violates "state in known places"
- Component-attached state bags — registries, violates "no magic"

### D2. Widget Tree Lifecycle

**Current:** Full rebuild every frame. Tree is destroyed and reconstructed from scratch. UiState, Animator, PanelManager persist across frames.

**Available patterns:**

- **A. Rebuild every frame (current).** Stateless builders, deterministic output. Simple. No dirty tracking.
  - Inputs: World state, UI state structs
  - Outputs: fresh WidgetTree each frame
  - Depends on: D15 (scroll must persist outside tree)

- **B. Retained tree with explicit mutations.** Tree persists. Systems mutate it via WidgetId handles.
  - Inputs: stable WidgetIds
  - Outputs: long-lived tree, needs dirty/layout invalidation
  - Depends on: D1 (state can live on tree), D8 (builders become updaters)

- **C. Hybrid: retain structure, rebuild content.** Tree skeleton persists (panels, containers). Leaf content (labels, values) rebuilt each frame.
  - Inputs: stable container IDs + fresh content
  - Outputs: partial rebuild, partial persistence
  - Depends on: D1, D8

**Excluded by constraints:**
- Virtual DOM / diff-and-patch — indirection layer, violates "no magic"
- Reactive bindings / signals — indirection, violates "explicit"

### D3. mod.rs Decomposition

**Current:** ui/mod.rs is 7.2k lines. Contains WidgetTree (arena, layout, hit-test, draw, scroll, tooltip positioning), WidgetNode, geometry types, ZTier, perf metrics.

**Available patterns:**

- **A. Extract by concern within WidgetTree.** Split WidgetTree's methods into files by operation: `tree.rs` (core arena + insert/remove), `layout.rs` (measure + position), `hit_test.rs`, `tree_draw.rs` (draw command generation), `tooltip_layout.rs`.
  - Inputs: WidgetTree struct definition stays in one file
  - Outputs: impl blocks spread across files via `impl WidgetTree` in each
  - Depends on: D16

- **B. Extract types to dedicated files.** Move geometry types (Rect, Size, Constraints, Edges, Position, Sizing) to `geometry.rs`. Move WidgetNode + ZTier to `node.rs`. WidgetTree stays in mod.rs but shrinks.
  - Inputs: type definitions
  - Outputs: smaller mod.rs, more files
  - Depends on: nothing

- **C. Both A and B.** Types to their own files AND methods split by concern.
  - Inputs: everything
  - Outputs: mod.rs becomes a thin re-export hub
  - Depends on: D16

**Excluded by constraints:**
- Trait-based decomposition (Layout trait, HitTestable trait) — violates "no traits between modules"

### D4. Widget Type Set

**Current:** Closed enum (`Widget`) with 16+ variants. All widget types known at compile time.

**Available patterns:**

- **A. Closed enum (current).** Exhaustive match. No extensibility. Simple.
  - Inputs: known widget set
  - Outputs: compiler-enforced completeness
  - Depends on: nothing

- **B. Struct-per-widget with enum wrapper and inherent methods.** Each variant becomes its own struct (`ButtonWidget`, `LabelWidget`) in its own file. Enum wraps them: `Widget::Button(ButtonWidget)`. Each struct has inherent `fn measure()`, `fn draw()` methods (not trait methods — plain `impl ButtonWidget`). Central layout/draw matches on enum and delegates: `Widget::Button(b) => b.draw(...)`. This keeps the closed enum guarantee while moving per-widget logic out of the monolithic match arms.
  - Inputs: same data
  - Outputs: per-widget logic in per-widget files, enum still closed, compiler-enforced exhaustiveness
  - Depends on: nothing (D16 is an incentive, not a dependency)

**Excluded by constraints:**
- Trait objects (`Box<dyn Widget>`) — indirection, dynamic dispatch
- Entity-component widget model — registry pattern, violates "no magic"
- Open enum / plugin system — violates "explicit"

### D5. Callback Dispatch

**Current:** String keys. Widget gets `on_click: String`. App has centralized `dispatch_click(key: &str)` with string matching.

**Available patterns:**

- **A. String keys (current).** Simple. Greppable. Central dispatch.
  - Inputs: callback key strings
  - Outputs: one dispatch site in App
  - Depends on: nothing

- **B. Enum variants with payloads.** `UiAction::SelectTab(usize)`, `UiAction::PanelClose(PanelKind)`, `UiAction::EntitySelect(Entity)`, etc. Type-safe. Exhaustive match in dispatch. Compiler catches missing handlers. Payloads replace the current runtime string parsing (`"sidebar::tab::0".strip_prefix`). Strictly more explicit than string keys by the project's own "explicit" criterion — the connection between producer and consumer is compile-time verified.
  - Inputs: widget carries `Option<UiAction>` on WidgetNode (not on Widget enum)
  - Outputs: `match action { ... }` in App
  - Depends on: nothing (lives on WidgetNode, independent of D4)

- **C. (Entity, Action) pairs.** Like sim — callback carries entity + action type. Dispatch can route by entity.
  - Inputs: entity-scoped actions
  - Outputs: dispatch routes by entity then action
  - Depends on: nothing

**Excluded by constraints:**
- Closures / `Box<dyn Fn()>` — indirection, not greppable
- Event bus / pub-sub — message passing, violates sim principles
- Signal/slot — indirection

### D6. Layout Model

**Current:** Custom layout. Measure pass (intrinsic size) → layout pass (position children). Row/Column auto-layout. Fixed/Percent/Center positioning. Constraints (min/max). Padding/margin.

**Available patterns:**

- **A. Current custom layout.** Hand-written measure + layout for each widget kind. Works. Understood.
  - Inputs: widget tree + constraints
  - Outputs: computed rects
  - Depends on: D4 (layout code matches on Widget enum)

- **B. Flexbox subset.** Formalize the Row/Column model as a proper flexbox subset (main axis, cross axis, flex-grow, flex-shrink, align-items, justify-content).
  - Inputs: flex properties on containers
  - Outputs: same computed rects, but more expressive
  - Depends on: D4

**Excluded by constraints:**
- Constraint-based (Cassowary-style) — our layout problems are 1-dimensional (row or column). Constraint solvers add value for 2-dimensional relational layout, which we don't need. Impedance mismatch, not a complexity match.
- CSS-style cascading layout — magic inheritance, violates "explicit"
- Auto-layout with implicit rules — violates "explicit"

### D7. Input Routing

**Current:** UiState handles raw events → hit-test → update hovered/focused/pressed → App polls results (click_event, map_click). Early-return pattern: UI consumes first, game gets remainder.

**Available patterns:**

- **A. Hit-test + poll (current).** Input handlers set state, App polls it.
  - Inputs: raw winit events
  - Outputs: UiState fields (hovered, click_event, map_click)
  - Depends on: D1 (UiState location)

- **B. Return Option\<UiAction\>.** Input handler returns `Option<UiAction>` directly. `None` = pass-through to game. `Some(action)` = UI consumed the event. Simpler than poll (no stale state), simpler than two-phase (no collection). Natural Rust pattern — just a return value.
  - Inputs: raw winit events
  - Outputs: `Option<UiAction>`, handled immediately by caller
  - Depends on: D5 (if D5=B, action is typed; if D5=A, return is `Option<String>`)

- **C. Two-phase: route then dispatch.** Phase 1: hit-test determines target widget. Phase 2: widget-specific handler produces action. Analogous to sim's collect-then-apply.
  - Inputs: raw events
  - Outputs: collected actions, applied after
  - Depends on: D5

**Excluded by constraints:**
- Event bubbling / capture (DOM-style) — implicit propagation, violates "explicit"
- Observer pattern — indirection

### D8. Builder Pattern

**Current:** Free functions `build_*(tree: &mut WidgetTree, ...) → WidgetId`. Called every frame. Stateless.

**Available patterns:**

- **A. Free functions (current).** `build_sidebar(tree, state) → WidgetId`. One builder per panel/screen.
  - Inputs: tree + relevant state slices
  - Outputs: root WidgetId
  - Depends on: D2 (rebuild-every-frame makes this natural)

- **B. Builder structs.** `SidebarBuilder { state }.build(tree) → WidgetId`. Same thing but namespaced.
  - Inputs: state bundled in struct
  - Outputs: root WidgetId
  - Depends on: nothing

- **C. Thin macro_rules! helpers.** NOT a DSL. Just `macro_rules!` that expand 1:1 to existing `tree.insert()` + `tree.set_*()` calls, reducing per-widget boilerplate. Inspectable via `cargo expand`. Scope constraint: macro_rules! only, no proc macros, no nesting beyond one level, each macro maps to exactly one tree operation.
  - Inputs: macro invocations in builders
  - Outputs: tree insertions (identical to hand-written code)
  - Depends on: D4

**Excluded by constraints:**
- JSX-style / template DSL — indirection layer, opaque
- Proc macro DSL — opaque, hard to debug
- Reactive component model — violates "explicit"

### D9. Theme/Styling

**Current:** Single `Theme` struct with flat fields. Colors, sizes, durations. Passed explicitly to builders.

**Available patterns:**

- **A. Flat Theme struct (current).** One struct, all constants. Passed by reference.
  - Inputs: nothing (constants)
  - Outputs: consistent styling
  - Depends on: nothing

- **B. Theme struct with typed sub-sections.** `theme.button.bg_color`, `theme.panel.border_width`. Still flat, but grouped.
  - Inputs: nothing
  - Outputs: organized constants
  - Depends on: nothing

- **C. Per-widget style structs.** `ButtonStyle { bg, fg, border }` passed to each widget at construction.
  - Inputs: style struct per widget creation call
  - Outputs: per-widget visual control
  - Depends on: D4

**Note:** Regardless of A/B/C choice, the Theme should be `const`-constructible (`pub const GRUVBOX: Theme = Theme { ... }`). If the theme is genuinely immutable, this makes it a true compile-time constant with zero runtime cost. Orthogonal to the structural choice.

**Excluded by constraints:**
- CSS-style cascading / inheritance — magic, implicit
- Style sheets / external config — indirection layer
- Theme registry / provider pattern — registry

### D10. Animation Ownership

**Current:** `Animator` is a `HashMap<String, Animation>`. Lives on App. App queries `animator.get(key, now)` and manually applies values to widgets after layout.

**Available patterns:**

- **A. External animator, manual application (current).** Animation decoupled from tree. App code bridges them.
  - Inputs: animation key + wall-clock time
  - Outputs: interpolated f32, manually applied
  - Depends on: D1 (Animator lives somewhere known)

- **B. Animation targets on tree nodes.** Each node can have `Option<AnimationTarget>`. Layout/draw reads it automatically.
  - Inputs: animation params set on node
  - Outputs: automatic interpolation during layout/draw
  - Depends on: D2 (only sensible with retained tree)

- **C. Animation as a transform pass.** Separate pass between layout and draw that modifies positions/opacity. Like a sim phase.
  - Inputs: tree + active animations
  - Outputs: modified tree state
  - Depends on: D1

**Excluded by constraints:**
- Reactive animation bindings — magic
- CSS-style transitions — implicit

### D11. Draw Command Pipeline

**Current:** `DrawList` with `PanelCommand`, `TextCommand`, `RichTextCommand`, `SpriteCommand`. Tree walk generates commands. Renderer consumes them.

**Available patterns:**

- **A. Intermediate DrawList (current).** Tree → commands → renderer. Clean separation.
  - Inputs: laid-out tree
  - Outputs: DrawList
  - Depends on: D12 (commands ordered by Z-tier)

- **B. Direct rendering.** Tree walk calls renderer directly. No intermediate list.
  - Inputs: laid-out tree + renderer
  - Outputs: GPU calls
  - Depends on: nothing

- **C. Retained draw list with dirty regions.** Only regenerate commands for changed subtrees.
  - Inputs: diff info
  - Outputs: partial DrawList update
  - Depends on: D2 (requires retained tree)

**Excluded by constraints:**
- Scene graph abstraction — indirection layer
- Render tree separate from widget tree — two trees = complexity

### D12. Z-Ordering

**Current:** `ZTier` enum (Panel, Overlay, Modal, Tooltip). Roots assigned a tier. Draw order: tier first, insertion order second.

**Available patterns:**

- **A. Enum tiers (current).** Fixed number of layers. Simple. Predictable.
  - Inputs: tier assignment on roots
  - Outputs: draw order
  - Depends on: D14 (modals use Modal tier)

- **B. Numeric z-index with named constants.** Each root gets an integer. `const Z_PANEL: i32 = 0; const Z_MODAL: i32 = 200;` etc. Handles within-tier ordering naturally (no separate PanelManager.raise()). But: loses exhaustive match — adding a new tier is not compiler-enforced. The enum's advantage is that Rust's `match` catches missed tiers at compile time, which is the stronger "explicit" argument.
  - Inputs: z-index per root
  - Outputs: arbitrary ordering, no separate raise mechanism needed
  - Depends on: nothing

**Excluded by constraints:**
- Automatic z-ordering by focus/interaction — implicit, magic
- Stacking contexts (CSS-style) — complexity, implicit

### D13. Panel Lifecycle

**Current:** `PanelManager` tracks open panels by name string. Scroll offsets persist by name. Animated close via deadline.

**Available patterns:**

- **A. Name-keyed manager (current).** `HashMap<String, PanelEntry>`. Open/close/raise by name.
  - Inputs: panel name strings
  - Outputs: open set, draw order, scroll persistence
  - Depends on: D1 (PanelManager lives somewhere known), D5 (callbacks reference panel names)

- **B. Enum-keyed manager.** `HashMap<PanelKind, PanelEntry>`. Type-safe, exhaustive.
  - Inputs: PanelKind enum
  - Outputs: same as A but with compiler checks
  - Depends on: nothing

- **C. No manager — panels are just roots.** WidgetTree tracks its own roots. Open/close = insert/remove root. Scroll persistence lives in UiState or a dedicated map.
  - Inputs: WidgetIds
  - Outputs: tree manages itself
  - Depends on: D1 (scroll persistence location), D2

**Excluded by constraints:**
- Panel registry with dynamic registration — registry pattern
- Plugin-style panel loading — indirection

### D14. Modal System

**Current:** `ModalStack` with dim layers. Push creates dim + content at Modal tier. Focus scoped to Modal tier.

**Available patterns:**

- **A. Explicit modal stack (current).** Vec of modal entries. Push/pop. Dim layer is a widget.
  - Inputs: content root + options
  - Outputs: modal with dim + focus scoping
  - Depends on: D12 (Modal Z-tier), D7 (focus scoping in input)

- **B. Modal as a panel at Modal tier.** No separate stack. PanelManager handles modals as panels with tier=Modal. Focus scoping derived from "any Modal-tier panel open?"
  - Inputs: panel with tier
  - Outputs: simpler — one system handles both panels and modals
  - Depends on: D12, D13

**Excluded by constraints:**
- Modal service / promise-based — indirection, async
- Event-driven modal resolution — message passing

### D15. Scroll Management

**Current:** PanelManager persists scroll offsets by panel name. ScrollList/ScrollView widgets carry scroll_offset but it's rebuilt each frame. Offset restored from PanelManager during build.

**Available patterns:**

- **A. External persistence by key (current).** Scroll offsets stored in PanelManager (or similar) by name. Restored during build.
  - Inputs: panel name → offset mapping
  - Outputs: scroll position survives rebuild
  - Depends on: D2 (needed because tree rebuilds), D13

- **B. Scroll state in UiState.** Dedicated `HashMap<String, f32>` on UiState (or UiContext). Decoupled from PanelManager.
  - Inputs: scroll key → offset
  - Outputs: scroll survives rebuild, independent of panel concept
  - Depends on: D1

- **C. Retained scroll widgets.** If tree is retained (D2-B), scroll offset lives on the widget node itself. No external persistence needed.
  - Inputs: nothing extra
  - Outputs: natural persistence
  - Depends on: D2 (retained tree only)

**Excluded by constraints:**
- Scroll state synced via events — message passing
- Virtual scroll with lazy loading — over-engineering for current scale

### D16. File-to-Concern Mapping

**Current:** Loosely followed. Some files are one concern (sidebar.rs = sidebar builder). mod.rs bundles many concerns (tree, layout, hit-test, draw, geometry, tooltips).

**Available patterns:**

- **A. One struct/system per file.** Like sim systems. `widget_tree.rs` owns WidgetTree. `layout.rs` owns layout logic. `hit_test.rs` owns hit testing.
  - Inputs: defined concern boundaries
  - Outputs: small, focused files
  - Depends on: D3

- **B. One builder per file (current for panels).** Each screen/panel builder is its own file. Infrastructure (tree, layout) can be multi-concern per file.
  - Inputs: natural panel boundaries
  - Outputs: panel files are clean, infrastructure files are large
  - Depends on: nothing

- **C. Strict: one public function or struct per file.** Maximum granularity.
  - Inputs: every concern split
  - Outputs: many small files, potential navigation overhead
  - Depends on: nothing

**Excluded by constraints:**
- Grouping by "layer" (all views together, all models together) — doesn't match sim pattern
- Feature folders — over-engineering at this scale

---

**Gap check: "Are there orphan inputs no decision produces, or dead outputs nothing consumes?"**

- D2 (tree lifecycle) is consumed by D1, D8, D10, D11, D15. Central dependency — correct.
- D1 (state ownership) is consumed by D7, D10, D13, D15. Central dependency — correct.
- D16 (file mapping) is consumed by D3, D4. Correct.
- D5 (callbacks) is consumed by D7, D13. Correct.
- D12 (Z-ordering) is consumed by D11, D14. Correct.
- No orphans. No dead outputs.

**Dependency graph** (A → B means "A depends on B"):

```
D3 (mod.rs decomp) → D16 (file mapping)
D1 (state) → D16 (file mapping)
D7 (input) → D1 (state), D5 (callbacks)
D10 (animation) → D1 (state), D2 (tree lifecycle)
D13 (panels) → D1 (state), D5 (callbacks)
D15 (scroll) → D1 (state), D2 (tree lifecycle)
D11 (draw pipeline) → D12 (Z-ordering)
D14 (modals) → D12 (Z-ordering), D13 (panels)
D6 (layout) → D2 (tree lifecycle), D4 (widget set)
D8 (builders) → D1 (state), D4 (widget set)
D9 (theme) → D4 (widget set)
```

Foundation decisions (no dependencies): D2, D4, D5, D12, D16.

---

## Pass 3 — Specify

For each decision: edge cases, validation, error states, and integration seams.

### D1. State Ownership

**Option A (single UiContext)** edge cases:
- Borrow conflicts: if builder needs `&UiContext` for theme while also needing `&mut UiContext` for scroll offsets, you hit Rust borrow checker issues. Must split borrows carefully or use interior fields.
- Migration: moving 6 current locations into one struct is a large refactor touching every file that references any UI state.
- Integration seam: App holds `UiContext`. All `build_*` functions take `&mut UiContext` instead of individual state pieces.
- Error state: none — this is structural, not runtime.
- Validation: compile-time — if something is missing from UiContext, it won't compile.

**Option B (formalize split)** edge cases:
- Risk of ad-hoc fields creeping back onto App (current problem).
- Fix: document the closed set. Any new UI state must go into one of the named structs.
- Integration seam: App holds N named structs. Functions take the specific struct(s) they need.
- Validation: code review / linting — no compile-time enforcement.

**Option C (state on tree)** edge cases:
- Only viable with D2-B (retained tree). If tree rebuilds every frame, state is lost.
- Dependent on D2 decision. If D2 stays at A, this option is eliminated.

### D2. Widget Tree Lifecycle

**Option A (rebuild every frame)** edge cases:
- Performance ceiling: as widget count grows, rebuild cost grows linearly. Currently fine (measured via UiPerfMetrics).
- State loss: any per-widget state must be persisted externally (scroll, focus, selection).
- WidgetId instability: IDs change every frame. UiState.focused becomes stale after rebuild. Must re-resolve focus by widget identity (callback key or position).
- Integration seam: after rebuild, UiState must reconcile stale IDs. Currently handled by hit-test on next input.

**Option B (retained tree)** edge cases:
- Complexity explosion: dirty tracking, partial updates, stale widgets, lifecycle events.
- Rust ownership: long-lived mutable tree with cross-references is hard without unsafe or Rc.
- Builder pattern breaks: can't just call `build_sidebar()` — must diff against existing.
- Integration seam: every state change must trigger targeted tree mutation.

**Option C (hybrid)** edge cases:
- Two classes of widgets: persistent containers and ephemeral content. Must be clear which is which.
- Same complexity as B but only for the container skeleton.
- Diminishing returns: if most widgets are content, you're rebuilding most of the tree anyway.

### D3. mod.rs Decomposition

**Option A (split methods by concern)** edge cases:
- Rust allows `impl WidgetTree` blocks in multiple files (same crate). Clean.
- Risk: method discoverability — developer must know which file holds which methods.
- Fix: consistent naming. `tree_layout.rs` for layout methods. `tree_hit_test.rs` for hit-test.

**Option B (extract types)** edge cases:
- Geometry types (Rect, Size, etc.) are used everywhere. Moving them means updating imports in every UI file.
- But: this is a one-time mechanical refactor.

**Option C (both)** edge cases:
- mod.rs becomes ~100 lines of re-exports + WidgetTree struct definition.
- Maximum clarity but most files to navigate.

### D4. Widget Type Set

**Option A (closed enum)** edge cases:
- Adding a new widget requires touching enum definition + every match arm. With 16+ variants, this is already a tax.
- But: compiler catches missed arms. Strong guarantee.

**Option B (struct-per-widget + enum wrapper)** edge cases:
- Each variant is `Widget::Button(ButtonWidget)` where `ButtonWidget` is in `widgets/button.rs`.
- Layout/draw still match on enum — same guarantee.
- Per-widget file can hold widget-specific helpers (e.g., ButtonWidget::default_style).
- Risk: more files. But consistent with "one concern per file."

### D5. Callback Dispatch

**Option A (strings)** edge cases:
- Typos not caught at compile time. `"sidebar::tab::0"` vs `"sidebar:tab:0"`.
- Refactoring: rename a callback key → must grep all builders + dispatch.
- But: greppable. Simple. No type machinery.

**Option B (enum)** edge cases:
- Type-safe. Exhaustive match in dispatch. Compiler catches missing handlers.
- But: enum grows with every new interactive widget. Central definition required.
- Integration seam: Widget must carry `Option<UiAction>`. UiAction enum defined in ui module.

**Option C (entity+action)** edge cases:
- Only useful if callbacks are entity-scoped. Many UI actions aren't (toggle sidebar, open settings).
- Over-fitting to sim patterns where everything is entity-scoped.

### D6. Layout Model

**Option A (current custom)** edge cases:
- Each new widget requires manual layout code. But: widget set is closed, so this is bounded.
- No formal spec to test against. Bugs are discovered visually.

**Option B (flexbox subset)** edge cases:
- More expressive (flex-grow, justify-content). Handles dynamic sizing better.
- Risk: partial flexbox is confusing. Developers expect full flexbox if you name it that.
- Fix: don't call it flexbox. Call it "row/column layout with flex properties."

**Option C (constraint solver)** edge cases:
- Powerful but opaque. Debugging layout = debugging solver output.
- External dependency (cassowary crate) or significant implementation.
- Over-engineering for current needs (panels, lists, buttons).

### D7. Input Routing

**Option A (current poll)** edge cases:
- Polling means actions are delayed one frame (set in handle_input, consumed in update).
- Multiple clicks in one frame: only last one stored. Currently fine at 60fps.

**Option B (immediate return)** edge cases:
- Cleaner flow: event → action → immediately handled. No stale state.
- But: input handler must know about all possible actions (tighter coupling).

**Option C (two-phase)** edge cases:
- Mirrors sim's collect-then-apply. Consistent philosophy.
- Overhead of collecting into Vec for typically 0-1 actions per frame.
- But: principle matters more than overhead at this scale.

### D8. Builder Pattern

**Option A (free functions)** edge cases:
- Argument lists grow as builders need more state. `build_sidebar(tree, world, theme, animator, panel_mgr, ...)`.
- Fix: bundle state into fewer arguments (see D1).

**Option B (builder structs)** edge cases:
- More boilerplate. `SidebarBuilder { theme, world, ... }.build(tree)`.
- Marginal benefit over functions unless builders have configuration.

**Option C (declarative macros)** edge cases:
- Macros are hard to debug. Error messages are cryptic.
- Violates "explicit" if the macro hides widget tree insertions.
- But: if macro is thin (1:1 mapping to tree.insert calls), it's just syntax sugar.
- Risk: macro = magic in disguise. Proceed only if gain is significant.

### D9. Theme/Styling

**Option A (flat struct)** edge cases:
- Dozens of fields. Finding the right one requires knowing the naming convention.
- But: simple. All in one place. Greppable.

**Option B (grouped sub-sections)** edge cases:
- `theme.button.bg` is more discoverable than `theme.button_bg`.
- But: adds nested structs. Mild complexity increase.
- Integration seam: sub-structs can be borrowed independently (avoids borrow conflicts).

**Option C (per-widget styles)** edge cases:
- Every widget construction call needs a style argument. Verbose.
- But: maximum explicitness. No implicit styling.
- Risk: repetitive. Most buttons use the same style.

### D10. Animation Ownership

**Option A (external, manual)** edge cases:
- App code must bridge animator and tree. Every animation requires explicit application code.
- But: fully visible. No hidden animation behavior. Consistent with sim's "systems are explicit functions."

**Option B (on tree nodes)** edge cases:
- Animation happens implicitly during layout/draw. Less code in App.
- But: "magic" — widget moves without explicit code saying "move this widget."
- Violates "explicit." Rejected by constraints.

**Option C (transform pass)** edge cases:
- Explicit pass: `apply_animations(tree, animator, now)`. Called in main loop like a sim phase.
- Visible in main loop. Consistent with phase ordering.
- But: still needs per-animation application logic inside the pass.

### D11. Draw Command Pipeline

**Option A (intermediate DrawList)** edge cases:
- Extra allocation per frame. But: bounded and cheap.
- Clean separation: tree doesn't know about GPU. Renderer doesn't know about widgets.

**Option B (direct rendering)** edge cases:
- Fewer allocations. But: tree code now coupled to renderer API.
- Can't inspect draw commands for debugging.

**Option C (retained draw list)** edge cases:
- Only useful with D2-B (retained tree). Otherwise, entire list changes every frame anyway.

### D12. Z-Ordering

**Option A (enum tiers)** edge cases:
- Fixed 4 tiers. If you need 5, you edit the enum. Easy.
- But: can't order within a tier except by insertion order.
- Current PanelManager.raise() handles within-tier ordering. Works.

**Option B (numeric z-index)** edge cases:
- Arbitrary ordering. But: z-index conflicts, no semantic meaning.
- "What z-index is a tooltip?" vs "Tooltip tier." Former is worse.

### D13. Panel Lifecycle

**Option A (name strings)** edge cases:
- Same typo risk as D5-A. But: panel names are fewer and more stable.
- Scroll persistence naturally keyed by name.

**Option B (enum keys)** edge cases:
- Type-safe. But: enum grows with every panel type.
- Integration: PanelKind enum in ui module.

**Option C (no manager)** edge cases:
- Scroll persistence must live elsewhere. Open/close tracking must be inferred from tree state.
- Less explicit — you can't ask "is the sidebar open?" without scanning roots.

### D14. Modal System

**Option A (explicit stack)** edge cases:
- Nested modals work (stack depth > 1). Dim layers compose.
- Focus scoping is explicit: stack non-empty → scope to Modal tier.

**Option B (modals as panels)** edge cases:
- Simpler: one system for panels and modals.
- Stack semantics preserved via PanelManager's existing `draw_order: Vec<String>` — insertion order IS stack order. Close-topmost = pop last Modal-tier entry.
- Focus scoping: derive from "any root at Modal tier" instead of explicit stack check. Slightly less direct but mechanically equivalent.
- Real cost: dim layer management moves from ModalStack into PanelManager (or becomes a widget convention).

### D15. Scroll Management

**Option A (external by key)** edge cases:
- Keys must be unique and stable across frames. Panel names work for this.
- Non-panel scrollables (e.g., inline scroll in a modal) need their own key scheme.

**Option B (on UiState)** edge cases:
- Decoupled from PanelManager. Any scrollable gets a key.
- But: another HashMap to maintain. Stale keys accumulate unless GC'd.

**Option C (retained widgets)** edge cases:
- Cleanest but requires D2-B. Eliminated if D2 stays at A.

### D16. File-to-Concern Mapping

**Option A (one struct/system per file)** edge cases:
- WidgetTree is one struct but has many concerns (layout, hit-test, draw). Split into impl-block files.
- Concern: "where is method X?" → answer via naming convention.

**Option B (one builder per file, infrastructure flexible)** edge cases:
- Panel files are clean. Infrastructure files grow without bound.
- Current pain point: mod.rs is infrastructure that grew.

**Option C (strict: one function/struct per file)** edge cases:
- May produce files with 20-50 lines that feel unnecessary.
- Navigation overhead with many tiny files.

---

**Gap check: "What error states are unhandled? What integration seams are undefined?"**

- UiState.focused becoming stale after tree rebuild (D2-A) is acknowledged. Current reconciliation via hit-test works but is implicit. Should be documented.
- Borrow conflicts with single UiContext (D1-A) are a real Rust constraint. Must be evaluated before committing.
- No unhandled error states beyond what's noted. Integration seams are defined at the App boundary.

---

## Pass 4 — Refine

Cross-check decisions for conflicts, gaps, redundancies, and forced choices.

### Forced Choices

1. **D2-A forces D15-A or D15-B.** If tree rebuilds every frame, scroll state cannot live on the tree. D15-C is eliminated.
2. **D2-A forces D1-A or D1-B.** State cannot live on tree nodes. D1-C is eliminated.
3. **D2-A eliminates D10-B and D11-C.** Animation on nodes and retained draw lists need a retained tree.
4. **D16 constrains D3.** Whatever "one concern" means determines how mod.rs splits.

### Conflicts

1. **D1-A (single UiContext) vs D8-A (free functions with many args).** If all state is in one struct, builders take `&mut UiContext`. But if a builder only needs theme (immutable) and tree (mutable), a single `&mut UiContext` forces exclusive access to everything. Resolution: D1-A needs careful field-level borrowing or split-borrow methods. D1-B avoids this naturally.

2. **D5-B (enum callbacks) vs D13-A (string-keyed panels).** If callbacks become typed but panels stay string-keyed, there's inconsistency. Resolution: either both use strings or both use enums.

3. **D4-B (struct-per-widget) vs D16-C (strict one-per-file).** 16+ widget files is many, but consistent. Not a conflict — just a scale concern.

### Redundancies

1. **D13 (panel lifecycle) vs D14 (modal system).** Both manage "things that appear and disappear." If D14-B (modals as panels) is chosen, D14 merges into D13. This is a simplification opportunity.

2. **D5 (callbacks) and D7 (input routing)** overlap at the "what happens when user clicks" boundary. The callback type (D5) constrains the input return type (D7). These should be decided together.

### Gaps

1. **No decision for focus persistence.** When tree rebuilds (D2-A), focused WidgetId becomes stale. Need a decision: re-resolve by callback key? By position? By widget type + index? Currently implicit — should be explicit.
   - **Added: D17. Focus Reconciliation.**

2. **No decision for widget identity across frames.** Animations target widgets by string key, not WidgetId. Scroll offsets persist by panel name. Focus needs a similar identity scheme. This is a cross-cutting concern.
   - Subsumed into D17.

### D17. Focus Reconciliation (added)

**Current:** Focus is a WidgetId. After tree rebuild, old WidgetId is invalid. Next input event triggers hit-test which overwrites stale focus.

**Available patterns:**

- **A. Reconcile by callback key.** After rebuild, scan tree for widget with same `on_click` key as previously focused widget. Restore focus.
  - Inputs: previous focus callback key
  - Outputs: restored focus
  - Depends on: D5

- **B. Reconcile by path.** Store focus as a "path" (e.g., sidebar → tab 2 → scroll list → item 5). After rebuild, walk path to find equivalent widget.
  - Inputs: path description
  - Outputs: restored focus
  - Depends on: D2 (only needed with rebuild-every-frame)

- **C. Accept staleness.** Focus resets each frame unless input event restores it. Current behavior.
  - Inputs: nothing
  - Outputs: focus may flicker or jump on rebuild
  - Depends on: nothing

**Excluded by constraints:**
- Stable widget IDs across frames (generational keys) — requires retained tree or ID generation scheme that is essentially a registry.

### Resolution Summary

**Hard eliminations** (by constraints alone):
- D10-B eliminated (implicit animation, violates "explicit")
- D6-C eliminated (1D layout problems don't need a 2D constraint solver — impedance mismatch)

**Conditional eliminations** (eliminated IF D2 = A, i.e., rebuild every frame):
- D1-C eliminated (state on tree requires retained tree)
- D11-C eliminated (retained draw list requires retained tree)
- D15-C eliminated (scroll on widgets requires retained tree)

D2 is the most consequential undecided decision. It is currently A (rebuild every frame) and working. The infrastructure partially supports B (SlotMap arena, dirty flags on nodes). The real barrier to B is that all builders are written as "construct from scratch" functions — converting them is significant work, not a constraint violation. This document does not pre-decide D2.

**Natural pairings** (should be decided together):
- D5 + D7 + D13: callback type, input routing, panel keys — all share the string-vs-enum axis.
- D1 + D15: state ownership and scroll persistence — scroll is a state ownership question.
- D3 + D16: mod.rs decomposition and file mapping — same question at two scales.
- D13 + D14: panel lifecycle and modal system — potential merge.

**Remaining viable option sets per decision:**

| Decision | Remaining Options |
|----------|-------------------|
| D1. State ownership | A (single struct + split borrows) · B (formalized split) |
| D2. Tree lifecycle | A (rebuild/frame) · B (retained) · C (hybrid) |
| D3. mod.rs decomposition | A (methods by concern) · B (types to files) · C (both) |
| D4. Widget type set | A (closed enum) · B (struct-per-widget + enum + inherent methods) |
| D5. Callback dispatch | A (strings) · B (enum with payloads) |
| D6. Layout model | A (current custom) · B (flexbox subset) |
| D7. Input routing | A (poll) · B (return Option\<UiAction\>) · C (two-phase) |
| D8. Builder pattern | A (free functions) · B (builder structs) · C (thin macro_rules!) |
| D9. Theme/styling | A (flat) · B (grouped) · C (per-widget) |
| D10. Animation ownership | A (external manual) · C (transform pass) |
| D11. Draw pipeline | A (intermediate) · B (direct) |
| D12. Z-ordering | A (enum tiers) · B (numeric + named constants) |
| D13. Panel lifecycle | A (string keys) · B (enum keys) |
| D14. Modal system | A (explicit stack) · B (modals as panels) |
| D15. Scroll management | A (external by key) · B (on UiState) |
| D16. File mapping | A (one struct per file) · B (builder per file) · C (strict) |
| D17. Focus reconciliation | A (by callback key) · B (by path) · C (accept staleness) |

---

**Gap check: "Could a developer implement any decision in isolation without ambiguity?"**

Yes, given the dependency graph and forced choices documented above. The natural pairings should be decided as groups, but each decision's options are self-contained and implementable.

---

## Pass 5 — Proving Ground

Adversarial evaluation via parallel agent debate (Adaptive Sandbox Fan-Out + Opponent Processor pattern). Each decision cluster was evaluated by an independent agent that read the actual codebase, scored options against the constraints, and recommended with confidence intervals.

### Cluster 1: D2 — Tree Lifecycle

| Option | Fit | Migration | Key Risk | Confidence |
|--------|-----|-----------|----------|------------|
| **A: Rebuild/frame** | **78** | Low | State scatter proliferation | **90%** |
| B: Retained | 35 | High | Zombie UI state / stale nodes | 85% |
| C: Hybrid | 52 | Medium | Category collapse (persistent vs ephemeral blurs) | 75% |

**Verdict: D2-A (rebuild every frame).** Option A's weakness (state scatter) is organizational, not architectural — it is fixed by D1 (state ownership), not by changing the tree lifecycle. Option B violates "explicit" (dirty tracking, invalidation) and "simple" (lifecycle management). Option C is strictly harder to reason about than either A or B alone.

Key finding: the `dirty: bool` on WidgetNode and `mark_dirty()` are vestigial — they do no useful work in the rebuild-every-frame model. Should be removed to eliminate false signals.

**Philosophical alignment:** Builders are pure functions of UI state. The tree is derived, not accumulated. This matches the sim layer where systems are pure functions of World state.

### Cluster 2: D5 + D7 + D13 — String vs Enum Axis

| Config | Fit | Migration | Key Risk | Confidence |
|--------|-----|-----------|----------|------------|
| All strings (current) | 55 | None | Silent mismatches (18 unhandled callbacks found) | 85% |
| **All enums** | **82** | Medium | Enum growth | **80%** |
| Mixed | 60 | Medium | Inconsistency at seam | 75% |

**Verdict: All enums (D5-B + D7-B + D13-B).**

Enums are strictly more explicit than strings in Rust — this is a language-mechanical fact, not a preference. The compiler enforces exhaustive matching on enums. It does not enforce it on strings.

Key findings from codebase analysis:
- 22 distinct callback strings exist across 12 files.
- 6 carry runtime-parsed payloads (`"sidebar::tab::0"` → `starts_with` + `parse`).
- 18 callback strings have NO handler in `dispatch_click` — the `_ =>` arm silently swallows them. This is a live bug invisible to the compiler.
- The code already trends toward enums: `MODAL_DISMISS` constants, `MouseButton` enum, `ZTier` enum.

Dynamic data-driven callbacks (`event_choice`, `context_menu`) stay as `UiAction::EventChoice(String)` — an honest boundary between static and dynamic.

Migration is compiler-guided: change the type, fix every error. ~1-2 hours.

### Cluster 3: D1 + D15 — State Ownership + Scroll

| Config | Fit | Borrow Ergonomics | Migration | Key Risk | Confidence |
|--------|-----|-------------------|-----------|----------|------------|
| **D1-A + D15-B** | **82** | Good (World-proven) | Medium | God-struct growth | **75%** |
| D1-B + D15-A | 60 | Best | Low | State scatter recurrence + scroll coupling | 85% |
| D1-B + D15-B | 70 | Good | Low-Med | Argument-list bloat + scatter recurrence | 80% |

**Verdict: D1-A + D15-B (single UiContext + decoupled scroll).**

World already proves the pattern: one struct with pub sub-struct groupings for split borrows (`world.body`, `world.mind`, `world.gis`). UiContext mirrors this:

```rust
pub struct UiContext {
    pub input: UiState,
    pub animator: Animator,
    pub modals: ModalStack,
    pub panels: PanelManager,
    pub scroll: HashMap<ScrollKey, f32>,
    pub sidebar: SidebarState,
}
```

Compile-time enforcement: new UI state must go into an existing sub-struct or justify a new one. The ad-hoc `sidebar_active_tab` / `sidebar_scroll_offset` problem cannot recur.

Scroll decoupled from PanelManager is unconditionally correct: sidebar scrollables and modal-internal scrollables are not panels.

### Cluster 4: D3 + D16 + D4 — File Organization

| Config | Fit | Sim Consistency | Navigation | Migration | Confidence |
|--------|-----|-----------------|------------|-----------|------------|
| Minimal extraction | 35 | Low | High | Low | 85% |
| **Method-split** | **75** | **High** | **Low** | Medium | **80%** |
| Full decomposition | 55 | Moderate (misleading) | High | High | 70% |

**Verdict: Method-split (D3-C + D4-A + D16-A).**

Key insight: **"one concern" in the UI layer means one operation, not one widget type.** This mirrors the sim layer exactly — `hunger.rs` contains `run_hunger(world)`, not `Hunger::tick()`. Likewise, `tree_draw.rs` contains the draw pass for all widgets, not `ButtonWidget::draw()`.

The decomposition axis is the operation (layout, draw, hit-test), and WidgetTree is the UI's World. Widget types are data variants (components), not decomposition units.

Proposed file plan:

| File | ~Lines | Concern |
|------|--------|---------|
| `mod.rs` | 100 | Re-exports |
| `geometry.rs` | 180 | Size, Rect, Constraints, Edges |
| `node.rs` | 45 | WidgetNode, ZTier |
| `tree.rs` | 290 | WidgetTree struct + arena ops |
| `tree_layout.rs` | 1,035 | layout(), measure_node() |
| `tree_draw.rs` | 750 | draw(), draw_node() |
| `tree_hit_test.rs` | 75 | hit_test(), focusable_widgets() |
| `tree_scroll.rs` | 180 | scroll helpers |
| `tree_tooltip.rs` | 90 | tooltip positioning |
| `tree_anim.rs` | 130 | opacity/alpha helpers |
| `status_bar.rs` | 200 | build_status_bar |
| `pause_overlay.rs` | 30 | build_pause_overlay |
| `hover_tooltip.rs` | 170 | build_hover_tooltip |
| `event_log.rs` | 250 | build_event_log |
| `entity_inspector.rs` | 275 | build_entity_inspector |

The 5 free-standing builders in mod.rs move to their own files. Tests split to accompany the file they test. mod.rs drops from 8,597 lines to ~100.

### Cluster 5: Independent Decisions

| Decision | Recommendation | Fit | Key Reason | Confidence |
|----------|---------------|-----|------------|------------|
| D6. Layout | **A (current custom)** | 85 | Already IS the project's flexbox subset under its own names; renaming adds nothing | 90% |
| D8. Builders | **A (free functions)** | 80 | Mirrors sim `run_*` convention; argument-list concern solved by D1, not by builder structs | 92% |
| D9. Theme | **A (flat struct)** | 82 | Fields already grouped by prefix convention; const-constructible | 80% |
| D10. Animation | **A (external manual)** | 88 | Fully explicit — animation effect visible at call site, not hidden in a pass | 95% |
| D11. Draw | **A (intermediate DrawList)** | — | Not contested — clean separation, debuggable | — |
| D12. Z-ordering | **A (enum tiers)** | — | Exhaustive match catches missed tiers at compile time | — |
| D14. Modals | **B (modals as panels)** | 75 | Eliminates redundant management layer; draw_order already provides stack semantics | 72% |
| D17. Focus | **C (accept staleness)** | 78 | At 60fps, focus re-established on next input event — no visible problem to solve | 85% |

---

## Final Decision Map

| # | Decision | Winner | Confidence | Migration |
|---|----------|--------|------------|-----------|
| D1 | State ownership | **Single UiContext + split borrows** | 75% | Medium |
| D2 | Tree lifecycle | **Rebuild every frame** | 90% | None (current) |
| D3 | mod.rs decomposition | **Extract types + split methods by concern** | 80% | Medium |
| D4 | Widget type set | **Closed enum (current)** | 80% | None |
| D5 | Callback dispatch | **Enum with payloads (UiAction)** | 80% | Medium |
| D6 | Layout model | **Current custom** | 90% | None |
| D7 | Input routing | **Return Option\<UiAction\>** | 80% | Medium (with D5) |
| D8 | Builder pattern | **Free functions** | 92% | None |
| D9 | Theme/styling | **Flat struct (const-constructible)** | 80% | None |
| D10 | Animation ownership | **External manual** | 95% | None |
| D11 | Draw pipeline | **Intermediate DrawList** | — | None |
| D12 | Z-ordering | **Enum tiers** | — | None |
| D13 | Panel lifecycle | **Enum keys (PanelKind)** | 80% | Medium (with D5) |
| D14 | Modal system | **Modals as panels** | 72% | Low |
| D15 | Scroll management | **Decoupled on UiContext** | 75% | Low |
| D16 | File mapping | **One struct/operation per file** | 80% | Medium (with D3) |
| D17 | Focus reconciliation | **Accept staleness** | 85% | None |

**Summary:** Of 17 decisions, 9 are "keep current" (no migration). 8 require changes, but they cluster into 3 migrations that can be done independently:

1. **UiAction enum migration** (D5 + D7 + D13): Define `UiAction` + `PanelKind` enums, compiler-guided refactor. ~2 hours.
2. **UiContext consolidation** (D1 + D15): Bundle state into one struct with sub-fields, decouple scroll. ~3 hours.
3. **mod.rs decomposition** (D3 + D16): Split 8.6k-line mod.rs into ~15 focused files. ~4 hours.

These are independent and can be done in any order. Total estimated mechanical effort: ~1 day.

### Lowest-Confidence Decisions (candidates for revisiting)

- **D14 (72%):** Merging modals into panels is principled but the margin is narrow. Current ModalStack is clean at ~80 lines. Revisit if the systems start diverging.
- **D1 (75%):** UiContext mirrors World, but UI has more heterogeneous state concerns. Revisit if sub-struct boundaries feel unnatural during implementation.
- **D15 (75%):** Decoupling scroll from PanelManager is correct, but the key type (string vs enum) depends on D13. If D13 goes to enum, scroll keys should match.
