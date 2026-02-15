# 05 — Region Settings

**Scope:** The `region_settings` master type and ALL sub-types, `building_bin`, forest biome composition, and the invisible `mapgendata → region_settings` dependency.
**Purpose:** The consuming LLM uses this to implement region-based variation — how different areas of the world have different terrain composition, city styles, and generation parameters.

---

## THE INVISIBLE DEPENDENCY: region_settings → mapgendata

This is the single most important thing to understand about region_settings.

**Mapgen JSON never explicitly references region_settings.** There is no `"region"` key in any mapgen definition. Yet mapgen functions depend heavily on region data for groundcover, building selection, forest composition, and terrain replacement.

### How It Works

```
1. Overmap generation starts for a position (tripoint_abs_omt)

2. The overmap queries the region_settings for that position:
   region_settings& region = overmap_buffer.get_settings(position)

3. When mapgen runs for a tile, it creates a mapgendata context:
   mapgendata dat(position, map, density, when, mission)

4. The constructor stores a const reference to the region:
   dat.region = overmap_buffer.get_settings(over)     // line 76 in mapgendata.cpp

5. Mapgen functions access region through dat:
   dat.region.get_settings_forest()
   dat.region.default_groundcover
   dat.region.get_settings_city()
   dat.region.get_settings_terrain_furniture().resolve(ter_id)
```

**If you implement mapgen without region_settings, every mapgen function that calls `dat.region` will crash or produce empty output.** The region settings are invisible in JSON but mandatory in code.

> **IMPLEMENT FULLY:** The mapgendata → region_settings reference is non-negotiable. Every mapgendata constructor must receive or look up the active region_settings.

---

## region_settings (TOP LEVEL)

JSON type: `"region_settings"`. Defined in `src/regional_settings.h`.

The master container that holds or references all sub-type settings.

### Fields

| Field | C++ Type | JSON Key | Description |
|---|---|---|---|
| `id` | `region_settings_id` | `"id"` | Unique identifier (e.g. `"default"`) |
| `default_oter` | `array<oter_str_id, 21>` | `"default_oter"` | Default overmap terrain per z-level [-10..+10] |
| `default_groundcover` | `weighted_int_list<ter_id>` | `"default_groundcover"` | Default terrain for ground filling |
| `city_spec` | `region_settings_city_id?` | `"cities"` | City generation settings |
| `forest_composition` | `region_settings_forest_mapgen_id` | `"forest_composition"` | Forest biome mapping |
| `forest_trail` | `region_settings_forest_trail_id?` | `"forest_trails"` | Trail generation |
| `weather` | `weather_generator_id` | `"weather"` | Weather system reference |
| `overmap_feature_flag` | `region_settings_feature_flag` | `"feature_flag_settings"` | Feature enable/disable |
| `overmap_forest` | `region_settings_forest_id?` | `"forests"` | Forest noise thresholds |
| `overmap_river` | `region_settings_river_id?` | `"rivers"` | River generation params |
| `overmap_lake` | `region_settings_lake_id?` | `"lakes"` | Lake generation params |
| `overmap_ocean` | `region_settings_ocean_id?` | `"ocean"` | Ocean generation params |
| `overmap_highway` | `region_settings_highway_id?` | `"highways"` | Highway generation |
| `overmap_ravine` | `region_settings_ravine_id?` | `"ravines"` | Ravine generation |
| `overmap_connection` | `region_settings_overmap_connection` | `"connections"` | Network connection IDs |
| `region_terrain_and_furniture` | `region_settings_terrain_furniture_id` | `"terrain_furniture"` | Terrain/furniture replacement rules |
| `region_extras` | `region_settings_map_extras_id` | `"map_extras"` | Map extra distribution |
| `place_swamps` | `bool` | `"place_swamps"` | Enable swamp generation (default: true) |
| `place_roads` | `bool` | `"place_roads"` | Enable road generation (default: true) |
| `place_railroads` | `bool` | `"place_railroads"` | Enable railroad generation (default: false) |
| `place_railroads_before_roads` | `bool` | `"place_railroads_before_roads"` | Order railroads before roads (default: false) |
| `place_specials` | `bool` | `"place_specials"` | Enable overmap specials (default: true) |
| `neighbor_connections` | `bool` | `"neighbor_connections"` | Connect roads at region boundaries (default: true) |
| `max_urban` | `float` | `"max_urbanity"` | Maximum urban density (default: 8) |
| `urban_increase` | `array<float, 4>` | `"urbanity_increase"` | Per-direction urban gradient [N,E,S,W] |

> **IMPLEMENT FULLY:** The top-level region_settings must be loadable and queryable. Every sub-setting is optional (can be null), but `default_groundcover`, `forest_composition`, and `region_terrain_and_furniture` are functionally required.

---

## region_settings_city

City-specific generation parameters. Controls building pools and name generation.

### Fields

| Field | Type | Default | JSON Key | Description |
|---|---|---|---|---|
| `id` | `region_settings_city_id` | — | `"id"` | Identifier |
| `shop_radius` | `int` | 30 | `"shop_radius"` | Gaussian center for shop placement distance |
| `shop_sigma` | `int` | 20 | `"shop_sigma"` | Gaussian spread for shop distance |
| `park_radius` | `int` | 30 | `"park_radius"` | Gaussian center for park placement distance |
| `park_sigma` | `int` | 70 | `"park_sigma"` | Gaussian spread for park distance |
| `name_snippet` | `string` | `"<city_name>"` | `"name_snippet"` | City name template |
| `houses` | `building_bin` | empty | `"houses"` | Weighted residential buildings |
| `shops` | `building_bin` | empty | `"shops"` | Weighted commercial buildings |
| `parks` | `building_bin` | empty | `"parks"` | Weighted park/recreational buildings |

### Building Selection Algorithm

See `04_OVERMAP_GENERATION.md`, City Generation Pipeline, Step 5.

```
town_dist = (distance_to_center * 100) / max(city_size, 1)
shop_normal = max(shop_radius, normal_roll(shop_radius, shop_sigma))
park_normal = max(park_radius, normal_roll(park_radius, park_sigma))

if town_dist < shop_normal → pick from shops
else if town_dist < park_normal → pick from parks
else → pick from houses
```

> **IMPLEMENT FULLY:** Without city_spec, cities have no buildings.

---

## building_bin

The weighted building selection container. Used by `region_settings_city` (houses/shops/parks) and `region_settings_highway` (intersections/bends).

### Fields

| Field | Type | Description |
|---|---|---|
| `buildings` | `weighted_int_list<overmap_special_id>` | Weighted list of building specials |
| `finalized` | `bool` | Must be true before `pick()` is called |

### Methods

- `add(overmap_special_id, weight)` — add a building with selection weight
- `pick()` → `overmap_special_id` — randomly select proportional to weight
- `finalize()` — converts raw terrain type IDs to overmap_special IDs via `create_building_from()`

### JSON Format

```json
{
    "houses": {
        "house": 1000,
        "house_prepper": 20,
        "house_garage": 200
    }
}
```

Each key is an `overmap_special_id` (or `oter_type_id` that gets auto-wrapped), and the value is the integer weight.

> **IMPLEMENT FULLY:** building_bin is the core mechanism for populating cities. Without weighted selection, all buildings would be the same type.

---

## region_settings_forest

Forest noise threshold parameters.

### Fields

| Field | Type | Default | JSON Key | Description |
|---|---|---|---|---|
| `id` | `region_settings_forest_id` | — | `"id"` | Identifier |
| `noise_threshold_forest` | `double` | 0.25 | `"noise_threshold_forest"` | Perlin noise cutoff for sparse forest |
| `noise_threshold_forest_thick` | `double` | 0.3 | `"noise_threshold_forest_thick"` | Cutoff for dense forest |
| `noise_threshold_swamp_adjacent_water` | `double` | 0.3 | `"noise_threshold_swamp_adjacent_water"` | Swamp threshold near water |
| `noise_threshold_swamp_isolated` | `double` | 0.6 | `"noise_threshold_swamp_isolated"` | Swamp threshold far from water |
| `river_floodplain_buffer_distance_min` | `int` | 3 | `"river_floodplain_buffer_distance_min"` | Min river distance for floodplain |
| `river_floodplain_buffer_distance_max` | `int` | 15 | `"river_floodplain_buffer_distance_max"` | Max river distance for floodplain |
| `max_forest` | `float` | 0.395 | `"forest_threshold_limit"` | Maximum forest coverage (0.0-1.0) |
| `forest_increase` | `array<float, 4>` | [0,0,0,0] | `"forest_threshold_increase"` | Per-direction forest gradient [N,E,S,W] |

> **IMPLEMENT FULLY:** These thresholds control how noise maps to terrain. Without them, forest placement is uncalibrated.

---

## region_settings_forest_trail

Trail generation parameters for paths through forests.

### Fields

| Field | Type | Default | JSON Key | Description |
|---|---|---|---|---|
| `id` | `region_settings_forest_trail_id` | — | `"id"` | Identifier |
| `chance` | `int` | 1 | `"chance"` | one_in(X) chance to generate trail |
| `border_point_chance` | `int` | 2 | `"border_point_chance"` | one_in(X) per forest border point |
| `minimum_forest_size` | `int` | 50 | `"minimum_forest_size"` | Min forest area for trails |
| `random_point_min` | `int` | 4 | `"random_point_min"` | Min random trail waypoints |
| `random_point_max` | `int` | 50 | `"random_point_max"` | Max random trail waypoints |
| `random_point_size_scalar` | `int` | 100 | `"random_point_size_scalar"` | Scale factor for point sizing |
| `trailhead_chance` | `int` | 1 | `"trailhead_chance"` | one_in(X) for trailhead building |
| `trailhead_road_distance` | `int` | 6 | `"trailhead_road_distance"` | Distance from road for trailheads |
| `trailheads` | `building_bin` | empty | `"trailheads"` | Buildings at trail entry points |

> **STUB OK — TODO:** Forest trails are a nice-to-have. Implement after basic forest generation works.

---

## region_settings_forest_mapgen

Maps overmap terrain types to forest biome definitions.

### Fields

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `region_settings_forest_mapgen_id` | `"id"` | Identifier |
| `biomes` | `set<forest_biome_mapgen_id>` | `"biomes"` | All available forest biome definitions |
| `oter_to_biomes` | `map<oter_type_id, forest_biome_mapgen_id>` | (built at finalize) | OMT type → biome mapping |

At finalize time, iterates each biome's `terrains` set and builds the reverse map: for each terrain type, record which biome applies.

> **IMPLEMENT FULLY:** This is how forest mapgen knows which trees/plants/groundcover to place for a given forest OMT.

---

## forest_biome_mapgen

Individual forest biome definition — describes what a specific forest type looks like at the tile level.

### Fields

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `forest_biome_mapgen_id` | `"id"` | Identifier |
| `terrains` | `set<oter_type_str_id>` | `"terrains"` | Which OMT types use this biome |
| `biome_components` | `set<forest_biome_component_id>` | `"components"` | Feature generators (ordered by sequence) |
| `groundcover` | `weighted_int_list<ter_id>` | `"groundcover"` | Weighted groundcover terrain |
| `terrain_dependent_furniture` | `map<ter_id, furniture_rule>` | `"terrain_furniture"` | Per-terrain furniture rules |
| `sparseness_adjacency_factor` | `int` | `"sparseness_adjacency_factor"` | Density modifier for neighbor checks |
| `item_group_chance` | `int` | `"item_group_chance"` | one_in(X) to spawn items |
| `item_spawn_iterations` | `int` | `"item_spawn_iterations"` | Item spawn passes |
| `item_group` | `item_group_id` | `"item_group"` | Item group to spawn |

> **SCOPE BOUNDARY:** `item_group` references item_group_id (EXCLUDED). Store the ID but do not resolve.

---

## forest_biome_component

Individual feature type within a forest biome (trees, rocks, bushes, grass patches, etc.).

### Fields

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `forest_biome_component_id` | `"id"` | Identifier |
| `types` | `weighted_int_list<ter_furn_id>` | `"types"` | Weighted terrain/furniture choices |
| `sequence` | `int` | `"sequence"` | Order in generation (lower = earlier) |
| `chance` | `int` | `"chance"` | one_in(X) roll to place this feature per tile |

### How Biome Composition Works

When generating a forest tile:
1. Sort components by `sequence` (ascending)
2. For each component, roll `one_in(chance)`:
   - If success: pick from `types` weighted list → place terrain/furniture
   - If fail: try next component
3. If no component succeeds, place `groundcover`
4. After terrain placed, check `terrain_dependent_furniture`: if current terrain matches, roll for furniture

This creates layered placement: rare features (giant trees) have high `chance` values and low `sequence`, common features (grass, dirt) have low `chance` and high `sequence`.

> **IMPLEMENT FULLY:** Forest biome composition is essential for varied, natural-looking forests.

---

## forest_biome_terrain_dependent_furniture

Furniture spawned on specific terrain types within forests.

### Fields

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `furniture` | `weighted_int_list<furn_id>` | `"furniture"` | Weighted furniture choices |
| `chance` | `int` | `"chance"` | one_in(X) roll to place |

Example: on `t_tree` terrain, 1-in-10 chance to place `f_mushroom`.

---

## groundcover_extra

Terrain/furniture distribution for groundcover with optional clustering.

### Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `default_ter` | `ter_id` | `t_null` | Fallback terrain (resolved at finalize) |
| `percent_str` | `map<string, double>` | empty | Terrain/furniture string → percentage |
| `boosted_percent_str` | `map<string, double>` | empty | Clustered variant percentages |
| `mpercent_coverage` | `int` | 0 | Coverage (parts per million: 100% = 1,000,000) |
| `boost_chance` | `int` | 0 | Chance per tile to use boosted list |
| `boosted_mpercent_coverage` | `int` | 0 | Coverage for boosted features |
| `boosted_other_mpercent` | `int` | 1 | Fallback weight for boosted list |

### Clustering Algorithm

```
for each tile:
    if one_in(boost_chance):
        pick from boosted_weightlist at boosted_mpercent_coverage density
    else:
        pick from weightlist at mpercent_coverage density
```

This creates natural-looking clusters: most tiles get normal groundcover, but some tiles get boosted into dense patches of a single feature.

> **STUB OK — TODO:** Clustering is a nice visual touch. Implement basic groundcover first, add clustering later.

---

## region_settings_river

River generation parameters.

### Fields

| Field | Type | Default | JSON Key | Description |
|---|---|---|---|---|
| `id` | `region_settings_river_id` | — | `"id"` | Identifier |
| `river_scale` | `int` | 1 | `"river_scale"` | Width/amplitude of rivers |
| `river_frequency` | `double` | 1.5 | `"river_frequency"` | Frequency multiplier (lower = more branches) |
| `river_branch_chance` | `double` | 64 | `"river_branch_chance"` | Percentage chance to branch |
| `river_branch_remerge_chance` | `double` | 4 | `"river_branch_remerge_chance"` | Percentage chance branch re-merges |
| `river_branch_scale_decrease` | `double` | 1 | `"river_branch_scale_decrease"` | Scale reduction per branch level |

> **IMPLEMENT FULLY:** River parameters control river appearance. Without them, all rivers are identical.

---

## region_settings_lake

Lake generation parameters.

### Fields

| Field | Type | Default | JSON Key | Description |
|---|---|---|---|---|
| `id` | `region_settings_lake_id` | — | `"id"` | Identifier |
| `noise_threshold_lake` | `double` | 0.25 | `"noise_threshold_lake"` | Noise cutoff for lake placement |
| `lake_size_min` | `int` | 20 | `"lake_size_min"` | Minimum lake size in tiles |
| `lake_depth` | `int` | -5 | `"lake_depth"` | Z-level depth |
| `surface` | `oter_str_id` | `lake_surface` | `"surface_ter"` | Surface water terrain |
| `shore` | `oter_str_id` | `lake_shore` | `"shore_ter"` | Shore terrain |
| `interior` | `oter_str_id` | `lake_water_cube` | `"interior_ter"` | Deep water terrain |
| `bed` | `oter_str_id` | `lake_bed` | `"bed_ter"` | Lake bed terrain |
| `shore_extendable_overmap_terrain` | `vec<oter_str_id>` | empty | `"shore_extendable_overmap_terrain"` | Terrains that extend into shore |
| `shore_extendable_overmap_terrain_aliases` | `vec<alias>` | empty | `"shore_extendable_overmap_terrain_aliases"` | Shore alias rules |
| `invert_lakes` | `bool` | false | `"invert_lakes"` | Invert noise (negative = lake) |

> **IMPLEMENT FULLY:** Lake parameters define lake appearance and depth. Required for any water body generation.

---

## region_settings_ocean

Ocean generation parameters.

### Fields

| Field | Type | Default | JSON Key | Description |
|---|---|---|---|---|
| `id` | `region_settings_ocean_id` | — | `"id"` | Identifier |
| `noise_threshold_ocean` | `double` | 0.25 | `"noise_threshold_ocean"` | Noise cutoff |
| `ocean_size_min` | `int` | 100 | `"ocean_size_min"` | Minimum ocean size |
| `ocean_depth` | `int` | -9 | `"ocean_depth"` | Z-level depth |
| `ocean_start_north` | `int?` | null | `"ocean_start_north"` | Force ocean north of this y |
| `ocean_start_east` | `int?` | null | `"ocean_start_east"` | Force ocean east of this x |
| `ocean_start_west` | `int?` | null | `"ocean_start_west"` | Force ocean west of this x |
| `ocean_start_south` | `int?` | null | `"ocean_start_south"` | Force ocean south of this y |
| `sandy_beach_width` | `int` | 2 | `"sandy_beach_width"` | Beach border width |

> **STUB OK — TODO:** Oceans are optional for initial implementation. Store the settings but defer ocean generation.

---

## region_settings_ravine

Ravine generation parameters.

### Fields

| Field | Type | Default | JSON Key | Description |
|---|---|---|---|---|
| `id` | `region_settings_ravine_id` | — | `"id"` | Identifier |
| `num_ravines` | `int` | 0 | `"num_ravines"` | Number of ravines per region |
| `ravine_range` | `int` | 45 | `"ravine_range"` | Spread distance |
| `ravine_width` | `int` | 1 | `"ravine_width"` | Width in OMTs |
| `ravine_depth` | `int` | -3 | `"ravine_depth"` | Z-level depth |

> **STUB OK — TODO:** Ravines are optional. Store settings but defer generation.

---

## region_settings_highway

Highway generation and segment composition.

### Fields

| Field | Type | Default | JSON Key | Description |
|---|---|---|---|---|
| `id` | `region_settings_highway_id` | — | `"id"` | Identifier |
| `width_of_segments` | `int` | 2 | `"width_of_segments"` | Highway width in OMTs |
| `straightness_chance` | `double` | 0.6 | `"straightness_chance"` | Probability to continue straight |
| `reserved_terrain_id` | `oter_type_str_id` | — | `"reserved_terrain_id"` | Placeholder terrain |
| `reserved_terrain_water_id` | `oter_type_str_id` | — | `"reserved_terrain_water_id"` | Water crossing placeholder |
| `segment_flat` | `overmap_special_id` | — | `"segment_flat_special"` | Flat highway segment |
| `segment_ramp` | `overmap_special_id` | — | `"segment_ramp_special"` | Ramp segment |
| `segment_road_bridge` | `overmap_special_id` | — | `"segment_road_bridge_special"` | Road bridge |
| `segment_bridge` | `overmap_special_id` | — | `"segment_bridge_special"` | Water bridge |
| `segment_bridge_supports` | `overmap_special_id` | — | `"segment_bridge_supports_special"` | Bridge supports |
| `segment_overpass` | `overmap_special_id` | — | `"segment_overpass_special"` | Overpass |
| `clockwise_slant` | `overmap_special_id` | — | `"clockwise_slant_special"` | CW curve |
| `counterclockwise_slant` | `overmap_special_id` | — | `"counterclockwise_slant_special"` | CCW curve |
| `fallback_onramp` | `overmap_special_id` | — | `"fallback_onramp"` | Fallback on-ramp |
| `fallback_bend` | `overmap_special_id` | — | `"fallback_bend"` | Fallback bend |
| `fallback_three_way_intersection` | `overmap_special_id` | — | `"fallback_three_way_intersection"` | Fallback T-junction |
| `fallback_four_way_intersection` | `overmap_special_id` | — | `"fallback_four_way_intersection"` | Fallback cross |
| `four_way_intersections` | `building_bin` | empty | `"four_way_intersections"` | 4-way intersection pool |
| `three_way_intersections` | `building_bin` | empty | `"three_way_intersections"` | 3-way intersection pool |
| `bends` | `building_bin` | empty | `"bends"` | Bend pool |
| `interchanges` | `building_bin` | empty | `"interchanges"` | Interchange pool |
| `road_connections` | `building_bin` | empty | `"road_connections"` | Road connection pool |

> **STUB OK — TODO:** Highway composition with specials is complex. For initial implementation, use simple straight roads between cities.

---

## region_settings_overmap_connection

Network connection type IDs for road/trail/rail/sewer routing.

### Fields

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `trail_connection` | `overmap_connection_id` | `"trail_connection"` | Trail network type |
| `sewer_connection` | `overmap_connection_id` | `"sewer_connection"` | Sewer tunnel type |
| `subway_connection` | `overmap_connection_id` | `"subway_connection"` | Subway tunnel type |
| `rail_connection` | `overmap_connection_id` | `"rail_connection"` | Railroad type |
| `intra_city_road_connection` | `overmap_connection_id` | `"intra_city_road_connection"` | Roads within cities |
| `inter_city_road_connection` | `overmap_connection_id` | `"inter_city_road_connection"` | Roads between cities |

> **IMPLEMENT FULLY:** At minimum, `intra_city_road_connection` and `inter_city_road_connection` are required for functional road networks.

---

## region_settings_terrain_furniture

Collection of terrain/furniture replacement rules for post-mapgen regional resolution.

### Fields

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `region_settings_terrain_furniture_id` | `"id"` | Identifier |
| `ter_furn` | `set<region_terrain_furniture_id>` | `"ter_furn"` | All replacement rule references |

### region_terrain_furniture (individual rule)

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `region_terrain_furniture_id` | `"id"` | Identifier |
| `replaced_ter_id` | `ter_id` | `"ter_id"` | Abstract terrain to match |
| `replaced_furn_id` | `furn_id` | `"furn_id"` | Abstract furniture to match |
| `terrain` | `weighted_int_list<ter_id>` | `"replace_with_terrain"` | Weighted concrete terrain choices |
| `furniture` | `weighted_int_list<furn_id>` | `"replace_with_furniture"` | Weighted concrete furniture choices |

### Resolution Algorithm

```
resolve(ter_id input) → ter_id output:
    for each rule in ter_furn:
        if rule.replaced_ter_id == input:
            output = rule.terrain.pick()    // weighted random
            return resolve(output)          // recurse (handle chaining)
    return input                            // no match, return unchanged
```

This is invoked by `resolve_regional_terrain_and_furniture()` after mapgen — see `02_TERRAIN_AND_FURNITURE.md`, REGION_PSEUDO section.

> **IMPLEMENT FULLY:** This is how biome variation works. Without it, abstract terrain like `t_region_groundcover` is never resolved to actual terrain.

---

## region_settings_map_extras

Map extra distribution across the region.

### Fields

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `region_settings_map_extras_id` | `"id"` | Identifier |
| `extras` | `set<map_extra_collection_id>` | `"extras"` | Collection of map extra pools |

### map_extra_collection

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `id` | `map_extra_collection_id` | `"id"` | Identifier |
| `chance` | `unsigned int` | `"chance"` | one_in(X) chance to spawn extras per OMT |
| `values` | `weighted_int_list<map_extra_id>` | `"extras"` | Weighted extra choices |

The `filtered_by(mapgendata)` method filters extras by validity for the current context (terrain type, location, etc.).

> **STUB OK — TODO:** Map extras add scattered features (crashed vehicles, corpses, loot). Implement after basic mapgen works.

---

## region_settings_feature_flag

Whitelist/blacklist for enabling/disabling overmap generation features.

### Fields

| Field | Type | JSON Key | Description |
|---|---|---|---|
| `blacklist` | `set<string>` | `"blacklist"` | Feature IDs to disable |
| `whitelist` | `set<string>` | `"whitelist"` | Feature IDs to force-enable |

Used to disable rivers, lakes, etc. in specific regions.

> **IMPLEMENT FULLY:** Simple but essential for region customization.

---

## HOW region_settings CONTROLS GENERATION

Summary of what each sub-type affects:

| Sub-type | Controls | Used By |
|---|---|---|
| `region_settings_city` | Building pools (houses/shops/parks) | City generation |
| `region_settings_forest` | Forest noise thresholds | `place_forests()` |
| `region_settings_forest_mapgen` | Forest biome mapping | Forest tile mapgen |
| `forest_biome_mapgen` | Tree/plant/groundcover composition | Forest tile mapgen |
| `forest_biome_component` | Individual feature types | Forest tile mapgen |
| `region_settings_forest_trail` | Trail parameters | `place_forest_trails()` |
| `region_settings_river` | River branching | `place_rivers()` |
| `region_settings_lake` | Lake noise/depth | `place_lakes()` |
| `region_settings_ocean` | Ocean boundaries | `place_oceans()` |
| `region_settings_ravine` | Ravine count/depth | `place_ravines()` |
| `region_settings_highway` | Highway composition | `place_highways()` |
| `region_settings_overmap_connection` | Network type IDs | Road/trail routing |
| `region_settings_terrain_furniture` | Abstract → concrete terrain | Post-mapgen resolution |
| `region_settings_map_extras` | Extra distribution | Post-mapgen extras |
| `region_settings_feature_flag` | Feature enable/disable | All generation phases |
| `default_groundcover` | Fallback terrain | Mapgen `fill_groundcover` |
| `default_oter` | Fallback overmap terrain | Unassigned OMT tiles |

---

## SUB-TYPE COUNT

Total distinct sub-types documented: **18**

1. `region_settings` (top level)
2. `region_settings_city`
3. `region_settings_forest`
4. `region_settings_forest_trail`
5. `region_settings_forest_mapgen`
6. `region_settings_river`
7. `region_settings_lake`
8. `region_settings_ocean`
9. `region_settings_ravine`
10. `region_settings_highway`
11. `region_settings_overmap_connection`
12. `region_settings_terrain_furniture`
13. `region_settings_map_extras`
14. `region_settings_feature_flag`
15. `forest_biome_mapgen`
16. `forest_biome_component`
17. `forest_biome_terrain_dependent_furniture`
18. `building_bin`

Plus supporting types: `groundcover_extra`, `region_terrain_furniture`, `map_extra_collection`, `shore_extendable_overmap_terrain_alias`.
