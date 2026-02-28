# Phase 3: Final Verdicts (Post-Debate Synthesis)

## Tiebreaker Rule Applied
When Compressor and Architect both have strong arguments, KEEP the line.
Cost of extra line = tokens. Cost of missing design constraint = drift.

## Final Verdicts by Section

### What This Is
- KEEP as-is (2 lines of prose)

### Architecture (Non-Negotiable)
- L10: MODIFY — Update for sub-structs: "World groups property tables into sub-structs (BodyTables, MindTables) each containing HashMap<Entity,T>"
- L11-17: KEEP
- L15: MODIFY — Append: "No system registry or scheduler."
- ADD: "All shared state lives on World. No mutable state outside it."
- ADD: "Single-threaded. Do not add concurrency."
- ADD: "Data-driven: creature/item types in data/*.kdl, not hardcoded. loading.rs maps KDL to entities."

### Core Types
- CUT code block (lines 21-24) — Types in components.rs, constraint in prose
- KEEP trimmed field listing: alive (HashSet, not HashMap), pending_deaths (Vec), events (EventLog ring buffer). CUT rng (duplicate), CUT tiles (own section removed).
- KEEP line 33 (no casting Entity/Tick)

### Entity Lifecycle
- CUT code block (lines 37-46) — Prose rules are the constraints
- KEEP "NEVER manually .remove()" (lines 48-49) — Single most important rule
- KEEP "spawn in any phase" (lines 51-52)
- MODIFY paths in prose if any remain referencing flat world.hungers

### System Iteration and Mutation (MERGED from Pending-Death + Mutation Pattern)
- KEEP prose: "Skip pending_deaths entities" + "Collect changes first, then apply"
- CUT both code examples — 8 system files demonstrate the pattern
- ADD U1: "Sort entities by e.0 before processing. HashMap iteration order is non-deterministic."
- Three rules, no code block

### Main Loop Phases
- COMPRESS code block to compact form: phase names + 1-line descriptions
- KEEP Phase 4/5 classification rule
- ADD Phase 1-3 classification rules (Architect's formulation)
- Full 5-phase classification list replaces current 2-line rule

### Code Rules
- COMPRESS L224+L227: "Missing table entry = skip silently (if let Some). Never .unwrap() on lookups."
- MODIFY L225-226: Mention sub-structs for helpers
- KEEP L228-232 (no #[allow])
- ADD: "Do not use unsafe without explicit approval."
- ADD: "Do not replace HashMap without profiling data showing >5ms per tick."

### TileMap
- ELIMINATE section — Module privacy enforces accessor-only. Reference to .workflow/architecture.md is nice-to-know, not constraint.

### Spatial Scale
- KEEP L167 (1 tile = 1 meter)
- KEEP L169 (unit comments on constants)
- CUT L170 (64×64 default — configurable, discoverable)
- CUT L171 (melee range — in combat.rs source)
- CUT L172-173 (diagonal duplicate, forward-reference)

### Gait System
- KEEP intro (biped/quadruped split) — Lines 177-178
- KEEP data model (GaitProfile, current_gait) — Lines 180-181
- KEEP cooldown table — Lines 183-190 (Architect won: opaque arrays in code, table IS the spec)
- KEEP diagonal formula — Line 192
- KEEP walk default/situational — Lines 194-196

### Data Files (KDL)
- ELIMINATE as standalone section — Absorb one line into Architecture ("data-driven: data/*.kdl")
- CUT example block, crate link, "no code changes" line

### Adding a New System
- CUT step 2 (fn signature — duplicate of Architecture L11)
- COMPRESS step 5 to "Write a unit test (see Testing)"
- ADD step about deterministic iteration order
- KEEP steps 1, 3, 4, 6

### Adding a New Property Table
- MODIFY all steps for sub-struct paths
- KEEP all steps (zombie prevention checklist)

### Adding a New Event Type
- KEEP all steps
- DO NOT add EventLog API surface (Compressor won: 3 methods discoverable)

### Event Log
- ELIMINATE section — Ring buffer fact in Core Types L30, API discoverable

### What NOT To Do
- ELIMINATE section — All items either cut as duplicate or absorbed into Architecture/Code Rules

### UI Architecture
- KEEP mirror metaphor (lines 251-253) — Architect won: foundational "why"
- MODIFY UiContext bullet: trim descriptive location, add "no ad-hoc UI state on App"
- MODIFY WidgetTree bullet: trim "Lives on App"
- KEEP builders, Widget enum, PanelKind, Theme bullets
- MODIFY UiAction bullet: keep type info, cut exhaustive match claim
- COMPRESS one-concern-per-file: trim "one builder per panel" (already in builders bullet)
- KEEP mod.rs rule
- ADD: "No traits between UI modules. Exception: TextMeasurer at hardware boundary."

### UI Frame Lifecycle
- COMPRESS listing: keep phase names, trim to essentials
- MODIFY Dispatch line: cut "Exhaustive — no catch-all arm"
- KEEP "No dirty tracking. No retained tree state. No diff-and-patch."

### Adding a New UI Panel
- MODIFY step 1: expand for collect_*/Info convention (U3)
- KEEP steps 2-5

### Adding a New Widget / Adding a New UiAction
- KEEP as-is

### UI What NOT To Do
- ELIMINATE section — TextMeasurer exception absorbed into UI Architecture, App state rule absorbed into UiContext bullet

### Testing
- KEEP prose rule (every system ships with unit test)
- CUT code example — Pattern in every test file
- KEEP property-based tests line
- CUT validate_world line (duplicate of Main Loop)

## Estimated Result
- ~4 sections eliminated (Event Log, TileMap, What NOT To Do, UI What NOT To Do, Data Files standalone)
- 1 section merged (Pending-Death + Mutation → System Iteration and Mutation)
- All code examples removed except: Main Loop phases (compressed), Gait cooldown table
- ~3 additions (U1 deterministic iteration, Phase 1-3 rules, collect_* convention)
- Target: ~180-200 lines (down from 332)
