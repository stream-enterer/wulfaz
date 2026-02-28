# Phase 1: Section-Level Audit Verdicts

## Agent A — Simulation Core (lines 1-117)

### What This Is (1-6)
- Lines 1-6: **KEEP** — Design intent, pattern names (blackboard, EAV) non-obvious

### Architecture (8-17)
- Line 10 (World is HashMap): **MODIFY** — Drift: World now uses sub-structs (BodyTables, MindTables, GisTables)
- Line 11 (plain functions): **KEEP** — Prevents trait/object systems (1), design intent (2)
- Line 12 (one system per file): **KEEP** — Prevents monolith files (1)
- Lines 13-14 (no message passing/traits): **KEEP** — Prevents 3 specific mistakes (1)
- Line 15 (phase order matters): **KEEP** — Design intent (2), order is load-bearing
- Lines 16-17 (seeded RNG only): **KEEP** — Prevents non-deterministic RNG (1)
- **ADD U1**: Sort entities by e.0 before processing for determinism

### Core Types (19-33)
- Lines 21-24 (Entity/Tick newtypes): **KEEP** — Prevents raw u64 confusion (1)
- Line 26 (Key World fields header): **MODIFY** — "not property tables" confusing given sub-structs
- Line 27 (alive: HashSet): **KEEP** — Prevents HashMap for alive (1)
- Line 28 (pending_deaths: Vec): **KEEP** — Design intent (2)
- Line 29 (rng: StdRng): **CUT** — Duplicates lines 16-17
- Line 30 (events: EventLog): **KEEP** — Prevents Vec<Event> (1)
- Line 31 (tiles: TileMap): **COMPRESS** — "flat Vec arrays" is descriptive; merge with TileMap section
- Line 33 (no casting Entity/Tick): **KEEP** — Prevents cross-casting (1)

### Entity Lifecycle (35-53)
- Lines 37-46 (code block): **MODIFY** — world.hungers → world.mind.hungers (sub-struct path)
- Lines 48-49 (NEVER manual .remove): **KEEP** — Single most important rule (1)
- Lines 51-52 (spawn in any phase): **KEEP** — Prevents confusion about visibility (1, 3)

### Pending-Death Rule (54-64)
- Lines 56-57 (prose rule): **KEEP** — Prevents acting on dead entities (1)
- Lines 59-63 (code example): **COMPRESS** — Merge into System Mutation Pattern example; redundant + drifted path

### System Mutation Pattern (66-85)
- Lines 68-69 (collect-then-apply rule): **KEEP** — Prevents mutation-during-iteration (1)
- Lines 71-84 (code example): **MODIFY** — Fix world.hungers → world.mind.hungers

### Main Loop Phases (87-117)
- Lines 89-113 (phase block): **KEEP** — Phase specification IS design intent (2)
- Lines 115-117 (Phase 4 vs 5 rule): **KEEP** — Prevents phase misclassification (1)
- **ADD**: Phase 1-3 classification rules (not just 4 vs 5)

## Agent B — Simulation Periphery (lines 119-248)

### Adding a New System (119-127)
- Step 1 (create file): **KEEP**
- Step 2 (write fn signature): **CUT** — Duplicate of line 11
- Step 3 (add pub mod): **KEEP**
- Step 4 (add call in main.rs): **KEEP**
- Step 5 (write unit test): **CUT** — Duplicate of Testing section
- Step 6 (cargo build + validate): **KEEP**
- **ADD U1**: Deterministic iteration order step

### Adding a New Property Table (128-138)
- Lines 130, 132, 133, 134, 136: **MODIFY** — Drift: flat HashMap → sub-structs
- Lines 135, 138: **KEEP** — Zombie bug prevention

### Adding a New Event Type (140-145)
- All steps: **KEEP** — All pass both rubrics

### Event Log (147-152)
- Lines 149-151 (ring buffer description + no Vec): **CUT** — Triple-stated (line 30, line 244)
- Lines 151-152 (API surface): **KEEP** — Unique API info

### TileMap (154-163)
- Line 156 (accessor-only): **KEEP**
- Lines 158-161 (code example): **CUT** — Discoverable from code
- Line 163 (architecture.md ref): **MODIFY** — Path should be .workflow/architecture.md

### Spatial Scale (165-173)
- Line 167 (1 tile = 1 meter): **KEEP** — Design intent
- Line 169 (unit comments): **KEEP** — Documentation discipline
- Line 170 (64×64 default): **CUT** — Descriptive, discoverable
- Line 171 (melee range): **KEEP** — Design intent
- Line 172 (diagonal √2): **CUT** — Duplicate of line 192
- Line 173 (see Gait System): **CUT** — Forward-reference only

### Gait System (175-196)
- Lines 177-178 (intro): **KEEP** — Design intent
- Lines 180-181 (data model): **KEEP**
- Lines 183-190 (cooldown table): **CUT** — Reference data, not constraint; maintenance burden
- Line 192 (diagonal formula): **KEEP** — Specific formula
- Lines 194-196 (walk default, fast situational): **KEEP** — Design intent

### Data Files (KDL) (198-220)
- Line 200 (content in data/*.kdl): **KEEP** — Core architecture
- Line 201 (kdl crate link): **CUT** — Discoverable from Cargo.toml
- Lines 203-217 (KDL example): **CUT** — Duplicates actual data files
- Lines 219-220 (no code changes): **KEEP** — Prevents hardcoding

### Code Rules (222-232)
- Line 224 + 227: **COMPRESS** — Merge: missing entry = skip silently (if let Some)
- Lines 225-226 (helpers on World): **MODIFY** — Should mention sub-structs
- Lines 228-232 (no #[allow]): **KEEP** — Prevents warning suppression

### What NOT To Do (234-247)
- Line 236 (no traits between systems): **CUT** — Duplicate of line 14
- Line 237 (no scheduler): **KEEP** — Prevents over-engineering
- Line 238 (no message passing): **CUT** — Duplicate of lines 13-14
- Line 239 (no manual .remove): **CUT** — Duplicate of lines 48-49
- Line 240 (no multiple systems per file): **CUT** — Duplicate of line 12
- Line 241 (no shared mutable state outside World): **KEEP** — Unique negative-space
- Line 242 (no unsafe): **KEEP** — Safety gate
- Line 243 (no thread_rng): **CUT** — Duplicate of lines 16-17
- Line 244 (no Vec<Event>): **CUT** — Duplicate of line 30
- Lines 245-246 (no HashMap replacement without profiling): **KEEP** — Non-obvious threshold
- Line 247 (no concurrency): **KEEP** — Design intent

## Agent C — UI (lines 249-332)

### UI Architecture (249-270)
- Lines 251-253 (mirror metaphor): **KEEP** — Design intent
- Lines 255-257 (UiContext): **MODIFY** — Drop "Analogous to World" (redundant) + "Lives on App" (descriptive)
- Lines 258-259 (WidgetTree ephemeral): **MODIFY** — Drop "Lives on App as self.ui_tree" (descriptive)
- Lines 260-261 (builders): **KEEP** — Design intent + prevents trait builders
- Lines 262-263 (Widget closed enum): **KEEP** — Prevents trait objects
- Line 264 (UiAction enum): **CUT** — Fails uniqueness; restated in lines 302, 311
- Line 265 (PanelKind enum): **KEEP** — Prevents string-based IDs
- Lines 266-267 (Theme flat struct): **KEEP** — Design intent, prevents Theme-in-UiContext
- Lines 268-269 (one concern per file): **COMPRESS** — "one builder per panel" already in 261; trim
- Line 270 (mod.rs rule): **KEEP** — Prevents logic creep

### UI Frame Lifecycle (272-282)
- Lines 274-279 (lifecycle phases): **MODIFY** — Drop "Exhaustive — no catch-all arm" (redundant with 302)
- Line 282 (no dirty tracking): **KEEP** — Prevents 3 specific mistakes

### Adding a New UI Panel (284-290)
- Lines 286-290 (5-step checklist): **KEEP**
- **ADD U3**: collect_*/Info convention step

### Adding a New Widget (292-296)
- Lines 294-296: **KEEP**

### Adding a New UiAction (298-302)
- Lines 300-302: **KEEP** — Canonical location for no-catchall rule

### UI What NOT To Do (304-312)
- Line 306 (no dirty flags): **CUT** — Duplicate of lines 258, 282
- Lines 307-308 (no traits, TextMeasurer exception): **KEEP** — Broader scope than 262
- Line 309 (no persistent state on WidgetTree): **CUT** — Duplicate of 258-259
- Line 310 (no ad-hoc state on App): **KEEP** — Unique prohibition
- Line 311 (no catch-all): **CUT** — Duplicate of line 302
- Line 312 (no multiple builders per file): **CUT** — Positive form in 260-261, 268-269

### Testing (314-332)
- Line 316 (every system ships with test): **KEEP**
- Lines 318-327 (test example): **KEEP** — Example IS the teaching
- Lines 329-330 (property tests): **KEEP** — Design intent + scaffolding
- Line 331 (validate_world every tick): **CUT** — Duplicate of lines 112, 126

## Totals

- **KEEP**: ~45 blocks
- **CUT**: ~22 items (mostly cross-section duplicates)
- **COMPRESS**: 4 merges
- **MODIFY**: ~10 items (mostly sub-struct drift + descriptive trimming)
- **ADD**: 3 items (U1 deterministic iteration, U3 collect_* convention, Phase 1-3 rules)
