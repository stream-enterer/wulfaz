# 10 — Antipattern Checklist

**Scope:** Every "you will miss this" trap that causes silent failures, broken output, or wasted effort when porting CDDA's map generation systems.
**Purpose:** The consuming LLM reads this BEFORE implementation to avoid known pitfalls. Each entry describes a specific failure mode, why it's hidden, and how to fix it.

---

## TIER 0 — Won't Load / Crash

These cause immediate failures during data loading or map generation. You cannot ship without fixing them.

---

### TRAP: Mapgen Without Palettes

- **SYMPTOM:** Every building in mapgen "rows" renders as `fill_ter` (usually grass or dirt). Buildings are blank rectangles.
- **SYSTEM YOU PORTED:** JSON mapgen — `"rows"` grid, `fill_ter`, terrain placement.
- **SYSTEM YOU MISSED:** Palette system — the `"palettes"` array that resolves ASCII characters to terrain/furniture/trap IDs.
- **WHY IT'S HIDDEN:** The `"rows"` data looks self-contained — it's just strings of characters. But those characters are meaningless without palette resolution. The palette reference is a small `"palettes": ["house_general"]` line easily overlooked.
- **TIER:** 0
- **FIX:** Implement palette loading (`07_PALETTE_SYSTEM.md`), symbol resolution, and palette composition BEFORE testing any mapgen output.

---

### TRAP: Overmap Specials Without overmap_location

- **SYMPTOM:** Overmap special placement silently fails — the `is_valid()` check returns false for every candidate position. No buildings, no specials, no structures on the overmap.
- **SYSTEM YOU PORTED:** Overmap specials — the `overmap_special` type with terrain arrays and placement constraints.
- **SYSTEM YOU MISSED:** `overmap_location` — the eligibility filter that defines which overmap terrains a special can replace.
- **WHY IT'S HIDDEN:** `overmap_location` is referenced by a single field (`locations`) in `overmap_special`. It looks like a simple string reference, but it's a runtime filter that must be loaded, finalized, and queryable.
- **TIER:** 0
- **FIX:** Load `overmap_location` definitions before attempting overmap special placement. Each location defines allowed terrain types and flags.

---

### TRAP: Overmap Specials Without overmap_connection

- **SYMPTOM:** Buildings are placed on the overmap but never connect to the road network. Isolated buildings float in wilderness with no access paths.
- **SYSTEM YOU PORTED:** Overmap specials with terrain placement.
- **SYSTEM YOU MISSED:** `overmap_connection` — the system that generates road/trail/sewer connections between specials and the existing network.
- **WHY IT'S HIDDEN:** Connection data is a small sub-array in the overmap special definition. The special places correctly without connections — it just has no roads leading to it.
- **TIER:** 0
- **FIX:** Implement `overmap_connection` subtypes and the line-drawing algorithm that connects buildings to roads.

---

### TRAP: City Generation Without building_bin

- **SYMPTOM:** City generation produces empty street grids — roads exist but no buildings fill the lots. Or crash: building selection from empty pool.
- **SYSTEM YOU PORTED:** City generation pipeline — street network, lot subdivision.
- **SYSTEM YOU MISSED:** `building_bin` — the weighted selection container that `region_settings_city` uses to choose which building type fills each lot.
- **WHY IT'S HIDDEN:** `building_bin` is a helper type inside `region_settings`, not a top-level JSON type. It's populated during `region_settings` finalization, not during initial loading.
- **TIER:** 0
- **FIX:** Implement `building_bin` as part of `region_settings_city`. Ensure `finalize()` populates the building pools before city generation runs.

---

### TRAP: Overmap Terrain Without Directional Suffixes

- **SYMPTOM:** Rotated buildings crash or render as a single orientation. Line-drawing terrain (roads, rivers) has no bends or intersections — only straight segments.
- **SYSTEM YOU PORTED:** `oter_type_t` with base ID and symbol.
- **SYSTEM YOU MISSED:** The directional suffix system that generates 4 (rotatable) or 16 (line-drawing) variants from each base type.
- **WHY IT'S HIDDEN:** The base `oter_type_t` looks complete — it has an ID, symbol, color. But the actual `oter_t` instances used by the overmap are generated variants with suffixes like `_north`, `_east`, `_ns`, `_nesw`.
- **TIER:** 0
- **FIX:** Implement the suffix generation system documented in `04_OVERMAP_GENERATION.md`. Generate all variants during `oter_type_t::finalize()`.

---

### TRAP: Mapgen Without Nested Mapgen

- **SYMPTOM:** Nested chunk references fail silently. `place_nested` produces empty areas. Buildings with room-level variety (bedrooms, bathrooms, kitchens as nested chunks) have blank interior sections.
- **SYSTEM YOU PORTED:** Top-level JSON mapgen with `"rows"` and palettes.
- **SYSTEM YOU MISSED:** `nested_mapgen` — the reusable sub-chunk system that palette `"nested"` entries and `place_nested` depend on.
- **WHY IT'S HIDDEN:** Nested mapgen uses the same JSON structure as top-level mapgen, so it looks like "more of the same." But it has no `om_terrain` — it's keyed by a nested chunk ID, not an overmap terrain type.
- **TIER:** 0
- **FIX:** Load `nested_mapgen` entries into a separate registry keyed by ID. Implement `place_nested` to stamp nested chunks into the parent map.

---

### TRAP: Mapgen Without Rows + Palette

- **SYMPTOM:** Most buildings are featureless rectangles of `fill_ter`. Only buildings with C++ hardcoded mapgen or direct `"terrain"`/`"furniture"` placement keys render correctly.
- **SYSTEM YOU PORTED:** The mapgen dispatch system and terrain/furniture placement.
- **SYSTEM YOU MISSED:** The `"rows"` + palette combination that >90% of CDDA buildings use.
- **WHY IT'S HIDDEN:** Looking at mapgen JSON, `"rows"` is just one of many keys. It seems optional. In practice, virtually every building uses it.
- **TIER:** 0
- **FIX:** Implement `"rows"` parsing (24 strings × 24 characters) and palette symbol resolution as the PRIMARY mapgen path, not an alternative.

---

## TIER 1 — Empty / Dead Output

Data loads without errors, but map generation produces empty, unpopulated, or visually broken output.

---

### TRAP: Mapgen Without Monster Groups

- **SYMPTOM:** `place_monster` and `place_monsters` entries resolve to nothing. Buildings that should have zombies, animals, or hostile creatures are empty. The world is lifeless.
- **SYSTEM YOU PORTED:** Mapgen placement keys including `"monsters"`.
- **SYSTEM YOU MISSED:** `monstergroup` type — the weighted group definitions that `place_monsters` references by ID.
- **WHY IT'S HIDDEN:** The mapgen JSON just says `"monster": "GROUP_ZOMBIE"`. It looks like a simple spawn. But `GROUP_ZOMBIE` is a weighted distribution defined in a separate JSON file.
- **TIER:** 1
- **FIX:** Load `monstergroup` definitions. Resolve `place_monsters` references to actual creature spawns.

---

### TRAP: Mapgen Without Vehicle Groups

- **SYMPTOM:** `place_vehicles` entries resolve to nothing. Parking lots are empty. Roads have no wrecks. Gas stations have no cars.
- **SYSTEM YOU PORTED:** Mapgen placement keys including `"vehicles"`.
- **SYSTEM YOU MISSED:** `vehicle_group` type — the weighted distribution of vehicle prototypes that `place_vehicles` references.
- **WHY IT'S HIDDEN:** Vehicle groups are defined in separate JSON files. The mapgen reference is just a group ID string.
- **TIER:** 1
- **FIX:** Load vehicle group and vehicle prototype definitions. Resolve `place_vehicles` references. Note: full vehicle instantiation is a SCOPE BOUNDARY — but the group lookup must exist even if placement is stubbed.

---

### TRAP: Mapgen Without Trap Definitions

- **SYMPTOM:** `"traps"` palette entries and `place_traps` keys reference undefined trap IDs. Traps either don't appear or crash on lookup.
- **SYSTEM YOU PORTED:** Palette system with `"traps"` category, mapgen `place_traps` key.
- **SYSTEM YOU MISSED:** `trap` type definitions — the actual trap data (symbol, color, visibility, effects) that trap IDs resolve to.
- **WHY IT'S HIDDEN:** The palette just maps a character to a trap ID string. It works syntactically. But the string must resolve to a loaded `trap` definition for the tile to display anything.
- **TIER:** 1
- **FIX:** Load trap definitions from `data/json/traps.json`. Resolve trap ID strings to loaded data.

---

### TRAP: Mapgen Without Field Type Definitions

- **SYMPTOM:** `"fields"` palette entries and `place_fields` produce nothing. Fire, smoke, acid, and other environmental effects don't appear where mapgen intends them.
- **SYSTEM YOU PORTED:** Mapgen `place_fields` key, palette `"fields"` category.
- **SYSTEM YOU MISSED:** `field_type` definitions — the data that defines how fields look, behave, and decay.
- **WHY IT'S HIDDEN:** Field placement just references a field type ID string. Without loaded definitions, the ID has no meaning.
- **TIER:** 1
- **FIX:** Load `field_type` definitions from `data/json/field_type.json`. Fields must at minimum render (symbol + color at their intensity level).

---

### TRAP: Forest Mapgen Without Biome Components

- **SYMPTOM:** Forest overmap terrain generates but all forest tiles are uniform grass/dirt with no trees, bushes, or undergrowth. The forest is a featureless green plain.
- **SYSTEM YOU PORTED:** Forest overmap terrain placement, basic `fill_ter` for forest tiles.
- **SYSTEM YOU MISSED:** `forest_biome_component` and `forest_biome_mapgen` — the biome system that populates forest tiles with terrain features (trees, bushes, undergrowth) based on noise thresholds.
- **WHY IT'S HIDDEN:** Forest mapgen doesn't use the normal `"rows"` + palette system. It uses `region_settings_forest` with a completely separate biome-driven generation algorithm.
- **TIER:** 1
- **FIX:** Implement the forest biome generation pipeline documented in `05_REGION_SETTINGS.md`. Forest tiles must query `region_settings_forest` for biome data.

---

## TIER 2 — Static / Non-Interactive

World generates and looks approximately correct, but nothing responds to interaction.

---

### TRAP: Terrain Without bash_info

- **SYMPTOM:** Walls, doors, furniture — all indestructible. Players cannot break anything. Explosions have no environmental effect. The world is a museum.
- **SYSTEM YOU PORTED:** Terrain and furniture loading with `bash_info` fields.
- **SYSTEM YOU MISSED:** The bash resolution system — the runtime code that applies damage, checks thresholds, and executes terrain transforms.
- **WHY IT'S HIDDEN:** `bash_info` loads cleanly as data. It looks "done" in the data model. But without `map::bash()` or equivalent runtime code, the data is never queried.
- **TIER:** 2
- **FIX:** Implement basic bash: strength vs threshold → terrain/furniture transform. Item drops from `drop_group` can be stubbed.

---

### TRAP: Furniture Without Examine Actions

- **SYMPTOM:** All furniture is decorative. Beds can't be slept in. Water sources produce no water. Workbenches don't enable crafting. Signs have no text.
- **SYSTEM YOU PORTED:** Furniture loading with `examine_action` field.
- **SYSTEM YOU MISSED:** The `iexamine` dispatch system — the runtime code that maps action names to behavior functions.
- **WHY IT'S HIDDEN:** `examine_action` loads as a string. It looks like simple data. But the string must dispatch to actual behavior code.
- **TIER:** 2
- **FIX:** Implement a basic examine dispatch. Start with `"none"` (no effect) and `"sign"` (display text). Add other actions incrementally.

---

### TRAP: Terrain Without Connect Groups

- **SYMPTOM:** All walls display as `#`. All fences display as the same character. No visual connection between adjacent wall segments. Buildings look like ASCII rectangles instead of box-drawn rooms.
- **SYSTEM YOU PORTED:** Terrain loading with `connect_groups` and `connect_to_groups` fields.
- **SYSTEM YOU MISSED:** The runtime neighbor-analysis system that selects glyphs based on adjacent terrain.
- **WHY IT'S HIDDEN:** Connect group data loads cleanly. Walls have the right group IDs. But without the render-time lookup, the base symbol is used for every instance.
- **TIER:** 2
- **FIX:** Implement 4-neighbor analysis at render time. Build a 4-bit bitmask, look up the rotated symbol. See `09_SUPPORTING_SYSTEMS.md` §4.

---

### TRAP: Map Extras Using update_mapgen

- **SYMPTOM:** Map extras that use `update_mapgen` as their generator method silently fail. Post-generation additions (crashed helicopters, roadblocks, corpse scenes) that modify existing terrain don't appear.
- **SYSTEM YOU PORTED:** Map extras with function-based and mapgen-based generators.
- **SYSTEM YOU MISSED:** The `update_mapgen` path — map extras that MODIFY existing terrain rather than replacing it.
- **WHY IT'S HIDDEN:** Map extras have three generator methods (`map_extra_function`, `mapgen`, `update_mapgen`). The first two create terrain from scratch. The third modifies existing terrain and has a different application signature.
- **TIER:** 2
- **FIX:** Implement `update_mapgen` as a distinct code path that reads existing terrain before writing. See `06_LOCAL_MAPGEN.md` §Update Mapgen.

---

### TRAP: Overmap Specials Without city_building

- **SYMPTOM:** Cities have houses from the building_bin but no special buildings (banks, hospitals, police stations). The overmap special system works for wilderness placement but not for city-integrated buildings.
- **SYSTEM YOU PORTED:** Overmap specials as a standalone placement system.
- **SYSTEM YOU MISSED:** `city_building` — which is NOT a separate type but a dynamically created `overmap_special` from `oter_type_t` entries with `"flags": ["CITY_BUILDING"]`. These are integrated into city generation via `building_bin`, not the normal special placement pipeline.
- **WHY IT'S HIDDEN:** `city_building` appears in JSON as a separate concept but in code is constructed from `oter_type_t`. The mapping between city buildings and overmap specials is implicit.
- **TIER:** 2
- **FIX:** During `oter_type_t` finalization, generate `overmap_special` entries for types with `CITY_BUILDING` flag. Feed these into `building_bin`.

> **PORTING TRAP:** `city_building` is the most confusing type in CDDA's overmap system because it appears to be a separate JSON type but is actually synthesized from oter_type_t at finalization time.

---

### TRAP: Mapgen Without Region Settings

- **SYMPTOM:** No groundcover variation — every tile outside buildings is the same terrain. No biome variation — forests, fields, and swamps all look identical. Default fill terrain everywhere.
- **SYSTEM YOU PORTED:** Mapgen with terrain placement and palettes.
- **SYSTEM YOU MISSED:** `region_settings` — the invisible dependency that mapgen reads through `mapgendata` to select groundcover, forest features, and biome-appropriate terrain.
- **WHY IT'S HIDDEN:** Mapgen JSON never references region_settings by name. The connection is through `mapgendata`, which stores a `const region_settings&` obtained from the overmap. Mapgen functions call `mapgendata.region()` to get biome parameters.
- **TIER:** 2
- **FIX:** Implement `region_settings` loading and wire it into `mapgendata`. See `05_REGION_SETTINGS.md` for the full sub-type inventory.

> **PORTING TRAP:** The region_settings dependency is the single most invisible dependency in CDDA's mapgen system. Nothing in mapgen JSON hints at its existence. You must read the C++ to discover it.

---

### TRAP: Multi-Z Buildings Without Roof Generation

- **SYMPTOM:** Buildings have walls and floors on z=0 but no roof on z=1. Looking down from above shows open sky where roofs should be. Rain falls indoors.
- **SYSTEM YOU PORTED:** Mapgen for z=0 (ground floor).
- **SYSTEM YOU MISSED:** Roof generation — the automatic or explicit z+1 terrain that covers buildings.
- **WHY IT'S HIDDEN:** Many mapgen entries only define z=0. Roofs are either auto-generated from terrain flags or defined in separate mapgen entries for z=1.
- **TIER:** 2
- **FIX:** Implement roof terrain placement. Check for `roof` field in `ter_t` for auto-generation, or load z=1 mapgen entries for explicit roofs.

---

## TIER 3 — Atmospherically Wrong

The world is playable but feels artificial, static, or unrealistic.

---

### TRAP: Terrain Without Seasonal Variants

- **SYMPTOM:** Trees are always green. Grass never turns brown. Crops never visually mature. The world exists in perpetual summer.
- **SYSTEM YOU PORTED:** Terrain with base symbol and color.
- **SYSTEM YOU MISSED:** Season-specific symbol/color overrides in terrain definitions.
- **WHY IT'S HIDDEN:** Seasonal data is optional — most terrain works fine without it. The visual wrongness only becomes apparent after extended play.
- **TIER:** 3
- **FIX:** Implement season lookup at render time. Requires a world clock/calendar system.

---

### TRAP: Buildings Without Lighting Model

- **SYMPTOM:** Building interiors are fully visible from outside. There's no darkness indoors. Walls don't block vision. Entering a building is visually identical to standing outside.
- **SYSTEM YOU PORTED:** Terrain with `TRANSPARENT` flag.
- **SYSTEM YOU MISSED:** The FOV/LOS system that uses the transparency flag to determine visible tiles.
- **WHY IT'S HIDDEN:** The `TRANSPARENT` flag loads correctly. Walls have `TRANSPARENT: false`. But without a FOV algorithm that reads the flag, vision is omniscient.
- **TIER:** 3
- **FIX:** Implement FOV using the transparency cache. Binary (transparent vs opaque) is sufficient initially.

---

### TRAP: Terrain Without Field Propagation

- **SYMPTOM:** Fire is a static orange glyph that never spreads. A burning building stays as a single fire tile forever. Smoke never fills rooms. Gas never diffuses.
- **SYSTEM YOU PORTED:** Field types with `percent_spread`, `half_life`, intensity levels.
- **SYSTEM YOU MISSED:** The per-tick field processing system that actually executes spread and decay.
- **WHY IT'S HIDDEN:** Field type definitions describe HOW fields should behave. The definitions load cleanly. But without `map::process_fields()`, the behavior never executes.
- **TIER:** 3
- **FIX:** Implement field processing: decay (reduce intensity over time), spread (propagate to adjacent tiles based on `percent_spread`). See `09_SUPPORTING_SYSTEMS.md` §2.

---

## UNDERSCOPING TRAPS

Systems where the data model appears "done" but the runtime behavior is completely missing. The porting LLM may claim completion after loading data, not realizing the data is inert without its runtime system.

---

### TRAP: Field Types Loaded But No Propagation

- **SYMPTOM:** Fields are placed by mapgen and render at their initial intensity forever. No spread. No decay. No entity effects.
- **SYSTEM YOU PORTED:** `field_type` loading, field placement in mapgen.
- **SYSTEM YOU MISSED:** `map::process_fields()` — the per-tick simulation loop.
- **WHY IT'S HIDDEN:** The data model is complete and correct. Fields render. Everything looks "done" in a static screenshot. The failure only manifests over time — fire that should spread doesn't.
- **TIER:** Underscoping
- **FIX:** Implement the field processing loop as a simulation system. Even basic decay (intensity reduction over time) is better than static fields.

---

### TRAP: Trap Types Loaded But No Trigger Logic

- **SYMPTOM:** Traps are visible on tiles but stepping on them does nothing. Bear traps, land mines, pit traps — all purely decorative terrain.
- **SYSTEM YOU PORTED:** `trap` type loading, trap placement in mapgen.
- **SYSTEM YOU MISSED:** The trigger system — weight threshold checks, visibility detection, the `trap_function` dispatch that executes effects.
- **WHY IT'S HIDDEN:** Traps render correctly with their defined symbols and colors. They look "done." But without trigger logic, they're floor decorations.
- **TIER:** Underscoping
- **FIX:** Implement movement-triggered trap checks: when an entity enters a tile with a trap, check `trigger_weight` against entity weight, then dispatch `trap_function`.

---

### TRAP: Connect Groups Defined But No Neighbor Analysis

- **SYMPTOM:** Every wall tile displays the same base symbol regardless of neighbors. Buildings are `####` rectangles instead of box-drawn rooms.
- **SYSTEM YOU PORTED:** Terrain with `connect_groups` field loaded.
- **SYSTEM YOU MISSED:** The render-time neighbor analysis that builds a 4-bit bitmask and selects the rotated symbol.
- **WHY IT'S HIDDEN:** Connect group data loads cleanly. The data model is correct. But the rendering path uses the base symbol, not the connected variant.
- **TIER:** Underscoping
- **FIX:** At render time: check 4 cardinal neighbors for matching connect groups, build bitmask, look up rotated symbol.

---

### TRAP: Terrain Flags Loaded But No FOV System

- **SYMPTOM:** The `TRANSPARENT` flag exists on every terrain type but vision is not affected. Players see through walls.
- **SYSTEM YOU PORTED:** Terrain with all 139 flags including `TRANSPARENT`.
- **SYSTEM YOU MISSED:** The FOV/LOS algorithm that reads the transparency flag to determine visible tiles.
- **WHY IT'S HIDDEN:** Flags load as a bitset. `TRANSPARENT` is one of 139 flags. Nothing in the data loading code hints that this particular flag requires a rendering subsystem.
- **TIER:** Underscoping
- **FIX:** Implement FOV using `TRANSPARENT` flag. Shadowcasting or raycasting — algorithm choice is independent of the data model.

---

### TRAP: Seasonal Variants Defined But No World Clock

- **SYMPTOM:** Terrain has season-specific symbol/color data but the world is always in the default season. Variants never activate.
- **SYSTEM YOU PORTED:** Terrain with seasonal override fields.
- **SYSTEM YOU MISSED:** A `calendar` / world clock system that tracks the current season.
- **WHY IT'S HIDDEN:** The terrain data is complete with all four seasons. But nothing in the terrain loading code creates a clock.
- **TIER:** Underscoping
- **FIX:** Implement a basic calendar that tracks in-game days and derives seasons. Wire season into terrain rendering.

---

### TRAP: bash_info Loaded But No Damage System

- **SYMPTOM:** Terrain has full destruction data (`str_min`, `str_max`, `ter_set`) but nothing ever queries it. The world is indestructible.
- **SYSTEM YOU PORTED:** Terrain/furniture with `bash_info` struct fully loaded.
- **SYSTEM YOU MISSED:** `map::bash()` — the function that applies damage, checks thresholds, and transforms terrain.
- **WHY IT'S HIDDEN:** `bash_info` is a well-defined data struct. It loads, validates, and stores cleanly. The failure is that no code path ever calls the bash logic.
- **TIER:** Underscoping
- **FIX:** Implement a bash action: input (entity strength + tool bonus) → compare against `str_min`/`str_max` → transform terrain via `ter_set`/`furn_set`.

---

### TRAP: examine_action Names Loaded But No Dispatch

- **SYMPTOM:** Furniture has `examine_action: "workbench"` but examining it does nothing. All interaction is a no-op.
- **SYSTEM YOU PORTED:** Furniture with `examine_action` string field.
- **SYSTEM YOU MISSED:** The `iexamine` dispatch table mapping strings to C++ functions.
- **WHY IT'S HIDDEN:** `examine_action` is a string. It loads. It stores. It validates (it's just a string). But "workbench" must map to actual behavior code.
- **TIER:** Underscoping
- **FIX:** Create a dispatch table: `action_name → handler_function`. Start with `"none"` and `"sign"`, add incrementally.

---

### TRAP: Overmap Specials Loaded But No Placement Algorithm

- **SYMPTOM:** Overmap special definitions load correctly but the overmap is bare wilderness. No buildings, no structures, no specials anywhere.
- **SYSTEM YOU PORTED:** `overmap_special` type loading with terrain arrays and constraints.
- **SYSTEM YOU MISSED:** The placement algorithm — the 23-step overmap generation pipeline that scores positions, checks constraints, and places specials.
- **WHY IT'S HIDDEN:** The data model for specials is complex enough to feel like the whole system. But the data only describes WHAT can be placed — the WHERE and HOW require the placement algorithm.
- **TIER:** Underscoping
- **FIX:** Implement the overmap generation pipeline documented in `04_OVERMAP_GENERATION.md` §Overmap Generation Phase Order.

---

## WULFAZ-SPECIFIC ANTIPATTERNS

Traps specific to the Wulfaz project context — not about missing CDDA systems, but about porting mistakes.

---

### TRAP: Porting item_group Because Mapgen References It

- **SYMPTOM:** Weeks spent implementing CDDA's item system (item types, material system, item groups, spawn distributions) because mapgen's `place_items` references `item_group_id`.
- **SYSTEM YOU PORTED:** The entire item system.
- **SYSTEM YOU MISSED:** Nothing — you ported too much. Items are **EXCLUDED** from Wulfaz's scope.
- **WHY IT'S HIDDEN:** `place_items` is used in nearly every mapgen entry. It looks essential. But Wulfaz has its own item system that doesn't need CDDA's item types.
- **TIER:** Wulfaz-specific
- **FIX:** Stub `place_items`, `place_loot`, `sealed_item`, and `place_corpses` to no-ops. Mark item_group references as EXCLUDED. Do not load CDDA item data.

---

### TRAP: Mirroring C++ Class Hierarchies in Rust

- **SYMPTOM:** Fighting the borrow checker endlessly. Spaghetti lifetime annotations. `Arc<Mutex<>>` everywhere. Runtime panics from double borrows.
- **SYSTEM YOU PORTED:** CDDA's inheritance-heavy class hierarchy (`mapgen_function` base → builtin/json/nested/update subclasses).
- **SYSTEM YOU MISSED:** Nothing — the APPROACH is wrong. Wulfaz uses blackboard ECS with HashMap property tables, not OOP inheritance.
- **WHY IT'S HIDDEN:** CDDA's C++ code is the reference. It's natural to mirror its structure. But Rust's ownership model makes deep inheritance hierarchies painful.
- **TIER:** Wulfaz-specific
- **FIX:** Use Wulfaz's established pattern: data in HashMap property tables, behavior in plain functions. Convert CDDA's class hierarchy to an enum + data struct pattern. See `CLAUDE.md` architecture rules.

---

### TRAP: Porting CDDA Content

- **SYMPTOM:** Implementing specific CDDA building names (`house_01`), terrain IDs (`t_wall_w`), creature types (`mon_zombie`). All of which are wrong for Wulfaz's setting.
- **SYSTEM YOU PORTED:** CDDA content data (specific buildings, terrain variants, creature types).
- **SYSTEM YOU MISSED:** Nothing — the SCOPE is wrong. Wulfaz needs CDDA's SYSTEMS, not its CONTENT.
- **WHY IT'S HIDDEN:** The systems and content are intermixed in CDDA's JSON files. It's easy to port both when you only need the system.
- **TIER:** Wulfaz-specific
- **FIX:** Port the SYSTEM (palette loading, symbol resolution, composition logic). Create Wulfaz-specific content in KDL format with Wulfaz's own terrain IDs and building layouts.

---

### TRAP: Implementing JSON Parser for Mapgen

- **SYMPTOM:** Building a JSON parser for mapgen data files. Implementing `JsonObject`, `JsonArray`, CDDA's `mandatory`/`optional` pattern.
- **SYSTEM YOU PORTED:** CDDA's JSON data format.
- **SYSTEM YOU MISSED:** Nothing — the FORMAT is wrong. Wulfaz uses KDL, not JSON.
- **WHY IT'S HIDDEN:** The reference documents describe JSON structure extensively. It feels like JSON is required. But Wulfaz should express the same SCHEMA in KDL syntax.
- **TIER:** Wulfaz-specific
- **FIX:** Translate the data SCHEMA (what fields exist, what types they have, how they relate) to KDL format. Use the `kdl` crate for parsing. The schema is what matters, not the serialization format.

---

### TRAP: Implementing All iexamine Actions

- **SYMPTOM:** Implementing 50+ distinct examine action functions (workbench, water_source, bed, bulletin_board, autoclave, elevator, harvest_plant, piano...) before any of them are needed.
- **SYSTEM YOU PORTED:** All 50+ examine action implementations.
- **SYSTEM YOU MISSED:** Nothing — the SCOPE is too broad. Most actions are CDDA-setting-specific.
- **WHY IT'S HIDDEN:** The actions are listed in `iexamine.cpp` and it feels incomplete to stub them. But most actions (autoclave, cardreader, elevator) are specific to CDDA's post-apocalyptic setting and irrelevant to Wulfaz.
- **TIER:** Wulfaz-specific
- **FIX:** Implement the dispatch INTERFACE, then stub all actions to display furniture name/description. Add specific actions only when Wulfaz gameplay requires them.

---

### TRAP: Implementing Mutable Overmap Specials Before Fixed

- **SYMPTOM:** Weeks spent on the constraint-satisfaction solver for mutable specials (join points, rules, piece matching) when no fixed specials work yet.
- **SYSTEM YOU PORTED:** Mutable overmap specials — the complex system with joins and constraint satisfaction.
- **SYSTEM YOU MISSED:** Nothing — the ORDER is wrong. Fixed specials are 10× simpler and cover 90% of use cases.
- **WHY IT'S HIDDEN:** Mutable specials look more "general" and "correct." It feels like implementing the harder system first saves rework. In practice, fixed specials are sufficient for initial gameplay.
- **TIER:** Wulfaz-specific
- **FIX:** Implement fixed overmap specials first. Verify they place correctly with connections and locations. Only then consider mutable specials.

---

### TRAP: Full Road Network Generation Before Simple Grid Roads

- **SYMPTOM:** Implementing Bezier curves for highways, complex intersection algorithms, and terrain-aware pathfinding for road generation when no buildings exist on the overmap yet.
- **SYSTEM YOU PORTED:** CDDA's full highway/road generation system.
- **SYSTEM YOU MISSED:** Nothing — the COMPLEXITY is premature.
- **WHY IT'S HIDDEN:** CDDA's road generation is well-documented and appears as a prerequisite for city generation. But a simple grid-based road system is sufficient for initial testing.
- **TIER:** Wulfaz-specific
- **FIX:** Implement simple grid roads (straight lines between cities) first. Add curves, terrain avoidance, and highway specials after basic city generation works.

> **PORTING TRAP:** The seven Wulfaz-specific antipatterns above are not about missing systems — they're about doing too much, doing it in the wrong format, or doing it in the wrong order. The consuming LLM must resist the urge to "complete" CDDA's systems before verifying they're needed in Wulfaz's context.

---

## SUMMARY

| Tier | Count | Description |
|---|---|---|
| 0 — Won't Load | 7 | Immediate failures during loading or generation |
| 1 — Empty Output | 5 | Data loads but generation produces nothing |
| 2 — Non-Interactive | 7 | World generates but nothing responds to interaction |
| 3 — Atmospherically Wrong | 3 | Playable but feels artificial |
| Underscoping | 8 | Data model "done" but runtime missing |
| Wulfaz-Specific | 7 | Porting mistakes specific to this project |
| **Total** | **37** | |

### Reading Order

1. Read Tier 0 traps BEFORE starting any implementation
2. Read Wulfaz-specific traps BEFORE starting any implementation
3. Read Tier 1-2 traps before claiming any subsystem is "done"
4. Read Underscoping traps as a checklist when reviewing completed work
5. Read Tier 3 traps when planning post-MVP polish
