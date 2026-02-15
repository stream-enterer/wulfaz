# 00 — CDDA Map/Terrain System Dependency Graph

**Scope:** All map, terrain, building, and overmap generation systems relevant to the Wulfaz port.
**Purpose:** The consuming LLM reads this FIRST to understand what systems exist, how they connect, and what to load in what order.

---

## THE INVISIBLE DEPENDENCY PROBLEM

CDDA's map generation is NOT a self-contained system. It is a thin orchestration layer that coordinates **dozens** of interconnected type registries. Porting "mapgen" alone produces buildings that are empty shells — no furniture, no monsters, no traps, no field effects — because mapgen is just a coordinate system that references other types by string ID.

The failure mode is silent: code compiles, mapgen runs, buildings generate, but every `place_monsters` call finds no monstergroup definition, every palette symbol lookup for furniture finds nothing, and every `place_traps` reference resolves to null. The map looks like a grid of floor tiles with walls.

**But also:** some systems that LOOK necessary are **explicitly excluded** from the Wulfaz port. The most dangerous is `item_group` — mapgen JSON is littered with `"items"`, `"loot"`, and `"sealed_item"` placement keys that reference `item_group_id`. Implementing `item_group` to satisfy these references is wasted work because Wulfaz has a completely different setting and item system. The porting LLM must recognize these references, stub them as no-ops, and move on.

This graph distinguishes:
- **IN SCOPE** — required for the Wulfaz port
- **INTERFACE-ONLY** — the ID type and basic schema are needed so other systems can reference them, but internals are not ported
- **EXCLUDED** — exists in CDDA but explicitly not part of the Wulfaz port

---

## DEPENDENCY GRAPH

### connect_group
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none — self-contained definitions)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** terrain, furniture (connect_groups / connects_to / rotates_to bitset fields)
- **EXCLUDED CONNECTIONS:** (none)

### field_type
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** mapgen (place_fields), submap (field layer per tile), bash_info (hit_field, destroyed_field)
- **EXCLUDED CONNECTIONS:** (none)

### terrain (ter_t)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** connect_group (for connect_groups/connects_to/rotates_to bitsets)
- **SOFT DEPENDENCIES:** field_type (bash hit_field/destroyed_field), trap (trap_id_str field), harvest_list (harvest_by_season)
- **DEPENDENTS:** furniture, mapgen, palette, overmap_terrain (indirectly), construction, submap, ter_furn_transform, gate
- **EXCLUDED CONNECTIONS:** item types (base_item, liquid_source_item_id — references itype_id which is EXCLUDED), harvest_list (setting-specific)

### furniture (furn_t)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** connect_group
- **SOFT DEPENDENCIES:** field_type (via bash_info)
- **DEPENDENTS:** mapgen, palette, construction, submap, ter_furn_transform
- **EXCLUDED CONNECTIONS:** item types (crafting_pseudo_item, deployed_item, base_item — all itype_id, EXCLUDED)

### monstergroup
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** INTERFACE-ONLY

- **HARD DEPENDENCIES:** MONSTER types (EXCLUDED from scope except interface)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** mapgen (place_monsters), palette (monster symbol mappings)
- **EXCLUDED CONNECTIONS:** full MONSTER/SPECIES internals

### vehicle_group
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** INTERFACE-ONLY

- **HARD DEPENDENCIES:** vehicle definitions (EXCLUDED from scope except interface)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** mapgen (place_vehicles), palette (vehicle symbol mappings)
- **EXCLUDED CONNECTIONS:** full vehicle/vehicle_part internals

### trap
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none)
- **SOFT DEPENDENCIES:** field_type (some traps spawn fields on trigger)
- **DEPENDENTS:** terrain (trap_id_str field), mapgen (place_traps), submap (trap layer per tile)
- **EXCLUDED CONNECTIONS:** item types (some traps reference items)

### overmap_terrain (oter_type_t / oter_t)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none — self-contained, directional suffixes are derived at finalize)
- **SOFT DEPENDENCIES:** overmap_land_use_code (optional land_use_code field)
- **DEPENDENTS:** overmap_special (overmaps array), overmap_connection (subtypes reference terrain), city_building, mapgen (om_terrain string match), overmap generation, omt_placeholder
- **EXCLUDED CONNECTIONS:** (none)

### overmap_land_use_code
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none)
- **SOFT DEPENDENCIES:** overmap_terrain (provides alternate symbol)
- **DEPENDENTS:** overmap_terrain (optional land_use_code field)
- **EXCLUDED CONNECTIONS:** (none)

### overmap_connection
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_terrain (subtypes reference terrain IDs via oter_str_id)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** overmap_special (connections array), region_settings (overmap_connection inline), overmap generation (road routing)
- **EXCLUDED CONNECTIONS:** (none)

### overmap_location
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_terrain (location defines eligible terrain sets)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** overmap_special (locations array — placement eligibility)
- **EXCLUDED CONNECTIONS:** (none)

### city
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none)
- **SOFT DEPENDENCIES:** overmap_terrain (city placement operates on overmap terrain grid)
- **DEPENDENTS:** overmap generation (city center placement, road networks)
- **EXCLUDED CONNECTIONS:** (none)

### overmap_special
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_terrain (overmaps array entries), overmap_location (locations array), overmap_connection (connections array)
- **SOFT DEPENDENCIES:** mapgen (content for each OMT is generated via mapgen matching om_terrain)
- **DEPENDENTS:** city_building (same data structure, different placement), region_settings_city (building_bin), region_settings_highway (segment/intersection specials), overmap generation
- **EXCLUDED CONNECTIONS:** (none)

### overmap_special_migration
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_special
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** save/load (migration of old saves)
- **EXCLUDED CONNECTIONS:** (none)

### city_building
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_terrain, overmap_location, overmap_connection (same structure as overmap_special)
- **SOFT DEPENDENCIES:** mapgen (content generation)
- **DEPENDENTS:** region_settings_city (building_bin picks from city_building pool)
- **EXCLUDED CONNECTIONS:** (none)

> **PORTING TRAP:** city_building and overmap_special share the same data structure but are loaded by separate registrations ("city_building" vs "overmap_special"). City buildings are selected by the city generator from weighted pools in region_settings_city, NOT placed as standalone overmap specials. Missing city_building means cities have roads but no buildings.

### mapgen
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** terrain, furniture (for row ASCII mapping and fill_ter), palette (for symbol resolution)
- **SOFT DEPENDENCIES:** trap, field_type, monstergroup, vehicle_group, ter_furn_transform, nested_mapgen, update_mapgen
- **DEPENDENTS:** overmap_terrain (matched via om_terrain string), map_extra (generator_method: mapgen/update_mapgen)
- **EXCLUDED CONNECTIONS:** item_group (items/loot/sealed_item placement keys reference item_group_id — EXCLUDED)

> **SCOPE BOUNDARY:** mapgen JSON contains `"items"`, `"loot"`, and `"sealed_item"` placement keys that reference `item_group_id`. These are EXCLUDED from the Wulfaz port. The porting LLM should parse and ignore these keys (or skip them with a log warning), NOT implement item_group loading.

### palette
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** terrain, furniture (symbol mappings to ter_id/furn_id)
- **SOFT DEPENDENCIES:** trap, field_type, monstergroup, vehicle_group
- **DEPENDENTS:** mapgen (palettes array in JSON mapgen)
- **EXCLUDED CONNECTIONS:** item_group (palette symbol -> item mappings exist in CDDA but are EXCLUDED)

### nested_mapgen
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** mapgen (shares mapgen_function_json_base structure), palette
- **SOFT DEPENDENCIES:** (same as mapgen)
- **DEPENDENTS:** mapgen (place_nested references nested_mapgen_id), map_extra (generator_method: mapgen)
- **EXCLUDED CONNECTIONS:** item_group (same as mapgen)

### update_mapgen
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** mapgen (shares mapgen_function_json_base structure), palette
- **SOFT DEPENDENCIES:** (same as mapgen)
- **DEPENDENTS:** map_extra (generator_method: update_mapgen), missions (runtime map modifications)
- **EXCLUDED CONNECTIONS:** item_group (same as mapgen)

### map_extra
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none — self-contained definitions)
- **SOFT DEPENDENCIES:** nested_mapgen (when generator_method is "mapgen"), update_mapgen (when generator_method is "update_mapgen")
- **DEPENDENTS:** map_extra_collection, region_settings_map_extras
- **EXCLUDED CONNECTIONS:** (none)

### map_extra_collection
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** map_extra (weighted list references map_extra_id)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings_map_extras
- **EXCLUDED CONNECTIONS:** (none)

### omt_placeholder
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_terrain (resolves to actual terrain at finalize)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** overmap generation (placeholder resolution)
- **EXCLUDED CONNECTIONS:** (none)

### region_settings (top-level)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** All region_settings sub-types (city, forest, river, lake, ocean, ravine, highway, forest_trail, terrain_furniture, forest_mapgen, map_extras), weather_generator (link only)
- **SOFT DEPENDENCIES:** (none — but many fields are optional)
- **DEPENDENTS:** mapgendata (carries `const region_settings&` reference — every mapgen function has access), overmap generation
- **EXCLUDED CONNECTIONS:** weather_generator (link exists, full weather system not in scope)

### region_settings_city
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_special_id (via building_bin — houses, shops, parks)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings, city generation (building slot filling)
- **EXCLUDED CONNECTIONS:** (none)

### region_settings_forest / region_settings_forest_trail
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none for forest; forest_trail has building_bin -> overmap_special_id for trailheads)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings, overmap generation (forest/trail placement)
- **EXCLUDED CONNECTIONS:** (none)

### region_settings_forest_mapgen
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** forest_biome_mapgen (biomes set)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings (forest_composition field)
- **EXCLUDED CONNECTIONS:** (none)

### forest_biome_mapgen
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** forest_biome_component, terrain types (groundcover, terrain_dependent_furniture), overmap_terrain types (terrains set)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings_forest_mapgen
- **EXCLUDED CONNECTIONS:** item_group (item_group field for forest item spawns — EXCLUDED)

### forest_biome_component
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** terrain, furniture (ter_furn_id weighted list)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** forest_biome_mapgen (biome_components set)
- **EXCLUDED CONNECTIONS:** (none)

### region_settings_terrain_furniture
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** region_terrain_furniture (set of IDs)
- **SOFT DEPENDENCIES:** terrain, furniture (for resolution)
- **DEPENDENTS:** region_settings (REGION_PSEUDO terrain/furniture resolution)
- **EXCLUDED CONNECTIONS:** (none)

### region_terrain_furniture
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** terrain, furniture (concrete replacement lists)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings_terrain_furniture
- **EXCLUDED CONNECTIONS:** (none)

### region_settings_river / lake / ocean / ravine
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_terrain (lake references oter_str_id for surface/shore/interior/bed)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings, overmap generation (water body placement)
- **EXCLUDED CONNECTIONS:** (none)

### region_settings_highway
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** overmap_special_id (segment/intersection specials), overmap_terrain types (reserved terrain IDs), building_bin (intersection types)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings, highway generation
- **EXCLUDED CONNECTIONS:** (none)

### region_settings_map_extras
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** map_extra_collection
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** region_settings
- **EXCLUDED CONNECTIONS:** (none)

### construction
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** terrain, furniture (pre/post terrain/furniture IDs)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** (player-facing feature — no other systems depend on it for mapgen)
- **EXCLUDED CONNECTIONS:** item requirements (material costs reference item types — EXCLUDED)

### gate
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** terrain, furniture (gate open/close terrain references)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** iexamine actions (controls_gate examine function)
- **EXCLUDED CONNECTIONS:** (none)

### ter_furn_transform
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** terrain, furniture (transform source/target IDs)
- **SOFT DEPENDENCIES:** field_type (some transforms involve fields)
- **DEPENDENTS:** mapgen (ter_furn_transforms placement key), runtime terrain aging/decay
- **EXCLUDED CONNECTIONS:** (none)

### rotatable_symbol
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

- **HARD DEPENDENCIES:** (none)
- **SOFT DEPENDENCIES:** (none)
- **DEPENDENTS:** terrain/furniture rendering (symbol selection based on rotation context)
- **EXCLUDED CONNECTIONS:** (none)

### item_group
**Tier:** [EXCLUDED — NOT IN WULFAZ SCOPE]
**Wulfaz scope:** EXCLUDED

- **NOTE:** item_group is referenced by mapgen (items/loot/sealed_item placement keys), palette (symbol -> item mappings), bash_info/deconstruct_info (drop_group), and forest_biome_mapgen (item_group field). All of these references should be parsed-and-ignored or stubbed as no-ops. Do NOT implement item_group loading.

---

## DEPENDENCY DIAGRAM

```
                    +-----------------+
                    | connect_group   |
                    +--------+--------+
                             |
              +--------------+--------------+
              |                             |
      +-------v-------+            +-------v-------+
      |   terrain     |            |  furniture    |
      |   (ter_t)     |            |  (furn_t)     |
      +---+---+---+---+            +---+---+---+---+
          |   |   |                    |   |   |
          |   |   +----+               |   |   +----+
          |   |        |               |   |        |
    +-----v---v---+  +-v-----------+ +-v---v---+  +-v-----------+
    |  palette    |  | ter_furn_   | |  mapgen  |  | construct-  |
    |             |  | transform   | |          |  | ion         |
    +------+------+  +-------------+ +----+-----+  +-------------+
           |                              |
           +------------+-----------------+
                        |
                  +-----v------+       +-------------+
                  | overmap_   |       |   trap      |
                  | terrain    |       +------+------+
                  +---+--+-----+              |
                      |  |                    |
        +-------------+  +--------+     +-----v------+
        |                         |     | field_type |
  +-----v--------+         +-----v---+ +------------+
  | overmap_     |         | overmap_ |
  | location     |         | connect- |
  +------+-------+         | ion      |
         |                 +-----+----+
         |                       |
    +----v-----------+           |
    | overmap_       +-----------+
    | special        |
    +----+-----------+
         |
    +----v-----------+     +-----------------+
    | city_building  |     | region_settings |----> (all sub-types)
    +----+-----------+     +--------+--------+
         |                          |
    +----v-----------+     +--------v--------+
    | region_settings|     | mapgendata      |
    | _city          |     | (runtime)       |
    +----------------+     +-----------------+

  +-------------+    +----------------+
  | monstergroup|    | vehicle_group  |    INTERFACE-ONLY
  +------+------+    +-------+--------+
         |                   |
         +----->  mapgen  <--+

  +-------------+
  | item_group  |    EXCLUDED (referenced by mapgen but NOT ported)
  +-------------+
```

---

## DATA LOADING ORDER

Extracted from `src/init.cpp` `DynamicDataLoader::initialize()`. Only map/terrain-relevant types shown, in registration order. The add() order is the order types are registered for JSON loading; actual load order depends on which JSON files are encountered first, but the finalize order (below) enforces correct sequencing.

### Registration Order (init.cpp add() calls)

| # | Line | JSON type string | C++ loader | Wulfaz scope |
|---|------|-----------------|------------|-------------|
| 1 | 264 | (pre-init) | `init_mapdata()` — registers iexamine actors | IN |
| 2 | 275 | `"connect_group"` | `connect_group::load` | IN |
| 3 | 281 | `"field_type"` | `field_types::load` | IN |
| 4 | 311 | `"furniture"` | `load_furniture` | IN |
| 5 | 312 | `"terrain"` | `load_terrain` | IN |
| 6 | 313 | `"ter_furn_migration"` | `ter_furn_migrations::load` | IN |
| 7 | 314 | `"monstergroup"` | `MonsterGroupManager::LoadMonsterGroup` | INTERFACE-ONLY |
| 8 | 337 | `"item_group"` | `item_controller->load_item_group` | EXCLUDED |
| 9 | 347 | `"vehicle_part"` | `vehicles::parts::load` | INTERFACE-ONLY |
| 10 | 351 | `"vehicle"` | `vehicles::load_prototype` | INTERFACE-ONLY |
| 11 | 352 | `"vehicle_group"` | `VehicleGroup::load` | INTERFACE-ONLY |
| 12 | 353 | `"vehicle_placement"` | `VehiclePlacement::load` | INTERFACE-ONLY |
| 13 | 359 | `"trap"` | `trap::load_trap` | IN |
| 14 | 398 | `"oter_id_migration"` | `overmap::load_oter_id_migration` | IN |
| 15 | 400 | `"overmap_terrain"` | `overmap_terrains::load` | IN |
| 16 | 401 | `"oter_vision"` | `oter_vision::load_oter_vision` | IN |
| 17 | 402 | `"construction_category"` | `construction_categories::load` | IN |
| 18 | 403 | `"construction_group"` | `construction_groups::load` | IN |
| 19 | 404 | `"construction"` | `load_construction` | IN |
| 20 | 405 | `"mapgen"` | `load_mapgen` | IN |
| 21 | 406 | `"overmap_land_use_code"` | `overmap_land_use_codes::load` | IN |
| 22 | 407 | `"overmap_connection"` | `overmap_connections::load` | IN |
| 23 | 408 | `"overmap_location"` | `overmap_locations::load` | IN |
| 24 | 409 | `"city"` | `city::load_city` | IN |
| 25 | 410 | `"overmap_special"` | `overmap_specials::load` | IN |
| 26 | 411 | `"overmap_special_migration"` | `overmap_special_migration::load_migrations` | IN |
| 27 | 412 | `"city_building"` | `city_buildings::load` | IN |
| 28 | 413 | `"map_extra"` | `MapExtras::load` | IN |
| 29 | 414 | `"omt_placeholder"` | `map_data_placeholders::load` | IN |
| 30 | 416 | `"region_settings_river"` | `region_settings_river::load_region_settings_river` | IN |
| 31 | 417 | `"region_settings_lake"` | `region_settings_lake::load_region_settings_lake` | IN |
| 32 | 418 | `"region_settings_ocean"` | `region_settings_ocean::load_region_settings_ocean` | IN |
| 33 | 419 | `"region_settings_ravine"` | `region_settings_ravine::load_region_settings_ravine` | IN |
| 34 | 420 | `"region_settings_forest"` | `region_settings_forest::load_region_settings_forest` | IN |
| 35 | 421 | `"region_settings_highway"` | `region_settings_highway::load_region_settings_highway` | IN |
| 36 | 422 | `"region_settings_forest_trail"` | `region_settings_forest_trail::load_region_settings_forest_trail` | IN |
| 37 | 424 | `"region_settings_city"` | `region_settings_city::load_region_settings_city` | IN |
| 38 | 426 | `"region_settings_terrain_furniture"` | `region_settings_terrain_furniture::load_region_settings_terrain_furniture` | IN |
| 39 | 428 | `"region_terrain_furniture"` | `region_terrain_furniture::load_region_terrain_furniture` | IN |
| 40 | 430 | `"region_settings_forest_mapgen"` | `region_settings_forest_mapgen::load_region_settings_forest_mapgen` | IN |
| 41 | 432 | `"region_settings_map_extras"` | `region_settings_map_extras::load_region_settings_map_extras` | IN |
| 42 | 434 | `"forest_biome_component"` | `forest_biome_component::load_forest_biome_feature` | IN |
| 43 | 436 | `"forest_biome_mapgen"` | `forest_biome_mapgen::load_forest_biome_mapgen` | IN |
| 44 | 438 | `"map_extra_collection"` | `map_extra_collection::load_map_extra_collection` | IN |
| 45 | 440 | `"region_settings"` | `region_settings::load_region_settings` | IN |
| 46 | 470 | `"gate"` | `gates::load` | IN |
| 47 | 481 | `"palette"` | `mapgen_palette::load` | IN |
| 48 | 482 | `"rotatable_symbol"` | `rotatable_symbols::load` | IN |
| 49 | 494 | `"ter_furn_transform"` | `ter_furn_transform::load_transform` | IN |

### Finalize Order (init.cpp finalize phase)

Finalization resolves string IDs to int IDs, validates cross-references, and derives computed data. Key entries in order:

| # | Description | Function |
|---|------------|----------|
| 1 | Overmap terrain finalize | `overmap_terrains::finalize()` — generates directional variants (_north/_east/_south/_west) |
| 2 | Map extras finalize | `map_extra::finalize_all()` |
| 3 | Mapgen weights | `calculate_mapgen_weights()` |
| 4 | Mapgen parameters | `overmap_specials::finalize_mapgen_parameters()` |
| 5 | Overmap connections | `overmap_connections::finalize()` |
| 6 | Region settings | `region_settings::finalize_all()` |
| 7 | Terrain/furniture | `finalize_mapdata()` (resolves trap IDs, connect groups, etc.) |

### Consistency Check Order (init.cpp check phase)

| # | Check | Function |
|---|-------|----------|
| 1 | Mapgen definitions | `check_mapgen_definitions` |
| 2 | Mapgen palettes | `mapgen_palette::check_definitions` |
| 3 | Furniture and terrain | `check_furniture_and_terrain` |
| 4 | Constructions | `check_constructions` |
| 5 | Overmap land use codes | `overmap_land_use_codes::check_consistency` |
| 6 | Overmap connections | `overmap_connections::check_consistency` |
| 7 | Overmap terrain | `overmap_terrains::check_consistency` |
| 8 | Overmap locations | `overmap_locations::check_consistency` |
| 9 | Cities | `city::check_consistency` |
| 10 | Overmap specials | `overmap_specials::check_consistency` |
| 11 | Map extras | `MapExtras::check_consistency` |
| 12 | Traps | `trap::check_consistency` |
| 13 | Gates | `gates::check` |

---

## LOADING ORDER DEPENDENCIES

The registration order in init.cpp is NOT the actual constraint order — CDDA loads all JSON files in directory order and resolves cross-references at finalize time. However, certain finalization steps MUST happen before others:

1. **connect_group** must load before terrain/furniture (they reference connect_group IDs)
2. **terrain and furniture** must load before mapgen/palette (they reference ter_str_id/furn_str_id)
3. **overmap_terrain** must finalize before overmap_special/overmap_connection (they reference oter_str_id, and finalize generates directional variants)
4. **overmap_location** must load before overmap_special (specials reference locations for placement eligibility)
5. **overmap_connection** must load before overmap_special (specials reference connections for road routing)
6. **All region_settings sub-types** must load before region_settings (top-level references sub-type IDs)
7. **forest_biome_component** must load before forest_biome_mapgen (mapgen references components)
8. **forest_biome_mapgen** must load before region_settings_forest_mapgen (forest_mapgen references biomes)
9. **map_extra** must load before map_extra_collection (collections reference extras)
10. **map_extra_collection** must load before region_settings_map_extras
11. **palette** must load before mapgen finalization (mapgen resolves palette references)
12. **trap** must load before terrain finalization (terrain resolves trap_id_str to trap_id)

> **PORTING TRAP:** In Wulfaz (KDL-based loading), you must ensure this dependency order is respected. Load types in dependency tiers: Tier 0 first (connect_group, field_type, terrain, furniture), then Tier 1 (trap, monstergroup, vehicle_group, overmap_terrain, etc.), then Tier 2 and above.
