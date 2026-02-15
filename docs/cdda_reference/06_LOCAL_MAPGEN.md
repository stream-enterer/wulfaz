# 06 — Local Mapgen

**Scope:** How individual overmap terrain tiles (24x24) are generated — function hierarchy, JSON format, placement keys, nested/update mapgen, C++ hardcoded functions.
**Purpose:** The consuming LLM uses this to implement the system that turns overmap terrain IDs into actual tile-level content (walls, floors, furniture, monsters, etc.).

---

## MAPGEN FUNCTION HIERARCHY

```
mapgen_function (abstract base)
├── mapgen_function_builtin         — C++ hardcoded (function pointer)
├── mapgen_function_json            — JSON data-driven (top-level)
├── mapgen_function_json_nested     — JSON nested (sub-chunks)
└── update_mapgen_function_json     — JSON update (post-generation mods)
```

All mapgen functions take `mapgendata&` and modify the map in place.

### Registration

- **Top-level mapgen** is registered by `om_terrain` string → stored in `oter_mapgen[terrain_id]` as a weighted list
- **Nested mapgen** is registered by `nested_mapgen_id` → stored in `nested_mapgens` map
- **Update mapgen** is registered by `update_mapgen_id` → stored in `update_mapgens` map
- **Builtin mapgen** is registered by function name string → `get_mapgen_cfunction(name)` returns function pointer

### Mapgen Weights

Multiple mapgen entries can target the same `om_terrain`. One is randomly selected proportional to weight:

```json
[
    { "om_terrain": "house", "weight": 100, "object": {...} },
    { "om_terrain": "house", "weight": 50,  "object": {...} }
]
```

Weight 100 vs 50 = 67% vs 33% chance. Default weight: 1000.

> **IMPLEMENT FULLY:** Weighted selection is essential for building variety — multiple house layouts for the same terrain type.

---

## MAPGEN PHASES

Operations are applied in strict phase order:

| Phase | Name | Description |
|---|---|---|
| 0 | `removal` | Remove existing content (for update mapgen) |
| 1 | `terrain` | Place base terrain (must exist before furniture) |
| 2 | `furniture` | Place furniture on terrain |
| 3 | `default_` | All other placements (items, monsters, traps, etc.) |
| 4 | `nested_mapgen` | Apply nested mapgen sub-chunks |
| 5 | `transform` | ter_furn_transforms |
| 6 | `faction_ownership` | Set faction ownership zones |
| 7 | `zones` | Place zones (missions, safe areas) |

> **PORTING TRAP:** Terrain MUST be placed before furniture. If phases are not enforced, furniture placement will silently fail or produce invalid state.

---

## JSON MAPGEN FORMAT

JSON type: `"mapgen"`.

### Top-Level Structure

```json
{
    "type": "mapgen",
    "om_terrain": "house",
    "weight": 100,
    "fill_ter": "t_floor",
    "rotation": [0, 3],

    "rows": [
        "########################",
        "#......................#",
        "#..TTcc..BBB..........#",
        "########################"
    ],

    "terrain": { ".": "t_floor", "#": "t_wall" },
    "furniture": { "T": "f_table", "c": "f_chair", "B": "f_bed" },

    "palettes": ["house_palette"],

    "set": [
        { "point": "terrain", "id": "t_door_c", "x": 12, "y": 0 }
    ],

    "place_monsters": [
        { "monster": "GROUP_ZOMBIE", "x": [5, 18], "y": [5, 18], "chance": 30 }
    ]
}
```

### The "rows" System

- Array of strings, exactly 24 strings of 24 characters each (for standard 24x24 OMTs)
- Each character maps to terrain via `"terrain"` key, and optionally to furniture/traps/items via `"furniture"`/`"traps"` keys or palettes
- Characters not defined in any mapping default to `fill_ter`
- `mapgensize` can override the 24x24 default (rare)

### fill_ter

Default terrain for the entire tile. Used when:
- No `"rows"` defined
- A character in rows has no terrain mapping
- Can be a string ID, weighted list, or parameter reference

### predecessor_mapgen / fallback_predecessor_mapgen

- `predecessor_mapgen`: Exact `oter_id` that must have been generated first. This mapgen layers ON TOP of the predecessor's output.
- `fallback_predecessor_mapgen`: Used if predecessor is missing.
- If present, `expects_predecessor()` returns true and `fill_ter` is optional.

### The "set" Array

Direct coordinate-based operations (not character-mapped):

**Point operations:**
```json
{ "point": "terrain", "id": "t_door_c", "x": 12, "y": 0 }
{ "point": "furniture", "id": "f_table", "x": 5, "y": 5 }
{ "point": "trap", "id": "tr_beartrap", "x": 8, "y": 8 }
{ "point": "radiation", "amount": 50, "x": 10, "y": 10 }
{ "point": "bash", "x": 3, "y": 3 }
{ "point": "burn", "x": 3, "y": 3 }
```

**Line operations:**
```json
{ "line": "terrain", "id": "t_wall", "x": 0, "y": 0, "x2": 23, "y2": 0 }
```

**Square operations:**
```json
{ "square": "terrain", "id": "t_floor", "x": 1, "y": 1, "x2": 22, "y2": 22 }
```

All support `"chance"` (1-in-N) and `"repeat"` (count).

### Rotation

- `"rotation": 0` — fixed rotation (0=north, 1=east, 2=south, 3=west)
- `"rotation": [0, 3]` — random rotation from range
- Applied to the entire mapgen output

---

## NESTED MAPGEN

JSON type: `"nested_mapgen"`. Reusable sub-chunks stamped into parent mapgen via `place_nested`.

### Structure

```json
{
    "type": "nested_mapgen",
    "id": "house_bedroom",
    "weight": 100,
    "rows": [ ... ],
    "terrain": { ... },
    "furniture": { ... },
    "place_*": [ ... ]
}
```

### How It Differs from Top-Level

| Aspect | Top-Level | Nested |
|---|---|---|
| Registration | By `om_terrain` | By `nested_mapgen_id` |
| Size | Full 24x24 | Any size (offset into parent) |
| fill_ter | Supported | Not supported (parent provides) |
| predecessor | Supported | Not supported |
| Called by | Overmap generation | `place_nested` in parent |
| Parameters | Own scope | Inherits parent + own `nest` scope |

### Invocation

```json
{
    "place_nested": [
        { "chunks": [["house_bedroom", 50], ["house_study", 30]], "x": 0, "y": 12 }
    ]
}
```

Weighted random selection: the `chunks` array picks one nested mapgen by weight. `"null"` chunks are valid (do nothing).

> **IMPLEMENT FULLY:** Many buildings use nested mapgen for room layouts. Without it, most building interiors are empty.

---

## UPDATE MAPGEN

JSON type: `"update_mapgen"`. Applies modifications to already-generated terrain.

### Structure

```json
{
    "type": "update_mapgen",
    "id": "house_decay",
    "rows": [ ... ],
    "set": [ ... ],
    "place_furniture": [ ... ],
    "faction_owner": [ ... ]
}
```

### How It Differs

| Aspect | Top-Level | Update |
|---|---|---|
| Timing | During generation | After generation complete |
| Target | Empty map | Existing terrain |
| Mirroring | No | `mirror_horizontal`, `mirror_vertical` |
| Offset | Always (0,0) | Arbitrary offset |
| Verification | No | Optional collision check |
| Use case | Create buildings | Modify, decay, add ownership |

Used by: map extras, missions, faction camp building, post-cataclysm damage.

> **IMPLEMENT FULLY:** Update mapgen is how map extras and missions modify terrain after generation.

---

## MAPGEN PARAMETERS

Parameterized generation allows the same mapgen to produce different results based on context.

### Declaration

```json
{
    "type": "mapgen",
    "om_terrain": "house",
    "parameters": {
        "num_beds": { "type": "int", "scope": "omt", "default": 2 },
        "wall_style": {
            "type": "ter_str_id", "scope": "overmap_special",
            "default": { "distribution": [["t_wall_w", 50], ["t_wall_b", 50]] }
        }
    }
}
```

### Scopes

| Scope | Resolution | Shared Across |
|---|---|---|
| `overmap_special` | Once per special (building complex) | All OMTs in the special |
| `omt` | Once per OMT | Single 24x24 tile |
| `omt_stack` | Once per vertical stack | All z-levels at same XY |
| `nest` | Once per nested call | Single nested invocation |

### Usage in Placement

```json
{ "place_furniture": [{ "furn": "f_bed", "x": 5, "y": 5, "repeat": {"key": "num_beds"} }] }
```

> **STUB OK — TODO:** Parameters add significant variety but are not required for basic mapgen. Implement after core mapgen works.

---

## PLACEMENT KEYS — IN SCOPE

For each key: what it does, JSON format, and what external system it depends on.

### place_terrain — **IN SCOPE**
Places terrain at specific coordinates (outside of rows).
```json
{ "ter": "t_wall", "x": [0, 5], "y": 0 }
```
**Depends on:** terrain type registry (`ter_t`)

### place_furniture — **IN SCOPE**
Places furniture at coordinates.
```json
{ "furn": "f_table", "x": 10, "y": 10 }
```
**Depends on:** furniture type registry (`furn_t`)

### place_traps — **IN SCOPE**
Places traps at coordinates.
```json
{ "trap": "tr_beartrap", "x": 8, "y": 8, "chance": 10 }
```
**Depends on:** trap type registry (`trap`)

### place_fields — **IN SCOPE**
Places field effects (fire, gas, blood).
```json
{ "field": "fd_blood", "x": 12, "y": 12, "intensity": 3 }
```
**Depends on:** field type registry (`field_type`)

### place_monsters — **IN SCOPE**
Spawns monster groups.
```json
{ "monster": "GROUP_ZOMBIE", "x": [5, 18], "y": [5, 18], "chance": 30, "density": 0.5 }
```
**Depends on:** monstergroup registry (`monstergroup`)

### place_monster — **IN SCOPE**
Spawns a single specific monster.
```json
{ "monster": "mon_zombie", "x": 12, "y": 12, "friendly": false }
```
**Depends on:** monster type registry (`mtype`)

### place_vehicles — **IN SCOPE**
Spawns vehicles.
```json
{ "vehicle": "car", "x": 5, "y": 5, "chance": 25, "rotation": 90 }
```
**Depends on:** vehicle prototype/group registry (INTERFACE ONLY)

### place_npcs — **IN SCOPE**
Spawns NPCs.
```json
{ "class": "bandit", "x": 12, "y": 12, "target": true }
```
**Depends on:** NPC class registry

### place_signs — **IN SCOPE**
Places signs with readable text.
```json
{ "signage": "Welcome to town!", "x": 10, "y": 0 }
```
**Depends on:** terrain with SIGN flag

### place_graffiti — **IN SCOPE**
Places graffiti text on terrain.
```json
{ "text": "Was here", "x": 5, "y": 5 }
```
**Depends on:** cosmetic text storage on submap

### place_rubble — **IN SCOPE**
Places rubble/debris terrain.
```json
{ "rubble_type": "t_rubble", "x": 8, "y": 8, "overwrite": true }
```
**Depends on:** terrain types, RUBBLE flag

### place_liquids — **IN SCOPE**
Places liquid terrain.
```json
{ "liquid": "water_clean", "x": 10, "y": 10, "amount": [1, 5] }
```
**Depends on:** liquid item types (SCOPE BOUNDARY — store ID, defer item logic)

### place_zones — **IN SCOPE**
Places faction zones (safe areas, no-go zones).
```json
{ "type": "NPC_NO_INVESTIGATE", "faction": "your_followers", "x": 0, "y": 0, "x2": 23, "y2": 23 }
```
**Depends on:** zone type registry

### place_remove_all — **IN SCOPE**
Clears all entities from a tile.
```json
{ "x": 5, "y": 5 }
```
**Depends on:** (none — pure removal)

### place_ter_furn_transforms — **IN SCOPE**
Applies bulk terrain/furniture transforms.
```json
{ "transform": "fungicide", "x": [0, 23], "y": [0, 23] }
```
**Depends on:** ter_furn_transform registry (see `02_TERRAIN_AND_FURNITURE.md`)

### place_nested — **IN SCOPE**
Stamps nested mapgen sub-chunks.
```json
{ "chunks": [["house_bedroom", 50], ["null", 50]], "x": 0, "y": 12 }
```
**Depends on:** nested_mapgen registry

### place_computers — **IN SCOPE**
Places computer terminals.
```json
{ "name": "Lab Terminal", "security": 3, "x": 12, "y": 12, "options": [...], "failures": [...] }
```
**Depends on:** computer terminal system

### place_toilets / place_gaspumps / place_vendingmachines — **IN SCOPE**
Places specific furniture with special properties.
```json
{ "x": 8, "y": 8, "amount": [0, 100] }
```
**Depends on:** specific furniture IDs

### faction_owner — **IN SCOPE**
Sets faction ownership of an area.
```json
{ "id": "hells_raiders", "x": 0, "y": 0, "x2": 23, "y2": 23 }
```
**Depends on:** faction registry

---

## PLACEMENT KEYS — EXCLUDED

### place_items — **EXCLUDED**
Places item groups. `"item": "kitchen_set"` references `item_group_id`.
**WHY EXCLUDED:** Wulfaz has a different setting and item system. CDDA item groups contain hundreds of modern-world items (canned food, ammunition, tools). Porting them is wasted work. Stub as no-op.

### place_loot — **EXCLUDED**
Single loot item placement. `"group": "guns_common"` references `item_group_id`.
**WHY EXCLUDED:** Same as place_items — references item_group system.

### sealed_item — **EXCLUDED**
Places items sealed inside terrain (e.g., safe with contents). References `item_group_id`.
**WHY EXCLUDED:** Combines terrain interaction with item_group — both require the excluded item system.

### place_corpses — **EXCLUDED**
Places corpses/skeletons. References `mtype_id` for corpse type.
**WHY EXCLUDED:** Requires item-as-corpse system (corpses are items in CDDA) which is EXCLUDED.

> **PORTING TRAP:** Mapgen JSON is LITTERED with `"items"`, `"place_items"`, and `"loot"` keys. These must be silently ignored during mapgen loading, not cause parse errors. The porting LLM must recognize and skip them.

---

## C++ HARDCODED MAPGEN

These mapgen functions are implemented in C++ (`src/mapgen_functions.cpp`) and cannot be data-driven. Each handles specific overmap terrain types:

| Function | Overmap Terrain | Description |
|---|---|---|
| `mapgen_forest` | `forest` variants | Natural forest biome (uses forest_biome_component system) |
| `mapgen_river_straight` | `river_*` straight variants | Straight river segments |
| `mapgen_river_curved` | `river_*` curved variants | Curved river segments |
| `mapgen_river_curved_not` | `river_c_not_*` variants | River curves without certain connections |
| `mapgen_subway` | `subway_*` (straight/curved/end/tee/four_way) | Underground subway tunnels |
| `mapgen_lake_shore` | `lake_shore` | Freshwater lake shoreline |
| `mapgen_ocean_shore` | `ocean_shore` | Saltwater ocean shoreline |
| `mapgen_ravine_edge` | `ravine_edge` | Cliff/drop terrain |

> **PORTING TRAP:** These functions must be reimplemented in Rust — they cannot be loaded from data files. `mapgen_forest` is the most complex, using the entire `forest_biome_component` / `forest_biome_mapgen` / `region_settings_forest_mapgen` chain from `05_REGION_SETTINGS.md`.

---

## MAPGEN FLAGS

Applied to entire mapgen entries to control behavior:

| Flag | Description |
|---|---|
| `dismantle_all_before_placing_terrain` | Clear everything before terrain phase |
| `erase_all_before_placing_terrain` | Erase all existing data |
| `allow_terrain_under_furniture` | Don't clear furniture when placing terrain |
| `avoid_creatures` | Skip placement if creature collision |
| `no_underlying_rotate` | Don't rotate underlying content |

---

## EXECUTION FLOW SUMMARY

```
1. Overmap identifies OMT terrain ID (e.g., "house_north")
2. Look up oter_mapgen["house"] → weighted list of mapgen_function
3. Select one by weight
4. Create mapgendata context (includes region_settings, neighbors, etc.)
5. Call mapgen_function::generate(mapgendata)
   a. Phase 0 (removal): clear specified areas
   b. Phase 1 (terrain): apply rows→terrain mapping, fill_ter, set terrain ops
   c. Phase 2 (furniture): apply rows→furniture mapping, place_furniture
   d. Phase 3 (default): place_monsters, place_traps, place_fields, etc.
   e. Phase 4 (nested): apply place_nested chunks
   f. Phase 5 (transform): apply ter_furn_transforms
   g. Phase 6 (faction): set faction_owner
   h. Phase 7 (zones): place_zones
6. Post-mapgen: resolve REGION_PSEUDO terrain, apply map extras
```
