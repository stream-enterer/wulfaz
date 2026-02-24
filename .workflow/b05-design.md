# B05 — Door Placement & Passage Carving

Preprocessor extension. Runs after `classify_walls_floors()`, before
`save_paris_binary()`. Modifies `TileMap` terrain in-place. Does not modify
`BuildingRegistry`, `building_id`, `block_id`, or `quartier_id` arrays.

## Constraints

### Pipeline position

```
preprocess.rs main():
  rasterize_paris()          // includes classify_walls_floors()
  rasterize_water()          // A08
  load_addresses()           // A07
  load_occupants()           // A07
  ▶ place_doors()            // B05 — NEW
  save_paris_binary()
```

Entry point in `loading_gis.rs`:
```rust
pub fn place_doors(tiles: &mut TileMap, buildings: &BuildingRegistry)
```

Call site in `preprocess.rs` (between occupant loading and binary save):
```rust
println!("Placing doors...");
loading_gis::place_doors(&mut tiles, &buildings);
```

### Terrain transitions

Only these transitions are permitted. All others are bugs.

| Before | After | Trigger |
|--------|-------|---------|
| Wall | Door | Door candidate selected |
| Wall | Floor | Passage interior (Phase 2) or small building fix (Phase 4) |
| Floor | Garden | Garden conversion |

Road, Courtyard, Water, Bridge, Fixture are never modified.

### Rendering glyphs

| Terrain | Glyph | Example wall run |
|---------|-------|------------------|
| Wall | `#` | `######` |
| Door (closed) | `+` | `##+##` |
| Door (open) | `-` | `##-##` (future) |

Renderer already maps `Terrain::Door => '+'`. Open/closed differentiation
is not yet implemented; all doors render as `+`. When door state is added,
open doors switch to `-`.

### Temperature

- Door: 17°C (transition zone between Floor 18°C and Road 16°C).
- Carved passage interior tiles use Floor (18°C), not Door, to avoid
  cold corridor artifacts. Door only at passage endpoints.
- Garden: 15°C (outdoor green space, 3°C cooler than Floor).

### Downstream contracts

- **B03 (entity spawning):** Every building with occupant data has ≥1 Door
  tile reachable from the Road network. Door tile's `building_id` identifies
  its building. B03 can pathfind from any Road tile to a building's Door.
- **B06 (interior generation):** Door tiles have `terrain == Door` and retain
  `building_id`. B06 must not place furniture on Door tiles.

## Workflow Protocol

### Step completion ritual

After completing each step:
1. `cargo test` — all tests pass.
2. Commit with standardized message (template below each step).
3. Mark the step's checkbox `- [x]`.

### Commit message format

```
B05 P{phase}S{step}: {short description}
```

### Phase completion ritual

After all steps in a phase pass:
1. Verify exit criteria (run diagnostics, check counts).
2. Update `checkpoint.md` with current B05 status and modified files.
3. Commit checkpoint update: `B05 P{phase}: Phase {phase} complete — {summary}`

### Failure protocol

- If `cargo test` fails: fix before committing. Do not advance.
- If exit criteria fail: do not advance to next phase. Fix and re-verify.
- Pre-commit hook runs `cargo fmt`, `cargo clippy -D warnings`, `cargo test` automatically.

### Final completion

After Phase 4 exit criteria pass:
1. Delete `SCALE-B05` entry from `backlog.md`.
2. Update `checkpoint.md` with final B05 status.
3. Final commit: `B05: Door placement complete`

---

## Diagnostic baseline

From `cargo run --bin building_diag`:

| Metric | Value |
|--------|-------|
| BATI=1 buildings | 21,035 |
| With interior (Floor > 0) | 20,368 (96.8%) |
| All-wall (no Floor) | 667 (3.2%) |
| Road-adjacent walls | 15,418 (75.7%) |
| Courtyard-only access | 4,938 (24.2%) |
| Landlocked (no Road or Courtyard neighbor) | 12 (0.06%) |
| Dual-door candidates (Road + Courtyard) | 14,504 (71.2%) |
| Island courtyard regions (4-connected) | 10,128 (875K tiles) |
| Arcis BATI=1 with Floor | 437 |
| Arcis landlocked | 1 |
| Arcis buildings with occupants | 273 (1,814 records) |

---

## Phase 1 — MVP

**Goal:** Every BATI=1 building with interior space gets doors. Entities can
pathfind from Road to building interior. Unblocks B03.

**Approach:** Convert ALL valid wall tiles to Door (no spacing heuristic).
This is intentionally crude — maximizes connectivity, defers visual quality.

### Entry criteria

- `classify_walls_floors()` has run. Every BATI=1 building tile is Wall or Floor.
- `BuildingRegistry` is populated with tile lists and BATI classification.

### Steps

- [x] **Step 1 — Door candidate detection and placement.**

Iteration pattern (mirrors `classify_walls_floors` in `loading_gis.rs`):
```rust
let mut door_tiles: Vec<(usize, usize)> = Vec::new();
for bdata in &buildings.buildings {
    if bdata.bati != 1 { continue; }
    let bid = bdata.id;                       // BuildingId
    for &(cx, cy) in &bdata.tiles {           // tiles: Vec<(i32, i32)>
        let ux = cx as usize;
        let uy = cy as usize;
        if tiles.get_terrain(ux, uy) != Some(Terrain::Wall) { continue; }
        // ... candidate checks using ux, uy, bid ...
    }
}
for (x, y) in door_tiles { tiles.set_terrain(x, y, Terrain::Door); }
```

For each Wall tile, check cardinal neighbors `(cx+dx, cy+dy)` with
`dx, dy` in `[(-1,0), (1,0), (0,-1), (0,1)]`. Cast to usize; if either
coord is negative, skip (map boundary).

1. **Exterior candidate:** any neighbor where
   `tiles.get_terrain(nx, ny).map_or(false, |t| t.is_walkable())` AND
   `tiles.get_building_id(nx, ny) != Some(bid)`. Uses `Terrain::is_walkable()`
   method (true for Road, Floor, Door, Courtyard, Garden, Bridge, Fixture).
2. **Interior access:** any neighbor where
   `tiles.get_terrain(nx, ny) == Some(Terrain::Floor)` AND
   `tiles.get_building_id(nx, ny) == Some(bid)`.
3. If BOTH pass, push `(ux, uy)` to `door_tiles`.

Collect-then-apply: batch `set_terrain` calls after iteration.

Skip buildings with `bati != 1`. Skip buildings with zero tiles.

Edge cases handled automatically:
- Wall adjacent to Water: `is_walkable()` returns false → no exterior candidate.
- Wall at map boundary: `get_terrain` returns None → `map_or(false, ...)` → skip.
- Building tiles overwritten by BATI=2/3 (Courtyard/Garden/Fixture terrain):
  `get_terrain != Some(Wall)` → skipped.

Commit: `B05 P1S1: Place doors on all BATI=1 buildings`

- [x] **Step 2 — Garden conversion.**

For each building where `nom_bati: Option<String>` contains "parc" or "jardin":
```rust
if let Some(ref name) = bdata.nom_bati {
    let lower = name.to_lowercase();
    if lower.contains("parc") || lower.contains("jardin") { ... }
}
```
(24 buildings match.) Iterate the building's tiles. For each tile that is
`Terrain::Floor`, set `terrain = Garden`. Do NOT convert Door tiles.

This runs AFTER door placement so doors are preserved.

Commit: `B05 P1S2: Convert park/garden buildings to Garden terrain`

- [x] **Step 3 — Per-building door validation.**

For each BATI=1 building with ≥1 Floor or Garden tile: scan its tiles for at
least one Door. Log any doorless buildings (id, quartier, tile count).

Expected: 667 doorless buildings (the all-wall buildings). 0 doorless
buildings that have interior space.

Commit: `B05 P1S3: Validate per-building door coverage`

- [x] **Step 4 — Diagnostic log.**

```
Door placement: {N} doors on {M} buildings (avg {N/M:.1}/building)
Gardens: {G} buildings, {T} tiles converted
Doorless buildings with interior: {D} (expected 0)
Doorless buildings without interior: {W} (all-wall, expected 667)
```

Commit: `B05 P1S4: Add door placement diagnostic log`

### Exit criteria

- Every BATI=1 building with ≥1 Floor tile has ≥1 Door tile.
- 24 garden buildings have Floor→Garden conversion.
- `cargo run --bin building_diag` still runs clean on the output.
- `cargo test` passes.
- Arcis coverage: ≥430/437 buildings with doors (98.4%).

**Phase gate:** Run exit criteria checks. If all pass, update `checkpoint.md` and commit:
`B05 P1: Phase 1 complete — MVP door placement`

### What this defers

- Door density is maximal (buildings have 20+ doors). Phase 3 fixes this.
- 12 landlocked buildings remain without doors. Phase 2 fixes this.
- 667 all-wall buildings remain without doors. Phase 4 fixes this.
- Island courtyard reachability is unknown — measure with Phase 2's
  connectivity BFS to quantify before committing to piercing scope.

---

## Phase 2 — Connectivity

**Goal:** Every courtyard region and every landlocked building is reachable
from the Road network via walkable tiles.

### Entry criteria

Phase 1 complete. Doors placed on all non-landlocked buildings.

### Steps

- [ ] **Step 1 — BFS carve routing utility.**

Implement a shared BFS function:
```rust
fn bfs_to_walkable(
    tiles: &TileMap,
    seeds: &[(i32, i32)],
    max_depth: usize,
) -> Option<Vec<(i32, i32)>>
```

BFS from `seeds` outward through Wall and Floor tiles (treating both as
traversable). Cardinal movement only (4-dir). Stop when a tile with
walkable terrain outside any building is reached (Road, Courtyard, Garden,
Bridge, Fixture with no building_id, or terrain already Door). Return the
path from seed to the tile adjacent to the walkable destination (exclusive
of destination). Return None if max_depth exceeded.

Implementation: use `VecDeque<(i32, i32)>` as queue, `HashSet<(i32, i32)>`
for visited. Track parents via `HashMap<(i32, i32), (i32, i32)>`. On
reaching a goal tile, walk the parent chain back to the seed to reconstruct
the path. Coordinates are `i32` throughout BFS; cast to `usize` only for
TileMap accessor calls. Skip neighbors where `nx < 0 || ny < 0`.

Max depth: 50 tiles (track depth per-node, or limit BFS expansions).

Commit: `B05 P2S1: Implement BFS carve routing utility`

- [ ] **Step 2 — Landlocked building passage carving.**

For each BATI=1 building where Phase 1 placed zero doors AND the building
has ≥1 Floor tile (12 buildings):

1. Collect all Wall tiles of this building as BFS seeds.
2. Call `bfs_to_walkable(tiles, &seeds, 50)`.
3. If path found (path = Vec of tiles from building wall to destination wall):
   - First tile (building's own wall): set `terrain = Door`.
   - Middle tiles (all except first and last): set `terrain = Floor`.
     These tiles retain their original `building_id` (may belong to
     intervening buildings). This is expected — they become walkable
     corridor tiles. Do not change `building_id`.
   - Last tile (wall of destination building): set `terrain = Door`.
4. If not found: log warning with building id and quartier. Skip.

Expected: 11 carved, 1 unreachable (data artifact, depth >50).

Depth distribution from diagnostic: 2 at depth 1, 4 at depth 2, 2 at
depth 3, 3 at depth 4–5, 1 unreachable.

Commit: `B05 P2S2: Carve passages for landlocked buildings`

- [ ] **Step 3 — Island courtyard piercing.**

1. Build courtyard regions via 4-connected BFS over all `Terrain::Courtyard`
   tiles. Each region is a `Vec<(usize, usize)>`. Also build a
   `HashSet<(usize, usize)>` per region for O(1) membership tests.
2. Classify each region: **connected** if any tile has a cardinal neighbor
   with `terrain == Road`. Otherwise **island**. (Door neighbors are NOT
   sufficient — the Door could face another island courtyard.)
3. For each island region (process smallest to largest):
   a. Scan boundary tiles (courtyard tiles with ≥1 non-courtyard cardinal
      neighbor). For each boundary tile's cardinal neighbors: if the
      neighbor has `terrain == Wall` and `get_building_id` returns
      `Some(bid)`, collect `bid`. These are **perimeter buildings**.
   b. Among perimeter buildings, find one with a Door tile adjacent to Road
      (road-connected building). Query: does any of this building's tiles
      have `terrain == Door` AND a cardinal neighbor with `terrain == Road`?
   c. On that building, find a Wall tile where: (a) ≥1 cardinal neighbor is
      in THIS region's tile set (`region_set.contains(&(nx, ny))`), AND
      (b) ≥1 cardinal neighbor is `Terrain::Floor` with same building_id.
      Set `terrain = Door`. If already Door, courtyard is already
      connected — skip.
   d. If no perimeter building has road access: find a perimeter building
      adjacent to a **connected** courtyard region instead. Pierce through
      that building (same logic as step c). After piercing, mark this
      region as **connected** so subsequent island regions can chain
      through it.
   e. If no viable building found: log warning. Leave as island.

Process regions from smallest to largest. This ensures small nested
courtyards that depend on larger outer courtyards being connected are
handled after those outer courtyards are pierced.

Commit: `B05 P2S3: Pierce island courtyards to Road network`

- [ ] **Step 4 — Global connectivity validation.**

Find a starting Road tile by scanning the grid for the first
`get_terrain(x, y) == Some(Terrain::Road)`. BFS through all walkable
terrain (`is_walkable()` tiles). After BFS:
- Count visited Door tiles vs total Door tiles.
- Count visited Courtyard tiles vs total Courtyard tiles.
- Build courtyard regions again (or reuse), count how many have ≥1 visited tile.

Log:
```
Connectivity: {D}/{DT} doors reachable ({P:.1}%)
Courtyards: {CR}/{CT} regions reachable ({CP:.1}%)
Landlocked passages: {L}/12 carved (1 unreachable data artifact)
```

Target: >95% courtyard regions reachable, 100% doors reachable.

Commit: `B05 P2S4: Add global connectivity validation`

### Exit criteria

- Global BFS reaches 100% of Door tiles.
- Global BFS reaches >95% of courtyard regions.
- 11/12 landlocked buildings have passages carved.
- `cargo test` passes.

**Phase gate:** Run exit criteria checks. If all pass, update `checkpoint.md` and commit:
`B05 P2: Phase 2 complete — full connectivity`

---

## Phase 3 — Door Quality

**Goal:** Replace "all valid candidates → Door" with a spacing heuristic.
Buildings get 1–3 doors instead of 20+.

### Entry criteria

Phase 2 complete. All connectivity guarantees established.

### Steps

**Note:** Phase 3 REPLACES Phase 1's Step 1 entirely. The `place_doors`
function is rewritten to use candidate detection → run grouping → selection
instead of converting all candidates. Garden conversion (Step 2) and
validation (Steps 3-4) remain unchanged.

- [ ] **Step 1 — Facade run detection.**

Compute door candidates using the same criteria as Phase 1 Step 1 (exterior +
interior adjacency), but collect candidates WITHOUT converting to Door.

Group each building's candidates into **facade runs** via 4-connected BFS
over the candidate set. Two candidate tiles are in the same run if they share
a cardinal edge. Use a `HashSet<(usize, usize)>` of candidates, BFS from
each unvisited candidate to find its connected component.

Classify each run by **facing**: check the exterior walkable neighbor of the
run's tiles. If any tile's exterior neighbor is `Terrain::Road`, the run is
**Road-facing**. If any is `Terrain::Courtyard`, it is **Courtyard-facing**.
A run can be both (dual-facing). Store this per run.

Order tiles within each run: walk from one endpoint along cardinal edges.
For L-shaped runs, use BFS traversal order (the exact order doesn't matter
for midpoint selection — index `len / 2` picks a tile near the geometric
center regardless).

Output: `Vec<Vec<(usize, usize)>>` per building — list of runs, each run
an ordered list of cardinally-adjacent candidate tiles, with facing metadata.

Commit: `B05 P3S1: Detect facade runs with facing metadata`

- [ ] **Step 2 — Door selection heuristic.**

For each facade run, select tile positions:

| Run length | Doors | Positions |
|-----------|-------|-----------|
| 1–2 | 1 | First tile |
| 3–10 | 1 | Midpoint |
| 11–20 | 2 | ⅓ and ⅔ |
| 21+ | `max(2, len / 10)` | Evenly spaced: indices `i * len / count` |

Midpoint for even-length runs: index `len / 2` (integer division). For the
⅓/⅔ positions: indices `len / 3` and `2 * len / 3`.

Each selected tile must pass **interior adjacency**: ≥1 cardinal neighbor is
Floor or Garden with `get_building_id == Some(bid)`. If the selected index
fails, slide outward in both directions (index ±1, ±2, …) and pick the
first candidate that passes. If no candidate in the run passes, skip the
entire run.

Convert selected tiles to Door. All other candidates remain Wall. Then run
garden conversion (same as Phase 1 Step 2).

Commit: `B05 P3S2: Select doors via spacing heuristic`

- [ ] **Step 3 — Dual-door guarantee.**

After selection, check each building that has facade runs facing BOTH Road
and Courtyard (use the per-run facing metadata from Step 1). Verify it has
≥1 Door adjacent to Road AND ≥1 Door adjacent to Courtyard. If either side
is missing, force the midpoint of the longest run on that side (that passes
the interior adjacency check, with the same slide fallback).

This preserves the courtyard connectivity established in Phase 2. Without
this fixup, the spacing heuristic could remove the only courtyard-facing
door from a perimeter building, disconnecting the courtyard.

Commit: `B05 P3S3: Ensure dual-door buildings keep both facings`

- [ ] **Step 4 — Door-floor adjacency validation.**

After all placement (including garden conversion), check every Door tile:
does it have ≥1 cardinal neighbor that is Floor or Garden? Log violations.
Not a hard failure — violating doors are still walkable — but indicates a
selection bug.

Commit: `B05 P3S4: Validate door-floor adjacency`

### Exit criteria

- Average doors per building drops to 1–3 range (from 6+ in Phase 1).
- Phase 2's connectivity BFS still passes (re-run validation).
- Zero door-floor adjacency violations.
- `cargo test` passes.

**Phase gate:** Run exit criteria checks. If all pass, update `checkpoint.md` and commit:
`B05 P3: Phase 3 complete — door quality heuristic`

---

## Phase 4 — Edge Cases

**Goal:** Handle the 667 all-wall buildings. Full diagnostic suite.

### Entry criteria

Phase 3 complete.

### Steps

- [ ] **Step 1 — Small building interior fix.**

Run BEFORE door candidate detection (prepend to `place_doors` pipeline).

Full `place_doors` internal ordering (Phases 3+4 combined):
1. Small building interior fix (this step)
2. Door candidate detection → facade run grouping → selection (Phase 3)
3. Landlocked passage carving (Phase 2)
4. Island courtyard piercing (Phase 2)
5. Garden conversion
6. Validation + diagnostic log

For each BATI=1 building with zero Floor tiles:
- **≤4 tiles (318 buildings):** Skip. Too small for occupants. No door.
- **5–20 tiles (334 buildings):** Convert one Wall tile to Floor. Pick the
  tile with the most same-building cardinal neighbors (most interior):
  count how many of `(cx±1, cy), (cx, cy±1)` have
  `get_building_id == Some(bid)`. If tied, pick the tile closest to the
  building centroid (centroid = average of all tile `(cx, cy)` as `f32`).
- **21+ tiles (15 buildings):** Re-classify with a RELAXED criterion (new
  logic, not `classify_walls_floors`): a tile is Floor if ≥3 of its 4
  cardinal neighbors belong to the same building. Otherwise Wall. This is
  less strict than the original (which requires all 4), recovering interior
  tiles from thin/irregular rasterization. Apply to ALL tiles of the
  building, overwriting the previous all-Wall classification.

After this step, normal door placement handles the newly-interior buildings.

Commit: `B05 P4S1: Fix all-wall buildings with interior conversion`

- [ ] **Step 2 — Strict per-building door check.**

Every BATI=1 building with ≥1 Floor or Garden tile must have ≥1 Door tile.
Zero tolerance. Buildings with zero Floor AND zero Garden (≤4 tile skips)
are excluded from this check.

Log any failures with building id, quartier, superficie, tile count.
Expected: 0 failures.

Commit: `B05 P4S2: Strict zero-tolerance door check`

- [ ] **Step 3 — Full diagnostic log.**

```
Small buildings: {C} converted to Floor, {S} skipped (≤4 tiles)
Door placement: {N} doors on {M} buildings (avg {N/M:.1}/building)
  Road-facing: {R}  Courtyard-facing: {CF}
Dual-door fixup: {DF} additional doors on {DB} buildings
Island courtyards: {CR}/{CT} connected ({CP:.1}%)
Landlocked passages: {L}/12 carved (1 unreachable)
Gardens: {G} buildings, {GT} tiles converted
Validation:
  Door-floor adjacency violations: {V}
  Doorless buildings (with interior): {DL}
  Connectivity: {DP:.1}% doors reachable, {CP:.1}% courtyards reachable
```

Commit: `B05 P4S3: Full diagnostic log for door placement pipeline`

### Exit criteria

- Zero doorless buildings with interior space.
- Full diagnostic log emitted.
- `cargo test` passes.
- `cargo run --bin building_diag` output consistent with expectations.

**Phase gate:** Run exit criteria checks. If all pass, update `checkpoint.md` and commit:
`B05 P4: Phase 4 complete — edge cases handled`

---

## Final completion

When Phase 4 exit criteria pass:
1. Delete the `SCALE-B05` entry from `.workflow/backlog.md`.
2. Update `.workflow/checkpoint.md` with final B05 status (all phases complete, modified files, test count).
3. Commit: `B05: Door placement complete`

---

## Known limitations

1. **One unreachable landlocked building** (carve depth >50). Data artifact.
   Logged and skipped. Zero impact on Arcis.
2. **Some island courtyards may remain** if enclosed entirely by all-wall
   buildings with no viable door position. Target >95% connected.
3. **No door orientation metadata.** Terrain enum stores a single value per
   tile. If rendering needs door facing direction later, add a parallel
   data array.
4. **No per-floor doors.** Buildings are 2D footprints. Upper-floor access
   (stairwells) is implicit.
5. **No open/closed door state yet.** All doors render as `+` (closed).
   When door state is added, open doors switch to `-`.
