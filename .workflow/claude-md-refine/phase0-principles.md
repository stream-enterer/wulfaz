# Phase 0: Classified Principles

## CONFIRMED (CLAUDE.md states it, code follows it, design docs agree)

C1. World is HashMap<Entity,T> property tables + TileMap [Architecture §1]
C2. Systems are plain functions: `fn run_*(world: &mut World, tick: Tick)` [Architecture §2]
C3. One system per file in src/systems/ [Architecture §3]
C4. Systems communicate only through shared World state — no message passing, no traits [Architecture §4]
C5. Phase-ordered sequential main loop — order matters [Architecture §5]
C6. All randomness through world.rng (seeded StdRng) — deterministic replay [Architecture §6]
C7. Entity(u64) and Tick(u64) newtypes — no raw integers, no cross-casting [Core Types]
C8. alive is HashSet<Entity>, pending_deaths is Vec<Entity> [Core Types]
C9. events is EventLog (ring buffer), not Vec [Core Types + Event Log]
C10. tiles is TileMap — use accessor methods only [TileMap]
C11. Entity lifecycle: spawn → insert components → pending_deaths → despawn [Entity Lifecycle]
C12. ONLY run_death calls World::despawn() [Entity Lifecycle]
C13. NEVER manually .remove() from individual tables [Entity Lifecycle]
C14. Systems MUST skip pending_deaths entities [Pending-Death Rule]
C15. Collect-then-apply mutation pattern [System Mutation Pattern]
C16. 5 phases: Environment → Needs → Decisions → Actions → Consequences [Main Loop]
C17. Phase rule: external state change = Phase 4, consequences = Phase 5 [Main Loop]
C18. run_death() ALWAYS last in Phase 5 [Main Loop]
C19. validate_world() runs every tick in debug builds [Main Loop]
C20. Adding New System checklist (6 steps) [Adding a New System]
C21. Adding New Property Table checklist (5 steps) — despawn + validate [Adding New Property Table]
C22. Adding New Event Type checklist (4 steps) — tick field required [Adding New Event Type]
C23. EventLog ring buffer with configurable max depth [Event Log]
C24. 1 tile = 1 meter, every spatial constant must have unit comment [Spatial Scale]
C25. Diagonal movement cost √2 (141/100 fixed-point) [Spatial Scale + Gait]
C26. Never .unwrap() on table lookups — use if let Some [Code Rules]
C27. Helpers go as World methods in world.rs — no utils.rs [Code Rules]
C28. Missing table entry = skip entity, no log, no panic [Code Rules]
C29. No #[allow(...)]/[expect(...)] — fix the cause [Code Rules]
C30. No traits/interfaces between systems [What NOT To Do]
C31. No system registry or scheduler [What NOT To Do]
C32. No unsafe without approval [What NOT To Do]
C33. No HashMap replacement without profiling data >5ms [What NOT To Do]
C34. No concurrency in simulation loop [What NOT To Do]
C35. Widget is closed enum, exhaustive match [UI Architecture]
C36. UiAction enum for callbacks, exhaustive dispatch_click [UI Architecture]
C37. PanelKind enum for panel identity, no string names [UI Architecture]
C38. Theme is flat struct, const-constructible, passed by &Theme [UI Architecture]
C39. One concern per file: tree_*.rs split, one builder per panel [UI Architecture]
C40. mod.rs is module declarations + re-exports only [UI Architecture]
C41. WidgetTree is ephemeral — rebuilt every frame [UI Architecture + Frame Lifecycle]
C42. No dirty tracking, no retained tree state, no diff-and-patch [Frame Lifecycle]
C43. Builders are free functions: fn build_*(...) -> WidgetId [UI Architecture]
C44. UI Frame Lifecycle: Build → Layout → Draw → Input → Dispatch [Frame Lifecycle]
C45. Adding New Panel checklist (5 steps) [Adding New Panel]
C46. Adding New Widget checklist (3 steps) [Adding New Widget]
C47. Adding New UiAction checklist (3 steps) [Adding New UiAction]
C48. No dirty flags or change tracking [UI What NOT To Do]
C49. No traits between UI modules (exception: TextMeasurer) [UI What NOT To Do]
C50. No persistent state on WidgetTree [UI What NOT To Do]
C51. No ad-hoc UI state on App — all persistent UI state → UiContext [UI What NOT To Do]
C52. No _ => catch-all in dispatch_click [UI What NOT To Do]
C53. No multiple builders or tree operations in one file [UI What NOT To Do]
C54. Every new system ships with unit test [Testing]
C55. Property-based tests in tests/invariants.rs [Testing]
C56. KDL data files for content, engine doesn't hardcode entity types [Data Files]
C57. Gait system with DF-style tiers, biped/quadruped profiles [Gait System]
C58. Default map 64×64, melee range = same tile [Spatial Scale]

## UNSTATED (Code follows it, CLAUDE.md doesn't mention it)

U1. Deterministic entity processing: sort by entity ID (e.0) before iteration
    — Every system does this. Critical for deterministic replay. CLAUDE.md mentions
    deterministic replay via RNG but not via iteration order.
    → SHOULD ADD (Rubric B: consistent pattern, non-obvious, violating it breaks determinism)

U2. Spatial index rebuilt post-movement (twice per tick)
    — Main loop rebuilds after wander so eating/combat see updated positions.
    → SHOULD NOT ADD (implementation detail of main loop, not a constraint for new systems)

U3. collect_* functions extract game state into *Info structs for builders
    — Consistent naming convention, every builder has paired collect function.
    → SHOULD ADD as brief mention in builder convention (Rubric B: naming convention, non-obvious)

U4. Builder info structs (*Info) have pub fields
    — All info structs use pub fields, consistent with UiContext split-borrow rationale.
    → Already implied by builder signature pattern description. SKIP.

U5. WindowFrame shared builder pattern for titled/closeable windows
    — Reusable chrome builder returning struct with handles.
    → SHOULD NOT ADD (discoverable from code, not a constraint)

U6. Colors are [f32; 4] sRGB RGBA with hex()/hex_a() helpers
    — Consistent convention but discoverable from code.
    → SHOULD NOT ADD (discoverable)

U7. Event pushed AFTER decision, BEFORE pending_deaths.push() for lethal events
    — Already stated in "Adding a New Event Type" step 3. SKIP (already confirmed as C22).

U8. Read-only helpers take &World, only main system fn takes &mut World
    — Good practice but enforced by Rust borrow checker. SKIP (compiler-enforced).

## ASPIRATIONAL (CLAUDE.md states it, design docs prescribe it, not fully implemented yet)

A1. UiContext as single struct with pub sub-fields (D1)
    — CLAUDE.md describes this. Design docs prescribe it (Phase 2 migration).
    Current code: UiContext EXISTS and is used, but D1 migration may not be 100% complete.
    → KEEP (scaffolding for intended architecture)

A2. D14: Modals as panels (not separate ModalStack)
    — Design docs prescribe but at lowest confidence (72%). Not in CLAUDE.md currently.
    → DO NOT ADD (low confidence, may be revised)

## DRIFTED (CLAUDE.md states something, code has evolved past it)

D1. CLAUDE.md says "UiContext... Analogous to World. Lives on App as `self.ui`"
    — Need to verify this is still accurate. The structure exists.
    → VERIFY during rewrite (minor wording check)

D2. CLAUDE.md references `architecture.md` for TileMap storage layout
    — File may be `.workflow/architecture.md` not `architecture.md`. Minor path issue.
    → VERIFY and fix reference if needed

D3. D3 (mod.rs decomposition) and D5 (UiAction enum) were listed as aspirational
    in architecture docs but are now IMPLEMENTED in the codebase.
    — The architecture docs are stale, not CLAUDE.md. CLAUDE.md correctly describes
    the current state. No drift in CLAUDE.md itself.
    → NO ACTION NEEDED for CLAUDE.md

## DESCRIPTIVE (CLAUDE.md states it but it's just describing current code, not constraining)

DESC1. "Key World fields" listing (alive, pending_deaths, rng, events, tiles)
    — Partially descriptive, partially constraining. The types (HashSet vs HashMap,
    ring buffer vs Vec) are constraints. The field names are descriptive.
    → COMPRESS: keep type constraints, trim field listing

DESC2. KDL code example showing Goblin/Wolf definitions
    — Demonstrates format but format is self-evident from KDL files.
    → CUT the example, keep the principle "Content in data/*.kdl, engine doesn't hardcode"

DESC3. System Mutation Pattern code example (17 lines)
    — The example is useful for showing both pending_death filter AND collect-then-apply
    together. The pattern is non-obvious enough to warrant an example.
    → KEEP (the example IS the teaching)

DESC4. Pending-Death Rule code example (5 lines)
    — Somewhat redundant with the mutation pattern example which also shows filtering.
    → COMPRESS: merge into mutation pattern section or make briefer

DESC5. Entity Lifecycle code example
    — Shows spawn/kill/despawn flow. Non-obvious (especially "ONLY run_death despawns").
    → KEEP

DESC6. Test example code (8 lines)
    — Shows World::new_with_seed pattern and general shape. Useful for new contributors.
    → KEEP but could be slightly shorter

DESC7. TileMap code example (2 lines)
    — Shows accessor pattern. Brief enough. Keep.
    → KEEP

DESC8. Main Loop Phases code block (12 lines of comments)
    — Describes what each phase does. This IS the phase specification.
    → KEEP (this is design intent, not description)

DESC9. "WidgetTree is ephemeral — destroyed and rebuilt from scratch every frame.
    It is NOT persistent state. Lives on App as `self.ui_tree`."
    — The "Lives on App as self.ui_tree" part is descriptive. The rest is a constraint.
    → COMPRESS: keep constraint, trim location detail

## SUMMARY

- **Confirmed:** 58 principles
- **Unstated (should add):** 2 (U1: deterministic iteration order, U3: collect_* convention)
- **Unstated (skip):** 5
- **Aspirational (keep):** 1 (A1: UiContext architecture)
- **Drifted:** 0 significant (2 minor path/wording checks)
- **Descriptive (candidates for cutting/compression):** 9 items evaluated, 4 can be compressed/cut
