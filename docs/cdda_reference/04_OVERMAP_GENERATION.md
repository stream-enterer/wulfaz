# 04 — Overmap Generation

**Scope:** Overmap terrain types, overmap specials, connections, locations, city/highway/water generation, and noise-based terrain distribution.
**Purpose:** The consuming LLM uses this to implement world-scale map structure — the 180x180 grid of overmap terrain tiles and how they are generated.

---

## OVERMAP TERRAIN TYPE: oter_type_t

Defined in `src/omdata.h`. JSON type: `"overmap_terrain"`. This is the **base type** — individual rotation/line variants are generated at finalize time as `oter_t` instances.

### Key Fields

| Field | C++ Type | JSON Key | Description |
|---|---|---|---|
| `id` | `oter_type_str_id` | `"id"` | Unique identifier (e.g. `"forest"`, `"house"`) |
| `name` | `translation` | `"name"` | Display name |
| `symbol` | `uint32_t` | `"sym"` | Unicode glyph for overmap display |
| `color` | `nc_color` | `"color"` | Display color |
| `see_cost` | `see_costs` | `"see_cost"` | Vision obstruction level |
| `travel_cost_type` | `oter_travel_cost_type` | `"travel_cost_type"` | Pathfinding cost category |
| `land_use_code` | `overmap_land_use_code_id` | `"land_use_code"` | Land classification for alternate display |
| `default_map_data` | `map_data_summary_id` | `"default_map_data"` | Default mapgen data |
| `looks_like` | `vec<string>` | `"looks_like"` | Tileset inheritance chain |
| `extras` | `string` | `"extras"` | Extra identifier for map_extras system |
| `mondensity` | `int` | `"mondensity"` | Monster spawn density multiplier |
| `entry_EOC` | `eoc_id` | `"entry_EOC"` | Effect triggered on player entry |
| `exit_EOC` | `eoc_id` | `"exit_EOC"` | Effect triggered on player exit |
| `static_spawns` | `overmap_static_spawns` | `"static_spawns"` | One-time creature spawns |
| `uniform_terrain` | `ter_str_id?` | `"uniform_terrain"` | If set, all submaps use this terrain (optimization) |
| `flags` | `bitset<oter_flags>` | `"flags"` | Behavior flags (see below) |
| `connect_group` | `string` | `"connect_group"` | Group for connection rendering |

### See Cost Enum

Controls vision obstruction on the overmap:

| Value | Meaning | Approximate Opacity |
|---|---|---|
| `all_clear` | Fully transparent (open terrain) | 0 |
| `none` | No horizontal obstacles | 0 |
| `low` | Low obstacles (fences, hedges) | ~0.1 |
| `medium` | Medium obstacles (scattered trees) | ~0.2 |
| `spaced_high` | Tall but scattered (sparse forest) | ~0.4 |
| `high` | Dense obstruction (thick forest) | ~0.9 |
| `full_high` | Nearly opaque (building walls) | ~0.99 |
| `opaque` | Fully blocked (solid structure) | 1.0 |

### Travel Cost Type Enum

| Value | Use |
|---|---|
| `other` | Default/unclassified |
| `impassable` | Cannot travel through |
| `highway` | Fastest road travel |
| `road` | Normal road |
| `field` | Open field |
| `dirt_road` | Unpaved road |
| `trail` | Hiking trail |
| `forest` | Forest (slow) |
| `shore` | Shoreline |
| `swamp` | Swamp (very slow) |
| `water` | Water body |
| `air` | Open air (z > 0) |
| `structure` | Building interior |
| `roof` | Rooftop |
| `basement` | Underground structure |
| `tunnel` | Tunnel system |

### oter_flags

| Flag | Description | Priority |
|---|---|---|
| `known_down` | Has known stairs/access going down | IMPLEMENT FULLY |
| `known_up` | Has known stairs/access going up | IMPLEMENT FULLY |
| `no_rotate` | Single variant — no directional suffixes | IMPLEMENT FULLY |
| `should_not_spawn` | Excluded from random placement | IMPLEMENT FULLY |
| `water` | Water body (generic) | IMPLEMENT FULLY |
| `river_tile` | River terrain | IMPLEMENT FULLY |
| `has_sidewalk` | Has sidewalks (city roads) | STUB OK |
| `road` | Road terrain | IMPLEMENT FULLY |
| `highway` | Highway terrain | IMPLEMENT FULLY |
| `highway_reserved` | Highway placeholder (pre-finalization) | STUB OK |
| `highway_special` | Highway intersection/ramp | STUB OK |
| `bridge` | Bridge over water | STUB OK |
| `ignore_rotation_for_adjacency` | Don't consider rotation in neighbor checks | STUB OK |
| `line_drawing` | Uses 16-variant line system (roads, rivers) | IMPLEMENT FULLY |
| `subway_connection` | Subway network tile | STUB OK |
| `requires_predecessor` | Must connect to existing terrain | STUB OK |
| `lake` | Lake surface | IMPLEMENT FULLY |
| `lake_shore` | Lake shoreline | IMPLEMENT FULLY |
| `ocean` | Ocean surface | STUB OK |
| `ocean_shore` | Ocean shoreline | STUB OK |
| `ravine` | Ravine terrain | STUB OK |
| `ravine_edge` | Ravine edge | STUB OK |
| `generic_loot` | Contains generic loot | STUB OK |
| `risk_extreme/high/low` | Danger level indicators | STUB OK |
| `source_*` (16 types) | Resource source indicators (ammo, food, medicine, etc.) | STUB OK |

---

## THE DIRECTIONAL SUFFIX SYSTEM

CDDA overmap terrains can have directional variants. At finalize time, `oter_type_t::finalize()` generates `oter_t` instances with suffixes.

### Three Modes

**1. Rotatable terrains** (default — no `no_rotate` or `line_drawing` flag):

Generates 4 variants:
```
forest_north  (dir=0, rotation=0°)
forest_east   (dir=1, rotation=90°)
forest_south  (dir=2, rotation=180°)
forest_west   (dir=3, rotation=270°)
```

Rotation adds direction values modulo 4:
```
get_rotated(current_dir + rotation) = directional_peers[(current + rot) % 4]
```

**2. Line-drawing terrains** (`line_drawing` flag set):

Generates 16 variants using a 4-bit NESW connectivity mask:

| Bits | Index | Suffix | Symbol | Shape |
|---|---|---|---|---|
| `0000` | 0 | `_isolated` | (none) | No connections |
| `0001` | 1 | `_end_south` | `│` | Dead end facing south |
| `0010` | 2 | `_end_west` | `─` | Dead end facing west |
| `0011` | 3 | `_ne` | `└` | Corner NE |
| `0100` | 4 | `_end_north` | `│` | Dead end facing north |
| `0101` | 5 | `_ns` | `│` | Straight N-S |
| `0110` | 6 | `_es` | `┌` | Corner ES |
| `0111` | 7 | `_nes` | `├` | T-junction NES |
| `1000` | 8 | `_end_east` | `─` | Dead end facing east |
| `1001` | 9 | `_wn` | `┘` | Corner WN |
| `1010` | 10 | `_ew` | `─` | Straight E-W |
| `1011` | 11 | `_new` | `┴` | T-junction NEW |
| `1100` | 12 | `_sw` | `┐` | Corner SW |
| `1101` | 13 | `_nsw` | `┤` | T-junction NSW |
| `1110` | 14 | `_esw` | `┬` | T-junction ESW |
| `1111` | 15 | `_nesw` | `┼` | Four-way |

The bits encode: `W=bit3, S=bit2, E=bit1, N=bit0`.

Line rotation: `rotate(line, dir)` = bit-rotate left by dir positions within 4 bits.

Mapgen uses 5 suffixes mapping from line index:
```
_straight  (indices 5, 10)
_curved    (indices 3, 6, 9, 12)
_end       (indices 1, 2, 4, 8)
_tee       (indices 7, 11, 13, 14)
_four_way  (index 15)
```

**3. Non-rotatable terrains** (`no_rotate` flag):

Single variant with no suffix. The terrain ID is used directly.

> **IMPLEMENT FULLY:** The line-drawing system is critical for roads, rivers, and rail. Without it, all road tiles would show the same symbol regardless of which directions they connect to.

---

## oter_t — Overmap Terrain Instance

Each `oter_t` is a specific rotation/line variant of an `oter_type_t`. Stored in a global factory indexed by `oter_id` (integer) and `oter_str_id` (string).

### Fields

| Field | C++ Type | Description |
|---|---|---|
| `type` | `oter_type_t*` | Pointer to base type |
| `id` | `oter_str_id` | Full ID with suffix (e.g. `"road_ns"`) |
| `dir` | `om_direction::type` | Direction for rotatable types (north/east/south/west) |
| `symbol` | `uint32_t` | Rotated display symbol |
| `symbol_alt` | `uint32_t` | Land-use-code alternate symbol |
| `line` | `size_t` | Line index for line-drawing types (0-15) |

### Key Methods

```
get_mapgen_id()     → type_id + mapgen suffix (_straight, _curved, etc.)
get_rotated(dir)    → oter_id of rotated variant
get_dir()           → current direction
get_line()          → current line index
get_rotation()      → int rotation (0-3) for tileset rendering
has_connection(dir) → whether this tile connects in a given direction
```

> **PORTING TRAP:** The mapgen ID differs from the display ID. `road_ns` (display) maps to mapgen ID `road_straight` with rotation. Mapgen functions are keyed by the mapgen ID, not the display ID.

---

## OVERMAP SPECIALS (FIXED)

Overmap specials define multi-tile structures placed on the overmap (buildings, labs, military bases). JSON type: `"overmap_special"`.

### overmap_special (main class)

| Field | C++ Type | JSON Key | Description |
|---|---|---|---|
| `id` | `overmap_special_id` | `"id"` | Unique identifier |
| `subtype` | `enum` | `"subtype"` | `fixed` or `mutable_` |
| `constraints` | `placement_constraints` | (see below) | Placement rules |
| `data` | `overmap_special_data*` | (polymorphic) | Fixed or mutable data |
| `rotatable` | `bool` | `"rotatable"` | Whether special can be rotated (default: true) |
| `flags` | `set<string>` | `"flags"` | Behavior flags |
| `priority` | `int` | `"priority"` | Placement priority (higher = placed first) |
| `default_locations` | `set<overmap_location_id>` | `"locations"` | Default location constraints |
| `monster_spawns` | `overmap_special_spawns` | `"monster_spawns"` | Monster spawn config |
| `mapgen_params` | `mapgen_parameters` | `"mapgen_parameters"` | Parameters passed to mapgen |

### Placement Constraints

| Field | C++ Type | JSON Key | Description |
|---|---|---|---|
| `city_size` | `interval<int>` | `"city_sizes"` | Required city size range [min, max] |
| `city_distance` | `interval<int>` | `"city_distance"` | Required distance from nearest city [min, max] |
| `occurrences` | `interval<int>` | `"occurrences"` | How many times this can appear [min, max] |

### Overmap Special Flags

| Flag | Description |
|---|---|
| `CLASSIC` | Spawns in classic game mode |
| `URBAN` | Appropriate for urban areas |
| `WILDERNESS` | Appropriate for wilderness areas |
| `BLOB` | Scatter similar terrain in a 5x5 area around placement |
| `OVERMAP_UNIQUE` | At most 1 per overmap (180x180 area) |
| `GLOBALLY_UNIQUE` | At most 1 across all overmaps |
| `CITY_UNIQUE` | At most 1 per city |

### fixed_overmap_special_data

The `terrains[]` array defines what goes where:

```cpp
struct overmap_special_terrain {
    tripoint_rel_omt p;                    // Position offset within the special
    oter_str_id terrain;                   // Overmap terrain to place
    set<overmap_location_id> locations;    // Where this tile can be placed
    set<string> flags;                     // Placement flags
    optional<faction_id> camp_owner;       // Camp faction (if any)
    translation camp_name;                 // Camp name
};
```

The `connections[]` array defines roads/paths to the special:

```cpp
struct overmap_special_connection {
    tripoint_rel_omt p;                      // Connection origin point
    optional<tripoint_rel_omt> from;         // Hint for initial direction
    cube_direction initial_dir;              // Forced initial direction
    oter_type_str_id terrain;                // Target terrain type for the connection
    overmap_connection_id connection;         // Connection type (road, rail, etc.)
    bool existing;                           // Reuse existing terrain (don't overwrite)
};
```

### Placement Algorithm

1. For each overmap being generated, iterate specials by priority (highest first)
2. For each special, check `occurrences` constraint
3. Find candidate positions where all `terrains[].locations` are satisfied
4. Check `city_distance` and `city_size` constraints
5. Score rotation options: `score_rotation_at()` checks if terrain at each offset position matches the location constraint
6. Place terrains at offsets, then route connections

### JSON Example

```json
{
    "type": "overmap_special",
    "id": "Police Station",
    "subtype": "fixed",
    "overmaps": [
        { "point": [0, 0, 0], "overmap": "police_north", "locations": ["land"] }
    ],
    "connections": [
        { "point": [0, -1, 0], "terrain": "road", "connection": "local_road" }
    ],
    "city_distance": [0, 10],
    "city_sizes": [4, -1],
    "occurrences": [0, 3],
    "flags": ["CLASSIC", "URBAN"]
}
```

> **IMPLEMENT FULLY:** Fixed overmap specials are the primary mechanism for placing buildings, labs, and other structures. They must work for the world to have any interesting locations.

---

## OVERMAP SPECIALS (MUTABLE)

Mutable specials use a **constraint-satisfaction system** for procedural shape generation. Instead of a fixed array of terrains, they define **rules** with **join points** that connect pieces together.

### Core Concepts

- **Join points**: Connection interfaces between pieces. Each has an ID and an `opposite_id`.
- **Pieces**: Individual terrain tiles with joins on their faces.
- **Rules**: Patterns of pieces that can be placed together, with weight and max count.
- **Resolution**: Iteratively place rules, matching join points between pieces.

### Key Data Types

**mutable_overmap_join:**
| Field | Type | Description |
|---|---|---|
| `id` | `string` | Join identifier |
| `opposite_id` | `string` | ID of the complementary join |
| `into_locations` | `set<overmap_location_id>` | Where this join can connect |
| `priority` | `unsigned` | Resolution priority |

**mutable_overmap_terrain:**
| Field | Type | Description |
|---|---|---|
| `terrain` | `oter_str_id` | Terrain to place |
| `locations` | `set<overmap_location_id>` | Placement constraints |
| `joins` | `map<direction, join>` | Joins on each face |
| `connections` | `map<direction, connection>` | Road/path connections |

**mutable_overmap_placement_rule:**
| Field | Type | Description |
|---|---|---|
| `name` | `string` | Rule name |
| `pieces` | `vec<piece>` | Terrain pieces in this rule |
| `outward_joins` | `vec<(pos_dir, join)>` | External joins (connect to other rules) |
| `max` | `int_distribution` | Maximum instances of this rule |
| `weight` | `int` | Selection weight |

> **STUB OK — TODO:** Mutable overmap specials are a complex constraint-satisfaction system. For initial implementation, only support fixed overmap specials. The mutable special data schema should be documented so the data model is ready, but the generation algorithm should be marked as TODO.

---

## CITY_BUILDING

City buildings are **NOT a separate data type**. They are dynamically created as `overmap_special` instances from `oter_type_t` terrain types:

```cpp
overmap_special_id create_building_from(const oter_type_str_id& base) {
    // Create a fixed overmap_special with:
    // - Single terrain at (0,0,0)
    // - Default locations: land + swamp
    // - No connections
    // - No placement constraints
    // - ID: "FakeSpecial_" + base_type_id
    return new_special_id;
}
```

### Key Differences from True Overmap Specials

| Aspect | Overmap Special | City Building |
|---|---|---|
| **Size** | Multi-tile (any shape) | Single OMT (1 tile) |
| **Connections** | Has road/path connections | No connections |
| **Constraints** | city_distance, city_sizes, occurrences | None (placed by city gen) |
| **Selection** | Globally by occurrence count | By `building_bin` weighted pools |
| **Placement** | Anywhere matching constraints | Only in city building slots |

### How City Gen Selects Buildings

City generation creates building slots along streets. Each slot is filled by querying `region_settings_city`'s three `building_bin` pools:

1. Calculate `town_dist` = distance from building slot to city center (as percentage of city size)
2. Roll against normal distributions:
   - If `town_dist < shop_normal`: pick from **shops** pool
   - Else if `town_dist < park_normal`: pick from **parks** pool
   - Else: pick from **houses** pool
3. Check uniqueness constraints (`CITY_UNIQUE`, `OVERMAP_UNIQUE`, `GLOBALLY_UNIQUE`)
4. Retry up to 10 times if constraints fail

Default parameters: `shop_radius=30, shop_sigma=20, park_radius=30, park_sigma=70`.

> **IMPLEMENT FULLY:** City building selection is essential for populated cities. Without building_bin, all city slots would be empty.

---

## OVERMAP CONNECTION

Connections define how roads, trails, rail, sewers, and subways route between points on the overmap. JSON type: `"overmap_connection"`.

### Structure

```cpp
class overmap_connection {
    overmap_connection_id id;
    list<subtype> subtypes;       // Multiple terrain options for routing

    class subtype {
        oter_type_str_id terrain;        // Terrain type for this connection segment
        int basic_cost;                  // Pathfinding cost (lower = preferred)
        set<overmap_location_id> locations;  // Where this subtype can be placed
        set<flag> flags;                 // orthogonal, perpendicular_crossing
    };
};
```

### Subtype Flags

| Flag | Description |
|---|---|
| `orthogonal` | Can only follow orthogonal (N/S/E/W) paths |
| `perpendicular_crossing` | Can cross perpendicular connections |

### Routing Behavior

When placing a connection between two overmap specials:

1. **Pick subtype** for the ground terrain at each position: `pick_subtype_for(ground_oter)` checks which subtype's `locations` match the current terrain
2. **Pathfind** using A* with `basic_cost` as edge weight
3. **Place** connection terrain along the path, using the line-drawing system for the terrain type

A connection can have **multiple subtypes** — for example, a road connection might have a "paved road on grass" subtype and a "bridge on water" subtype, allowing roads to cross water by switching to bridge terrain.

> **IMPLEMENT FULLY:** Connections are how roads reach buildings. Without them, overmap specials float in isolation with no road access.

---

## OVERMAP LOCATION

Locations define **where** overmap specials and connections can be placed. JSON type: `"overmap_location"`.

### Structure

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `overmap_location_id` | `"id"` | Unique identifier (e.g. `"land"`, `"water"`, `"swamp"`) |
| `terrains` | `set<oter_type_str_id>` | `"terrains"` | Compatible terrain type IDs |
| `flags` | `vec<string>` | `"flags"` | Named oter_flags — at finalize, all terrains with these flags are added |

### How It Works

- At finalize time, all `oter_t` instances are scanned. Any terrain matching a flag in `flags` is added to `terrains`.
- `test(oter_id)` checks if the given terrain's type is in the `terrains` set.
- Overmap specials reference locations by ID. Each terrain position in a special has a set of allowed locations.

### Common Locations

| ID | Matches |
|---|---|
| `land` | Default field/forest/etc. terrain |
| `water` | Water body terrain |
| `swamp` | Swamp terrain |
| `wilderness` | Non-city terrain |

> **IMPLEMENT FULLY:** Locations prevent buildings from being placed on water and roads from being placed on buildings. Without them, placement is unconstrained chaos.

---

## OVERMAP LAND USE CODE

An alternative display layer for the overmap. When viewing by land use code, each terrain shows its land_use_code's symbol and color instead of its own.

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `overmap_land_use_code_id` | `"id"` | Identifier |
| `land_use_code` | `int` | `"land_use_code"` | Numeric classification code |
| `name` | `translation` | `"name"` | Display name |
| `detailed_definition` | `translation` | `"detailed_definition"` | Long description |
| `symbol` | `uint32_t` | `"sym"` | Display glyph |
| `color` | `nc_color` | `"color"` | Display color |

Predefined codes: `forest`, `wetland`, `wetland_forest`, `wetland_saltwater`.

> **STUB OK — TODO:** Land use codes are a display convenience. Store them in the data model but the alternate display mode can be deferred.

---

## CITY GENERATION PIPELINE

City generation happens during overmap creation. The pipeline is a sequence of steps defined primarily in `src/overmap_city.cpp`.

### Step 1: City Count Calculation

```
omts_per_overmap = OMAPX * OMAPY                    (= 32,400)
city_map_coverage_ratio = 1.0 / 2^(CITY_SPACING)    (option: 0-10)
omts_per_city = (city_size * 2 + 1)^2 * 3/4
num_cities = omts_per_overmap * coverage_ratio / omts_per_city
```

### Step 2: City Center Placement

- Find candidate positions on default terrain (field/forest)
- Randomly select from candidates
- Remove nearby candidates (within 2-tile radius) to prevent overlap
- Place initial road intersection (4-way) at center

### Step 3: City Size Randomization

```
base_size = rng(CITY_SIZE - 1, max_city_size)
Modifier:
  33% → tiny:  size * 1/3
  33% → small: size * 2/3
  17% → large: size * 3/2
  17% → huge:  size * 2
Clamp to [2, 55]
```

### Step 4: Street Network Generation

For each city, iterate 4 cardinal directions from center:
1. Lay out main street using `build_city_street(direction, full_size)`
2. At each step, 1-in-4 chance (`BUILDINGCHANCE=4`) to place building on each side
3. Spawn perpendicular streets at intervals
4. Alternating block widths: 2-tile and 3-5 tile blocks
5. Recursively shrink street length as distance increases

### Step 5: Building Placement

For each building slot:
1. Calculate `town_dist = (distance_to_center * 100) / max(city_size, 1)`
2. Select from building_bin pools (shops/parks/houses) based on town_dist
3. Check `CITY_UNIQUE` / `OVERMAP_UNIQUE` / `GLOBALLY_UNIQUE` flags
4. Retry up to 10 times if constraints fail
5. Place the selected overmap_special at the slot position

### Step 6: Flood Fill

After streets and buildings placed, flood-fill from city tiles to mark enclosed areas as part of the city.

> **STUB OK — TODO:** Road network generation within cities is algorithmically complex. For initial implementation, a simplified grid-based road layout is acceptable. Document CDDA's actual algorithm for future reference but mark the full algorithm as TODO.

---

## HIGHWAY GENERATION

Inter-city highway placement is managed by `src/overmap_highway.cpp`.

### Pipeline

1. **Grid calculation**: Highway intersection points on a global grid with configurable spacing and variance
2. **Connection identification**: Determine which cardinal directions need highways based on grid neighbors
3. **Endpoint selection**: Find edge points, apply random deviation (`HIGHWAY_MAX_DEVIANCE`)
4. **Path drawing**: Connect endpoints through center using segments:
   - 2-way: straight or 90° corner
   - 3-way: T-intersection
   - 4-way: cross intersection
5. **Bend placement**: Insert curve segments from weighted `building_bin`
6. **Water crossing**: Detect water, insert bridge/ramp specials
7. **Finalization**: Replace highway placeholder terrain with actual highway terrain

Configuration: `width_of_segments=2`, `straightness_chance=0.6`, various overmap_special IDs for segments/bends/intersections.

> **STUB OK — TODO:** Highway generation between cities is algorithmically complex. For initial implementation, simplified straight-line highways between cities are acceptable.

---

## WATER BODY GENERATION

### Rivers (`src/overmap_water.cpp`)

1. **Endpoint selection**: Up to 2 major rivers per overmap. Start from N/W edge, end at S/E edge. Continue rivers from adjacent overmaps.
2. **Bezier curve**: Generate smooth path using control points at 1/3 and 2/3 of the distance, offset by amplitude = distance/2.
3. **River tracing**: Place river tiles in radius (`river_scale`) around curve points. Apply perpendicular meander at each step.
4. **Branching**: Every `river_branch_chance` tiles, spawn a branch. Branches either re-merge downstream or extend outward. Recursive with `river_scale - branch_scale_decrease`.
5. **Shore generation**: For each river tile, check 4-cardinal neighbors. Build directional mask → assign shore/center variants.

### Lakes

1. **Noise evaluation**: `om_noise_layer_lake` (8 octaves, 0.5 persistence, 0.002 frequency). Apply `pow(noise, 4)` for sharp thresholds.
2. **Flood fill**: For each tile exceeding `noise_threshold_lake`, flood-fill to find contiguous lake area.
3. **Size filter**: Skip lakes smaller than `lake_size_min` tiles.
4. **Rendering**: Interior tiles get `lake_surface`, edge tiles get `lake_shore`. Fill z-levels down to `lake_depth`.

### Oceans

Similar to lakes but with directional boundary parameters (`ocean_start_north/south/east/west`) and gradient adjustments near boundaries.

> **STUB OK — TODO:** Water body generation uses Bezier curves and multi-octave noise. For initial implementation, simplified river and lake placement (noise threshold + flood fill) is sufficient. Full branching/meander can be deferred.

---

## OVERMAP NOISE

Noise functions for terrain distribution are defined in `src/overmap_noise.h` and `.cpp`. Each noise layer uses `scaled_octave_noise_3d()`.

### Noise Layers

| Layer | Octaves | Persistence | Frequency | Post-Processing | Use |
|---|---|---|---|---|---|
| `forest` | 4 | 0.5 | 0.03 | `pow(n,2) - pow(n2,3)*0.5` | Forest density |
| `floodplain` | 4 | 0.5 | 0.05 | `pow(n,2)` | Swamp/floodplain |
| `lake` | 8 | 0.5 | 0.002 | `pow(n,4)` | Lake placement |
| `ocean` | 8 | 0.5 | 0.002 | `pow(n,4)` | Ocean placement |

### Key Properties

- **Seed derivation**: From game seed via modulo, ensures reproducibility
- **Global coordinates**: Uses absolute OMT position for cross-overmap continuity
- **Forest clearing**: The subtraction of a second noise term (`r - d * 0.5`) creates natural clearings in forests
- **Sharp thresholds**: `pow(n, 4)` for lakes/oceans makes them rare and clearly bounded

> **STUB OK — TODO:** Overmap noise tuning is extensive. For initial implementation, use simple Perlin noise with default parameters. CDDA's specific noise configuration is setting-dependent and won't be ported directly.

---

## omt_placeholder

An `omt_placeholder` is a temporary terrain type used during overmap generation. It marks positions that should be filled by a specific overmap special or connection but haven't been resolved yet.

During finalization:
1. The overmap generation pipeline places placeholders at positions earmarked for specials
2. Special placement resolves placeholders by replacing them with actual terrain
3. Highway generation uses `reserved_terrain_id` and `reserved_terrain_water_id` as placeholders
4. After all generation phases complete, any remaining placeholders are resolved or replaced with default terrain

> **STUB OK — TODO:** Placeholder resolution is an implementation detail of the generation pipeline. For initial implementation, generate terrain directly without a placeholder phase.

---

## OVERMAP GENERATION PHASE ORDER

The complete overmap generation sequence (from `overmap::generate()` in `src/overmap.cpp`):

```
 1. Calculate urbanity & forestosity (noise-based regional character)
 2. place_rivers()          — Bezier curves with branching
 3. place_lakes()           — Noise threshold + flood fill
 4. place_oceans()          — Boundary-directed noise
 5. place_forests()         — Multi-layer noise → forest density
 6. place_swamps()          — Floodplain noise near rivers
 7. place_ravines()         — Procedural gorge generation
 8. polish_river()          — Shore tile assignment
 9. place_highways()        — Grid-based inter-city routing
10. place_cities()          — City seed placement
11. place_highway_interchanges()
12. build_cities()          — Street network + building slots
13. place_forest_trails()   — Trails through large forests
14. place_roads()           — Inter-city road connections
15. place_railroads()       — Rail network (optional)
16. place_specials()        — Labs, military bases, etc.
17. finalize_highways()     — Replace placeholders
18. place_forest_trailheads()
19. polish_river()          — Final shore cleanup
20. generate_sub()          — Sublevels (z < 0) for each overmap tile
21. generate_over()         — Overlevels (z > 0)
22. place_mongroups()       — Monster spawn zones
23. place_radios()          — Radio tower signals
```

> **IMPLEMENT FULLY:** The phase order matters. Rivers must exist before forests (swamps depend on river proximity). Cities must exist before roads (roads connect cities). Specials must come after roads (they need road connections).
