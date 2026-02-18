# Backlog

Incomplete work only. Delete entries when done.
See `architecture.md` for technical spec on all SCALE tasks.
GIS data reference: `~/Development/paris/PROJECT.md`

## Phase A — Chunked Map + GIS Loading

Goal: See Paris on screen. No entities.

Map dimensions: 6,309 x 4,753 tiles at 1m/tile (vertex-crop of all buildings + 30m padding).
That is ~99 x 75 chunks at 64×64 = ~7,400 chunks, ~30M tiles.

- **SCALE-A07a** — Occupant normalization improvements. Needs: A07 (done). Blocks: B06, B03, C04, C05.
  - A07 core pipeline is implemented (de078ba). Current match rates: addresses 65.6% (19,112/29,164), occupants 19.7% (194,892/990,108). Two issues to fix:
  - **Year extraction broken**: all occupants land in Year 0. The `source.publication_date` column format isn't a "YYYY..." string. Inspect actual values in the GeoPackage, fix parsing to extract the 16 snapshot years.
  - **Normalization gaps** (from top-10 unmatched streets):
    - `St-Honoré` (5,608) — GeoPackage uses bare `St-` without "Rue " prefix; address file has full form. Need to normalize both sides identically.
    - `Faub.-St-Denis` (3,485), `Faub.-St-Honoré` (2,080) — compound abbreviation `Faub.-St-` not expanded. Expand "faub.-" before "st-" or handle compound.
    - `boul. Voltaire` (2,290) — "boul." abbreviation not handled. Add "boul." → "boulevard".
    - `Faub.-Poissonnière` (1,825) — "Faub.-" with capital works, but check case handling.
    - `Charenton` (2,152), `Charonne` (1,796), `Sèvres` (1,723), `Provence` (1,651), `Cherche-Midi` (1,945) — bare street names without type prefix. These may match if address file also lacks prefix, or may need the normalizer to strip prefixes from both sides.
  - After fixes, re-run preprocessor and compare match rates. Target: >50% occupant match rate.

- **SCALE-A08** — Seine + bridge placement. Needs: A03.
  - **Preprocessor** (extend `preprocess.rs`): runs after block/building rasterization, before writing binary tiles.
  - The Seine is NOT in any GIS dataset. Hardcoded polygon vertices in `loading_gis.rs`.
  - River band: approximately lat 48.856-48.860, tile rows ~2600-3050 (y-axis, from top). Not uniform width.
  - Island exclusions: Ile de la Cite (quartier "Cite") and Ile Saint-Louis (quartier "Ile Saint-Louis"). Only overwrite tiles that are currently Road (tiles with building_id or block_id are already Courtyard or building from A03).
  - Method: define river as a polygon (~20-30 vertices tracing both banks), rasterize with `scanline_fill`, mark resulting tiles as Water unless they already have building_id or block_id.
  - Bridges (hardcoded, 1830s-era): Pont Royal, Pont du Carrousel, Pont des Arts, Pont Neuf, Pont au Change, Pont Notre-Dame, Pont de l'Arcole, Pont Saint-Louis, Pont Marie, Pont de la Tournelle. Each bridge: two endpoints (tile coordinates), fill the rectangle between them with Bridge tiles (~5-8 wide).
  - Set `quartier_id` on water tiles to 0 (unassigned). Bridge tiles get quartier from the nearest bank.
  - Result baked into `paris.tiles` binary. Game loads Water/Bridge terrain the same as any other tile.

## Phase B — Entities in One Neighborhood

Goal: ~200 entities with full AI on the real map.

- **SCALE-B05** — Door placement + passage carving. Needs: A03. Blocks: B06, B03. **BLOCKED: design review required.**
  - **Preprocessor** (extend `preprocess.rs`): runs after wall/floor classification, same pattern as classify_walls_floors. Static tile modification baked into `paris.tiles`.
  - Place Door tiles: for each building, find a wall tile adjacent to both a floor tile and a Road or Courtyard tile. That tile becomes a Door.
  - Landlocked buildings (no wall tile adjacent to Road or Courtyard): carve a 1-tile passage through intervening buildings to the nearest Road or Courtyard. This models the narrow covered passages (allées) that provided access to interior buildings in dense Parisian blocks.
  - Garden buildings (24 "parc ou jardin"): convert their interior Floor tiles to Garden instead of Floor.
  - Game loads Door/Garden terrain from binary, no runtime classification needed.

- **SCALE-B06** — Building interior generation. Needs: B05, A07. **BLOCKED: design review required.**
  - **Preprocessor** (extend `preprocess.rs`): runs after door placement + address loading. Static tile modifications baked into `paris.tiles`.
  - Furnish building interiors based on occupant type. NAICS category from building registry (populated by A07 in preprocessor). Place furniture tiles:
    - Food stores → counters, barrels, shelves
    - Restaurants → tables, chairs, hearth
    - Clothing → looms, counters, fabric
    - Manufacturing → workbenches, anvils, forges
    - Residential/unknown → beds, table, chairs, hearth, chest
  - Buildings with no known occupant get default residential furnishing.
  - Small buildings (<15 floor tiles) get minimal furnishing (bed, table).
  - Requires new Terrain variants for furniture types (or a separate furniture tile layer in Chunk).

- **SCALE-B01** — Spatial index. `HashMap<(i32,i32), SmallVec<[Entity; 4]>>` on World, rebuilt from positions each tick. Blocks: B02.

- **SCALE-B02** — Convert spatial queries. `run_combat`, `run_eating`, `run_decisions` target selection use spatial index, not full position scan. Needs: B01.

- **SCALE-B03** — GIS-aware entity spawning. Needs: A07, B05. **BLOCKED: design review required.**
  - The building registry (populated by A03 + A07) already knows each building's occupants, addresses, and NAICS categories. This task spawns actual entities from that data.
  - For known occupants (3.7% of population): spawn entity with real name, real occupation, at their building's floor tiles. Position from building's tile list in the registry.
  - For generated occupants (96.3%): see C05 for the procedural generation rules.
  - Single neighborhood first: filter to one QUARTIER (recommend "Arcis" — 825 buildings, dense, central, ~150m×300m).
  - The full data pipeline reference (address → building → people) is documented in SCALE-A07 and `~/Development/paris/PROJECT.md`.

- **SCALE-B04** — Increase A* node limit to 32K. Stopgap for larger-map pathing. Replaced by SCALE-D03.

## Phase C — Simulation LOD (1M Population)

Goal: Full city population. ~4K active, rest statistical.
Census population 1846: 1,034,196. Directory-listed people: 38,188 (3.7%).

- **SCALE-C01** — District definitions from GIS. Blocks: C02, C04.
  - 36 quartiers defined by the `QUARTIER` field on every building and block polygon. No separate quartier boundary geometry needed — derive bounds from the bounding box of all buildings with that QUARTIER value.
  - Quartier sizes range from 265 buildings (Palais de Justice) to 2,391 buildings (Temple).
  - Sub-district: blocks (`NUM_ILOT` field on buildings, `ID_ILOTS` on plot polygons) group ~30-100 buildings each. Use as LOD sub-units if quartier granularity is too coarse.
  - Per-district density derivable from: building count, total building area (SUPERFICIE sum), and occupant count from building registry (baked in by A07).

- **SCALE-C02** — LOD zone framework. Active/Nearby/Statistical derived from camera + district bounds. Needs: C01. Blocks: C03, C05.

- **SCALE-C03** — Zone-aware system filtering. Combat: Active only. Hunger: Active+Nearby. Statistical: no entity iteration. Needs: C02.

- **SCALE-C04** — District aggregate model + `run_district_stats`. Population, avg needs, death rates, resource flows as equations. Needs: C01, A07. **BLOCKED: design review required.**
  - Seed `population_by_type` from NAICS distribution per quartier. 22 industry categories. Aggregate from building registry occupant data (baked in by A07 preprocessor), not from raw GeoPackage.
  - City-wide distribution (1845): Manufacturing 18%, Food stores 13.5%, Clothing 11.7%, Furniture 8.2%, Legal 5.9%, Health 5.5%, Rentiers 4.5%, Arts 3.9%, Construction 3.6%. Use these as priors, adjust per quartier from actual registry data.

- **SCALE-C05** — Statistical population seeding. Every district outside active zone gets aggregate population. Needs: C02, C04, A07. **BLOCKED: design review required.**
  - Procedural population generation rules (for the 96% not in directories):
    - **Concierge**: every building with >4 floor tiles gets one. Ground floor.
    - **Shopkeeper household**: for each directory-listed person, generate spouse + 1-4 children + 0-1 apprentice. Place on ground floor and first upper floor.
    - **Bourgeois tenants**: buildings >100m² get 1-2 wealthy households on lower floors (rentiers, professionals). 3-5 people each.
    - **Working tenants**: remaining floor capacity filled with laborer households. Common unlisted occupations: blanchisseuse (laundress), couturière (seamstress), journalier (day laborer), domestique (servant), porteur d'eau (water carrier), chiffonnier (ragpicker), marchand ambulant (street vendor).
    - **Vertical stratification**: wealthiest on floor 1 (étage noble), progressively poorer upward, servants in garret.
    - **Floor estimate**: building height not in data. Estimate from SUPERFICIE: <50m² = 2 floors, 50-150m² = 3-4 floors, 150-400m² = 4-5 floors, >400m² = 5-6 floors. Multiply footprint area by floor count for total interior space.
    - **Density target**: ~116 people per 1,000m² of footprint (from census population / total building area). Adjust per quartier.
  - 16 available time snapshots from SoDUCo: 1829, 1833, 1839, 1842, 1845, 1850, 1855, 1860, 1864, 1871, 1875, 1880, 1885, 1896, 1901, 1907. A07 bakes all years into the metadata; active year selected at runtime via `world.active_year` (default 1845). Note: building geometry is fixed 1810-1836; post-1855 directory data increasingly references demolished buildings.

## Phase D — Seamless Transitions

Goal: Camera movement smoothly activates/deactivates zones.

- **SCALE-D01** — Hydration. Statistical → active: spawn entities from distribution at building positions. Batch ~100/tick. Needs: C05, B03.
- **SCALE-D02** — Dehydration. Active → statistical: collapse to district averages. Nearby zone buffers for ~200 ticks. Needs: C02.
- **SCALE-D03** — HPA* pathfinding. Chunk-level nav graph, border nodes, precomputed intra-chunk paths. Replaces B04. Needs: A02.
- **SCALE-D04** — Profile and tune. Zone radii, hydration batch size, tick budget per zone, entity count limits.

## Simulation Features (parallel or post-Phase B)

Developable on test map or integrated after Phase B.

- **SIM-001** — Plant growth (Phase 1). Food regeneration. Garden tiles only (24 in dataset). Needs: B05 (garden placement).
- **SIM-002** — Thirst (Phase 2). Requires Water tiles (Seine) and fountains (3 named "Fontaine" buildings + "Pompe de la Samaritaine" in data).
- **SIM-003** — Decay (Phase 1). Corpse decomposition.
- **SIM-004** — Tiredness/sleep (Phase 2). Rest cycles. Entities return to their home building.
- **SIM-005** — Injury (Phase 5). Non-binary damage states.
- **SIM-006** — Weather (Phase 1). Rain/drought/cold.
- **SIM-007** — Emotions/mood (Phase 2). Aggregate need state.
- **SIM-008** — Relationships (Phase 5). Bonds from interaction.
- **SIM-009** — Reputation (Phase 5). Observed behavior.
- **SIM-010** — Building (Phase 4). Requires inventory.
- **SIM-011** — Crafting (Phase 4). Requires recipes.
- **SIM-012** — Fluid flow (Phase 1). Cellular automaton. Needs: A08 (Seine placement).

## Deferred Rasterization Simplifications

Design decisions needed before implementation.

- **S06** — BATI=3 minor features (199 polygons: fountains, wells, kiosks). Currently skipped and logged. No terrain type exists for them. Decide: new terrain variant? Point-of-interest overlay? Ignore permanently?
- **S08** — Garden/courtyard polygons have no registry entry. Given a Garden tile, there's no way to look up which garden it is, its name, or metadata. Data is preserved in `paris.ron`. Decide: new GardenRegistry? Extend BuildingRegistry to BATI=2? Only needed if gameplay references named green spaces.

## Pending (threshold not yet met)

- **GROW-001** — Sub-struct grouping. Trigger: >25 World fields.
- **GROW-002** — Phase function grouping. Trigger: >30 system calls.
- **GROW-003** — System dependency analyzer. Trigger: >15 system files.
