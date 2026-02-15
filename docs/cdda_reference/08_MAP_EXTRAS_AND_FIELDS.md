# 08 — Map Extras, Fields, and Traps

**Scope:** Post-generation map extras, the field system (fire/gas/smoke), and the trap system.
**Purpose:** The consuming LLM uses this to implement the systems that add detail after base mapgen and provide dynamic environmental effects.

---

## MAP EXTRAS

Map extras are **post-generation additions** — scattered features (crashed vehicles, corpse scenes, loot stashes, environmental hazards) applied after base mapgen completes.

### map_extra Type

JSON type: `"map_extra"`.

| Field | C++ Type | JSON Key | Description |
|---|---|---|---|
| `id` | `map_extra_id` | `"id"` | Unique identifier |
| `generator_id` | `string` | `"generator_id"` | ID of the generator function or mapgen |
| `generator_method` | `enum` | `"generator_method"` | How this extra is generated (see below) |
| `autonote` | `bool` | `"autonote"` | Create map note when spawned |
| `symbol` | `uint32_t` | `"sym"` | Overmap display symbol |
| `color` | `nc_color` | `"color"` | Overmap display color |
| `name` | `translation` | `"name"` | Display name |
| `description` | `translation` | `"description"` | Description text |
| `min_max_zlevel` | `pair<int,int>?` | `"min_max_zlevel"` | Z-level range constraint |
| `flags` | `set<string>` | `"flags"` | Behavior flags |

### Generator Methods

| Method | Description |
|---|---|
| `map_extra_function` | C++ function pointer — hardcoded generation logic |
| `mapgen` | JSON mapgen entry — generates extra using standard mapgen |
| `update_mapgen` | JSON update_mapgen — modifies existing terrain |
| `null` | No generator (marker only) |

### How Extras Are Applied

1. After base mapgen generates terrain, the system checks `region_settings_map_extras`
2. For each `map_extra_collection`, roll `one_in(chance)` to trigger
3. If triggered, `filtered_by(mapgendata)` removes invalid extras (wrong z-level, wrong flags)
4. Pick one extra from the filtered weighted list
5. Apply the extra's generator to the map

### Invocation

```cpp
MapExtras::apply_function(extra_id, map, position);
```

- **Function-based:** Calls the C++ function pointer directly
- **Mapgen-based:** Runs a standard mapgen entry on the map
- **Update-based:** Runs an update_mapgen entry on the existing map

> **STUB OK — TODO:** Map extras add post-apocalyptic flavor (crashed helicopters, roadblocks, etc.). Implement the dispatch system first, add individual extras incrementally.

---

## FIELD SYSTEM

Fields represent environmental effects on tiles — fire, smoke, gas, blood, radiation, etc. Unlike terrain and furniture, multiple field types can coexist on a single tile, and fields have **intensity levels** that change over time.

### field_type

JSON type: `"field_type"`. Defined in `src/field_type.h`.

| Field | C++ Type | JSON Key | Description |
|---|---|---|---|
| `id` | `field_type_str_id` | `"id"` | Unique identifier (e.g. `"fd_fire"`) |
| `intensity_levels` | `vec<field_intensity_level>` | `"intensity_levels"` | 1+ levels (see below) |
| `priority` | `int` | `"priority"` | Display priority (higher = drawn on top) |
| `half_life` | `time_duration` | `"half_life"` | Time for field to decay one intensity |
| `decay_amount_factor` | `int` | `"decay_amount_factor"` | Multiplier for decay rate |
| `percent_spread` | `int` | `"percent_spread"` | % chance to spread to adjacent tile per turn |
| `phase` | `phase_id` | `"phase"` | Physical phase: GAS, LIQUID, SOLID |
| `underwater_age_speedup` | `time_duration` | `"underwater_age_speedup"` | Faster aging underwater |
| `outdoor_age_speedup` | `time_duration` | `"outdoor_age_speedup"` | Faster aging outdoors |
| `accelerated_decay` | `bool` | `"accelerated_decay"` | Non-linear decay |
| `dangerous` | `bool` | `"dangerous"` | Harms entities on contact |
| `transparent` | `bool` | `"transparent"` | Doesn't block vision |
| `moppable` | `bool` | `"moppable"` | Can be mopped up |
| `indestructible` | `bool` | `"indestructible"` | Immune to removal |
| `display_items` | `bool` | `"display_items"` | Items visible through field |
| `display_field` | `bool` | `"display_field"` | Field itself visible |
| `has_fire` | `bool` | `"has_fire"` | Triggers fire interactions |
| `has_acid` | `bool` | `"has_acid"` | Triggers acid interactions |
| `has_elec` | `bool` | `"has_elec"` | Triggers electrical interactions |
| `has_fume` | `bool` | `"has_fume"` | Triggers fume interactions |
| `wandering_field` | `field_type_str_id` | `"wandering_field"` | Field type created when spreading |
| `bash_info` | `map_fd_bash_info?` | `"bash"` | How field responds to bashing |
| `immunity_data` | `field_immunity_data` | `"immunity_data"` | What grants immunity |
| `immune_mtypes` | `set<mtype_id>` | `"immune_mtypes"` | Monster types immune to this field |

### field_intensity_level

Each field has 1+ intensity levels (typically 3: light, medium, heavy):

| Field | Type | Description |
|---|---|---|
| `name` | `translation` | Display name at this intensity |
| `symbol` | `uint32_t` | Display character |
| `color` | `nc_color` | Display color |
| `dangerous` | `bool` | Harmful at this intensity |
| `transparent` | `bool` | Vision pass-through at this intensity |
| `move_cost` | `int` | Movement speed penalty |
| `light_emitted` | `float` | Light output |
| `translucency` | `float` | Partial transparency amount |
| `concentration` | `int` | Gas concentration level |
| `convection_temperature_mod` | `int` | Temperature effect |
| `intensity_upgrade_chance` | `int` | Chance to escalate to next level |
| `intensity_upgrade_duration` | `time_duration` | Time between escalation checks |
| `monster_spawn_chance` | `int` | Chance to spawn monsters |
| `monster_spawn_group` | `mongroup_id` | Monster group to spawn |
| `field_effects` | `vec<field_effect>` | Status effects applied to entities |
| `extra_radiation_min/max` | `int` | Radiation damage range |
| `scent_neutralization` | `int` | Scent reduction |

### Common Field Types

| ID | Description | Has Fire | Spreads |
|---|---|---|---|
| `fd_fire` | Active fire | Yes | Yes (to FLAMMABLE terrain) |
| `fd_smoke` | Smoke | No | Yes (rises) |
| `fd_blood` | Blood splatter | No | No |
| `fd_acid` | Corrosive acid | No | Slowly |
| `fd_toxic_gas` | Toxic gas | No | Yes |
| `fd_tear_gas` | Tear gas | No | Yes |
| `fd_electricity` | Electrical arc | No | No |
| `fd_plasma` | Plasma | Yes | No |
| `fd_incendiary` | Incendiary material | Yes | Yes |
| `fd_fungal_haze` | Fungal spores | No | Yes |
| `fd_slime` | Slime trail | No | No |
| `fd_web` | Spider web | No | No |

> **STUB OK — TODO:** Field propagation (fire spreading to adjacent FLAMMABLE tiles, smoke rising, gas diffusing) is a per-tick simulation algorithm. For initial implementation, placed fields are static — they exist where mapgen put them but don't spread. The field_type data model should be fully implemented so fields can be placed and rendered. Propagation is a separate TODO.

> **IMPLEMENT FULLY:** The `field_type` data model (loading, storage per tile, rendering) must work from day one — mapgen places fields and they must be visible.

---

## TRAP SYSTEM

Traps are hidden or visible hazards placed on tiles. Each tile can have at most one trap (stored in the `trp` layer of `maptile_soa`).

### trap Type

JSON type: `"trap"`. Defined in `src/trap.h`.

| Field | C++ Type | JSON Key | Description |
|---|---|---|---|
| `id` | `trap_str_id` | `"id"` | Unique identifier (e.g. `"tr_beartrap"`) |
| `sym` | `int` | `"sym"` | ASCII display symbol |
| `color` | `nc_color` | `"color"` | Display color |
| `name` | `translation` | `"name"` | Display name |
| `visibility` | `int` | `"visibility"` | Detection difficulty (lower = easier to spot) |
| `avoidance` | `int` | `"avoidance"` | Difficulty to avoid triggering |
| `difficulty` | `int` | `"difficulty"` | Disarming difficulty (0=trivial, 99=impossible) |
| `trap_radius` | `int` | `"trap_radius"` | Area of effect (0 = single tile) |
| `benign` | `bool` | `"benign"` | Non-dangerous (e.g. funnel, cot) |
| `always_invisible` | `bool` | `"always_invisible"` | Never shown on map |
| `trigger_weight` | `mass` | `"trigger_weight"` | Minimum weight to trigger (default: 500g) |
| `sound_threshold` | `pair<int,int>` | `"sound_threshold"` | Sound level range that triggers |
| `funnel_radius_mm` | `int` | `"funnel_radius_mm"` | Water collection radius (funnel traps) |
| `comfort` | `int` | `"comfort"` | Comfort level (cot, bed roll) |
| `floor_bedding_warmth` | `temperature_delta` | `"floor_bedding_warmth"` | Warmth (sleeping bag) |
| `act` | `trap_function` | `"action"` | C++ trigger function |
| `map_regen` | `update_mapgen_id` | `"map_regen"` | Map regeneration on trigger |
| `spell_data` | `fake_spell` | `"spell_data"` | Spell cast on trigger |
| `eocs` | `vec<eoc_id>` | `"eocs"` | Effects on conditions |
| `components` | `vec<comp>` | `"components"` | Disassembly yields |
| `flags` | `set<flag_id>` | `"flags"` | Behavior flags |
| `vehicle_data` | `vehicle_handle_trap_data` | `"vehicle_data"` | Vehicle interaction |

### Trap Functions (C++ actions)

Common trap functions from the `trapfunc` namespace:

| Function | Description |
|---|---|
| `none` | No effect (marker only) |
| `beartrap` | Leg hold — immobilizes |
| `snare_light` / `snare_heavy` | Snare traps — varying strength |
| `board` | Board/nail trap — foot damage |
| `caltrops` / `caltrops_glass` | Caltrops — foot damage |
| `tripwire` | Tripwire — stumble/fall |
| `crossbow` / `shotgun` | Ranged turret traps |
| `landmine` | Explosive mine |
| `pit` / `pit_spikes` / `pit_glass` | Pit traps — falling damage |
| `blade` | Spinning blade trap |
| `boobytrap` | Generic explosive |
| `goo` | Sticky goo |
| `lava` | Lava tile damage |
| `sinkhole` | Ground collapse |
| `cast_spell` | Casts a defined spell |
| `map_regen` | Regenerates terrain (update_mapgen) |

### How Mapgen Places Traps

1. Via palette symbol mapping: `"traps": { "^": "tr_beartrap" }`
2. Via `place_traps` key: `{ "trap": "tr_beartrap", "x": 8, "y": 8, "chance": 10 }`
3. Via terrain's inherent trap: `ter_t::trap_id_str` → some terrain types have built-in traps

> **STUB OK — TODO:** Trap trigger logic is complex (weight thresholds, sound detection, visibility checks, disarming). For initial implementation, load trap definitions and place them via mapgen. The trigger/disarm mechanics are a separate TODO. Traps should be visible/renderable from day one.

> **PORTING TRAP:** Some "traps" in CDDA are actually benign furniture-like objects (cots, bed rolls, funnels). They use the trap system because they need per-tile placement with visibility rules. The porting LLM should not assume all traps are hazards.
