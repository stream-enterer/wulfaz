# B05 — Door Placement & Passage Carving

Design produced via concentric planning (enumerate → shape → specify → refine).
Tiered from MVP to full polish.

## Diagnostic Baseline

From `building_diag` run on the current Paris dataset:

| Metric | Value |
|--------|-------|
| BATI=1 buildings | 21,035 |
| With interior (Floor > 0) | 20,368 (96.8%) |
| All-wall (no Floor) | 667 (3.2%) |
| Road-adjacent walls | 15,418 (75.7%) |
| Courtyard-only access | 4,938 (24.2%) |
| Landlocked (neither) | 12 (0.06%) |
| Dual-door candidates (Road + Courtyard) | 14,504 (71.2%) |
| Island courtyard regions | 10,128 (875K tiles) |
| Arcis BATI=1 with Floor | 437 |
| Arcis landlocked | 1 |
| Arcis with occupants | 273 (1,814 records) |

Key insight: landlocked buildings are negligible (12). The dominant topology
problem is **island courtyards** (10K regions) and the 4,938 buildings whose
only external access faces those courtyards.

However, 14,504 buildings have **both** Road and Courtyard access. When these
get doors on both sides, their interior provides a walkable path from Road
through the building into the courtyard. This means most island courtyards
become reachable automatically once dual-door buildings are handled.

---

## Complete Feature Inventory

### Core Placement

**F01 — External door candidate detection.**
For each BATI=1 building, identify wall tiles that have a cardinal neighbor
which is (a) walkable and (b) outside this building (different building_id
or no building_id). These are door candidates.
- Inputs: TileMap (terrain, building_id), BuildingRegistry (BATI=1, tile lists).
- Outputs: `Vec<(i32, i32)>` of candidate tiles per building.
- Edge: Wall tile adjacent to Water — Water is not walkable, no candidate. Correct by default.
- Edge: Wall tile at map boundary — `get_terrain` returns None. Skip. Correct.
- Edge: Building tile overwritten by BATI=2/3 (now Courtyard/Garden/Fixture terrain) — no
  longer Wall, so not considered. Building_id is a remnant. Correct.
- Edge: Building with zero tiles — skip.

**F02 — Facade run detection.**
Group a building's door candidates into contiguous runs via 4-connected
adjacency. Two candidate tiles are in the same run if they share a cardinal
edge. This naturally splits at corners and gaps.
- Inputs: Candidate tiles from F01.
- Outputs: `Vec<Vec<(i32, i32)>>` — list of runs per building.
- Edge: Single isolated candidate (no adjacent candidates) — run of length 1.
- Edge: L-shaped facade wrapping a corner — stays as one run if tiles are
  cardinally connected. This is correct: an L still has physical continuity.
- Edge: Building with candidates on all 4 sides — splits into 4+ runs
  (breaks wherever tiles are not cardinally adjacent).

**F03 — Door selection heuristic.**
Within each facade run, select which tiles become Doors. Controls density:
not every candidate needs to become a door.
- Inputs: Facade runs from F02.
- Outputs: Final set of `(i32, i32)` tiles to convert to Door.
- Selection rules:
  - Run length 1–2: 1 door (first tile).
  - Run length 3–10: 1 door at midpoint.
  - Run length 11–20: 2 doors at ⅓ and ⅔ positions.
  - Run length 21+: 1 door per 10 tiles, evenly spaced.
- All selected tiles must pass the **door-floor adjacency check**: at least
  one cardinal neighbor is Floor or Garden (building interior). If midpoint
  fails, slide to nearest candidate that passes.
- Edge: All candidates in a run fail the adjacency check (entire facade is a
  1-tile-wide wing with no interior behind it) — no door for this run. Building
  may still get doors from other runs.

**F04 — Dual-door guarantee.**
After F03, verify that buildings with both Road-adjacent and Courtyard-adjacent
facade runs have at least one door on each side. If F03 selected only road-facing
doors, force the best courtyard-facing candidate (midpoint of longest courtyard
run that passes the adjacency check), and vice versa.
- Inputs: Selected doors from F03, facade runs from F02, terrain type of each
  run's external neighbors.
- Outputs: Additional Door tiles where the guarantee was missing.
- Edge: Building has courtyard-adjacent candidates but all fail the adjacency
  check — cannot place a courtyard door. This building's courtyard side remains
  sealed. Acceptable; the courtyard must then be reached through another building.

### Connectivity

**F05 — Island courtyard piercing.**
For each island courtyard region (4-connected Courtyard component with no
Road neighbor), find a perimeter building that already has road access and
add a courtyard-facing door to it.
- Inputs: Courtyard regions (computed via 4-connected BFS), TileMap,
  building doors already placed by F03/F04.
- Outputs: Additional Door tiles on TileMap.
- Algorithm:
  1. Build courtyard regions (BFS over Courtyard terrain).
  2. Identify island regions (no tile has a Road cardinal neighbor).
  3. For each island region, scan boundary tiles and collect building_ids of
     adjacent Wall tiles. These are the perimeter buildings.
  4. Among perimeter buildings, find one that already has a road-reachable Door
     (query TileMap: does this building have a Door tile with a Road cardinal
     neighbor, or a Door reachable from Road via walkable BFS?). Simplified
     check: does the building have any Door tile adjacent to Road?
  5. On that building, find a Wall tile that is adjacent to both the courtyard
     region and an interior Floor tile. Convert to Door. (If the tile is already
     Door from an earlier placement pass, the courtyard is already connected —
     skip.)
  6. If no perimeter building has road access: chain — find a perimeter building
     adjacent to a *connected* courtyard region, and pierce through that instead.
     This handles nested courtyards.
- Edge: Very large island courtyard with many perimeter buildings — pick the
  building with the shortest interior path between its road-door and the
  courtyard-facing wall (approximation: fewest tiles, or just pick first found).
- Edge: Island courtyard enclosed entirely by all-wall buildings (no Floor) —
  cannot pierce. Log warning, leave as island.
- Edge: Nested courtyards (courtyard inside a courtyard) — the chaining step
  handles this by connecting to an already-connected courtyard region.
- Error: If no path can be found, log warning. Don't panic.

**F06 — Landlocked building passage carving.**
For the 12 landlocked buildings (no wall tile adjacent to any walkable external
terrain), carve a passage through intervening buildings.
- Inputs: Landlocked buildings (F01 produces zero candidates), TileMap.
- Outputs: Carved passage tiles on TileMap.
- Algorithm: Use F07 (BFS routing) to find shortest path from building's wall
  tiles through Wall/Floor tiles of other buildings to nearest Road or Courtyard.
  Apply F08 terrain semantics.
- Diagnostic baseline: 2 at depth 1, 4 at depth 2, 2 at depth 3, 3 at depth 4–5,
  1 unreachable (>50 tiles). The unreachable building is likely a data artifact.
- Edge: Carve path traverses a building with occupants — historically correct
  (allée through the ground floor). Carved tiles retain the traversed building's
  building_id.
- Edge: Carve path goes through only Wall tiles (thin building, no Floor to
  traverse) — that's fine; Wall → Floor for interior, Wall → Door for endpoints.

**F07 — Carve path routing.**
BFS from a set of seed tiles outward through Wall and Floor tiles (treating
them as traversable) until a Road or Courtyard tile is reached. Returns the
ordered path.
- Inputs: Seed tiles (wall tiles of the landlocked building, or courtyard
  boundary tiles), TileMap.
- Outputs: `Vec<(i32, i32)>` — ordered path from seed to destination (exclusive
  of the destination Road/Courtyard tile itself).
- Cardinal movement only (4-dir). Uniform cost (BFS, not A*).
- Max search depth: 50 tiles. Beyond that, abandon and report.
- Shared by F05 (fallback for unreachable courtyards) and F06 (landlocked
  buildings).

**F08 — Passage terrain semantics.**
Define what terrain carved tiles become.
- Path endpoints (first and last tile, which are wall tiles of the source and
  destination buildings): **Door**.
- Interior path tiles (Wall/Floor tiles of traversed buildings): **Floor**.
  This preserves 18°C thermal behavior and avoids a cold corridor of 17°C Door
  tiles through the building. The Door-Floor-Door sandwich is historically
  correct: you enter the allée through a door, walk through the ground floor,
  exit through another door.
- Building_id is NOT changed — carved tiles retain ownership of whatever
  building they were part of.

### Edge Cases

**F09 — Small building policy.**
667 all-wall buildings (no Floor tiles). They cannot receive doors under the
standard rule (Door requires a Floor neighbor).
- Classification:
  - ≤4 tiles (318 buildings): **Skip.** These are genuinely tiny structures —
    boundary walls, sheds, ruins. Too small for occupants. No door, no spawning.
  - 5–20 tiles (334 buildings): **Convert one tile to Floor.** Pick the tile
    with the most same-building cardinal neighbors (most interior). This gives
    the building at least one Floor tile, enabling normal door placement.
  - 21+ tiles (15 buildings): **Convert all non-edge tiles to Floor.** These
    are significant structures that should have been classified with interior
    tiles. Likely caused by thin or irregular polygon rasterization. Apply the
    same wall/floor classification logic but using 8-connected neighbors instead
    of 4-connected (more permissive — a tile is Floor if all 8 neighbors belong
    to the same building).
- Inputs: BuildingRegistry, TileMap.
- Outputs: Modified terrain (some Wall → Floor), then normal door placement.
- Must run BEFORE F01 (door candidate detection).

**F10 — Garden conversion.**
24 buildings with "parc ou jardin" in `nom_bati`: convert interior Floor tiles
to Garden.
- Door tiles are NOT converted — they remain Door.
- Must run AFTER door placement (F03), so doors are already placed and
  won't be affected by the Floor → Garden change.
- Edge: Small garden building where all Floor tiles become Garden and Door
  now neighbors only Garden — this is fine. Door-Garden adjacency is valid
  (Garden is walkable interior space, just outdoors).
- Edge: Garden building that is also landlocked — door placed by F06 first,
  then garden conversion applies. No conflict.

### Validation

**F11 — Door-floor adjacency check.**
Every Door tile must have ≥1 cardinal neighbor that is Floor or Garden.
Violations mean a door was placed on a wall tile with no interior behind it.
- Run after all placement is complete (including garden conversion).
- Output: count of violations. Log each violating tile coordinate.
- Not a hard failure — violating doors are still walkable and functional for
  pathfinding. But they indicate a logic error in placement.

**F12 — Per-building door check.**
Every BATI=1 building with ≥1 Floor or Garden tile must have ≥1 Door tile.
- Run after all placement is complete.
- Output: list of doorless buildings (BuildingId, quartier, superficie, tile count).
- Expected: 0 after all tiers are implemented. In Tier 1, some all-wall
  buildings will appear here (acceptable).
- Buildings with zero Floor AND zero Garden are excluded (they were skipped
  by F09 policy for ≤4 tiles).

**F13 — Global connectivity validation.**
BFS from a single Road tile, following all walkable terrain. After BFS, check
that every Door tile and every Courtyard tile was visited.
- Output: visited counts by terrain type, unreachable Door count, unreachable
  Courtyard tile count.
- Performance: ~30M tiles, BFS is O(n). Under 2 seconds. Fine for preprocessing.
- Not a hard failure — some deep island courtyards may remain unreachable.
  Report the count and locations.

**F14 — Courtyard reachability check.**
After F13's BFS, count how many courtyard regions have ≥1 visited tile vs.
how many are entirely unvisited (still island).
- Output: `X/Y courtyard regions reachable (Z%)`.
- This is the primary success metric for F05 (island courtyard piercing).

### Integration

**F15 — Pipeline integration.**
New public function in `loading_gis.rs`:
```rust
pub fn place_doors(tiles: &mut TileMap, buildings: &BuildingRegistry)
```
Called in `preprocess.rs` between `rasterize_paris()` (line 163, which includes
`classify_walls_floors`) and `save_paris_binary()` (line 198). The function
modifies `tiles` in-place. No changes to BuildingRegistry needed (building_id
on tiles is preserved; Door tiles retain their original building_id).

Call site in `preprocess.rs`:
```rust
// After rasterize + water + addresses + occupants, before save:
println!("Placing doors...");
loading_gis::place_doors(&mut tiles, &buildings);
```

**F16 — Operation ordering.**
Within `place_doors` (full pipeline — each tier implements its subset):
1. F09 — Small building fix (Wall → Floor for eligible all-wall buildings)
2. F01 — Door candidate detection
3. F02 — Facade run detection
4. F03 — Door selection (Tier 1: all candidates; Tier 3: heuristic)
5. F04 — Dual-door guarantee fixup
6. F05 — Island courtyard piercing
7. F06 — Landlocked passage carving (using F07 routing + F08 semantics)
8. F10 — Garden conversion
9. F11–F14 — Validation suite
10. Diagnostic log output

Tier 1 runs steps 2, 4 (simplified), 8, 9 (partial), 10.
Tier 2 adds steps 6, 7, and full 9.
Tier 3 replaces step 4 and adds steps 3, 5.
Tier 4 adds step 1 and completes all remaining.

**F17 — Diagnostic reporting.**
Log the following during preprocessing:
```
  Small buildings: 334 converted to Floor, 318 skipped (≤4 tiles)
  Door placement: 127,432 doors on 20,368 buildings (avg 6.3/building)
    Road-facing: 89,201  Courtyard-facing: 38,231
  Dual-door fixup: 412 additional doors on 389 buildings
  Island courtyards: 9,847/10,128 connected (97.2%)
  Landlocked passages: 11/12 carved (1 unreachable)
  Gardens: 24 buildings, 1,847 tiles converted
  Validation:
    Door-floor adjacency violations: 0
    Doorless buildings: 0
    Connectivity: 100.0% doors reachable, 97.2% courtyards reachable
```
(Numbers are estimates. Actual values from implementation.)

**F18 — Downstream contracts.**
- **B03 (entity spawning):** Every building with occupant data in the registry
  has ≥1 Door tile reachable from the Road network. B03 can pathfind from any
  Road tile to a building's Door. The Door tile's `building_id` identifies which
  building it belongs to.
- **B06 (interior generation):** Door tiles have `terrain == Door` and retain
  their `building_id`. B06 must not place furniture on Door tiles. B06 can query
  `get_terrain(x, y) == Door` to identify no-furniture zones within a building.

---

## Tiered Implementation Plan

### Tier 1 — MVP (unblock B03)

**Goal:** Every building with interior space gets at least one door. Entities
can pathfind from Road to building interior. Crude but functional.

**Features:**
- **F01** — Door candidate detection.
- **F15** — Pipeline integration (`place_doors` function + call site).
- **Simplified F03** — ALL candidates become Doors. No facade runs, no spacing
  heuristic. Every valid wall tile (walkable exterior neighbor + Floor interior
  neighbor) is set to Door.
- **F10** — Garden conversion (24 buildings, trivial, independent).
- **F12** — Per-building door check (basic validation).
- **F17** (partial) — Log door count and doorless building count.

**What this achieves:**
- 75.7% of buildings get road-facing doors (all valid wall tiles → Door).
- 14,504 dual-access buildings get doors on BOTH sides automatically (because
  all candidates are converted). This connects most island courtyards via
  Road → Door → Floor → Door → Courtyard.
- 24.2% courtyard-only buildings get courtyard doors. Many of their courtyards
  become reachable through dual-access neighbors.
- 0.06% landlocked buildings remain without doors (12 total, 1 in Arcis).
- 3.2% all-wall buildings remain without doors (667 total, 6 in Arcis).
- Garden buildings converted.

**What's deferred:** Door density is maximal (every candidate wall tile is a
door). Buildings have 20+ doors where they should have 2–3. Visually crude.
12 landlocked buildings unreachable. Some island courtyards remain isolated.

**Unknown until measured:** How many island courtyards become connected after
Tier 1? The 14,504 dual-access buildings get doors on both sides, which should
connect most courtyards. But the exact number depends on whether every island
courtyard has at least one dual-access perimeter building. Run F13 (global
connectivity BFS) as a diagnostic after Tier 1 to measure actual reachability
before committing to Tier 2's scope.

**Estimated scope:** ~100 lines of Rust. One function. One afternoon.

**Arcis coverage:** 430/437 buildings accessible (98.4%). Sufficient for B03.

---

### Tier 2 — Connectivity Hardening

**Goal:** Close the remaining connectivity gaps. Every courtyard region and
every landlocked building is reachable from the Road network.

**Features:**
- **F05** — Island courtyard piercing. For each island courtyard region, find
  a perimeter building with road access, place a courtyard-facing door.
- **F06** — Landlocked building passage carving (12 buildings).
- **F07** — Carve path routing (BFS utility, shared by F05 and F06).
- **F08** — Passage terrain semantics (Door-Floor-Door sandwich).
- **F13** — Global connectivity validation (BFS from Road, check all Doors
  and Courtyards visited).
- **F14** — Courtyard reachability check.

**What this achieves:**
- Island courtyards connected: target >95% of 10,128 regions.
- 11 of 12 landlocked buildings connected (1 unreachable data artifact).
- Global connectivity validated with quantified metrics.

**Depends on:** Tier 1 complete (doors already placed on non-landlocked buildings).

**Estimated scope:** ~250 lines. Courtyard region BFS + perimeter building
detection + piercing logic + passage carving + validation. The BFS routing
is the most complex single piece.

---

### Tier 3 — Door Quality

**Goal:** Replace "all valid tiles → Door" with a controlled heuristic. Buildings
get a realistic number of doors, properly spaced. Visual quality improves
significantly.

**Features:**
- **F02** — Facade run detection (4-connected candidate grouping).
- **F03** (full) — Door selection heuristic (midpoint, spacing rules by run length).
- **F04** — Dual-door guarantee (after selection, verify both-side coverage).
- **F11** — Door-floor adjacency check (validate selection quality).

**What this achieves:**
- Door count drops from ~6 per building average (all candidates) to ~2–3
  (heuristic selection). Buildings look like buildings, not colanders.
- Dual-door guarantee preserves courtyard connectivity from Tier 2.
- Adjacency validation catches placement bugs.

**Risk:** Reducing door count could break connectivity if the heuristic removes
a door that was the only courtyard link. F04 mitigates this — dual-door
guarantee runs as a fixup pass and forces courtyard doors where needed.

**Depends on:** Tier 2 complete (connectivity guarantees must be maintained
through the transition from all-doors to selected-doors).

**Estimated scope:** ~150 lines. Facade detection is a standard connected-
component algorithm. Selection is a simple spacing function. Dual-door
fixup is a scan + force.

---

### Tier 4 — Edge Cases & Completeness

**Goal:** Handle the long tail. Every building in the dataset is correctly
processed, including pathological cases.

**Features:**
- **F09** — Small building policy (667 all-wall buildings: ≤4 tiles skipped,
  5–20 tiles get synthetic Floor, 21+ tiles get reclassified).
- **F12** (strict) — Per-building door check with zero tolerance (every building
  with Floor or Garden must have a door).
- **F17** (full) — Comprehensive diagnostic logging with all metrics.
- **F18** — Downstream contracts documented and validated.

**What this achieves:**
- 349 more buildings gain interiors and doors (5+ tile all-wall buildings).
- 318 tiny buildings (≤4 tiles) formally excluded from door/spawn logic.
- Zero doorless buildings in the validation output (excluding the ≤4 tile
  exclusions and the 1 unreachable data artifact).
- Full diagnostic suite for ongoing regression detection.

**Depends on:** Tier 3 complete (door selection heuristic must be in place
before adding more buildings to the pool).

**Estimated scope:** ~100 lines. Small building fix is simple iteration.
Validation is straightforward. Diagnostics are print statements.

---

## Cross-Cutting Concerns

### Pipeline Position

```
preprocess.rs main():
  rasterize_paris()          // includes classify_walls_floors()
  rasterize_water()          // A08
  load_addresses()           // A07
  load_occupants()           // A07
  ▶ place_doors()            // B05 — NEW
  save_paris_binary()
```

`place_doors` modifies only `TileMap` terrain values. It does not modify
`BuildingRegistry`, `building_id` arrays, `block_id` arrays, or `quartier_id`
arrays. Door tiles inherit the building_id of the wall tile they replace.

### Terrain Transition Rules

| Before | After | Condition |
|--------|-------|-----------|
| Wall | Door | Wall tile is a selected door candidate |
| Wall | Floor | Carved passage interior tile (F06/F08) |
| Floor | Garden | Garden building interior (F10) |
| Wall | Floor | Small building interior fix (F09) |

No other terrain transitions occur. Road, Courtyard, Water, Bridge, Fixture
are never modified by B05.

### Temperature Impact

- Door: 17°C target (between Floor 18°C and Road 16°C). Thermal transition zone.
- Carved passages use Floor (18°C) in the interior with Door (17°C) only at
  endpoints. No cold corridor artifacts.
- Garden conversion (Floor 18°C → Garden 15°C) is a 3°C drop. Correct: outdoor
  green space is cooler than building interior.

### Known Limitations

1. **One unreachable landlocked building** (carve depth >50). Likely a data
   artifact. Logged and skipped. 0 impact on Arcis.
2. **Some island courtyards may remain** if enclosed entirely by all-wall
   buildings or by buildings with no viable door position. Target >95% connected.
3. **No door orientation metadata.** Terrain enum stores a single value per tile.
   If future rendering needs door facing direction, a parallel data array would
   be needed. Not worth adding now.
4. **No per-floor door placement.** Buildings are 2D footprints. Upper-floor
   access (stairwells) is implicit, not modeled in the tile map.
