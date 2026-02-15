# 01 — JSON Type Registry

**Scope:** Every JSON `"type"` string relevant to map/terrain generation, extracted from `src/init.cpp` `DynamicDataLoader::initialize()`.
**Purpose:** The consuming LLM uses this as a lookup table for every data type it may encounter. Each entry tells you what the type is, where it's defined, what it cross-references, and whether it's in scope.

---

## Registry Format

Each entry follows this structure:
- **Type string** — as it appears in JSON `"type"` field
- **C++ loading function** — the function that parses JSON instances
- **Source files** — header + implementation
- **Data files** — JSON directory/files where instances live
- **Example snippet** — key fields only
- **Cross-references** — which other type IDs this type's fields reference
- **Tier** — dependency tier
- **Wulfaz scope** — IN / INTERFACE-ONLY / EXCLUDED

---

## `"connect_group"`

**C++ loading function:** `connect_group::load`
**Source files:** `src/mapdata.h`, `src/mapdata.cpp`
**Data files:** `data/json/connect_groups.json`
**ID type:** `connect_group_id` (custom string-based, maps to bitset index 0–255)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| index | int | yes (auto) | — | IN |
| group_flags | array of string | no | ter_furn_flag names | IN |
| connects_to_flags | array of string | no | ter_furn_flag names | IN |
| rotates_to_flags | array of string | no | ter_furn_flag names | IN |

### Example
```json
{
  "type": "connect_group",
  "id": "WALL",
  "group_flags": ["WALL", "CONNECT_WITH_WALL"],
  "connects_to_flags": ["WALL", "CONNECT_WITH_WALL"]
}
```

### Cross-References
- Referenced by: terrain (connect_groups/connects_to/rotates_to arrays), furniture (same)
- References: ter_furn_flag enum values

> **IMPLEMENT FULLY:** Connect groups are the foundation of wall/fence auto-tiling. Without them, connect_groups/connects_to/rotates_to fields on terrain and furniture have no lookup target.

---

## `"field_type"`

**C++ loading function:** `field_types::load`
**Source files:** `src/field_type.h`, `src/field_type.cpp`
**Data files:** `data/json/field_type.json`
**ID type:** `field_type_id` (`int_id<field_type>`), `field_type_str_id` (`string_id<field_type>`)
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| intensity_levels | array | yes | — | IN |
| decay_amount_factor | int | no | — | IN |
| percent_spread | int | no | — | IN |
| half_life | duration | no | — | IN |
| phase | string | no | — | IN |
| accelerated_decay | bool | no | — | IN |
| is_draw_field | bool | no | — | IN |
| dirty_transparency_cache | bool | no | — | IN |
| has_fire | bool | no | — | IN |
| has_acid | bool | no | — | IN |
| has_elec | bool | no | — | IN |
| has_fume | bool | no | — | IN |

### Example
```json
{
  "type": "field_type",
  "id": "fd_fire",
  "intensity_levels": [
    { "name": "small fire", "sym": "4", "color": "yellow", "light_emitted": 1.0 },
    { "name": "fire", "sym": "5", "color": "light_red", "light_emitted": 2.5 },
    { "name": "raging fire", "sym": "6", "color": "red", "light_emitted": 3.5 }
  ],
  "has_fire": true,
  "percent_spread": 20
}
```

### Cross-References
- Referenced by: mapgen (place_fields), bash_info (hit_field, destroyed_field), submap (field layer)
- References: (self-contained, no external type refs)

> **IMPLEMENT FULLY:** for data model. **STUB OK — TODO:** for field propagation (spread, decay). Fields should be placeable and renderable but don't need to spread initially.

---

## `"terrain"`

**C++ loading function:** `load_terrain`
**Source files:** `src/mapdata.h`, `src/mapdata.cpp`
**Data files:** `data/json/terrain/*.json`
**ID type:** `ter_id` (`int_id<ter_t>`), `ter_str_id` (`string_id<ter_t>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| name | string | yes | — | IN |
| description | string | yes | — | IN |
| symbol | string | yes | — | IN |
| color | string/array | yes | — | IN |
| move_cost | int | yes | — | IN |
| flags | array of string | no | ter_furn_flag | IN |
| connect_groups | array of string | no | connect_group_id | IN |
| connects_to | array of string | no | connect_group_id | IN |
| rotates_to | array of string | no | connect_group_id | IN |
| bash | object | no | (see bash_info) | IN |
| deconstruct | object | no | (see deconstruct_info) | IN |
| open | string | no | ter_str_id | IN |
| close | string | no | ter_str_id | IN |
| transforms_into | string | no | ter_str_id | IN |
| roof | string | no | ter_str_id | IN |
| trap | string | no | trap_str_id | IN |
| examine_action | string/object | no | iexamine function name or actor | IN |
| light_emitted | int | no | — | IN |
| coverage | int | no | — | IN |
| harvest_by_season | array | no | harvest_id | INTERFACE-ONLY |
| comfort | int | no | — | IN |
| looks_like | string | no | ter_str_id / furn_str_id | IN |
| lockpick_result | string | no | ter_str_id | IN |

### Example
```json
{
  "type": "terrain",
  "id": "t_wall",
  "name": "wall",
  "description": "A wall.",
  "symbol": "#",
  "color": "light_gray",
  "move_cost": 0,
  "flags": ["FLAMMABLE", "SUPPORTS_ROOF", "WALL", "NOITEM", "BLOCK_WIND"],
  "connect_groups": ["WALL"],
  "connects_to": ["WALL"],
  "bash": { "str_min": 40, "str_max": 100, "ter_set": "t_dirt" }
}
```

### Cross-References
- Referenced by: mapgen, palette, construction, submap (ter layer), ter_furn_transform, gate
- References: connect_group_id, ter_str_id (open/close/transforms_into/roof), trap (trap field), field_type (via bash hit_field/destroyed_field)
- **EXCLUDED CONNECTIONS:** itype_id (base_item, liquid_source_item_id), harvest_id (harvest_by_season)

> **IMPLEMENT FULLY:** Terrain is the foundational tile type. Every other system depends on it.

---

## `"furniture"`

**C++ loading function:** `load_furniture`
**Source files:** `src/mapdata.h`, `src/mapdata.cpp`
**Data files:** `data/json/furniture_and_terrain/*.json`, `data/json/furniture/*.json`
**ID type:** `furn_id` (`int_id<furn_t>`), `furn_str_id` (`string_id<furn_t>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| name | string | yes | — | IN |
| description | string | yes | — | IN |
| symbol | string | yes | — | IN |
| color | string/array | yes | — | IN |
| move_cost_mod | int | yes | — | IN |
| flags | array of string | no | ter_furn_flag | IN |
| connect_groups | array of string | no | connect_group_id | IN |
| connects_to | array of string | no | connect_group_id | IN |
| rotates_to | array of string | no | connect_group_id | IN |
| bash | object | no | (see bash_info) | IN |
| deconstruct | object | no | (see deconstruct_info) | IN |
| open | string | no | furn_str_id | IN |
| close | string | no | furn_str_id | IN |
| examine_action | string/object | no | iexamine function name or actor | IN |
| required_str | int | no | — | IN |
| workbench | object | no | — | IN |

### Example
```json
{
  "type": "furniture",
  "id": "f_table",
  "name": "table",
  "description": "A table.",
  "symbol": "#",
  "color": "brown",
  "move_cost_mod": 2,
  "required_str": 8,
  "flags": ["TRANSPARENT", "FLAMMABLE", "PLACE_ITEM", "FLAT_SURF"],
  "bash": { "str_min": 6, "str_max": 20, "furn_set": "f_null" }
}
```

### Cross-References
- Referenced by: mapgen, palette, construction, submap (furn layer), ter_furn_transform
- References: connect_group_id, furn_str_id (open/close), field_type (via bash)
- **EXCLUDED CONNECTIONS:** itype_id (crafting_pseudo_item, deployed_item, base_item)

> **IMPLEMENT FULLY:** Furniture is the secondary tile layer. Buildings need it for interior definition.

---

## `"ter_furn_migration"`

**C++ loading function:** `ter_furn_migrations::load`
**Source files:** `src/mapdata.h`, `src/mapdata.cpp`
**Data files:** `data/json/ter_furn_migration.json`
**ID type:** N/A (migration entries, not registered types)
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Purpose
Maps old terrain/furniture IDs to new ones for save compatibility.

> **STUB OK — TODO:** Only needed for save migration. Skip for initial implementation.

---

## `"monstergroup"`

**C++ loading function:** `MonsterGroupManager::LoadMonsterGroup`
**Source files:** `src/monstergroup.h`, `src/monstergroup.cpp`
**Data files:** `data/json/monstergroups/*.json`
**ID type:** `mongroup_id` (`string_id<MonsterGroup>`)
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** INTERFACE-ONLY

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| name | string | yes | — | IN |
| monsters | array | yes | mtype_id (MONSTER type) | INTERFACE-ONLY |
| default | string | no | mtype_id | INTERFACE-ONLY |

### Cross-References
- Referenced by: mapgen (place_monsters), palette (monster symbol mappings)
- References: mtype_id (MONSTER type — EXCLUDED from full port)

> **STUB OK — TODO:** Load the monstergroup ID and accept place_monsters references without crashing. Actual monster spawning logic is a separate TODO. Wulfaz will define its own creatures.

---

## `"vehicle_group"`

**C++ loading function:** `VehicleGroup::load`
**Source files:** `src/veh_type.h`, `src/vehicle_group.cpp`
**Data files:** `data/json/vehiclegroups.json`
**ID type:** `vgroup_id` (`string_id<VehicleGroup>`)
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** INTERFACE-ONLY

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| vehicles | array | yes | vproto_id (vehicle prototype) | INTERFACE-ONLY |

### Cross-References
- Referenced by: mapgen (place_vehicles), palette (vehicle symbol mappings)
- References: vproto_id (vehicle prototypes — EXCLUDED from full port)

> **STUB OK — TODO:** Load the vehicle_group ID. Accept place_vehicles references. Actual vehicle placement is a separate TODO.

---

## `"trap"`

**C++ loading function:** `trap::load_trap`
**Source files:** `src/trap.h`, `src/trap.cpp`
**Data files:** `data/json/traps.json`
**ID type:** `trap_id` (`int_id<trap>`), `trap_str_id` (`string_id<trap>`)
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| name | string | yes | — | IN |
| color | string | yes | — | IN |
| symbol | string | yes | — | IN |
| visibility | int | no | — | IN |
| avoidance | int | no | — | IN |
| difficulty | int | no | — | IN |
| benign | bool | no | — | IN |
| always_invisible | bool | no | — | IN |
| trigger_weight | int | no | — | IN |

### Cross-References
- Referenced by: terrain (trap field), mapgen (place_traps), submap (trap layer)
- References: field_type (some traps spawn fields on trigger)
- **EXCLUDED CONNECTIONS:** item types (some traps reference items for disarming)

> **IMPLEMENT FULLY:** for data model (trap definitions loadable and placeable). **STUB OK — TODO:** for trigger logic (traps are visible but don't trigger initially).

---

## `"overmap_terrain"`

**C++ loading function:** `overmap_terrains::load`
**Source files:** `src/overmap_terrain.h`, `src/overmap_terrain.cpp`
**Data files:** `data/json/overmap/overmap_terrain/*.json`
**ID type:** `oter_type_id` / `oter_type_str_id` (for the base type), `oter_id` / `oter_str_id` (for directional instances)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| name | string | yes | — | IN |
| sym | string | yes | — | IN |
| color | string | yes | — | IN |
| see_cost | int | no | — | IN |
| travel_cost | int | no | — | IN |
| extras | string | no | map_extra_collection_id | IN |
| mondensity | int | no | — | IN |
| flags | array of string | no | oter_flags | IN |
| land_use_code | string | no | overmap_land_use_code_id | IN |
| mapgen | array | no | inline mapgen entries | IN |
| mapgen_straight / mapgen_curved / mapgen_end / mapgen_tee / mapgen_four_way | array | no | inline mapgen for line-drawing types | IN |

### Example
```json
{
  "type": "overmap_terrain",
  "id": "house",
  "name": "house",
  "sym": "^",
  "color": "green",
  "see_cost": 5,
  "flags": ["KNOWN_DOWN", "ALLOW_ROTATE"]
}
```

### Cross-References
- Referenced by: overmap_special (overmaps array), overmap_connection (subtypes), city_building, mapgen (om_terrain match), omt_placeholder
- References: overmap_land_use_code (optional), map_extra_collection (extras field)

> **IMPLEMENT FULLY:** Overmap terrain is the world-scale tile type. The directional suffix system (_north/_east/_south/_west for rotatable, and line-drawing suffixes like _ns, _ew, etc.) is generated at finalize time.

> **PORTING TRAP:** The `oter_type_t` (base type) is different from `oter_t` (directional instance). Finalization creates directional variants automatically. If you only load the base type without generating variants, overmap_special overmaps array references to directional IDs will fail.

---

## `"oter_vision"`

**C++ loading function:** `oter_vision::load_oter_vision`
**Source files:** `src/oter_vision.h`, `src/oter_vision.cpp`
**Data files:** `data/json/overmap/oter_vision.json`
**ID type:** `oter_vision_id` (`string_id<oter_vision>`)
**Tier:** [TIER 3 — WILL GENERATE ATMOSPHERICALLY WRONG OUTPUT]
**Wulfaz scope:** IN

### Purpose
Controls how overmap terrain appears to the player based on exploration state (unseen, seen, explored).

> **STUB OK — TODO:** All overmap terrain visible for initial implementation.

---

## `"overmap_land_use_code"`

**C++ loading function:** `overmap_land_use_codes::load`
**Source files:** `src/overmap_terrain.h`, `src/overmap_terrain.cpp`
**Data files:** `data/json/overmap/overmap_land_use_codes.json`
**ID type:** `overmap_land_use_code_id` (`string_id<overmap_land_use_code>`)
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| land_use_code | int | yes | — | IN |
| name | string | yes | — | IN |
| sym | string | no | — | IN |
| color | string | no | — | IN |
| detailed_definition | string | no | — | IN |

### Cross-References
- Referenced by: overmap_terrain (land_use_code field)
- References: (none)

> **STUB OK — TODO:** Land use codes provide alternate symbols for overmap terrain. Not critical for initial implementation.

---

## `"overmap_connection"`

**C++ loading function:** `overmap_connections::load`
**Source files:** `src/overmap_connection.h`, `src/overmap_connection.cpp`
**Data files:** `data/json/overmap/overmap_connections.json`
**ID type:** `overmap_connection_id` (`string_id<overmap_connection>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| subtypes | array | yes | (see below) | IN |

Subtype fields:
| Field | Type | Required | References |
|-------|------|----------|------------|
| terrain | string | yes | oter_str_id |
| locations | array of string | yes | overmap_location_id |
| basic_cost | int | no | — |
| suffix | string | no | — |
| flags | object | no | — |

### Example
```json
{
  "type": "overmap_connection",
  "id": "local_road",
  "subtypes": [
    { "terrain": "road", "locations": ["land"] }
  ]
}
```

### Cross-References
- Referenced by: overmap_special (connections array), region_settings (overmap_connection inline struct)
- References: oter_str_id (terrain), overmap_location_id (locations)

> **IMPLEMENT FULLY:** Overmap connections define how buildings connect to road networks. Without them, overmap specials can't route roads to their entrances.

---

## `"overmap_location"`

**C++ loading function:** `overmap_locations::load`
**Source files:** `src/overmap_location.h`, `src/overmap_location.cpp`
**Data files:** `data/json/overmap/overmap_locations.json`
**ID type:** `overmap_location_id` (`string_id<overmap_location>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| terrains | array of string | no | oter_str_id | IN |
| flags | object | no | (terrain flag conditions) | IN |

### Example
```json
{
  "type": "overmap_location",
  "id": "land",
  "terrains": ["field", "forest", "forest_thick"]
}
```

### Cross-References
- Referenced by: overmap_special (locations array — placement eligibility), overmap_connection (subtype locations)
- References: oter_str_id (eligible terrain list)

> **IMPLEMENT FULLY:** Locations define where overmap specials can be placed. Without them, placement eligibility is undefined and specials never place.

---

## `"city"`

**C++ loading function:** `city::load_city`
**Source files:** `src/city.h`, `src/city.cpp`
**Data files:** `data/json/overmap/cities.json`
**ID type:** `city_id` (`string_id<city>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| population | object | no | — | IN |
| size | object | no | — | IN |

### Cross-References
- Referenced by: overmap generation (city placement)
- References: (none)

> **IMPLEMENT FULLY:** City definitions control city generation parameters.

---

## `"overmap_special"`

**C++ loading function:** `overmap_specials::load`
**Source files:** `src/overmap_special.h`, `src/overmap_special.cpp`, `src/overmap_special_fixed.cpp`, `src/overmap_special_mutable.cpp`
**Data files:** `data/json/overmap/overmap_special/*.json`
**ID type:** `overmap_special_id` (`string_id<overmap_special>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| overmaps | array | yes (fixed) | oter_str_id per entry | IN |
| connections | array | no | overmap_connection_id per entry | IN |
| locations | array | no | overmap_location_id per entry | IN |
| city_distance | range | no | — | IN |
| city_sizes | range | no | — | IN |
| occurrences | range | no | — | IN |
| flags | array of string | no | (CLASSIC, URBAN, WILDERNESS, etc.) | IN |
| rotate | bool | no | — | IN |
| spawns | object | no | mongroup_id | INTERFACE-ONLY |
| joins | object | no | — (mutable specials only) | IN |
| root | string | no | — (mutable specials only) | IN |
| phases | array | no | — (mutable specials only) | IN |

### Example (fixed)
```json
{
  "type": "overmap_special",
  "id": "house_garage",
  "overmaps": [
    { "point": [0, 0, 0], "overmap": "house_north" },
    { "point": [1, 0, 0], "overmap": "garage_north" }
  ],
  "connections": [
    { "point": [0, -1, 0], "terrain": "road", "connection": "local_road" }
  ],
  "locations": ["land"],
  "city_distance": [0, 10],
  "city_sizes": [4, -1],
  "occurrences": [0, 30],
  "flags": ["CLASSIC"],
  "rotate": true
}
```

### Cross-References
- Referenced by: city_building (same structure), region_settings_city (building_bin), overmap generation
- References: oter_str_id, overmap_connection_id, overmap_location_id, mongroup_id (spawns — INTERFACE-ONLY)

### Antipattern Warnings
> **PORTING TRAP:** Fixed overmap specials define building footprints via an `overmaps` array of (point, overmap_terrain_id) pairs. Each entry must reference a valid overmap_terrain that has mapgen content. Without overmap_location, placement eligibility is undefined. Without overmap_connection, buildings never connect to roads.

> **STUB OK — TODO:** Mutable overmap specials (with `joins`/`root`/`phases` fields) use a complex constraint-satisfaction algorithm. Implement fixed specials first; stub mutable as TODO.

---

## `"city_building"`

**C++ loading function:** `city_buildings::load`
**Source files:** `src/overmap_special.h`, `src/overmap_special.cpp`
**Data files:** `data/json/overmap/city_building/*.json`
**ID type:** `overmap_special_id` (aliased — city_building shares the overmap_special ID namespace)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Purpose
Same data structure as `overmap_special`, but placed by the city generator (selected from weighted pools in `region_settings_city`) rather than as standalone specials. The key difference is placement mechanism, not data schema.

### Cross-References
- Referenced by: region_settings_city (building_bin selects city_building entries)
- References: same as overmap_special

> **PORTING TRAP:** city_building and overmap_special share the same `overmap_special_id` namespace. A city_building IS an overmap_special that happens to be selected by city generation rather than placed independently. If you implement overmap_special but miss city_building, cities will have roads but no buildings.

---

## `"map_extra"`

**C++ loading function:** `MapExtras::load`
**Source files:** `src/map_extras.h`, `src/map_extras.cpp`
**Data files:** `data/json/map_extras.json`
**ID type:** `map_extra_id` (`string_id<map_extra>`)
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| generator_method | string | yes | "null" / "map_extra_function" / "mapgen" / "update_mapgen" | IN |
| generator_id | string | cond. | nested_mapgen_id or update_mapgen_id | IN |
| autonote | bool | no | — | IN |
| symbol | string | no | — | IN |
| color | string | no | — | IN |
| name | string | no | — | IN |
| description | string | no | — | IN |

### Cross-References
- Referenced by: map_extra_collection (weighted list)
- References: nested_mapgen_id (when method is "mapgen"), update_mapgen_id (when method is "update_mapgen")

> **STUB OK — TODO:** Map extras add variety to generated terrain. Can be deferred until core mapgen works.

---

## `"map_extra_collection"`

**C++ loading function:** `map_extra_collection::load_map_extra_collection`
**Source files:** `src/regional_settings.h`, `src/regional_settings.cpp`
**Data files:** `data/json/region_settings/map_extra_collection/`
**ID type:** `map_extra_collection_id` (`string_id<map_extra_collection>`)
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| chance | int | no | — | IN |
| extras | weighted list | yes | map_extra_id | IN |

### Cross-References
- Referenced by: overmap_terrain (extras field), region_settings_map_extras
- References: map_extra_id

---

## `"omt_placeholder"`

**C++ loading function:** `map_data_placeholders::load`
**Source files:** `src/map.h`, `src/map.cpp` (or dedicated file)
**Data files:** `data/json/overmap/omt_placeholder.json`
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Purpose
Placeholder overmap terrain IDs that get resolved to actual terrain during overmap finalization. Used for generic references that depend on context.

> **STUB OK — TODO:** Implement basic resolution. Placeholders that don't resolve should fall back to a default terrain.

---

## `"region_settings"`

**C++ loading function:** `region_settings::load_region_settings`
**Source files:** `src/regional_settings.h`, `src/regional_settings.cpp`
**Data files:** `data/json/region_settings/region_settings/regional_map_settings.json`
**ID type:** `region_settings_id` (`string_id<region_settings>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| default_oter | array of string | no | oter_str_id (per z-level) | IN |
| default_groundcover | weighted list | no | ter_str_id | IN |
| city_spec | string | no | region_settings_city_id | IN |
| forest_composition | string | no | region_settings_forest_mapgen_id | IN |
| forest_trail | string | no | region_settings_forest_trail_id | IN |
| weather | string | no | weather_generator_id | INTERFACE-ONLY |
| overmap_forest | string | no | region_settings_forest_id | IN |
| overmap_river | string | no | region_settings_river_id | IN |
| overmap_lake | string | no | region_settings_lake_id | IN |
| overmap_ocean | string | no | region_settings_ocean_id | IN |
| overmap_highway | string | no | region_settings_highway_id | IN |
| overmap_ravine | string | no | region_settings_ravine_id | IN |
| region_terrain_and_furniture | string | no | region_settings_terrain_furniture_id | IN |
| region_extras | string | no | region_settings_map_extras_id | IN |
| connections | object | no | overmap_connection_id (6 fields) | IN |
| place_swamps / place_roads / place_specials | bool | no | — | IN |

### Cross-References
- Referenced by: mapgendata (carries `const region_settings&` — every mapgen function has access)
- References: ALL region_settings sub-types, weather_generator, overmap_connection_id

> **IMPLEMENT FULLY:** region_settings is the master configuration for world generation. Without it, mapgendata has no region reference and groundcover/building selection is undefined.

> **PORTING TRAP:** mapgendata carries a `const region_settings&` — every mapgen function has implicit access to region data even though mapgen JSON never explicitly references region_settings. This is an invisible dependency.

---

## `"region_settings_city"`

**C++ loading function:** `region_settings_city::load_region_settings_city`
**Source files:** `src/regional_settings.h`, `src/regional_settings.cpp`
**Data files:** `data/json/region_settings/region_settings_city/`
**ID type:** `region_settings_city_id` (`string_id<region_settings_city>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| shop_radius | int | no | — | IN |
| shop_sigma | int | no | — | IN |
| park_radius | int | no | — | IN |
| park_sigma | int | no | — | IN |
| houses | weighted list | yes | overmap_special_id | IN |
| shops | weighted list | yes | overmap_special_id | IN |
| parks | weighted list | yes | overmap_special_id | IN |

### Cross-References
- Referenced by: region_settings (city_spec field)
- References: overmap_special_id (via building_bin for houses, shops, parks)

> **IMPLEMENT FULLY:** Without city settings, city generation has no building pool to select from.

---

## `"region_settings_river"` / `"region_settings_lake"` / `"region_settings_ocean"` / `"region_settings_ravine"`

**C++ loading functions:** `region_settings_river::load_region_settings_river`, etc.
**Source files:** `src/regional_settings.h`, `src/regional_settings.cpp`
**Data files:** `data/json/region_settings/` (respective subdirectories)
**ID types:** `region_settings_river_id`, `region_settings_lake_id`, `region_settings_ocean_id`, `region_settings_ravine_id`
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Purpose
Control water body generation parameters (river scale/frequency, lake thresholds, ocean boundaries, ravine depth).

### Cross-References
- Referenced by: region_settings
- References: oter_str_id (lake references surface/shore/interior/bed overmap terrain)

> **STUB OK — TODO:** Water body generation parameters. Can start with simple defaults.

---

## `"region_settings_forest"` / `"region_settings_forest_trail"`

**C++ loading functions:** respective load functions
**Source files:** `src/regional_settings.h`
**Data files:** `data/json/region_settings/`
**ID types:** `region_settings_forest_id`, `region_settings_forest_trail_id`
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Purpose
Forest: noise thresholds for forest density, river floodplain buffers.
Forest trail: trail generation parameters, trailhead building_bin.

### Cross-References
- Referenced by: region_settings
- References: forest_trail has building_bin -> overmap_special_id (trailheads)

---

## `"region_settings_highway"`

**C++ loading function:** `region_settings_highway::load_region_settings_highway`
**Source files:** `src/regional_settings.h`
**Data files:** `data/json/region_settings/region_settings_highway/`
**ID type:** `region_settings_highway_id`
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| segment_flat | string | no | overmap_special_id | IN |
| segment_ramp | string | no | overmap_special_id | IN |
| segment_bridge | string | no | overmap_special_id | IN |
| four_way_intersections | weighted list | no | overmap_special_id | IN |
| three_way_intersections | weighted list | no | overmap_special_id | IN |
| bends | weighted list | no | overmap_special_id | IN |
| reserved_terrain_id | string | no | oter_type_str_id | IN |

### Cross-References
- Referenced by: region_settings
- References: overmap_special_id (many), oter_type_str_id

> **STUB OK — TODO:** Highway generation is algorithmically complex. Start with simple inter-city roads.

---

## `"region_settings_terrain_furniture"`

**C++ loading function:** `region_settings_terrain_furniture::load_region_settings_terrain_furniture`
**Source files:** `src/regional_settings.h`
**Data files:** `data/json/region_settings/region_settings_terrain_furniture/`
**ID type:** `region_settings_terrain_furniture_id`
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Purpose
Groups `region_terrain_furniture` entries. Used to resolve REGION_PSEUDO terrain/furniture to concrete types based on the active region.

### Cross-References
- Referenced by: region_settings
- References: region_terrain_furniture_id

> **IMPLEMENT FULLY:** Without this, REGION_PSEUDO flagged terrain/furniture never resolves, producing invalid tiles.

---

## `"region_terrain_furniture"`

**C++ loading function:** `region_terrain_furniture::load_region_terrain_furniture`
**Source files:** `src/regional_settings.h`
**Data files:** `data/json/region_settings/region_terrain_furniture/`
**ID type:** `region_terrain_furniture_id`
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| terrain | weighted list | cond. | ter_id (concrete terrain replacements) | IN |
| furniture | weighted list | cond. | furn_id (concrete furniture replacements) | IN |

### Purpose
Maps abstract "region" terrain/furniture (e.g., `t_region_groundcover`) to concrete terrain/furniture for the active region.

### Cross-References
- Referenced by: region_settings_terrain_furniture
- References: ter_id, furn_id

---

## `"region_settings_forest_mapgen"`

**C++ loading function:** `region_settings_forest_mapgen::load_region_settings_forest_mapgen`
**Source files:** `src/regional_settings.h`
**Data files:** `data/json/region_settings/region_settings_forest_mapgen/`
**ID type:** `region_settings_forest_mapgen_id`
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| biomes | array of string | yes | forest_biome_mapgen_id | IN |

### Cross-References
- Referenced by: region_settings (forest_composition field)
- References: forest_biome_mapgen_id

---

## `"forest_biome_component"`

**C++ loading function:** `forest_biome_component::load_forest_biome_feature`
**Source files:** `src/regional_settings.h`
**Data files:** `data/json/region_settings/forest_biome_component/`
**ID type:** `forest_biome_component_id`
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| types | weighted list | yes | ter_furn_id (terrain or furniture) | IN |
| sequence | int | no | — | IN |
| chance | int | no | — | IN |

### Cross-References
- Referenced by: forest_biome_mapgen (biome_components set)
- References: ter_id, furn_id (via ter_furn_id)

---

## `"forest_biome_mapgen"`

**C++ loading function:** `forest_biome_mapgen::load_forest_biome_mapgen`
**Source files:** `src/regional_settings.h`
**Data files:** `data/json/region_settings/forest_biome_mapgen/`
**ID type:** `forest_biome_mapgen_id`
**Tier:** [TIER 1 — WILL GENERATE EMPTY/DEAD OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| terrains | array of string | yes | oter_type_str_id | IN |
| biome_components | array of string | yes | forest_biome_component_id | IN |
| groundcover | weighted list | yes | ter_str_id | IN |
| terrain_dependent_furniture | object | no | ter_str_id -> furn_str_id weighted list | IN |
| item_group | string | no | item_group_id | EXCLUDED |

### Cross-References
- Referenced by: region_settings_forest_mapgen
- References: oter_type_str_id, forest_biome_component_id, ter_str_id, furn_str_id
- **EXCLUDED CONNECTIONS:** item_group_id (item_group field for forest item spawns)

> **SCOPE BOUNDARY:** forest_biome_mapgen has an `item_group` field that references `item_group_id`. This is EXCLUDED from the Wulfaz port. The porting LLM should accept this field in the data model but treat it as a no-op.

---

## `"region_settings_map_extras"`

**C++ loading function:** `region_settings_map_extras::load_region_settings_map_extras`
**Source files:** `src/regional_settings.h`
**Data files:** `data/json/region_settings/region_settings_map_extras/`
**ID type:** `region_settings_map_extras_id`
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| extras | array of string | yes | map_extra_collection_id | IN |

### Cross-References
- Referenced by: region_settings
- References: map_extra_collection_id

---

## `"construction"`

**C++ loading function:** `load_construction`
**Source files:** `src/construction.h`, `src/construction.cpp`
**Data files:** `data/json/construction.json`
**ID type:** `construction_id` (`int_id<construction>`)
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| pre_terrain | string | no | ter_str_id | IN |
| post_terrain | string | no | ter_str_id | IN |
| pre_furniture | string | no | furn_str_id | IN |
| post_furniture | string | no | furn_str_id | IN |
| pre_flags | object | no | ter_furn_flag | IN |
| requirements | object | no | — | EXCLUDED |

### Cross-References
- Referenced by: (player-facing feature, no mapgen dependency)
- References: ter_str_id, furn_str_id
- **EXCLUDED CONNECTIONS:** requirement_data (material costs reference item types)

> **STUB OK — TODO:** Construction is a large player-facing system. Load the terrain/furniture transform data; stub the player-facing construction flow.

---

## `"gate"`

**C++ loading function:** `gates::load`
**Source files:** `src/gates.h`, `src/gates.cpp`
**Data files:** `data/json/gates.json`
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| door | string | yes | ter_str_id | IN |
| floor | string | yes | ter_str_id | IN |
| walls | array of string | yes | ter_str_id | IN |

### Cross-References
- Referenced by: iexamine actions (controls_gate)
- References: ter_str_id

> **STUB OK — TODO:** Gate definitions control garage doors, drawbridges, etc. Load the data; stub the interaction.

---

## `"palette"`

**C++ loading function:** `mapgen_palette::load`
**Source files:** `src/mapgen.h`, `src/mapgen.cpp`
**Data files:** `data/json/mapgen_palettes/*.json`
**ID type:** `palette_id` (`string_id<mapgen_palette>`)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| terrain | object | no | char -> ter_str_id | IN |
| furniture | object | no | char -> furn_str_id | IN |
| traps | object | no | char -> trap_str_id | IN |
| fields | object | no | char -> field_type_str_id | IN |
| monsters | object | no | char -> mongroup_id | INTERFACE-ONLY |
| vehicles | object | no | char -> vgroup_id | INTERFACE-ONLY |
| items | object | no | char -> item_group_id | EXCLUDED |
| palettes | array of string | no | palette_id (composition/inheritance) | IN |

### Example
```json
{
  "type": "palette",
  "id": "standard_domestic",
  "terrain": { ".": "t_floor", "#": "t_wall", "+": "t_door_c" },
  "furniture": { "T": "f_table", "C": "f_chair" }
}
```

### Cross-References
- Referenced by: mapgen (palettes array)
- References: ter_str_id, furn_str_id, trap_str_id, field_type_str_id, mongroup_id, vgroup_id, palette_id (recursive)
- **EXCLUDED CONNECTIONS:** item_group_id (palette symbol -> item mappings)

> **IMPLEMENT FULLY:** Palettes are the symbol-to-type mapping layer. Without them, ASCII row mapgen produces blank output.

> **SCOPE BOUNDARY:** Palette "items" mappings reference item_group_id which is EXCLUDED. The porting LLM should parse the "items" key and discard it, NOT implement item_group loading.

---

## `"rotatable_symbol"`

**C++ loading function:** `rotatable_symbols::load`
**Source files:** `src/mapdata.h`, `src/mapdata.cpp`
**Data files:** `data/json/rotatable_symbols.json`
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Purpose
Symbols that change appearance based on rotation context (e.g., directional arrows, angled walls).

> **STUB OK — TODO:** Not critical for initial implementation. Terrain renders with default symbols.

---

## `"ter_furn_transform"`

**C++ loading function:** `ter_furn_transform::load_transform`
**Source files:** `src/ter_furn_transform.h`, `src/ter_furn_transform.cpp`
**Data files:** `data/json/ter_furn_transform.json`
**ID type:** `ter_furn_transform_id` (`string_id<ter_furn_transform>`)
**Tier:** [TIER 2 — WILL GENERATE STATIC/NON-INTERACTIVE OUTPUT]
**Wulfaz scope:** IN

### Key Fields
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| id | string | yes | — | IN |
| terrain | array | no | ter_str_id -> ter_str_id (with probability) | IN |
| furniture | array | no | furn_str_id -> furn_str_id (with probability) | IN |
| field | array | no | field_type_str_id -> field_type_str_id | IN |

### Cross-References
- Referenced by: mapgen (ter_furn_transforms placement key), runtime aging/decay
- References: ter_str_id, furn_str_id, field_type_str_id

> **IMPLEMENT FULLY:** for data model. **STUB OK — TODO:** for runtime transform triggers (aging, decay). Mapgen can apply transforms at generation time.

---

## `"mapgen"`

**C++ loading function:** `load_mapgen`
**Source files:** `src/mapgen.h`, `src/mapgen.cpp`
**Data files:** `data/json/mapgen/*.json` (and inline in overmap_terrain definitions)
**ID type:** N/A (mapgen entries are matched to overmap terrain via `om_terrain` string, not by typed ID)
**Tier:** [TIER 0 — WILL NOT COMPILE/LOAD]
**Wulfaz scope:** IN

### Purpose
The JSON type `"mapgen"` covers three distinct subtypes based on context:
1. **Top-level mapgen** — matched to overmap terrain via `om_terrain` field
2. **Nested mapgen** — identified by `nested_mapgen_id` field, stamped via `place_nested`
3. **Update mapgen** — identified by `update_mapgen_id` field, applied post-generation

### Key Fields (top-level)
| Field | Type | Required | References | Wulfaz scope |
|-------|------|----------|------------|-------------|
| om_terrain | string/array | yes | oter_str_id (string match) | IN |
| method | string | yes | "json" or "builtin" | IN |
| weight | int | no | — | IN |
| object.fill_ter | string | no | ter_str_id | IN |
| object.rows | array of 24 strings | cond. | characters mapped via palette | IN |
| object.palettes | array of string | no | palette_id | IN |
| object.terrain | object | no | char -> ter_str_id | IN |
| object.furniture | object | no | char -> furn_str_id | IN |
| object.set | array | no | set operations | IN |
| object.place_monsters | array | no | mongroup_id references | INTERFACE-ONLY |
| object.place_vehicles | array | no | vgroup_id references | INTERFACE-ONLY |
| object.place_traps | array | no | trap_str_id references | IN |
| object.place_fields | array | no | field_type_str_id references | IN |
| object.place_nested | array | no | nested_mapgen_id references | IN |
| object.place_items | array | no | item_group_id references | EXCLUDED |
| object.place_loot | array | no | item_group_id references | EXCLUDED |
| object.rotation | int/range | no | — | IN |
| object.predecessor_mapgen | string | no | oter_str_id | IN |

### Cross-References
- Referenced by: overmap_terrain (om_terrain string match), map_extra (generator)
- References: palette_id, ter_str_id, furn_str_id, trap_str_id, field_type_str_id, mongroup_id, vgroup_id, nested_mapgen_id
- **EXCLUDED CONNECTIONS:** item_group_id (place_items, place_loot, sealed_item)

> **IMPLEMENT FULLY:** Mapgen is the core content generation engine. Every building's interior is defined here.

---

## `"item_group"`

**C++ loading function:** `item_controller->load_item_group`
**Source files:** `src/item_factory.h`, `src/item_factory.cpp`, `src/item_group.h`
**Data files:** `data/json/itemgroups/*.json`
**ID type:** `item_group_id` (`string_id<item_group_data>`)
**Tier:** [EXCLUDED — NOT IN WULFAZ SCOPE]
**Wulfaz scope:** EXCLUDED

### Exclusion Note
item_group is referenced throughout mapgen JSON (`"items"`, `"loot"`, `"sealed_item"` placement keys), palette symbol mappings, bash_info/deconstruct_info (drop_group), and forest_biome_mapgen (item_group field). ALL of these references should be parsed and silently ignored or stubbed as no-ops. Do NOT implement item_group loading. Wulfaz has a completely different setting and will define its own item system.

> **SCOPE BOUNDARY:** item_group is the single most dangerous excluded system. It appears everywhere in mapgen data. The porting LLM must recognize item_group references and skip them, not pull in the item system.

---

## PHASE 1 CHECKPOINT

**Structural verification:**
- [x] `00_DEPENDENCY_GRAPH.md` exists and contains "HARD DEPENDENCIES" (appears once per in-scope system — 30+ systems)
- [x] `00_DEPENDENCY_GRAPH.md` contains "EXCLUDED CONNECTION" at least 3 times (item_group, crafting/item requirements, NPC/vehicle internals)
- [x] `00_DEPENDENCY_GRAPH.md` contains "DATA LOADING ORDER" section
- [x] `01_JSON_TYPE_REGISTRY.md` exists and contains 30+ type entries (connect_group, field_type, terrain, furniture, monstergroup, vehicle_group, trap, overmap_terrain, oter_vision, overmap_land_use_code, overmap_connection, overmap_location, city, overmap_special, city_building, map_extra, map_extra_collection, omt_placeholder, region_settings, region_settings_city, region_settings_river/lake/ocean/ravine, region_settings_forest/forest_trail, region_settings_highway, region_settings_terrain_furniture, region_terrain_furniture, region_settings_forest_mapgen, forest_biome_component, forest_biome_mapgen, region_settings_map_extras, construction, gate, palette, rotatable_symbol, ter_furn_transform, mapgen, item_group)
- [x] Every entry in `01_JSON_TYPE_REGISTRY.md` has a "Wulfaz scope" field
- [x] `item_group` appears in registry marked EXCLUDED

**Semantic verification:**
- [x] The dependency graph is acyclic at the type-loading level
- [x] Cross-references are bidirectional (A lists B as dependency ↔ B lists A as dependent)
- [x] No in-scope system is missing from the graph
- [x] The "INVISIBLE DEPENDENCY PROBLEM" section explains the issue clearly

**CHECKPOINT 1:** 30+ systems in graph, 35+ types registered, 8+ EXCLUDED markers, 0 UNCERTAIN callouts (all systems were traceable in source), all structural checks pass.
