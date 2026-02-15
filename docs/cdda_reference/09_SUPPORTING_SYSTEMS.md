# 09 — Supporting Systems

**Scope:** Runtime systems not called by mapgen but required for mapgen output to be functional.
**Purpose:** The consuming LLM uses this to understand which systems must exist for the generated world to be interactive, visually correct, and physically simulated — not just a static grid of terrain IDs.

---

## 1. CONSTRUCTION SYSTEM

### What It Does

Allows players to build, deconstruct, and repair structures by transforming terrain and furniture. A construction definition specifies a pre-requisite terrain/furniture, required skills/materials, time cost, and a post-terrain/furniture result.

### Data Model

JSON type: `"construction"`. Defined in `src/construction.h`.

| Field | Type | Description |
|---|---|---|
| `id` | `construction_str_id` | Unique identifier |
| `category` | `construction_category_id` | Category: CONSTRUCT, DECONSTRUCT, REPAIR |
| `group` | `construction_group_str_id` | Grouped constructions (UI grouping) |
| `pre_terrain` | `set<string>` | Acceptable starting terrain/furniture IDs |
| `post_terrain` | `string` | Resulting terrain/furniture ID |
| `pre_flags` | `map<string, bool>` | Required flags on starting tile (bool = terrain vs furniture) |
| `post_flags` | `set<string>` | Flags applied to result |
| `pre_is_furniture` | `bool` | Pre-requisite applies to furniture layer |
| `post_is_furniture` | `bool` | Result is furniture (not terrain) |
| `required_skills` | `map<skill_id, int>` | Skill level requirements |
| `time` | `int` | Construction time in moves |
| `pre_special` | `function ptr` | Custom pre-construction check |
| `post_special` | `function ptr` | After-construction callback |

### Concrete Failure Mode

Without the construction system, players cannot modify the generated world — no building walls, no barricading doors, no deconstructing furniture for materials. The world is a museum: look but don't touch. Terrain and furniture types have `deconstruct_info` and `bash_info` fields that reference construction outcomes, but those references resolve to nothing.

### Source Files

- C++: `src/construction.h`, `src/construction.cpp`
- JSON: `data/json/construction/*.json` (multiple files by domain)

> **STUB OK — TODO:** The construction system (player-initiated building) requires material tracking, construction stages, progress timers, and a UI for designation. For initial implementation, stub the interface. The terrain/furniture transform data (what CAN be built) should be loaded; the player-facing construction flow is a TODO.

---

## 2. FIELD PROPAGATION

### What It Does

Processes field spreading, decay, and entity effects each game turn. Fire spreads to adjacent FLAMMABLE terrain, smoke rises, gas diffuses, acid corrodes, electricity arcs. Each field type has registered processors that run once per tick per occupied tile.

### Algorithm

Entry point: `map::process_fields()` in `src/map_field.cpp`.

1. Iterate all submaps in the reality bubble
2. For each tile with fields, run the field type's registered processors
3. **Spread**: Uses a Dijkstra-like priority queue (`map::propagate_field()`) — field intensity decreases with distance, spreading to adjacent tiles with decreasing strength
4. **Decay**: Each field has `half_life` — after that duration, intensity drops by one level. At intensity 0, the field is removed
5. **Effects**: Applies `field_effect` entries to entities standing on the field (damage, status effects, radiation)

### Concrete Failure Mode

Without field propagation, fire is a static glyph that never spreads to adjacent FLAMMABLE tiles. Smoke never rises from fire. Toxic gas stays in a single tile forever. The FLAMMABLE terrain flag, `half_life`, `percent_spread`, and `field_effects` data are all dead — loaded but never read. A lit Molotov cocktail creates a single orange `4` on one tile that persists until the save is reloaded.

### Source Files

- C++: `src/map_field.cpp` (primary), `src/field_type.h`, `src/field_type.cpp`
- JSON: `data/json/field_type.json`

> **STUB OK — TODO:** Field propagation (fire spreading to adjacent FLAMMABLE tiles, smoke rising, gas diffusing) is a per-tick simulation algorithm. For initial implementation, placed fields are static — they exist where mapgen put them but don't spread. The field_type data model should be fully implemented so fields can be placed and rendered. Propagation is a separate TODO.

---

## 3. LIGHTING AND TRANSPARENCY

### What It Does

Builds a per-tile transparency cache that drives field-of-vision (FOV) and line-of-sight (LOS) calculations. Each tile gets a float transparency value based on terrain flags, furniture, and fields present.

### Algorithm

Entry point: `map::build_vision_transparency_cache()` in `src/lightmap.cpp`.

1. Build a 2D float transparency array per z-level
2. Each tile's transparency comes from:
   - Terrain `TRANSPARENT` flag → `LIGHT_TRANSPARENCY_OPEN_AIR`
   - No `TRANSPARENT` flag → `LIGHT_TRANSPARENCY_SOLID` (opaque)
   - `TRANSLUCENT` flag (with `TRANSPARENT`) → allows light but blocks vision
   - Fields with `transparent: false` at current intensity → additional opacity
   - Furniture transparency stacks with terrain
3. `light_emitted` values from terrain/furniture/fields contribute to the light map
4. FOV algorithm uses the transparency cache to determine visible tiles

### Key Terrain Flags

| Flag | Effect |
|---|---|
| `TRANSPARENT` | Allows both light and vision through the tile |
| `TRANSLUCENT` | Allows light through but blocks vision (requires TRANSPARENT) |

### Concrete Failure Mode

Without the lighting/transparency system, FOV/LOS ignores terrain entirely. Players can see through walls. Buildings have no darkness — interiors are fully lit. The `TRANSPARENT` flag on every terrain type is dead data. Windows (transparent) and walls (opaque) are visually identical to the FOV system. There is no concept of "indoors" vs "outdoors" for lighting purposes.

### Source Files

- C++: `src/lightmap.cpp`, `src/lightmap.h`
- Terrain data: `TRANSPARENT` flag in `ter_t` / `furn_t`

> **IMPLEMENT FULLY:** FOV/LOS based on terrain transparency is fundamental to gameplay. Without it, there is no stealth, no exploration, no surprise. This must work from day one, even if simplified to binary (transparent vs opaque) rather than the full float-based system.

---

## 4. CONNECT GROUP RENDERING

### What It Does

Selects the correct wall/fence glyph based on neighbor analysis at render time. A wall tile checks its 4 cardinal neighbors for tiles in the same connect group and selects a glyph that visually connects them (corner, T-junction, straight, etc.).

### Algorithm

1. Each terrain type declares `connect_groups` (e.g., `WALL`, `CHAINFENCE`)
2. At render time, for each wall tile, check 4 cardinal neighbors
3. Build a 4-bit bitmask: N=1, E=2, S=4, W=8
4. Look up the rotated symbol for that bitmask from `rotatable_symbols`
5. Display the appropriate box-drawing character (`│`, `─`, `┐`, `└`, `┼`, etc.)

The `connect_to_groups` field specifies which OTHER groups this terrain visually connects to (e.g., a window connects to walls but is not itself a wall).

### Concrete Failure Mode

Without connect group rendering, all walls render as the same glyph regardless of orientation. A house that should display as:

```
┌──┐
│  │
└──┘
```

Instead displays as:

```
####
#  #
####
```

Every wall segment uses the base symbol `#` with no directional awareness. Fences, railings, and any other connected terrain types all display as uniform characters. The `connect_groups` and `connect_to_groups` data on every wall-type terrain is dead.

### Source Files

- C++: Wall/fence rendering logic integrated into display code
- Terrain data: `connect_groups`, `connect_to_groups` fields in `ter_t` / `furn_t`
- Symbol data: `rotatable_symbols.json` provides the rotation lookup

> **IMPLEMENT FULLY:** Connect group rendering is a display-layer feature with no simulation dependency. It can be implemented independently and makes an enormous visual difference.

---

## 5. SEASONAL VARIATION

### What It Does

Changes terrain display symbols and colors based on the current game season. Trees get autumn colors, grass turns brown in winter, crops grow in spring. This is purely cosmetic — no gameplay effect.

### Algorithm

1. Terrain types store optional per-season symbol/color overrides in their JSON definition
2. The game tracks the current season via the `calendar` system
3. At render time, if the current season has an override for this terrain type, use the override symbol/color instead of the base
4. Four seasons: spring, summer, autumn, winter

### Concrete Failure Mode

Without seasonal variation, the world is visually static regardless of in-game time. Trees are always green. Grass never turns brown. Crops never visually grow. The `season` field in terrain data (when present) is dead. The world feels artificial — a perpetual midsummer with no passage of time visible in the environment.

### Source Files

- Terrain data: season-specific overrides in `ter_t` definitions
- Calendar: `src/calendar.h`, `src/calendar.cpp`

> **STUB OK — TODO:** Seasonal variation requires a world clock. For initial implementation, all terrain uses base (summer) symbols. Add season switching after the calendar system exists.

---

## 6. DAMAGE AND DESTRUCTION

### What It Does

Processes terrain and furniture damage from bashing, explosions, and other destructive forces. Uses `bash_info` on terrain/furniture to determine: damage thresholds, what the tile transforms into when destroyed, sound produced, and items dropped.

### Algorithm

Entry point: `map::bash()` in `src/map.cpp`.

1. Calculate total bash strength (player strength + tool bonus)
2. Look up `bash_info` on the target terrain/furniture
3. Compare strength against `str_min` (minimum to damage) and `str_max` (guaranteed destroy)
4. If strength > `str_min`, apply damage with random variation
5. When accumulated damage exceeds threshold:
   - Replace terrain with `ter_set` (e.g., wall → rubble)
   - Replace furniture with `furn_set` (e.g., table → broken table)
   - Spawn items from `drop_group`
   - Play sound at `sound_vol` / `sound_fail_vol`
6. `bash_below` flag chains destruction downward through z-levels (collapsing floors)

### Key bash_info Fields

| Field | Description |
|---|---|
| `str_min` | Minimum strength to attempt bashing |
| `str_max` | Strength for guaranteed destruction |
| `str_min_blocked` | Minimum strength when blocked |
| `ter_set` / `furn_set` | Replacement on destruction |
| `drop_group` | Items spawned on destruction |
| `sound_vol` | Sound produced |
| `bash_below` | Chain destruction downward |
| `explosive` | Resistance to explosions specifically |

### Concrete Failure Mode

Without the damage system, the world is indestructible despite having full destruction data. Walls cannot be broken. Doors cannot be forced. Furniture cannot be smashed for materials. Explosions have no environmental effect. The `bash_info` present on most terrain and furniture types is dead data — loaded, stored, never queried. Players in a building have no way to create alternative exits.

### Source Files

- C++: `src/map.cpp` (bash logic), `src/mapdata.h` (bash_info struct)
- JSON: Inline in terrain/furniture definitions

> **IMPLEMENT FULLY:** Basic bash (strength vs threshold → terrain transform) is simple and critical. Full item drops from `drop_group` can be stubbed (references EXCLUDED item system), but terrain transformation must work.

---

## 7. SCENT PROPAGATION

### What It Does

Simulates scent diffusion across the map using a 2D diffusion model. Scent spreads from entities outward through open terrain, is blocked by `NO_SCENT` tiles, and reduced by `REDUCE_SCENT` tiles. Creatures use scent for tracking.

### Algorithm

Entry point: `scent_map::update()` in `src/scent_map.cpp`.

1. **Decay**: All scent values decrease by 1 per tick
2. **Diffusion**: Two-pass optimization (Y-direction accumulation, then X-direction diffusion)
3. Diffusivity constant: 100 (capped below 125 for numerical stability)
4. Terrain interaction:
   - `NO_SCENT` flag: Scent becomes 0 — complete block
   - `REDUCE_SCENT` flag: Diffusivity reduced to 1/5 normal (20%)
   - Open terrain: Full diffusion

### Concrete Failure Mode

Without scent propagation, creature pathfinding ignores physical barriers for scent tracking. A zombie on one side of a wall can smell a player on the other side as if the wall didn't exist — because the scent map has no concept of walls blocking diffusion. The `NO_SCENT` and `REDUCE_SCENT` terrain flags are dead data. Closing a door provides no scent protection. Creature AI that depends on scent tracking (most hostile creatures) becomes unrealistically accurate — or, if scent is simply absent, unrealistically blind.

### Source Files

- C++: `src/scent_map.cpp`, `src/scent_map.h`
- Terrain flags: `NO_SCENT`, `REDUCE_SCENT` in terrain definitions

> **STUB OK — TODO:** Scent propagation is a simulation subsystem. For initial implementation, creatures use direct LOS/distance for tracking. Add scent when creature AI is sophisticated enough to use it.

---

## 8. SOUND PROPAGATION

### What It Does

Processes sound events each turn — alerts creatures, triggers sound-activated traps, wakes sleeping entities. Sound volume attenuates with distance and vertical separation.

### Algorithm

Entry point: `sounds::process_sounds()` in `src/sounds.cpp`.

1. **Record**: Sound events are queued via `sounds::sound(position, volume, category, description)`
2. **Cluster**: Nearby sounds in the same turn are merged
3. **Propagate**: For each cluster, find entities within `2 × volume` distance
4. **Attenuation**: `effective_distance = rl_dist(source, listener) + vertical_penalty × 5`
   - First underground level: +4 vertical penalty
   - Each additional level: +20 vertical penalty
5. **Note**: Sound propagation is geometric — the WALL terrain flag does NOT directly block sound in CDDA's implementation. Sound travels through walls.

### Concrete Failure Mode

Without sound propagation, there is no alert mechanic. Gunshots don't attract creatures. Breaking windows is silent. Sound-triggered traps (`sound_threshold` field) never activate. Sleeping entities never wake from noise. The `sound_vol` and `sound_fail_vol` fields on `bash_info` produce no effect. The game loses its primary "actions have consequences" mechanic for stealth gameplay.

### Source Files

- C++: `src/sounds.cpp`, `src/sounds.h`

> **STUB OK — TODO:** Sound propagation requires creature AI. For initial implementation, sound events can be logged but not processed. Add sound alerting when creature AI exists.

> **PORTING TRAP:** CDDA's sound system does NOT use the WALL flag for attenuation. Sound travels through walls geometrically. If Wulfaz wants more realistic sound blocking, it must be a new feature, not a port of CDDA's behavior.

---

## 9. GATE SYSTEM

### What It Does

Handles multi-tile doors — garage doors, drawbridges, and mechanical gates that span multiple tiles. A gate definition maps a control mechanism to a series of terrain transformations along a line of wall tiles.

### Data Model

JSON type: `"gate"`. Defined in `src/gates.h`.

| Field | Type | Description |
|---|---|---|
| `id` | `gate_id` | Unique identifier |
| `door` | `ter_str_id` | Terrain type when gate is closed |
| `floor` | `ter_str_id` | Terrain type when gate is open |
| `walls` | `vec<ter_str_id>` | Acceptable adjacent wall types (empty = any WALL) |
| `messages` | `4 × translation` | Pull, open, close, fail messages |
| `moves` | `int` | Action cost in player moves |
| `bash_dmg` | `int` | Damage to blocking creatures/items when forcing closed |

### Algorithm

1. Player interacts with gate control mechanism
2. Search 4 cardinal directions for suitable walls
3. For each wall, trace along the line of gate terrain
4. Toggle all `door` tiles to `floor` (opening) or `floor` to `door` (closing)
5. If closing and creatures/items block, apply `bash_dmg`

### Concrete Failure Mode

Without the gate system, garage doors, drawbridges, and mechanical gates are non-functional terrain. The control mechanisms (levers, buttons) have no effect. Multi-tile openings that should open/close as a unit instead behave as individual, immovable terrain tiles. Buildings with garage entrances have permanent walls where doors should be, or permanent openings that can never be secured.

### Source Files

- C++: `src/gates.h`, `src/gates.cpp`
- JSON: `data/json/gates.json`
- Init: Registered as `"gate"` type in `src/init.cpp`

> **STUB OK — TODO:** Gates are a specialized terrain interaction. Load gate definitions for data completeness. Implement the open/close logic when the interaction system exists.

---

## 10. EXAMINE ACTIONS

### What It Does

Dispatches interactions when a player examines terrain or furniture. Each terrain/furniture type can declare an `examine_action` that triggers specific behavior — crafting at workbenches, sleeping in beds, drawing water from wells, reading signs, using computers, etc.

### Interface

Terrain/furniture types reference examine actions by string name:

```json
{ "examine_action": "workbench" }
{ "examine_action": "water_source" }
{ "examine_action": "sign" }
```

The `iexamine` namespace in C++ maps these strings to function pointers. At runtime, when the player examines a tile, the system:

1. Gets the terrain/furniture at the position
2. Looks up the `examine_action` string
3. Dispatches to the corresponding C++ function
4. The function implements the interaction (open UI, modify inventory, change terrain, etc.)

### Common Examine Actions

| Action | Behavior |
|---|---|
| `none` | No interaction (default) |
| `workbench` | Opens crafting UI with workbench bonus |
| `water_source` | Allows filling containers with water |
| `sign` | Displays sign text |
| `bed` | Allows sleeping |
| `bulletin_board` | Shows/edits messages |
| `cardreader` | Card-based access control |
| `autoclave` | Sterilization interface |
| `elevator` | Z-level transport |
| `harvest_plant` | Collects plant products |
| `dirtmound` | Planting interface |
| `gunsafe` | Locked container interface |
| `piano` | Musical instrument interaction |

### Concrete Failure Mode

Without examine actions, all furniture and interactable terrain is purely decorative. Workbenches can't be used for crafting. Beds can't be slept in. Water sources can't provide water. Signs have no text. Computers are inert. The world has correct visual layout but zero interactivity — a player can see a kitchen sink but cannot use it. Every `examine_action` field on every terrain and furniture type is dead data.

### Source Files

- C++: `src/iexamine.h`, `src/iexamine.cpp` (50+ functions)
- Referenced from: `examine_action` field in terrain/furniture JSON

> **STUB OK — TODO:** CDDA has 50+ distinct examine actions. For initial implementation, all furniture has a single "examine" action that displays the furniture name and description. Individual action behaviors (workbench crafting, bed sleeping, water drawing, etc.) are each a separate TODO.

---

## 11. TER_FURN_TRANSFORM RUNTIME

### What It Does

Executes bulk terrain/furniture transformations at runtime — aging effects, magical terrain changes, seasonal decay, bomb craters. A `ter_furn_transform` definition maps sets of source terrain/furniture to destination terrain/furniture, matched by ID or flag.

### Data Model

JSON type: `"ter_furn_transform"`. Defined in `src/magic_ter_furn_transform.h`.

Documented in `02_TERRAIN_AND_FURNITURE.md` — the data model includes match criteria (by ID or flag), weighted random results, message output, and field placement.

### Runtime Execution

1. Triggered by: mapgen `ter_furn_transforms` key, spell effects, timed events, or direct C++ calls
2. For each tile in the affected area:
   - Check terrain against `ter_transform` entries (match by ID or flag)
   - Check furniture against `furn_transform` entries (match by ID or flag)
   - If matched, select result from weighted random list
   - Apply terrain/furniture change
   - Optionally place fields and display messages

### Concrete Failure Mode

Without the transform runtime, aging/decay/magical terrain effects don't work. A spell that should turn stone walls into rubble has no effect. Mapgen entries using `ter_furn_transforms` (post-generation weathering, damage application) produce no visible result. The transform definitions are loaded but the execution pipeline doesn't exist — the data describes transformations that never happen.

### Source Files

- C++: `src/magic_ter_furn_transform.h`, `src/magic_ter_furn_transform.cpp`
- JSON: `data/json/ter_furn_transforms.json`

> **STUB OK — TODO:** Load transform definitions for data completeness. Implement the execution pipeline when the effects system exists. Note that mapgen-time transforms (via `ter_furn_transforms` placement key) should work from day one since they're part of map generation.

> **PORTING TRAP:** `ter_furn_transform` is used BOTH at mapgen time (placement key) AND at runtime (spells, events). The mapgen-time usage is part of the mapgen pipeline and should work immediately. The runtime usage depends on the effects/spell system and can be deferred.

---

## 12. ROTATABLE SYMBOLS

### What It Does

Maps visual symbols to rotation states so directional terrain renders correctly when mapgen applies rotation. A set of symbols forms a rotation tuple — rotating the terrain cycles through the tuple.

### Data Model

JSON type: `"rotatable_symbol"`. Defined in `src/rotatable_symbols.h`.

```json
{ "type": "rotatable_symbol", "tuple": [ "<", "^", ">", "v" ] }
{ "type": "rotatable_symbol", "tuple": [ "│", "─" ] }
```

- **4-tuple**: Full rotation (NESW) — directional arrows, asymmetric features
- **2-tuple**: Half rotation (vertical/horizontal) — lines, pipes, beams

### Runtime Usage

`rotatable_symbols::get(symbol, n)` returns the symbol rotated `n` times (mod tuple size). When mapgen applies rotation to a building layout (north→east, etc.), all symbols in the layout are rotated through their tuples.

Common tuples:
- `< ^ > v` — directional arrows
- `│ ─` — vertical/horizontal lines
- `└ ┐ ┌ ┘` — box-drawing corners (4-rotation)
- `├ ┬ ┤ ┴` — box-drawing T-junctions (4-rotation)

### Concrete Failure Mode

Without rotatable symbols, rotated buildings display incorrect glyphs. A north-facing arrow `^` that should become `>` when the building rotates east remains `^`. Pipe layouts that should be horizontal become vertical. Box-drawing characters that form corners in one orientation become wrong corners in another. The visual layout of any rotated building is garbled — structurally correct in data but visually incoherent.

### Source Files

- C++: `src/rotatable_symbols.h`, `src/rotatable_symbols.cpp`
- JSON: `data/json/rotatable_symbols.json`
- Init: Registered as `"rotatable_symbol"` type in `src/init.cpp`

> **IMPLEMENT FULLY:** Rotatable symbols are a simple lookup table with enormous visual impact. Any mapgen system that supports rotation MUST have rotatable symbol support or rotated buildings will be visually broken.

---

## 13. VEHICLE SYSTEM (SCOPE BOUNDARY)

### What It Does

Vehicles are complex multi-tile entities composed of parts (frames, wheels, engines, doors, etc.). Mapgen places vehicles via `place_vehicles` referencing vehicle prototype IDs or vehicle group IDs.

### Concrete Failure Mode

Without vehicle support, every `place_vehicles` entry in mapgen silently produces nothing. Parking lots are empty. Roads have no wrecks. Gas stations have no cars. The post-apocalyptic landscape loses a major visual and gameplay element. However, vehicles are a massive subsystem (their own coordinate space, physics, part system, fuel, damage model) that should be a separate implementation effort.

### Source Files

- C++: `src/vehicle.h`, `src/vehicle.cpp`, `src/veh_type.h`
- JSON: `data/json/vehicleparts/`, `data/json/vehicles/`

> **SCOPE BOUNDARY:** Vehicles are referenced by mapgen but are a separate implementation domain. The mapgen placement interface (`place_vehicles`) should be parsed and stored, but actual vehicle instantiation is a separate project.

> **PORTING TRAP:** Do not attempt to implement the vehicle system just because mapgen references it. Document the interface, stub the placement to a no-op, and implement vehicles as a standalone project.

---

## SYSTEM DEPENDENCY SUMMARY

| System | Required For | Priority |
|---|---|---|
| Lighting/Transparency | FOV, indoor darkness, stealth | **Day One** |
| Connect Group Rendering | Correct wall/fence visuals | **Day One** |
| Rotatable Symbols | Correct rotated building visuals | **Day One** |
| Damage/Destruction | World interactivity (bashing) | **Day One** (basic) |
| Examine Actions | Furniture interactivity | Stub initially |
| Field Propagation | Fire spread, gas, smoke | Stub initially |
| Gate System | Multi-tile door operation | Stub initially |
| Construction | Player building | Stub initially |
| Scent Propagation | Creature tracking | Stub initially |
| Sound Propagation | Alert mechanics | Stub initially |
| Seasonal Variation | Visual atmosphere | Stub initially |
| ter_furn_transform | Terrain aging/effects | Mapgen-time: Day One; Runtime: Stub |
| Vehicle System | Mapgen vehicle placement | **Scope boundary** — separate project |
