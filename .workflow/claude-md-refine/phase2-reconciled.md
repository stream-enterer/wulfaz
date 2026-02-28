# Phase 2: Cross-Section Reconciliation

## Structural Changes

### Sections Eliminated (absorbed elsewhere)
1. **Pending-Death Rule** → merged into new "System Iteration and Mutation"
2. **Event Log** → API surface absorbed into "Adding a New Event Type"
3. **What NOT To Do** → 6 items cut as redundant, 5 survivors absorbed into Architecture + Code Rules
4. **UI What NOT To Do** → 5 items cut as redundant, 2 survivors absorbed into UI Architecture

### New Merged Section
- **System Iteration and Mutation** = Pending-Death Rule + System Mutation Pattern + ADD U1 (deterministic iteration order)

### Reordering
Group conceptual sections first, then reference data, then checklists:
1. What This Is → Architecture → Core Types → Entity Lifecycle → System Iteration and Mutation → Main Loop Phases → Code Rules
2. TileMap → Spatial Scale → Gait System → Data Files (KDL)
3. Adding a New System → Adding a New Property Table → Adding a New Event Type
4. UI Architecture → UI Frame Lifecycle → Adding New Panel/Widget/UiAction → Testing

## ADD Placements
- **U1** (deterministic iteration order): System Iteration and Mutation section + brief mention in Adding a New System
- **U3** (collect_*/Info convention): Adding a New UI Panel step 1
- **Phase 1-3 rules**: Main Loop Phases, expanding the existing Phase 4/5 rule

## "What NOT To Do" Absorption Map
- No registry/scheduler → Architecture (append to phase order line)
- No shared mutable state outside World → Architecture (new bullet)
- No concurrency → Architecture (new bullet)
- No unsafe without approval → Code Rules
- No HashMap replacement without profiling → Code Rules

## "UI What NOT To Do" Absorption Map
- No traits between UI modules (TextMeasurer exception) → UI Architecture (new bullet)
- No ad-hoc UI state on App → UiContext bullet in UI Architecture

## Item-Level Verdicts (authoritative)

### Simulation Core
| Item | Verdict | Notes |
|------|---------|-------|
| What This Is (1-6) | KEEP | |
| Architecture L10 (World is HashMap) | MODIFY | Sub-struct grouping |
| Architecture L11 (plain functions) | KEEP | |
| Architecture L12 (one system per file) | KEEP | |
| Architecture L13-14 (no message passing) | KEEP | |
| Architecture L15 (phase order) | MODIFY | Append "No system registry or scheduler" |
| Architecture L16-17 (seeded RNG) | KEEP | |
| Architecture | ADD | "All shared state lives on World" |
| Architecture | ADD | "Single-threaded simulation loop. No concurrency." |
| Core Types L21-24 (newtypes) | KEEP | |
| Core Types L26 (header) | MODIFY | Rephrase for sub-structs |
| Core Types L27 (alive: HashSet) | KEEP | |
| Core Types L28 (pending_deaths) | KEEP | |
| Core Types L29 (rng) | CUT | Duplicate of Architecture L16-17 |
| Core Types L30 (EventLog) | KEEP | |
| Core Types L31 (tiles) | CUT | Descriptive, TileMap has own section |
| Core Types L33 (no casting) | KEEP | |
| Entity Lifecycle L37-46 | MODIFY | Update paths for sub-structs |
| Entity Lifecycle L48-49 | KEEP | Single most important rule |
| Entity Lifecycle L51-52 | KEEP | |
| Pending-Death Rule L54-64 | MERGE | Into System Iteration and Mutation |
| System Mutation Pattern L66-85 | MERGE | Into System Iteration and Mutation |
| System Iter & Mutation | ADD U1 | Deterministic iteration order |
| Main Loop Phases L89-113 | KEEP | |
| Main Loop Phases L115-117 | MODIFY | Expand to all 5 phases |

### Simulation Periphery
| Item | Verdict | Notes |
|------|---------|-------|
| Adding System step 1 (create file) | KEEP | |
| Adding System step 2 (fn signature) | CUT | Duplicate of Architecture L11 |
| Adding System step 3 (pub mod) | KEEP | |
| Adding System step 4 (add call) | KEEP | |
| Adding System step 5 (unit test) | COMPRESS | → "Write a unit test (see Testing)" |
| Adding System step 6 (cargo build) | KEEP | |
| Adding System | ADD | Deterministic iteration reminder |
| Adding Property Table L130-136 | MODIFY | Sub-struct paths |
| Adding Property Table L138 | KEEP | |
| Adding Event Type L140-145 | KEEP | |
| Adding Event Type | ADD | EventLog API surface from Event Log section |
| Event Log (entire section) | ELIMINATE | |
| TileMap L156 (accessor only) | KEEP | |
| TileMap L158-161 (code example) | CUT | |
| TileMap L163 (architecture ref) | MODIFY | Fix path |
| Spatial Scale L167 (1 tile = 1m) | KEEP | |
| Spatial Scale L169 (unit comments) | KEEP | |
| Spatial Scale L170 (64×64 default) | CUT | |
| Spatial Scale L171 (melee range) | KEEP | |
| Spatial Scale L172 (diagonal) | CUT | Duplicate of Gait L192 |
| Spatial Scale L173 (see Gait) | CUT | |
| Gait System L177-181 (intro + model) | KEEP | |
| Gait System L183-190 (cooldown table) | CUT | |
| Gait System L192 (diagonal formula) | KEEP | |
| Gait System L194-196 (walk default) | KEEP | |
| KDL L200 (data-driven) | KEEP | |
| KDL L201 (crate link) | CUT | |
| KDL L203-217 (example) | CUT | |
| KDL L219-220 (no code changes) | KEEP | |
| Code Rules L224+227 | COMPRESS | Merge into single rule |
| Code Rules L225-226 | MODIFY | Mention sub-structs |
| Code Rules L228-232 | KEEP | |
| Code Rules | ADD | No unsafe without approval |
| Code Rules | ADD | No HashMap replacement without profiling >5ms |
| What NOT To Do (entire section) | ELIMINATE | |

### UI
| Item | Verdict | Notes |
|------|---------|-------|
| UI Arch L251-253 (mirror metaphor) | KEEP | |
| UI Arch L255-257 (UiContext) | MODIFY | Trim descriptive parts, add no-ad-hoc-state rule |
| UI Arch L258-259 (WidgetTree) | MODIFY | Trim "Lives on App" |
| UI Arch L260-261 (builders) | KEEP | |
| UI Arch L262-263 (Widget enum) | KEEP | |
| UI Arch L264 (UiAction) | MODIFY | Keep type info, cut exhaustive match claim |
| UI Arch L265 (PanelKind) | KEEP | |
| UI Arch L266-267 (Theme) | KEEP | |
| UI Arch L268-269 (one concern) | COMPRESS | Trim "one builder per panel" (already in L261) |
| UI Arch L270 (mod.rs) | KEEP | |
| UI Arch | ADD | No traits between UI modules (TextMeasurer exception) |
| UI Lifecycle L274-278 | KEEP | |
| UI Lifecycle L279 (Dispatch) | MODIFY | Cut "Exhaustive — no catch-all arm" |
| UI Lifecycle L282 | KEEP | |
| Adding Panel L286 (step 1) | MODIFY | Expand for collect_*/Info (U3) |
| Adding Panel L287-290 | KEEP | |
| Adding Widget L294-296 | KEEP | |
| Adding UiAction L300-302 | KEEP | Canonical home for no-catchall |
| UI What NOT To Do (entire section) | ELIMINATE | |
| Testing L316-327 | KEEP | |
| Testing L329-330 | KEEP | |
| Testing L331 | CUT | Duplicate of Main Loop L112 |
