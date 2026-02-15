# 02 — Terrain and Furniture Types

**Scope:** Complete documentation of `ter_t`, `furn_t`, and all supporting types that define what the map is made of.
**Purpose:** The consuming LLM uses this to implement terrain and furniture type loading, flag checking, and tile-level interactions.

---

## BASE TYPE: map_data_common_t

Both `ter_t` and `furn_t` inherit from `map_data_common_t` (defined in `src/mapdata.h`). Every field listed here exists on BOTH terrain and furniture types.

### Key Fields (map_data_common_t)

| Field | C++ Type | JSON Key | Description |
|---|---|---|---|
| `name_` | `translation` | `"name"` | Display name shown to the player |
| `description` | `translation` | `"description"` | Longer description text |
| `symbol_` | `array<int, 4>` | `"symbol"` | ASCII symbol per season (Spring/Summer/Autumn/Winter) |
| `color_` | `array<nc_color, 4>` | `"color"` | Display color per season |
| `movecost` | `int` | `"move_cost"` / `"move_cost_mod"` | Movement cost in points. 0 = impassable. Terrain uses absolute cost; furniture uses modifier added to terrain cost |
| `light_emitted` | `int` | `"light_emitted"` | Light emission level (0 = none) |
| `heat_radiation` | `int` | `"heat_radiation"` | Heat radiation amount |
| `coverage` | `int` | `"coverage"` | Cover percentage. Values < 30 do not block line of sight |
| `floor_bedding_warmth` | `temperature_delta` | `"floor_bedding_warmth"` | Warmth provided for sleeping |
| `comfort` | `int` | `"comfort"` | Comfort level for rest quality |
| `fall_damage_reduction` | `int` | `"fall_damage_reduction"` | Flat damage reduction on fall (negative = increase) |
| `max_volume` | `volume` | `"max_volume"` | Max volume of items storable on tile (default: 1000 L) |
| `transparent` | `bool` | (derived) | Cached from TRANSPARENT/TRANSLUCENT flags at load time |
| `looks_like` | `string` | `"looks_like"` | Tileset fallback — references another terrain/furniture ID |
| `base_item` | `itype_id` | `"base_item"` | Underlying item type for this tile |
| `curtain_transform` | `ter_str_id` | `"curtain_transform"` | Terrain to switch to when curtains are toggled |
| `emissions` | `set<emit_id>` | `"emissions"` | Gas/smoke emission IDs (per-turn) |
| `lockpick_message` | `translation` | `"lockpick_message"` | Message on successful lockpick |
| `shoot` | `map_shoot_info*` | `"shoot"` | Ballistic interaction data (see below) |
| `harvest_by_season` | `array<harvest_id, 4>` | `"harvest_by_season"` | Harvest drop tables per season |
| `liquid_source_item_id` | `itype_id` | `"liquid_source"` | Liquid item this tile provides |
| `liquid_source_min_temp` | `double` | `"liquid_source_min_temp"` | Liquid temperature in Celsius (default: 4) |
| `liquid_source_count` | `pair<int, int>` | `"liquid_source_count"` | Charge range for finite liquid sources |
| `flags` | `set<string>` | `"flags"` | String flags (extensible, mod-friendly) |
| `bitflags` | `bitset<ter_furn_flag>` | (derived) | Fast bitfield mirror of known flags (built at load time) |
| `connect_groups` | `bitset<256>` | `"connect_groups"` | Connection group membership (passive — "I am a member of these groups") |
| `connect_to_groups` | `bitset<256>` | `"connects_to"` | Target groups to connect to (active — "I connect visually to tiles in these groups") |
| `rotate_to_groups` | `bitset<256>` | `"rotates_to"` | Target groups to rotate towards |
| `examine_func` | `iexamine_functions` | `"examine_action"` (string) | Hardcoded C++ examine function |
| `examine_actor` | `iexamine_actor*` | `"examine_action"` (object) | Data-driven examine actor |

> **SCOPE BOUNDARY:** `base_item`, `liquid_source_item_id`, and `harvest_by_season` reference `itype_id` and `harvest_id` which are EXCLUDED from the Wulfaz port. Store the string IDs in the data model so JSON loads correctly, but do not resolve them. Mark as TODO.

> **SCOPE BOUNDARY:** `emissions` references `emit_id` (gas/smoke system). Store the IDs but do not implement emission logic initially.

### map_shoot_info

| Field | Type | Default | Description |
|---|---|---|---|
| `chance_to_hit` | `int` | 100 | Base % chance a projectile hits this tile |
| `reduce_dmg_min` | `int` | 0 | Min damage reduction for ballistic shots |
| `reduce_dmg_max` | `int` | 0 | Max damage reduction for ballistic shots |
| `reduce_dmg_min_laser` | `int` | 0 | Min damage reduction for lasers |
| `reduce_dmg_max_laser` | `int` | 0 | Max damage reduction for lasers |
| `destroy_dmg_min` | `int` | 0 | Min damage to have a chance of destroying tile |
| `destroy_dmg_max` | `int` | 0 | Damage that guarantees destruction |
| `no_laser_destroy` | `bool` | false | Lasers cannot destroy this tile |

> **STUB OK — TODO:** Ranged combat and ballistic simulation are complex systems. Store `map_shoot_info` fields in the data model but do not implement projectile interaction initially.

---

## TERRAIN TYPE: ter_t

Defined in `src/mapdata.h`, inherits all `map_data_common_t` fields above. JSON type: `"terrain"`.

### Key Fields (ter_t only)

| Field | C++ Type | JSON Key | Description | References |
|---|---|---|---|---|
| `id` | `ter_str_id` | `"id"` | Unique string ID (e.g. `"t_wall"`) | — |
| `open` | `ter_str_id` | `"open"` | Transform to this terrain when opened (e.g. door opens) | another `ter_t` |
| `close` | `ter_str_id` | `"close"` | Transform to this terrain when closed | another `ter_t` |
| `transforms_into` | `ter_str_id` | `"transforms_into"` | Generic transform target (time-based decay, etc.) | another `ter_t` |
| `roof` | `ter_str_id` | `"roof"` | What terrain appears as the floor on z+1 above this tile | another `ter_t` |
| `lockpick_result` | `ter_str_id` | `"lockpick_result"` | Transform to when successfully lockpicked | another `ter_t` |
| `bash` | `map_ter_bash_info?` | `"bash"` | Bash interaction data (see Bash Info below) | — |
| `deconstruct` | `map_ter_deconstruct_info?` | `"deconstruct"` | Deconstruct interaction data | — |
| `trap_id_str` | `string` | `"trap"` | Trap ID string, resolved to `trap_id` at finalize | `trap` type |
| `trap` | `trap_id` | (resolved) | Resolved trap on this terrain | `trap` type |
| `phase_targets` | `vec<ter_str_id>` | `"phase_targets"` | Phase-change target terrains (e.g. ice/water/steam) | other `ter_t` |
| `phase_temps` | `vec<temperature>` | `"phase_temps"` | Temperature thresholds for phase change | — |
| `phase_method` | `string` | `"phase_method"` | Phase-change algorithm: `"thresholds"`, `"closest"`, `"gradient"` | — |
| `boltcut` | `activity_data_ter*` | `"boltcut"` | Bolt cutting action → result terrain | another `ter_t` |
| `hacksaw` | `activity_data_ter*` | `"hacksaw"` | Hacksaw action → result terrain | another `ter_t` |
| `oxytorch` | `activity_data_ter*` | `"oxytorch"` | Oxytorch action → result terrain | another `ter_t` |
| `prying` | `activity_data_ter*` | `"prying"` | Prying action → result terrain | another `ter_t` |
| `allowed_template_id` | `set<itype_id>` | `"allowed_template_id"` | Allowed template items on this terrain | item types (EXCLUDED) |

> **IMPLEMENT FULLY:** The `open`/`close`/`transforms_into`/`roof` transform chain is fundamental to door, window, and structural behavior. Every terrain→terrain transform ID must resolve correctly.

> **STUB OK — TODO:** Phase-change transforms (`phase_targets`/`phase_temps`/`phase_method`) implement temperature-dependent terrain changes (ice melting, water freezing). Store the fields but defer the phase-change simulation logic.

---

## FURNITURE TYPE: furn_t

Defined in `src/mapdata.h`, inherits all `map_data_common_t` fields above. JSON type: `"furniture"`.

### Key Fields (furn_t only)

| Field | C++ Type | JSON Key | Description | References |
|---|---|---|---|---|
| `id` | `furn_str_id` | `"id"` | Unique string ID (e.g. `"f_table"`) | — |
| `open` | `furn_str_id` | `"open"` | Transform to this furniture when opened | another `furn_t` |
| `close` | `furn_str_id` | `"close"` | Transform to this furniture when closed | another `furn_t` |
| `lockpick_result` | `furn_str_id` | `"lockpick_result"` | Transform to when lockpicked | another `furn_t` |
| `bash` | `map_furn_bash_info?` | `"bash"` | Bash interaction data | — |
| `deconstruct` | `map_furn_deconstruct_info?` | `"deconstruct"` | Deconstruct interaction data | — |
| `crafting_pseudo_item` | `itype_id` | `"crafting_pseudo_item"` | Pseudo-item providing tool qualities for crafting | item type (EXCLUDED) |
| `keg_capacity` | `volume` | `"keg_capacity"` | Liquid storage capacity | — |
| `bonus_fire_warmth_feet` | `temperature_delta` | `"bonus_fire_warmth_feet"` | Extra fire warmth at feet (default: 0.6°C) | — |
| `deployed_item` | `itype_id` | `"deployed_item"` | Item that deploys into this furniture | item type (EXCLUDED) |
| `move_str_req` | `int` | `"required_str"` | Strength required to push/move this furniture | — |
| `mass` | `mass` | `"mass"` | Physical mass of the furniture | — |
| `boltcut` | `activity_data_furn*` | `"boltcut"` | Bolt cutting action → result furniture | another `furn_t` |
| `hacksaw` | `activity_data_furn*` | `"hacksaw"` | Hacksaw action → result furniture | another `furn_t` |
| `oxytorch` | `activity_data_furn*` | `"oxytorch"` | Oxytorch action → result furniture | another `furn_t` |
| `prying` | `activity_data_furn*` | `"prying"` | Prying action → result furniture | another `furn_t` |
| `workbench` | `furn_workbench_info*` | `"workbench"` | Crafting workbench data | — |
| `plant` | `plant_data*` | `"plant_data"` | Plant growth configuration | — |
| `surgery_skill_multiplier` | `float*` | `"surgery_skill_multiplier"` | Surgery skill multiplier (autodoc) | — |

> **SCOPE BOUNDARY:** `crafting_pseudo_item` and `deployed_item` reference `itype_id` which is EXCLUDED. Store the IDs but do not resolve.

### furn_workbench_info

| Field | Type | Default | Description |
|---|---|---|---|
| `multiplier` | `float` | 1.0 | Crafting speed multiplier |
| `allowed_mass` | `mass` | max | Mass before penalty applies |
| `allowed_volume` | `volume` | max | Volume before penalty applies |

### plant_data

| Field | Type | Description | References |
|---|---|---|---|
| `transform` | `furn_str_id` | Furniture when plant reaches next growth stage | another `furn_t` |
| `base` | `furn_str_id` | Base furniture (pre-planting state / post-eaten state) | another `furn_t` |
| `growth_multiplier` | `float` | Growth speed multiplier (default: 1.0) | — |
| `harvest_multiplier` | `float` | Harvest yield multiplier (default: 1.0) | — |

---

## BASH INFO

Bash info defines what happens when a terrain, furniture, or field is destroyed by force. The hierarchy:

```
map_common_bash_info          (shared base — never instantiated alone)
├── map_ter_bash_info         (terrain bashing: result is ter_str_id)
├── map_furn_bash_info        (furniture bashing: result is furn_str_id)
└── map_fd_bash_info          (field bashing: result is implicit removal)
```

### map_common_bash_info (shared fields)

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `str_min` | `int` | `"str_min"` | Minimum strength to bash (-1 = not set → not bashable) |
| `str_max` | `int` | `"str_max"` | Maximum strength threshold (above this: guaranteed success) |
| `str_min_blocked` | `int` | `"str_min_blocked"` | Alternate str_min when adjacent furniture has BLOCKSDOOR flag |
| `str_max_blocked` | `int` | `"str_max_blocked"` | Alternate str_max when blocked |
| `str_min_supported` | `int` | `"str_min_supported"` | Alternate str_min when floor is supported from below |
| `str_max_supported` | `int` | `"str_max_supported"` | Alternate str_max when supported |
| `explosive` | `int` | `"explosive"` | Explosion magnitude on destruction |
| `sound_vol` | `int` | `"sound_vol"` | Sound volume on successful bash |
| `sound_fail_vol` | `int` | `"sound_fail_vol"` | Sound volume on failed bash |
| `collapse_radius` | `int` | `"collapse_radius"` | Tent collapse radius (default: 1) |
| `destroy_only` | `bool` | `"destroy_only"` | Only destroyed programmatically, not by player bashing |
| `bash_below` | `bool` | `"bash_below"` | This is a roof tile — bash attempts the tile below |
| `damage_profile` | `bash_damage_profile_id` | `"damage_profile"` | Damage type profile reference |
| `drop_group` | `item_group_id` | `"items"` | Items dropped on destruction |
| `sound` | `translation` | `"sound"` | Sound description on success |
| `sound_fail` | `translation` | `"sound_fail"` | Sound description on failure |
| `tent_centers` | `vec<furn_str_id>` | `"tent_centers"` | Tent center furniture IDs (for tent collapse logic) |
| `hit_field` | `(field_type_id, int)` | `"hit_field"` | Field spawned on any hit (type + intensity) |
| `destroyed_field` | `(field_type_id, int)` | `"destroyed_field"` | Field spawned on destruction (type + intensity) |

> **SCOPE BOUNDARY:** `drop_group` (`"items"`) references `item_group_id` which is EXCLUDED. The porting LLM should implement bash as terrain→terrain / furniture→furniture transform only, ignoring the item yield. Mark the item yield slot as TODO.

### map_ter_bash_info (terrain-specific)

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `ter_set` | `ter_str_id` | `"ter_set"` | Terrain to become after bashing |
| `ter_set_bashed_from_above` | `ter_str_id` | `"ter_set_bashed_from_above"` | Terrain if bashed from z+1 (defaults to `ter_set`) |

### map_furn_bash_info (furniture-specific)

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `furn_set` | `furn_str_id` | `"furn_set"` | Furniture to become after bashing (default: `f_null` = removed) |

### map_fd_bash_info (field-specific)

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `fd_bash_move_cost` | `int` | `"fd_bash_move_cost"` | Movement cost to bash this field (default: 100) |
| `field_bash_msg_success` | `translation` | `"field_bash_msg_success"` | Success message |

---

## DECONSTRUCT INFO

Careful dismantling (as opposed to bashing). Same hierarchy pattern:

```
map_common_deconstruct_info       (shared base)
├── map_ter_deconstruct_info      (result: ter_str_id)
└── map_furn_deconstruct_info     (result: furn_str_id)
```

### map_common_deconstruct_info (shared fields)

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `deconstruct_above` | `bool` | `"deconstruct_above"` | Also tear down the roof tile above |
| `drop_group` | `item_group_id` | `"items"` | Items yielded on deconstruct |
| `skill` | `map_deconstruct_skill?` | `"skill"` | Skill XP reward configuration |

> **SCOPE BOUNDARY:** `drop_group` references `item_group_id` (EXCLUDED). Implement as pure transform, ignore item yield.

### map_deconstruct_skill

| Field | Type | Description |
|---|---|---|
| `id` | `skill_id` | Skill to train |
| `min` | `int` | Minimum level to gain XP |
| `max` | `int` | Level cap for practice |
| `multiplier` | `double` | XP multiplier |

### map_ter_deconstruct_info

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `ter_set` | `ter_str_id` | `"ter_set"` | Terrain after deconstruction |

### map_furn_deconstruct_info

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `furn_set` | `furn_str_id` | `"furn_set"` | Furniture after deconstruction (default: `f_null`) |

---

## TOOL-USE ACTIVITY DATA

Boltcut, hacksaw, oxytorch, and prying actions share a common base (`activity_data_common`) with terrain- and furniture-specific result types.

### activity_data_common (shared fields)

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `valid_` | `bool` | (derived) | Whether this action is configured |
| `duration_` | `time_duration` | `"duration"` | How long the action takes |
| `message_` | `translation` | `"message"` | Completion message |
| `sound_` | `translation` | `"sound"` | Sound during action |
| `prying_data_` | `pry_data` | `"prying_data"` | Prying-specific sub-data |
| `byproducts_` | `vec<activity_byproduct>` | `"byproducts"` | Item byproducts |

### pry_data

| Field | Type | Default | Description |
|---|---|---|---|
| `prying_nails` | `bool` | false | Whether this is nail prying |
| `difficulty` | `int` | 0 | Prying difficulty level |
| `prying_level` | `int` | 0 | Required prying tool quality level |
| `noisy` | `bool` | false | Generates noise |
| `alarm` | `bool` | false | Triggers alarm |
| `breakable` | `bool` | false | Can break during prying |
| `failure` | `translation` | — | Failure message |

### activity_data_ter / activity_data_furn

Each adds one field:

| Field | Type | Description |
|---|---|---|
| `result_` | `ter_str_id` / `furn_str_id` | Terrain or furniture to transform into on completion |

> **STUB OK — TODO:** Tool-use actions require the crafting/activity system which is not ported initially. Store the data (especially `result_`) so transforms are known, but defer the activity execution pipeline.

---

## CONNECT GROUPS

Connect groups control the **visual auto-tiling** system — how walls connect to adjacent walls, how fences join up, etc. This is the system that makes `t_wall` display as `│`, `─`, `┌`, `┐`, `└`, `┘`, `├`, `┤`, `┬`, `┴`, `┼` depending on neighbors.

### Data Model

Defined in `src/mapdata.h`. Loaded from JSON as type `"connect_group"`.

```cpp
struct connect_group {
    connect_group_id id;       // String identifier (e.g. "WALL")
    int index;                 // Bit index in the bitset (0-255)
    set<ter_furn_flag> group_flags;         // Flags that make a tile a member
    set<ter_furn_flag> connects_to_flags;   // Flags that this group connects to
    set<ter_furn_flag> rotates_to_flags;    // Flags that this group rotates towards
};
```

Each `map_data_common_t` carries three `bitset<256>`:
- `connect_groups` — "I am a member of these groups" (passive)
- `connect_to_groups` — "I visually connect to tiles in these groups" (active)
- `rotate_to_groups` — "I rotate my symbol towards tiles in these groups" (active)

### Algorithm Concept

When rendering a tile at position (x, y):

1. **Connection check:** For each of the 4 cardinal + 4 diagonal neighbors, test whether the neighbor's `connect_groups` bitset overlaps with this tile's `connect_to_groups` bitset. This produces an 8-bit adjacency mask.

2. **Symbol selection:** The 8-bit mask maps to a box-drawing character. For walls:
   - `0b0101` (N+S connected) → `│`
   - `0b1010` (E+W connected) → `─`
   - `0b0110` (S+E) → `┌`
   - `0b1100` (E+N) → `┘`
   - etc.

3. **Rotation check:** Similar to connection, but for furniture/terrain that has a facing direction rather than box-drawing connections. The `rotate_to_groups` bitset determines which neighbor the tile "faces."

The `map` class methods that implement this:
- `get_known_connections(pos, connect_group_bitset)` → `uint8_t` adjacency mask
- `get_known_rotates_to(pos, rotate_to_group_bitset)` → `uint8_t` rotation mask

### JSON Example

```json
{
    "type": "connect_group",
    "id": "WALL",
    "group_flags": ["WALL", "CONNECT_WITH_WALL"],
    "connects_to_flags": ["WALL", "CONNECT_WITH_WALL", "WINDOW", "DOOR"]
}
```

This means: any terrain with the `WALL` or `CONNECT_WITH_WALL` flag is a member of the `WALL` group. Tiles in this group visually connect to adjacent tiles that have any of `WALL`, `CONNECT_WITH_WALL`, `WINDOW`, or `DOOR` flags.

> **IMPLEMENT FULLY:** Connect groups are essential for walls/fences to render correctly. Without them, every wall tile renders as the same symbol regardless of neighbors.

---

## EXAMINE ACTION (iexamine)

The examine system defines what happens when the player interacts with terrain/furniture.

### Two Mechanisms

1. **Hardcoded functions** (`examine_func`): C++ function pointers, set from JSON as `"examine_action": "function_name"`. Common examples: `none`, `gaspump`, `atm`, `vending`, `elevator`, `controls_gate`, `rubble`, `chainfence`, `bars`, `deployed_furniture`, `pit`, `safe`, `harvest_furn`, `harvest_ter`, `locked_object`, `locked_object_pickable`, `bulletin_board`, `curtains`, `flower_tulip`, `flower_poppy`.

2. **Data-driven actors** (`examine_actor`): Set from JSON as `"examine_action": { "type": "actor_name", ... }`. Registered actors: `appliance_convert_examine_actor`, `cardreader_examine_actor`, `eoc_examine_actor`, `mortar_examine_actor`.

### Wulfaz Approach

> **STUB OK — TODO:** Most examine actions are CDDA-gameplay-specific (ATMs, vending machines, gas pumps). For initial implementation, store the examine action string/type in the data model but implement only a few generic actions (door open/close via the `open`/`close` fields, which don't use the examine system). The examine dispatch can be a match on string → handler function, added incrementally.

---

## REGION_PSEUDO FLAG AND REGIONAL RESOLUTION

The `REGION_PSEUDO` flag (`TFLAG_REGION_PSEUDO`) marks **abstract placeholder** terrain and furniture that must be resolved to region-specific concrete variants after mapgen runs.

### How It Works

1. Mapgen places abstract terrain like `t_region_soil` or `t_region_groundcover` (these have the `REGION_PSEUDO` flag).

2. After mapgen completes, `resolve_regional_terrain_and_furniture()` runs (in `src/mapgen_functions.cpp`):

```
for each tile in the map:
    if terrain has REGION_PSEUDO flag:
        look up terrain in region_settings_terrain_furniture
        randomly pick a concrete replacement from weighted list
        replace terrain
    if furniture has REGION_PSEUDO flag:
        same process for furniture
```

3. The `region_settings_terrain_furniture` type maps abstract IDs to weighted replacement lists:

```json
{
    "type": "region_terrain_furniture",
    "id": "default_soil",
    "ter_id": "t_region_soil",
    "replace_with_terrain": [
        ["t_dirt", 70],
        ["t_clay", 20],
        ["t_mud", 10]
    ]
}
```

### Data Model

**`region_terrain_furniture`:**

| Field | Type | Description |
|---|---|---|
| `id` | `region_terrain_furniture_id` | Unique identifier |
| `replaced_ter_id` | `ter_id` | Abstract terrain to match |
| `replaced_furn_id` | `furn_id` | Abstract furniture to match |
| `terrain` | `weighted_int_list<ter_id>` | Weighted concrete terrain replacements |
| `furniture` | `weighted_int_list<furn_id>` | Weighted concrete furniture replacements |

**`region_settings_terrain_furniture`:**

| Field | Type | Description |
|---|---|---|
| `id` | `region_settings_terrain_furniture_id` | Unique identifier |
| `ter_furn` | `set<region_terrain_furniture_id>` | Collection of replacement mappings |

Resolution calls `resolve(ter_id)` which finds matching entries and picks from the weighted list. Supports chaining (the result may itself be REGION_PSEUDO, resolved on the next pass).

> **IMPLEMENT FULLY:** This is essential for biome variation. Without it, all regions look identical because mapgen JSON uses abstract terrain that never gets concretized.

---

## ter_furn_transform

A separate type (`"ter_furn_transform"`) that defines bulk terrain/furniture/trap/field transformations triggered by spells, effects, or events. Defined in `src/magic_ter_furn_transform.h`.

### Data Model

```cpp
class ter_furn_transform {
    ter_furn_transform_id id;

    // Transforms matched by specific ID
    map<ter_str_id,       ter_furn_data<ter_str_id>>   ter_transform;
    map<furn_str_id,      ter_furn_data<furn_str_id>>  furn_transform;
    map<trap_str_id,      ter_furn_data<trap_str_id>>  trap_transform;
    map<field_type_id,    ter_furn_data<field_type_id>> field_transform;

    // Transforms matched by flag (fallback if no ID match)
    map<string,           ter_furn_data<ter_str_id>>   ter_flag_transform;
    map<string,           ter_furn_data<furn_str_id>>  furn_flag_transform;
    map<string,           ter_furn_data<trap_str_id>>  trap_flag_transform;
};
```

### ter_furn_data\<T\> (per-entry)

| Field | Type | Description |
|---|---|---|
| `list` | `weighted_int_list<T>` | Possible results with weights (randomly selected) |
| `message` | `translation` | Message shown to player |
| `message_good` | `bool` | Whether message is positive (green) or negative (red) |

### Execution Order

When `transform(map, location)` is called:

1. Check terrain by specific ID → if no match, check by flag
2. Check furniture by specific ID → if no match, check by flag
3. Check trap by specific ID → if no match, check by flag
4. Check field by specific ID
5. Apply all matched transforms, show messages

### JSON Example

```json
{
    "type": "ter_furn_transform",
    "id": "fungicide",
    "terrain": [
        {
            "result": "t_dirt_barren",
            "valid_flags": ["FUNGUS"],
            "message": "The fungus here dies back."
        }
    ],
    "furniture": [
        {
            "result": "f_null",
            "valid_flags": ["FUNGUS"],
            "message": "The fungus here dies back."
        }
    ]
}
```

Results can also be weighted lists: `"result": [["t_tree_birch", 32], ["t_tree_elm", 32]]`.

> **STUB OK — TODO:** `ter_furn_transform` is primarily used by the magic/spell system (Magiclysm). Store the data model but defer the spell/effect integration. The transform execution logic itself is straightforward and can be ported independently.

---

## SEASONAL VARIANTS

Both `ter_t` and `furn_t` support per-season appearance changes via arrays indexed by `season_type` (4 seasons: SPRING=0, SUMMER=1, AUTUMN=2, WINTER=3).

- `symbol_[4]` — different ASCII character per season
- `color_[4]` — different color per season
- `harvest_by_season[4]` — different harvest drops per season

In JSON, seasonal variants are specified as:
```json
{
    "symbol": ".",
    "winter_symbol": "*",
    "color": "green",
    "winter_color": "white"
}
```

Seasons not explicitly overridden inherit from the base `symbol`/`color`.

> **IMPLEMENT FULLY:** Seasonal rendering is a core visual feature. Even if Wulfaz uses different seasons, the per-season array pattern should be preserved.

---

## TYPE ID RESOLUTION

CDDA uses a dual-ID scheme for both terrain and furniture:

- **`ter_str_id` / `furn_str_id`** — String-based IDs used in JSON and for human-readable lookups. Implemented as `string_id<ter_t>`.
- **`ter_id` / `furn_id`** — Integer-based IDs used at runtime for fast array indexing. Implemented as `int_id<ter_t>`.

Both are managed by `generic_factory<ter_t>` / `generic_factory<furn_t>`. The `int_id` dereferences to the full struct via `.obj()`. Submaps store `ter_id` and `furn_id` (integer IDs) per tile for cache-friendly access.

> **PORTING TRAP:** In Wulfaz, this maps naturally to a `Vec<ter_t>` with integer indices, plus a `HashMap<String, usize>` for string→index lookup. Do NOT use `HashMap<String, ter_t>` for the primary storage — tile data must use integer IDs for performance.

---

## COMPLETE FLAG INVENTORY

All 139 flags from `enum class ter_furn_flag` in `src/mapdata.h`. Organized by functional category. Each flag has a one-line behavioral description.

### Movement Flags

| Flag | Behavior | Priority |
|---|---|---|
| `FLAT` | Tile has no significant elevation change; allows placement of items | IMPLEMENT FULLY |
| `LIQUID` | Tile is a liquid (affects item placement and movement) | IMPLEMENT FULLY |
| `SWIMMABLE` | Entity can swim through this tile | IMPLEMENT FULLY |
| `DEEP_WATER` | Deep water — entities without swimming ability drown | IMPLEMENT FULLY |
| `SHALLOW_WATER` | Shallow water — wading possible, slows movement | IMPLEMENT FULLY |
| `WATER_CUBE` | Full cube of water (3D water body, not surface) | STUB OK |
| `CURRENT` | Water has a current that pushes entities | STUB OK |
| `ROUGH` | Rough terrain — slows movement, penalizes vehicles | IMPLEMENT FULLY |
| `UNSTABLE` | Unstable ground — chance of collapse or stumbling | STUB OK |
| `SHARP` | Sharp surface — damages entities passing through | IMPLEMENT FULLY |
| `ROAD` | Road surface — affects rollerblade speed and vehicle handling | IMPLEMENT FULLY |
| `RAIL` | Railroad track | STUB OK |
| `TINY` | Very small obstacle — does not block movement or sight | IMPLEMENT FULLY |
| `SHORT` | Short obstacle — can be seen and shot over | IMPLEMENT FULLY |
| `NOCOLLIDE` | No collision — entities pass through without interaction | IMPLEMENT FULLY |
| `SMALL_PASSAGE` | Only small creatures can pass | STUB OK |

### Vertical Movement Flags

| Flag | Behavior | Priority |
|---|---|---|
| `GOES_DOWN` | Stairs/ladder going down (z-1 access) | IMPLEMENT FULLY |
| `GOES_UP` | Stairs/ladder going up (z+1 access) | IMPLEMENT FULLY |
| `NO_FLOOR` | Open air — no floor at this tile (falling hazard) | IMPLEMENT FULLY |
| `ALLOW_ON_OPEN_AIR` | Furniture/terrain can exist on open air tiles | STUB OK |
| `SEEN_FROM_ABOVE` | Visible when looking down from z+1 | IMPLEMENT FULLY |
| `RAMP` | Ramp connecting z-levels | STUB OK |
| `RAMP_UP` | Upward ramp entry | STUB OK |
| `RAMP_DOWN` | Downward ramp entry | STUB OK |
| `RAMP_END` | End of a ramp | STUB OK |
| `CLIMBABLE` | Can be climbed (slow vertical movement) | STUB OK |
| `CLIMB_SIMPLE` | Easy to climb (no skill check) | STUB OK |
| `CLIMB_ADJACENT` | Can be climbed from an adjacent tile | STUB OK |
| `LADDER` | Ladder — fast vertical movement | IMPLEMENT FULLY |
| `DIFFICULT_Z` | Difficult vertical transition (slow/risky) | STUB OK |
| `ELEVATOR` | Elevator tile — multi-z transport | STUB OK |
| `FLOATS_IN_AIR` | This tile/furniture floats and doesn't need floor support | STUB OK |

### Vision / Light Flags

| Flag | Behavior | Priority |
|---|---|---|
| `TRANSPARENT` | Light and sight pass through this tile | IMPLEMENT FULLY |
| `TRANSLUCENT` | Partially transparent — reduced vision range through | IMPLEMENT FULLY |
| `NO_SIGHT` | Completely blocks sight (even if otherwise transparent) | IMPLEMENT FULLY |
| `Z_TRANSPARENT` | Light/sight passes between z-levels through this tile | STUB OK |
| `TRANSPARENT_FLOOR` | Floor is transparent — can see through to z-1 | STUB OK |
| `SUN_ROOF_ABOVE` | Sunlight passes through the roof above (greenhouse) | STUB OK |

> **STUB OK — TODO:** FOV/lighting is a significant algorithm (shadowcasting). For initial implementation, all tiles are visible. The TRANSPARENT flag should be stored on terrain types from day one so the data model is ready when FOV is implemented.

### Structural Flags

| Flag | Behavior | Priority |
|---|---|---|
| `WALL` | Upright wall — blocks movement, supports roof, used for auto-tiling | IMPLEMENT FULLY |
| `THIN_OBSTACLE` | Thin obstacle (fence, railing) — blocks movement but not a full wall | IMPLEMENT FULLY |
| `SUPPORTS_ROOF` | This terrain can support a roof above | IMPLEMENT FULLY |
| `COLLAPSES` | Tile collapses if support is removed (roof without walls) | STUB OK |
| `INDOORS` | Tile is considered indoors (affects weather, temperature) | IMPLEMENT FULLY |
| `MINEABLE` | Can be mined (pickaxe/drill) | STUB OK |
| `SINGLE_SUPPORT` | Only provides support in one direction | STUB OK |
| `WIRED_WALL` | Wall contains wiring (electrical system) | STUB OK |

### Door / Window Flags

| Flag | Behavior | Priority |
|---|---|---|
| `DOOR` | This is a door (affects pathfinding, NPC behavior, auto-open) | IMPLEMENT FULLY |
| `WINDOW` | This is a window (affects vision, connects to walls) | IMPLEMENT FULLY |
| `LOCKED` | Door/container is locked | IMPLEMENT FULLY |
| `PICKABLE` | Lock can be picked | IMPLEMENT FULLY |
| `OPENCLOSE_INSIDE` | Can only be opened/closed from inside | STUB OK |
| `ALARMED` | Opening triggers an alarm | STUB OK |
| `BARRICADABLE_DOOR` | Door can be barricaded | STUB OK |
| `BARRICADABLE_DOOR_DAMAGED` | Barricaded door in damaged state | STUB OK |
| `BARRICADABLE_DOOR_REINFORCED` | Reinforced barricaded door | STUB OK |
| `BARRICADABLE_WINDOW_CURTAINS` | Window with curtains that can be barricaded | STUB OK |
| `BLOCKSDOOR` | Furniture that boosts bash resistance when adjacent to doors | IMPLEMENT FULLY |

### Flammability Flags

| Flag | Behavior | Priority |
|---|---|---|
| `FLAMMABLE` | Can catch fire — normal burn rate, leaves charred remains | IMPLEMENT FULLY |
| `FLAMMABLE_HARD` | Harder to ignite but still burns (metal doesn't, thick wood does) | IMPLEMENT FULLY |
| `FLAMMABLE_ASH` | Burns completely to ash (no charred remains) | IMPLEMENT FULLY |
| `FIRE_CONTAINER` | Contains fire without spreading (fireplace, brazier) | IMPLEMENT FULLY |
| `SUPPRESS_SMOKE` | Reduces/eliminates smoke from fire on this tile | STUB OK |
| `USABLE_FIRE` | Fire here can be used for cooking/warmth | IMPLEMENT FULLY |

### Scent Flags

| Flag | Behavior | Priority |
|---|---|---|
| `NO_SCENT` | Completely blocks scent propagation | STUB OK |
| `REDUCE_SCENT` | Reduces scent propagation through this tile | STUB OK |
| `PERMEABLE` | Allows scent/gas to pass through | STUB OK |

### Container / Item Flags

| Flag | Behavior | Priority |
|---|---|---|
| `SEALED` | Container is sealed — contents are isolated from environment | STUB OK |
| `CONTAINER` | Can contain items (locker, box) | STUB OK |
| `LIQUIDCONT` | Can contain liquids | STUB OK |
| `NOITEM` | Items cannot be placed on this tile | IMPLEMENT FULLY |
| `DESTROY_ITEM` | Items on this tile are destroyed (lava, acid) | IMPLEMENT FULLY |
| `PLACE_ITEM` | Items can be deliberately placed here | STUB OK |
| `NO_PICKUP_ON_EXAMINE` | Examining does not trigger item pickup | STUB OK |
| `NO_SPOIL` | Items on this tile do not spoil (freezer, cold storage) | STUB OK |
| `DONT_REMOVE_ROTTEN` | Rotten items are not auto-cleaned from this tile | STUB OK |
| `AMMOTYPE_RELOAD` | Furniture can be reloaded with ammo (turret mount) | STUB OK |

### Nature / Organic Flags

| Flag | Behavior | Priority |
|---|---|---|
| `TREE` | This is a tree — affects logging, movement, and wind | IMPLEMENT FULLY |
| `SHRUB` | This is a shrub — smaller than tree, may be trampled | IMPLEMENT FULLY |
| `YOUNG` | Young tree — grows into full tree over time | STUB OK |
| `FLOWER` | This is a flower — decorative, may affect mood | STUB OK |
| `ORGANIC` | Partly organic material — affected by fungal conversion | IMPLEMENT FULLY |
| `PLANT` | This is a planted crop | STUB OK |
| `PLANTABLE` | Soil can be planted on | STUB OK |
| `PLOWABLE` | Soil can be plowed for farming | STUB OK |
| `FISHABLE` | Water tile where fishing is possible | STUB OK |
| `GRAZABLE` | Animals can graze here | STUB OK |
| `GRAZER_INEDIBLE` | Present but inedible for grazers | STUB OK |
| `BROWSABLE` | Animals can browse (eat leaves/twigs) | STUB OK |
| `FUNGUS` | Covered in fungus — used for fungal conversion logic | IMPLEMENT FULLY |
| `HARVESTED` | Has been harvested — won't produce fruit until regrown | STUB OK |
| `NATURAL_UNDERGROUND` | Natural underground terrain (cave walls, rock) | IMPLEMENT FULLY |

### Growth Stage Flags

| Flag | Behavior | Priority |
|---|---|---|
| `GROWTH_SEED` | Plant at seed stage | STUB OK |
| `GROWTH_SEEDLING` | Plant at seedling stage | STUB OK |
| `GROWTH_MATURE` | Plant at mature stage | STUB OK |
| `GROWTH_HARVEST` | Plant ready for harvest | STUB OK |
| `GROWTH_OVERGROWN` | Plant overgrown past harvest | STUB OK |
| `HARVEST_REQ_CUT1` | Harvesting requires a cutting tool (quality 1+) | STUB OK |

### Interactive Flags

| Flag | Behavior | Priority |
|---|---|---|
| `CONSOLE` | Computer console — triggers examine interaction | STUB OK |
| `MOUNTABLE` | Can mount equipment/weapons on this | STUB OK |
| `CAN_SIT` | Entity can sit on this (chair, bench) | STUB OK |
| `FLAT_SURF` | Flat hard surface for placing items (table, counter) | IMPLEMENT FULLY |
| `BUTCHER_EQ` | Provides butchering surface | STUB OK |
| `NANOFAB_TABLE` | Nanofabrication table | STUB OK |
| `AUTODOC` | Autodoc machine | STUB OK |
| `AUTODOC_COUCH` | Autodoc patient couch | STUB OK |
| `TRANSLOCATOR` | Teleportation pad | STUB OK |
| `TRANSLOCATOR_GREATER` | Greater teleportation pad | STUB OK |
| `WORKOUT_ARMS` | Arm exercise equipment | STUB OK |
| `WORKOUT_LEGS` | Leg exercise equipment | STUB OK |
| `SIGN` | Sign — displays text when examined (if adjacent) | IMPLEMENT FULLY |
| `SIGN_ALWAYS` | Sign — always displays text (even from distance) | STUB OK |
| `ACTIVE_GENERATOR` | Active power generator | STUB OK |
| `ALIGN_WORKBENCH` | Workbench aligns to adjacent wall | STUB OK |

### Weather / Environment Flags

| Flag | Behavior | Priority |
|---|---|---|
| `BLOCK_WIND` | Partially blocks wind | STUB OK |
| `THIN_ICE` | Thin ice — may break under weight | STUB OK |
| `THICK_ICE` | Thick ice — safe to walk on | STUB OK |
| `SWIM_UNDER` | Can swim under this tile (underwater passage) | STUB OK |
| `SALT_WATER` | Salt water (affects thirst, corrosion) | STUB OK |
| `MURKY` | Murky water — reduced visibility below surface | STUB OK |
| `NO_FLOOR_WATER` | Water tile without a floor (deep pool/chasm) | STUB OK |
| `CHOCOLATE` | Made of chocolate (yes, this is real — seasonal event) | STUB OK |

### Connection / Rendering Flags

| Flag | Behavior | Priority |
|---|---|---|
| `AUTO_WALL_SYMBOL` | Symbol is automatically selected based on neighbor connections | IMPLEMENT FULLY |
| `CONNECT_WITH_WALL` | Connects visually with the WALL connect group | IMPLEMENT FULLY |
| `NO_SELF_CONNECT` | Does not connect to other tiles of the same type | STUB OK |
| `ONE_DIMENSIONAL_X` | Only connects along the X axis | STUB OK |
| `ONE_DIMENSIONAL_Y` | Only connects along the Y axis | STUB OK |
| `ONE_DIMENSIONAL_Z` | Only connects along the Z axis | STUB OK |

### Special / Misc Flags

| Flag | Behavior | Priority |
|---|---|---|
| `ALLOW_FIELD_EFFECT` | Fields (fire, gas) can affect this tile even if normally blocked | IMPLEMENT FULLY |
| `HIDE_PLACE` | Creature on this tile is hidden from non-adjacent observers | STUB OK |
| `SMALL_HIDE` | Only small creatures can hide here | STUB OK |
| `RUBBLE` | This is rubble — can be cleared | IMPLEMENT FULLY |
| `PIT_FILLABLE` | Pit that can be filled in | STUB OK |
| `DIGGABLE` | Ground can be dug (shovel/digging tool) | STUB OK |
| `DIGGABLE_CAN_DEEPEN` | Digging makes it deeper (shallow pit → deep pit) | STUB OK |
| `EASY_DECONSTRUCT` | Can be deconstructed without tools | IMPLEMENT FULLY |
| `BURROWABLE` | Creatures can burrow through | STUB OK |
| `PHASE_BACK` | Entities can phase through (ethereal movement) | STUB OK |
| `MON_AVOID_STRICT` | Monsters strictly avoid this tile | IMPLEMENT FULLY |
| `REGION_PSEUDO` | Abstract placeholder — resolved by region settings post-mapgen (see section above) | IMPLEMENT FULLY |

**Total:** 139 flags. **IMPLEMENT FULLY:** ~40 flags. **STUB OK:** ~99 flags.

> **PORTING TRAP:** Flags are stored both as strings (`set<string>`) for extensibility and as a bitset (`enum_bitset<ter_furn_flag>`) for performance. In Wulfaz, use a bitset for the known flags (the 139 above) and keep a `HashSet<String>` for mod-added custom flags. The bitset is checked thousands of times per tick during pathfinding and FOV — string comparison would be a significant performance problem.

---

## TRANSFORM SUMMARY

All terrain/furniture transforms in one place for the porting LLM:

| Transform Type | Trigger | Terrain Result | Furniture Result |
|---|---|---|---|
| Open | Player/NPC opens | `ter_t::open` | `furn_t::open` |
| Close | Player/NPC closes | `ter_t::close` | `furn_t::close` |
| Bash | Force/strength | `bash.ter_set` | `bash.furn_set` |
| Deconstruct | Careful dismantling | `deconstruct.ter_set` | `deconstruct.furn_set` |
| Lockpick | Lockpick tool | `ter_t::lockpick_result` | `furn_t::lockpick_result` |
| Boltcut | Bolt cutter tool | `boltcut.result_` | `boltcut.result_` |
| Hacksaw | Hacksaw tool | `hacksaw.result_` | `hacksaw.result_` |
| Oxytorch | Oxy-acetylene torch | `oxytorch.result_` | `oxytorch.result_` |
| Prying | Pry bar tool | `prying.result_` | `prying.result_` |
| Generic transform | Time/condition | `ter_t::transforms_into` | — |
| Phase change | Temperature | `ter_t::phase_targets[]` | — |
| Curtain toggle | Player action | `curtain_transform` | — |
| Plant growth | Time | — | `plant.transform` |
| Roof relationship | Z-level gen | `ter_t::roof` (→ z+1 floor) | — |
| ter_furn_transform | Spell/effect | `ter_transform[id].pick()` | `furn_transform[id].pick()` |
| Region resolve | Post-mapgen | regional weighted list | regional weighted list |
