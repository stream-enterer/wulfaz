# Backlog

Incomplete work only. Delete entries when done.
See `architecture.md` for technical spec on all SCALE tasks.
GIS data reference: `~/Development/paris/PROJECT.md`

## Phase A — Chunked Map + GIS Loading

Goal: See Paris on screen. No entities.

Map dimensions: 6,309 x 4,753 tiles at 1m/tile (vertex-crop of all buildings + 30m padding).
That is ~99 x 75 chunks at 64×64 = ~7,400 chunks, ~30M tiles.

- **SCALE-A09** — Water/bridge polish. Needs: A08 (done). **Remaining known limitations:**
  - **Eastern coverage gap**: ~150-tile-wide hole in ALPAGE data (tiles ~4950-5100 X, ~3500-3900 Y). Road patch in the Seine near Pont d'Austerlitz. Components #12 (2777 tiles) and #13 (424 tiles) are data-gap artifacts, not real bridges. Fix: obtain APUR PLAN D'EAU shapefile, reproject from Lambert-93 (EPSG:2154) to WGS84 via ogr2ogr, feed through `rasterize_water_polygons()`.
  - **Western bridge coverage**: ALPAGE water polygons don't extend west of ~lon 2.336 (5 bridges: Invalides, Concorde, Royal, Carrousel, Arts). Same fix — supplemental data needed.
  - **North arm bridge gap**: No detected bridge components in the north arm between Pont Neuf and Ile Saint-Louis (Pont au Change, Notre-Dame, d'Arcole). Either ALPAGE data doesn't fully cover this arm or bridges merged with island road network. Needs investigation.
  - **Canal Saint-Martin**: not in the ALPAGE Vasserot Hydrography layer. Separate historical data source needed.
  - **Diagnostic match rate**: 7/15 in-coverage bridges match (47%). 7 confident matches (dist 2-6 tiles). 8 misses are north-arm bridges or small bridges without separate components.
  - **Bridge names**: Should name bridges as landmarks consistent with current naming system

## Phase B — Entities in One Neighborhood

Goal: ~200 entities with full AI on the real map.

- **SCALE-B06** — Building interior generation. Needs: B05, A07 (done). **BLOCKED: design review required.**
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

- **SCALE-C04** — District aggregate model + `run_district_stats`. Population, avg needs, death rates, resource flows as equations. Needs: C01, A07 (done). **BLOCKED: design review required.**
  - Seed `population_by_type` from NAICS distribution per quartier. 22 industry categories. Aggregate from building registry occupant data (baked in by A07 preprocessor), not from raw GeoPackage.
  - City-wide distribution (1845): Manufacturing 18%, Food stores 13.5%, Clothing 11.7%, Furniture 8.2%, Legal 5.9%, Health 5.5%, Rentiers 4.5%, Arts 3.9%, Construction 3.6%. Use these as priors, adjust per quartier from actual registry data.

- **SCALE-C05** — Statistical population seeding. Every district outside active zone gets aggregate population. Needs: C02, C04, A07 (done). **BLOCKED: design review required.**
  - Procedural population generation rules (for the 96% not in directories):
    - **Concierge**: every building with >4 floor tiles gets one. Ground floor.
    - **Shopkeeper household**: for each directory-listed person, generate spouse + 1-4 children + 0-1 apprentice. Place on ground floor and first upper floor.
    - **Bourgeois tenants**: buildings >100m² get 1-2 wealthy households on lower floors (rentiers, professionals). 3-5 people each.
    - **Working tenants**: remaining floor capacity filled with laborer households. Common unlisted occupations: blanchisseuse (laundress), couturière (seamstress), journalier (day laborer), domestique (servant), porteur d'eau (water carrier), chiffonnier (ragpicker), marchand ambulant (street vendor).
    - **Vertical stratification**: wealthiest on floor 1 (étage noble), progressively poorer upward, servants in garret.
    - **Floor estimate**: building height not in data. Estimate from SUPERFICIE: <50m² = 2 floors, 50-150m² = 3-4 floors, 150-400m² = 4-5 floors, >400m² = 5-6 floors. Multiply footprint area by floor count for total interior space.
    - **Density target**: ~116 people per 1,000m² of footprint (from census population / total building area). Adjust per quartier.
  - 3 active time snapshots from SoDUCo (filtered to best Vasserot overlap): 1845, 1850, 1855. Match rates: 40.1%, 37.1%, 38.0% (52,909 total matched occupants). Active year selected at runtime via `world.active_year` (default 1845). Building geometry is fixed 1810-1836.

## Phase D — Seamless Transitions

Goal: Camera movement smoothly activates/deactivates zones.

- **SCALE-D01** — Hydration. Statistical → active: spawn entities from distribution at building positions. Batch ~100/tick. Needs: C05, B03.
- **SCALE-D02** — Dehydration. Active → statistical: collapse to district averages. Nearby zone buffers for ~200 ticks. Needs: C02.
- **SCALE-D03** — HPA* pathfinding. Chunk-level nav graph, border nodes, precomputed intra-chunk paths. Replaces B04. Needs: A02 (done).
- **SCALE-D04** — Profile and tune. Zone radii, hydration batch size, tick budget per zone, entity count limits.

## Simulation Features (parallel or post-Phase B)

Developable on test map or integrated after Phase B.

- **SIM-001** — Plant growth (Phase 1). Food regeneration. Garden tiles only (24 in dataset). Needs: B05 (garden placement).
- **SIM-002** — Thirst (Phase 2). Requires Water tiles (Seine) and fountains (3 named "Fontaine" buildings + "Pompe de la Samaritaine" in data).
- **SIM-003** — Decay (Phase 1). Corpse decomposition.
- **SIM-004** — Tiredness/sleep (Phase 2). Rest cycles and daily movement. Entities return to their home building to sleep, leave for their workplace building in the morning. Reads `HomeBuilding`, `Workplace`, `Occupation` (schedule varies by occupation type). Needs: B06 (interiors for destination tiles).
- **SIM-005** — Injury (Phase 5). Non-binary damage states.
- **SIM-006** — Weather (Phase 1). Rain/drought/cold.
- **SIM-007** — Emotions/mood (Phase 2). Aggregate need state.
- **SIM-008** — Relationships (Phase 5). Bonds from interaction.
- **SIM-009** — Reputation (Phase 5). Observed behavior.
- **SIM-010** — Building (Phase 4). Requires inventory.
- **SIM-011** — Crafting (Phase 4). Requires recipes.
- **SIM-012** — Fluid flow (Phase 1). Cellular automaton. Needs: A08 (done, Seine placement).

## UI (placeholder — stripped, will rebuild)

- **UI-001** — Entity inspection. Click/hover on entity to see name, occupation, current action, needs. Reads `Name`, `Occupation`, `Intention`, `Hunger`, `Health`. Prerequisite for any player-facing simulation feedback.

## Deferred

### Map & GIS Polish

- **SCALE-B05-POLISH** — Door placement realism audit + selective fixes. Needs: B05 (done). Not blocking B06 or B03 — current doors are functional. Defer until pathfinding or visual artifacts surface.
  - **Workflow:** Phase 1 is a diagnostic audit that quantifies each issue with real numbers from `building_diag` output. Phase 2 is a developer walkthrough — present each issue with its measured severity, suggested fix, and trade-offs, then the developer decides which are worth fixing. Phase 3 implements only the approved fixes. No fix is prescribed in advance; the issues below are candidates with suggested approaches.
  - **Issue A — Passage carving bisects buildings.** BFS carves straight through other buildings' walls, creating phantom corridors. Real access routes in 1840s Paris were *allées couvertes* or narrow courtyards between buildings, not tunnels through rooms. Diagnostic: count how many buildings have passage tiles from a different landlocked building inside their footprint. Suggested fix: constrain carving to follow building-boundary edges (Wall-to-Wall between adjacent building_ids), or insert thin courtyard strips (no building_id) between buildings instead of tunneling through interiors.
  - **Issue B — 318 sealed small buildings (≤4 tiles).** Skipped entirely as "too small for occupants." But 4m² (2×2 at 1m/tile) is a plausible kiosk, guard post, or stairwell entrance — polygon-to-grid quantization shrinks real structures. Diagnostic: cross-reference with `nom_bati` and `superficie` to see how many are named or have non-trivial cadastral area. Suggested fix: give them a Door tile (most exterior-exposed wall) but no Floor, making them passable waypoints. Tag in registry so B06 skips furnishing.
  - **Issue C — Poor Floor placement in 5-20 tile buildings.** The most-interior-neighbor heuristic can place the single Floor tile where no exterior wall can reach it (e.g., deep corner of L-shapes), so door candidate detection still finds nothing. Diagnostic: after place_doors, count 5-20 tile buildings that got a Floor tile but still ended up doorless. Suggested fix: instead of maximizing interior depth, pick a Wall tile that has both same-building neighbors AND an exterior walkable neighbor, then convert its inward neighbor to Floor — guaranteeing adjacency to a viable door candidate.
  - **Issue D — Relaxed reclassification (≥3 neighbors) can breach walls.** For 21+ tile thin/irregular buildings, the ≥3 criterion turns exterior corner tiles into Floor, creating holes in the perimeter. Diagnostic: count Floor tiles in 21+ tile reclassified buildings that have a cardinal neighbor outside the building. Suggested fix: perimeter-closure pass after reclassification — revert any Floor tile with an exterior-facing neighbor back to Wall.
  - **Issue E — Carved passage tiles retain wrong building_id.** Passage Floor/Door tiles keep the building_id of the building they were carved through. Downstream effects: B06 furnishes them as part of the wrong building, B03 may spawn entities in the passage attributed to the wrong building, registry tile counts are wrong. Diagnostic: count passage tiles with mismatched building_id. Suggested fix: clear building_id on carved tiles (set to 0/None) or reassign to the landlocked building they serve.
  - **Issue F — Single-point courtyard connections.** Island courtyards connected by exactly 1 pierced Door tile have zero redundancy. Diagnostic: count courtyard regions with exactly 1 connection point and their sizes. Suggested fix: for regions above a size threshold (e.g., >20 tiles), require ≥2 connection points on different perimeter buildings.
  - **Issue G — Doors opening into dead-end pockets.** A Door is validated to have a Floor/Garden neighbor, but no check that the neighbor connects to the building's main interior. Irregular rasterization can create internal partitions. Diagnostic: BFS inward from each Door through same-building Floor/Garden — count doors where reachable interior is <50% of the building's total Floor tiles. Suggested fix: place an additional door on the disconnected interior component, or merge the partition by converting the thinnest internal wall segment.

- **SCALE-B03-POLISH** — Entity spawning quality improvements. Needs: B03 (done). Not blocking B06, C05, or SIM features — current spawning is functional. Defer until simulation artifacts surface.
  - **Workflow:** Same as B05-POLISH — diagnostic audit first, developer walkthrough, then implement approved fixes only.
  - **Issue A — Spawn clustering.** MVP places entities on random floor tiles. Multiple occupants of the same building can land on the same tile. Visually indistinguishable from a single entity without clicking. Diagnostic: after spawning, count tiles with >1 entity and max stack depth. Suggested fix: distribute entities across unique floor tiles (shuffle floor_tiles, pop one per entity). For buildings with more entities than floor tiles, allow stacking but log a warning.
  - **Issue B — Occupant name data quality.** SoDUCo OCR produces artifacts: inconsistent hyphenation, truncated names, stray punctuation, encoding issues. `loading_gis.rs` already does some cleanup during loading, but spawned entity names may still be messy. Diagnostic: dump all spawned names, count those with non-alphabetic characters (excluding spaces, hyphens, apostrophes). Suggested fix: normalize in `spawn_gis_entities` — collapse whitespace, strip leading/trailing punctuation, title-case.
  - **Issue C — Activity string cleaning.** Same OCR issues affect `Occupant.activity`. Diagnostic: dump unique activity values with frequency counts. Suggested fix: canonical activity mapping table (e.g. "boulangère" → "boulanger", "md." → "marchand de") applied during spawn. Improves both display and NAICS-based behavior grouping.
  - **Issue D — quartier_id on BuildingData.** `BuildingData.quartier` is a String; the tile system uses `quartier_id: u8`. Two representations of the same data. Diagnostic: N/A (structural concern). Suggested fix: add `quartier_id: u8` to `BuildingData` during preprocessing, matching the tile system's 1-based index. Enables O(1) numeric filtering, unifies representations, prepares for C02 zone framework.
  - **Issue E — Duplicate people across addresses.** The same person may appear at multiple addresses in SoDUCo data (e.g. a notaire listed at both office and residence). `occupants_nearest` returns one year's snapshot per building, but the same name+activity pair could spawn as two separate entities. Diagnostic: after spawning, count (name, activity) pairs appearing more than once. Suggested fix: deduplicate by (name, activity) tuple within the target quartier, keeping the first occurrence.
  - **Issue F — Detailed spawn diagnostic dump.** MVP logs aggregate counters and per-building `log::debug!` for no-floor skips. For debugging Issues A-E, additional granular dumps are needed. Diagnostic: N/A (tooling). Suggested fix: add `log::debug!` block after spawning (always compiled, gated by `RUST_LOG=debug` — not `#[cfg(debug_assertions)]`, since this runs once at startup) that logs: per-building spawn counts (building ID, floor tiles, occupants, entities), names containing non-alpha characters (feeds Issue B audit), activity frequency table (feeds Issue C audit), year distribution from `occupants_nearest` — `HashMap<u16, u32>` of which years were actually selected, showing whether the ±20 window pulls in unexpected snapshots. Note: tile stacking diagnostic belongs in Issue A, not here.

## Pending (threshold not yet met)

- **GROW-002** — Phase function grouping. Trigger: >30 system calls.
- **GROW-003** — System dependency analyzer. Trigger: >15 system files.
