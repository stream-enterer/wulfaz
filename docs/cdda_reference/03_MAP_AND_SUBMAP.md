# 03 — Map and Submap Data Model

**Scope:** Constants, coordinate systems, submap storage layout, z-levels, and the reality bubble.
**Purpose:** The consuming LLM uses this to implement the spatial data structures that terrain, furniture, and all tile data live in.

---

## CONSTANTS

All constants from `src/map_scale_constants.h` and related headers:

| Constant | Value | Meaning |
|---|---|---|
| `SEEX` | 12 | Submap width in tiles (map squares) |
| `SEEY` | 12 | Submap height in tiles (always equal to SEEX) |
| `MAPSIZE` | 11 | Reality bubble size in submaps per axis (odd, so player is centered) |
| `HALF_MAPSIZE` | 5 | `MAPSIZE / 2` — player's submap is at index (5, 5) in the bubble |
| `MAPSIZE_X` | 132 | Reality bubble width in tiles: `SEEX * MAPSIZE` = 12 * 11 |
| `MAPSIZE_Y` | 132 | Reality bubble height in tiles: `SEEY * MAPSIZE` = 12 * 11 |
| `MAX_VIEW_DISTANCE` | 60 | Maximum view/attack range: `SEEX * HALF_MAPSIZE` = 12 * 5 |
| `OMAPX` | 180 | Overmap width in overmap terrain tiles |
| `OMAPY` | 180 | Overmap height in overmap terrain tiles |
| `SEG_SIZE` | 32 | Save segment size in overmap terrain units |
| `MM_REG_SIZE` | 8 | Tile memory region size in submaps |
| `OVERMAP_DEPTH` | 10 | Z-levels below ground (not including z=0) |
| `OVERMAP_HEIGHT` | 10 | Z-levels above ground (not including z=0) |
| `OVERMAP_LAYERS` | 21 | Total z-levels: `OVERMAP_DEPTH + 1 + OVERMAP_HEIGHT` = range [-10, +10] |
| `NUM_SEASONS` | 4 | Spring, Summer, Autumn, Winter |
| `NUM_TERCONN` | 256 | Maximum number of connect groups |
| `MAX_ITEM_IN_SQUARE` | 4096 | Maximum items per tile |
| `DEFAULT_TILE_VOLUME` | 1000 L | Default max volume per tile |

### Critical Relationships

```
1 overmap terrain (OMT) = 2 x 2 submaps = 24 x 24 tiles
1 submap                 = 12 x 12 tiles (144 tiles)
1 overmap                = 180 x 180 OMTs = 360 x 360 submaps = 4320 x 4320 tiles
1 reality bubble         = 11 x 11 submaps = 132 x 132 tiles
```

---

## Z-LEVELS

Range: **z = -10** (deep underground) to **z = +10** (high altitude). **z = 0** is ground level. Total: **21 layers**.

Z-levels are stored as the `.z` component of tripoint coordinate types. Z is NEVER scaled or divided during coordinate conversions — only x and y are affected by scale operations.

### Z-Level Terrain Interaction

- `GOES_UP` flag: stairs/ladder access to z+1
- `GOES_DOWN` flag: stairs/ladder access to z-1
- `NO_FLOOR` flag: open air (falling hazard, rain passes through)
- `Z_TRANSPARENT` flag: light/sight passes between z-levels
- `TRANSPARENT_FLOOR` flag: can see through floor to z-1
- `SEEN_FROM_ABOVE` flag: tile visible when viewing from z+1
- `ter_t::roof` field: defines what terrain appears as floor on z+1

### Roof Generation

When terrain has a `roof` field set, the map generator creates the corresponding floor terrain on the z-level above. This is how buildings get floors on upper stories — the walls on z=0 each specify their roof terrain, which becomes the floor of z=1.

> **UI/UX DECISION:** Render one z-level at a time. Up/down keys switch levels. This is the sane default — multi-level transparency is a large feature.

> **STUB OK — TODO:** Ramp terrain (`RAMP`, `RAMP_UP`, `RAMP_DOWN`, `RAMP_END`) provides smooth z-level transitions. For initial implementation, use stairs/ladders only. Ramps can be added later — they require diagonal z-level movement which complicates pathfinding.

---

## SUBMAP DATA MODEL

A **submap** represents a 12x12 tile region. It is the atomic unit of map loading, saving, and memory management. Defined in `src/submap.h`.

### Per-Tile Data: maptile_soa

The per-tile data uses a **Structure-of-Arrays** (SoA) layout for cache efficiency:

```cpp
struct maptile_soa {
    mdarray<ter_id,         point_sm_ms>  ter;   // Terrain type (integer ID)
    mdarray<furn_id,        point_sm_ms>  frn;   // Furniture type (integer ID)
    mdarray<uint8_t,        point_sm_ms>  lum;   // Count of light-emitting items
    mdarray<colony<item>,   point_sm_ms>  itm;   // Item stacks per tile
    mdarray<field,          point_sm_ms>  fld;   // Field effects (fire, gas, etc.)
    mdarray<trap_id,        point_sm_ms>  trp;   // Trap per tile (limit: 1)
    mdarray<int,            point_sm_ms>  rad;   // Radiation level
};
```

Each `mdarray<T, point_sm_ms>` is a 12x12 2D array, indexed as `array[x][y]` where both x and y range from 0 to 11.

### Layer Summary

| Layer | Type | Per Tile | Description |
|---|---|---|---|
| **Terrain** | `ter_id` | 1 | The ground/wall/floor type. Always present. See `02_TERRAIN_AND_FURNITURE.md`. |
| **Furniture** | `furn_id` | 0-1 | Optional furniture on top of terrain. `f_null` = no furniture. |
| **Trap** | `trap_id` | 0-1 | Optional trap. Limit one trap per tile. |
| **Field** | `field` | 0-N | Field effects (fire, smoke, gas, blood, etc.). Multiple field types can coexist on one tile. See `field_type` in `01_JSON_TYPE_REGISTRY.md`. |
| **Items** | `colony<item>` | 0-4096 | Item stack on this tile. Max `MAX_ITEM_IN_SQUARE` items. |
| **Radiation** | `int` | 1 | Irradiation level (0 = none). |
| **Light items** | `uint8_t` | 1 | Count of items emitting light (optimization cache). |

> **SCOPE BOUNDARY:** The **items layer** (`itm`) references the item system which is EXCLUDED from the Wulfaz port. The porting LLM must know the slot exists (mapgen places items, serialization reads/writes them) but should not implement CDDA's item types. Stub the slot as an empty container. Wulfaz will use its own item system in this slot.

> **SCOPE BOUNDARY:** The **field layer** (`fld`) is IN SCOPE but the full `field_type` system with intensity levels, aging, and spread mechanics should be implemented incrementally. Start with fire and smoke.

### Uniform Submap Optimization

A submap can be **uniform** — a solid block of a single terrain type with no furniture, traps, fields, or items. When the `maptile_soa` pointer (`m`) is null, only `uniform_ter` (a `ter_id`) is stored.

Calling `ensure_nonuniform()` lazily allocates and fills the SoA arrays:
- All terrain slots set to `uniform_ter`
- All furniture set to `f_null`
- All traps set to `tr_null`
- Fields, items, radiation, light all zeroed

This is a significant memory optimization: open terrain (forests, fields, empty underground) is extremely common and would waste memory storing 144 identical terrain IDs plus 6 empty arrays.

> **IMPLEMENT FULLY:** The uniform submap optimization is critical for memory. Without it, a single overmap (360x360 submaps) would require ~47 million individual tile entries instead of ~130,000 submap headers.

### Additional Submap Data (non-SoA)

| Field | Type | Description |
|---|---|---|
| `spawns` | `Vec<spawn_point>` | Monster spawn definitions (position is `point_sm_ms`) |
| `vehicles` | `Vec<vehicle*>` | Vehicles whose origin is on this submap |
| `partial_constructions` | `Map<tripoint_sm_ms, partial_con>` | In-progress construction sites |
| `camp` | `basecamp*` | At most one basecamp per submap |
| `cosmetics` | `Vec<cosmetic_t>` | Graffiti and signage text overlays |
| `computers` | `Map<point_sm_ms, computer>` | Hackable computer terminals |
| `active_items` | `active_item_cache` | Items needing per-turn processing |
| `ephemeral_data` | `Map<point_sm_ms, tile_data>` | Per-tile damage tracking |
| `original_terrain` | `Map<point_sm_ms, ter_id>` | Pre-transformation terrain (for phase-change reversal) |
| `field_count` | `int` | Total active fields on this submap |
| `last_touched` | `time_point` | Last modification timestamp |
| `temperature_mod` | `int` | Temperature delta in Fahrenheit |
| `player_adjusted_map` | `bool` | Tracks whether player has modified this submap |

> **SCOPE BOUNDARY:** `vehicles` references the vehicle system (EXCLUDED — interface only). Store the slot but do not implement vehicle internals. `spawns` references `monstergroup_id` which is IN SCOPE.

---

## COORDINATE SYSTEMS

CDDA uses a **typed coordinate system** with compile-time safety. Every coordinate value encodes both its **origin** (what it's relative to) and its **scale** (what one unit represents).

### Origins

| Origin | Meaning | Typical Use |
|---|---|---|
| `relative` | A delta/offset — can be added to any other origin | Direction vectors, offsets |
| `abs` | Global absolute origin for the entire game world | World-space positions |
| `submap` | Relative to the corner of a submap | Tile indexing within a submap (range: 0..11) |
| `overmap_terrain` | Relative to the corner of an OMT | Tile indexing within an OMT (range: 0..23) |
| `overmap` | Relative to the corner of an overmap | OMT indexing within an overmap (range: 0..179) |
| `reality_bubble` | Relative to the corner of the loaded map | Tile indexing within the bubble (range: 0..131) |

### Scales

| Scale | Short | Tiles per Unit | Description |
|---|---|---|---|
| `map_square` | `ms` | 1 | The atomic tile — smallest addressable unit |
| `submap` | `sm` | 12 | One submap = 12 tiles |
| `overmap_terrain` | `omt` | 24 | One OMT = 2 submaps = 24 tiles |
| `segment` | `seg` | 768 | One save segment = 32 OMTs = 768 tiles |
| `overmap` | `om` | 4320 | One overmap = 180 OMTs = 4320 tiles |

### Coordinate Type Naming Convention

```
(tri)point_<origin>_<scale>(_ib)

point     = 2D (x, y)
tripoint  = 3D (x, y, z)
_ib       = guaranteed in-bounds (enables faster unsigned division)
```

### Key Coordinate Types

**Within a submap:**
| Type | Origin | Scale | Range | Use |
|---|---|---|---|---|
| `point_sm_ms` | submap | tile | x=[0,11], y=[0,11] | Indexing into `maptile_soa` arrays |
| `tripoint_sm_ms` | submap | tile | same + z | 3D variant |

**Absolute world coordinates:**
| Type | Origin | Scale | Range | Use |
|---|---|---|---|---|
| `point_abs_ms` | absolute | tile | unbounded | Global tile position |
| `point_abs_sm` | absolute | submap | unbounded | Global submap position |
| `point_abs_omt` | absolute | OMT | unbounded | Global OMT position |
| `point_abs_om` | absolute | overmap | unbounded | Global overmap position |

**Reality bubble coordinates:**
| Type | Origin | Scale | Range | Use |
|---|---|---|---|---|
| `point_bub_ms` | bubble | tile | x=[0,131], y=[0,131] | Tile within loaded map |
| `point_bub_sm` | bubble | submap | x=[0,10], y=[0,10] | Submap within loaded map |

**Within an overmap:**
| Type | Origin | Scale | Range | Use |
|---|---|---|---|---|
| `point_om_omt` | overmap | OMT | x=[0,179], y=[0,179] | OMT within an overmap |
| `point_om_sm` | overmap | submap | x=[0,359], y=[0,359] | Submap within an overmap |

**Within an OMT:**
| Type | Origin | Scale | Range | Use |
|---|---|---|---|---|
| `point_omt_ms` | OMT | tile | x=[0,23], y=[0,23] | Tile within an OMT |
| `point_omt_sm` | OMT | submap | x=[0,1], y=[0,1] | Submap within an OMT |

### Conversion Functions

**`project_to<TargetScale>(source)`** — converts between scales at the same origin.

- Scaling down (coarser): divides x,y using floor division (rounds toward negative infinity, handles negatives correctly)
- Scaling up (finer): multiplies x,y (gives the top-left corner of the coarser unit)

```
project_to<sm>(point_abs_ms(25, 30))   => point_abs_sm(2, 2)      // 25/12=2, 30/12=2
project_to<omt>(point_abs_ms(25, 30))  => point_abs_omt(1, 1)     // 25/24=1, 30/24=1
project_to<ms>(point_abs_sm(2, 3))     => point_abs_ms(24, 36)    // 2*12=24, 3*12=36
```

**`project_remain<TargetScale>(source)`** — splits into quotient + remainder. This is the most commonly used conversion because you typically need both "which submap?" and "where within that submap?".

```
(quotient, remainder) = project_remain<sm>(point_abs_ms(25, 30))
    quotient  = point_abs_sm(2, 2)     // submap (2,2)
    remainder = point_sm_ms(1, 6)      // tile (1,6) within that submap
                                       // because: 25 - 2*12 = 1, 30 - 2*12 = 6
```

The remainder's origin is automatically set based on the target scale:
- Remain by `sm` → origin `submap` (remainder is `point_sm_ms`)
- Remain by `omt` → origin `overmap_terrain` (remainder is `point_omt_ms` or `point_omt_sm`)

**`project_combine(coarse, fine)`** — inverse of `project_remain`. Reconstructs the finer coordinate from a coarse position plus offset.

```
project_combine(point_abs_sm(2, 2), point_sm_ms(1, 6)) => point_abs_ms(25, 30)
    // 2*12 + 1 = 25, 2*12 + 6 = 30
```

**`project_bounds<FineScale>(coarse)`** — returns the half-open rectangle of fine-scale coordinates contained within one coarse tile.

```
project_bounds<ms>(point_abs_sm(2, 2)) => rectangle from (24,24) to (36,36) exclusive
```

### Conversion Quick-Reference

```
abs_ms  ÷ 12   = abs_sm     (remainder: sm_ms,  range 0..11)
abs_ms  ÷ 24   = abs_omt    (remainder: omt_ms, range 0..23)
abs_sm  ÷ 2    = abs_omt    (remainder: omt_sm, range 0..1)
abs_sm  ÷ 360  = abs_om     (remainder: om_sm,  range 0..359)
abs_omt ÷ 180  = abs_om     (remainder: om_omt, range 0..179)
```

### Wulfaz Mapping

> **PORTING TRAP:** CDDA's typed coordinate system exists to prevent unit confusion at compile time. In Wulfaz (Rust), use newtype wrappers:
> ```rust
> struct TilePos(i32, i32);      // absolute tile position
> struct SubmapPos(i32, i32);    // absolute submap position
> struct LocalTile(u8, u8);     // tile within a submap (0..11)
> ```
> The conversion functions (`project_to`, `project_remain`, `project_combine`) should be implemented as methods on these types. Floor division for negative coordinates is critical — Rust's default integer division truncates toward zero, which is WRONG for this use case. Use `div_euclid` and `rem_euclid`.

---

## MAP CLASS OVERVIEW

The `map` class (`src/map.h`) manages the **reality bubble** — the actively simulated 11x11 grid of submaps centered on the player.

### Core Responsibilities

1. **Submap grid management**: Loads, stores, and unloads submaps as the player moves. The grid shifts when the player crosses a submap boundary.

2. **Tile access**: All per-tile queries go through `map`:
   ```cpp
   ter_id  ter(tripoint_bub_ms p)      // Get terrain at position
   furn_id furn(tripoint_bub_ms p)     // Get furniture at position
   bool    has_flag(flag, p)           // Check flag on terrain+furniture+vehicle
   int     move_cost(p)               // Combined movement cost
   ```

3. **Tile modification**: All mutations go through `map`:
   ```cpp
   bool ter_set(p, new_terrain)        // Set terrain
   bool furn_set(p, new_furniture)     // Set furniture
   void set(p, terrain, furniture)     // Set both at once
   ```

4. **Spatial queries**: Pathfinding, line-of-sight, area effects:
   ```cpp
   bool passable(p)                    // Can entities move through?
   bool impassable(p)                  // Blocked?
   bool is_bashable(p)                // Can be bashed?
   int  bash_strength(p)              // Strength needed to bash
   ```

5. **Connection queries** (for auto-tiling):
   ```cpp
   uint8_t get_known_connections(p, connect_group_bitset)   // 8-bit adjacency mask
   uint8_t get_known_rotates_to(p, rotate_to_group_bitset)  // Rotation target mask
   ```

### Internal Addressing

The `map` class converts `tripoint_bub_ms` (bubble-space tile coordinates) to submap index + local offset:

```
Given: tripoint_bub_ms(x=37, y=14, z=0)

submap_index_x = 37 / 12 = 3
submap_index_y = 14 / 12 = 1
local_x = 37 % 12 = 1
local_y = 14 % 12 = 2

Access: submaps[3][1][z=0].ter[1][2]
```

The submap grid is stored as a 3D array: `submaps[MAPSIZE][MAPSIZE][OVERMAP_LAYERS]`.

### Flag Checking

`map::has_flag(flag, p)` checks terrain, furniture, AND vehicle parts at that position. There are specialized variants:

| Method | Checks |
|---|---|
| `has_flag(flag, p)` | Terrain + furniture + vehicle parts |
| `has_flag_ter(flag, p)` | Terrain only |
| `has_flag_furn(flag, p)` | Furniture only |
| `has_flag_ter_or_furn(flag, p)` | Terrain + furniture (no vehicles) |

> **PORTING TRAP:** The general `has_flag` includes vehicle checks. Since vehicles are EXCLUDED (interface only), Wulfaz's `has_flag` should check terrain + furniture only. If vehicles are added later, this function must be updated.

---

## THE REALITY BUBBLE

The **reality bubble** is CDDA's core spatial simulation concept. Only the area around the player is fully simulated — everything else is stored on disk as static submap data.

### Dimensions

```
11 x 11 submaps = 132 x 132 tiles per z-level
Player is at the center: submap index (5, 5)
Maximum view distance: 60 tiles (center to edge)
21 z-levels loaded simultaneously: z = -10 to z = +10
```

### Movement and Loading

When the player moves to a new submap:

1. The entire grid shifts — submaps at the trailing edge are serialized and unloaded
2. New submaps at the leading edge are loaded from disk (or generated if visiting for the first time)
3. All `point_bub_ms` coordinates in the loaded map are recalculated
4. Active entities (monsters, NPCs) outside the bubble are suspended

### Simulation Boundary

- **Inside the bubble**: Full simulation — monsters move, fires spread, plants grow, items rot
- **Outside the bubble**: Static — no simulation until the player's bubble encompasses that area again
- This means time effectively "freezes" outside the bubble (with some catch-up mechanics)

### Wulfaz Implications

> **PORTING TRAP:** The reality bubble determines the simulation boundary. For Wulfaz's ECS-style simulation, this maps to: only iterate entities whose position falls within the bubble bounds. Entities outside the bubble should be flagged as suspended and skipped by all systems. The bubble center and bounds should be stored on `World` and updated when the camera/player moves.

> **IMPLEMENT FULLY:** The submap loading/unloading system is essential for large worlds. Without it, the entire world must fit in memory. Start with a simple version: load submaps from generated data (not disk) and don't worry about serialization initially.

---

## HIERARCHY SUMMARY

```
Overmap  (180 x 180 OMTs)
    │
    ├── Overmap Terrain (OMT)  (2 x 2 submaps = 24 x 24 tiles)
    │       │
    │       ├── Submap  (12 x 12 tiles)
    │       │       │
    │       │       └── Tile (map square)
    │       │             ├── Terrain layer    (always: ter_id)
    │       │             ├── Furniture layer   (optional: furn_id)
    │       │             ├── Trap layer        (optional: trap_id)
    │       │             ├── Field layer       (optional: 0-N fields)
    │       │             ├── Item layer        (optional: 0-4096 items) [EXCLUDED]
    │       │             └── Radiation level   (always: int)
    │       │
    │       └── Submap  (3 more submaps to complete the 2x2)
    │
    └── ... (180 * 180 = 32,400 OMTs per overmap)

Z-levels: -10 to +10 (21 layers, each a complete 2D map)

Reality Bubble: 11 x 11 submaps = 132 x 132 tiles (centered on player)
    └── Only this area is actively simulated
```
