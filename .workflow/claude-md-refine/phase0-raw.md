# Phase 0: Raw Principle Discovery

## Source 1: Codebase System Patterns

### File Structure
- 8 system files, each with exactly one `pub fn run_*` function
- All follow signature `fn run_*(world: &mut World, tick: Tick)` (some prefix `_tick`)
- `src/systems/mod.rs` is pure module declarations, no function bodies

### Mutation & Safety
- ALL systems use collect-then-apply pattern (no mutation during iteration)
- ALL entity-processing systems filter pending_deaths before processing
- NO `.unwrap()` on table lookups — always `get()` with `if let`/`filter_map()`

### Determinism
- Entity processing sorted by `e.0` (Entity ID) before iteration in all systems
- All randomness through `world.rng` (seeded StdRng)
- Event logging with tick coupling (every variant has `tick: Tick`)

### World Structure
- `World::despawn()` cleans up 14+ property tables (via BodyTables.remove + MindTables.remove + alive.remove)
- `validate_world()` checks 14 tables for zombie entities (key in table → entity in alive)
- Helper functions live as World methods, no separate utils.rs
- Spatial index rebuilt twice per tick: before Phase 1, after Phase 4 (movement)

### Main Loop
- 5-phase sequential loop in `run_one_tick()`
- Phase 1: Environment (temperature)
- Phase 2: Needs (hunger, fatigue)
- Phase 3: Decisions (decisions)
- Phase 4: Actions (wander, eating, combat) — spatial index rebuilt after wander
- Phase 5: Consequences (death — ALWAYS last)
- Debug validation after Phase 5
- Tick incremented at end

### Additional Conventions Found
- P1: Deterministic entity processing (sort by entity ID)
- P2: Exhaustive pattern matching (no catch-all arms)
- P3: Missing component = skip entity (no log, no panic)
- P4: Spatial index rebuilt post-movement
- P5: Events pushed AFTER decision, BEFORE pending_deaths.push()
- P6: Intention-based action gating (with legacy fallbacks)
- P7: Constants documented with real-world units
- P8: Fixed-point math for diagonal (141/100)
- P9: Every system file has `#[cfg(test)] mod tests`
- P10: Candidates collected, filtered, THEN sorted for determinism
- P11: Helper fns take `&World` (immutable), only main system fn takes `&mut World`

## Source 2: UI Architecture Patterns

### Builder Convention
- All builders: `pub fn build_*(tree: &mut WidgetTree, theme: &Theme, info: &*Info) -> WidgetId` (or tuple)
- Every builder has a paired `*Info` struct with pub fields
- Tests in builder file: `#[cfg(test)] mod tests` at bottom
- `collect_*` functions extract game state into Info structs (called in main.rs)

### Module Organization
- `mod.rs` is purely module declarations + re-exports (no function bodies)
- ~20 pub(crate) modules + ~18 private modules
- One concern per file: tree_layout.rs, tree_draw.rs, tree_hit_test.rs, tree_scroll.rs, tree_anim.rs, tree_tooltip.rs

### Widget System
- Widget is closed enum (18 variants), exhaustive match in layout/draw
- UiAction is closed enum (~20 variants), exhaustive match in dispatch_click (no `_ =>`)
- PanelKind is enum (9 variants), no string names
- ZTier enum: Panel, Overlay, Modal, Tooltip

### State Management
- UiContext: pub sub-fields (input, animator, modals, panels, scroll, sidebar)
- Theme: flat struct, const-constructible, passed by `&Theme`, not in UiContext
- WidgetTree is ephemeral — destroyed and rebuilt every frame
- No dirty tracking, no retained tree, no diff-and-patch

### Frame Lifecycle
- Build → Layout → Draw → Input → Dispatch → Render
- Layout called multiple times per frame for reflow
- Draw emits commands in Z-tier order (Panel → Overlay → Modal → Tooltip)

### WindowFrame Pattern
- Shared builder for titled/closeable windows
- Returns WindowFrame struct with handles (root, header, title, content, close_btn, content_width)
- Caller inserts screen-specific widgets into frame.content

### Colors
- All colors `[f32; 4]` sRGB RGBA
- `hex()` and `hex_a()` helpers

## Source 3: Architecture Document Decisions

### D1-D17 Summary
- D1: UiContext with pub sub-fields (ASPIRATIONAL — Phase 2 migration)
- D2: Full rebuild every frame (IMPLEMENTED)
- D3: mod.rs decomposition to ~15 files (IMPLEMENTED — was aspirational, now done)
- D4: Widget as closed enum (IMPLEMENTED)
- D5: UiAction enum callbacks (IMPLEMENTED — was aspirational, now done)
- D6: Current custom layout model (IMPLEMENTED)
- D7: Return Option<UiAction> from polling (IMPLEMENTED — bundled with D5)
- D8: Free function builders (IMPLEMENTED)
- D9: Theme as flat struct (IMPLEMENTED)
- D10: External animation ownership (IMPLEMENTED)
- D11: DrawList intermediate representation (IMPLEMENTED)
- D12: ZTier enum (IMPLEMENTED)
- D13: PanelKind enum (IMPLEMENTED — was aspirational, now done)
- D14: Modals as panels (ASPIRATIONAL — low priority)
- D15: Scroll on UiContext (ASPIRATIONAL — Phase 2)
- D16: One operation per file (IMPLEMENTED — bundled with D3)
- D17: Accept focus staleness (IMPLEMENTED)

### Non-Negotiable Principles from Architecture Docs
- Explicit over clever
- One concern per file
- State lives in known places
- Simple over flexible
- No traits as primary decomposition (exception: hardware boundary traits like TextMeasurer)

### B05 Door Placement (GIS pipeline — domain-specific, not CLAUDE.md material)
- Terrain transitions, temperature targets, building registry, door placement phases
- These are domain-specific implementation details, not architectural principles
