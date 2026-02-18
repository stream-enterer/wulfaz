# Checkpoint

## Active Task
Fix rasterization pass ordering regression — BATI=2 must carve courtyards AFTER BATI=1.

## Known Regression (commit 50fbf55)
The Halle au Blé (large circular building, center of city) lost its interior
courtyard. Previously it showed as a donut (Wall ring around Courtyard center).
Now it's a solid disk of Floor. Root cause: BATI=2 runs BEFORE BATI=1, then
BATI=1 overwrites the courtyard. BATI=2 must run AFTER BATI=1 to carve holes.

Correct ordering:
```
1. Blocks         → Courtyard + block_id + quartier_id
2. BATI=1         → Wall + building_id  (overwrites Courtyard)
3. BATI=2         → Courtyard/Garden + clear building_id  (carves holes in buildings)
4. classify_walls_floors  (on surviving BATI=1 tiles only)
5. fill_quartier_roads    (BFS quartier_id to Road tiles)
```

The fix is in `rasterize_paris()` in `src/loading_gis.rs` (~line 461).

## Rasterization Simplifications Audit

Complete inventory of every place the pipeline drops data, makes an
assumption, or takes a shortcut. Ordered by pipeline phase. Each entry
names the exact function/line, what is lost, and the consequence.

### Extraction Phase: Shapefile → RON

#### S01. Inner rings (polygon holes) are dropped
- **Where:** `extract_outer_ring()` at `src/loading_gis.rs` ~line 202.
- **What it does:** Takes only `shape.rings().first()`, discarding all
  subsequent rings.
- **What is lost:** Shapefiles encode polygon holes as inner rings (rings
  after the first, with opposite winding). Any polygon whose geometry
  contains a courtyard, lightwell, or passage as an inner ring will be
  rasterized as a solid fill instead of a ring.
- **Affected records:** Both BATI.shp (buildings) and Vasserot_Ilots.shp
  (blocks). Unknown how many polygons have inner rings — never checked.
- **Consequence:** Buildings with courtyards encoded as holes in the
  polygon (rather than as separate BATI=2 records) are solid. Block
  polygons with holes (e.g. a block with an internal plaza cut out)
  would also be solid. This is a distinct problem from the pass-ordering
  regression — even after flipping pass order, polygons with inner-ring
  courtyards will still be solid.
- **Detection:** After flipping pass order, any building that still
  appears filled where it should have a courtyard is an inner-ring case.
  Can also count: `shape.rings().len() > 1` during extraction and log.

#### S02. Non-polygon shapes silently dropped
- **Where:** `extract_blocks_from_shapefile()` ~line 253,
  `extract_buildings_from_shapefile()` ~line 307.
- **What it does:** `match shape { Polygon(p) => p, _ => continue }`.
- **What is lost:** Any record whose geometry is not `Shape::Polygon` —
  e.g. MultiPolygon, PolygonZ, PolygonM, or NullShape. Dropped with no
  log message, no count.
- **Affected records:** Unknown. Vasserot data is likely all Polygon, but
  this has never been verified. If even one record is MultiPolygon, its
  sub-polygons are all lost.
- **Consequence:** Silent data loss. Could miss buildings or blocks
  entirely with no indication.
- **Detection:** Log a count of non-Polygon shapes during extraction.

#### S03. Out-of-viewport polygons dropped
- **Where:** `extract_blocks_from_shapefile()` ~line 259,
  `extract_buildings_from_shapefile()` ~line 313.
  Filter function: `bbox_overlaps()` ~line 166.
- **What it does:** Computes each polygon's bounding box and checks
  overlap with the hardcoded VIEW_MIN/MAX_LON/LAT constants.
- **What is lost:** Any polygon whose bounding box falls entirely outside
  the viewport. This is intentional — the viewport covers central Paris.
- **Risk:** If the VIEW constants are wrong (e.g. off by a sign, wrong
  datum), entire swaths of data are silently dropped. The constants
  are never validated against the actual shapefile extent.
- **Consequence:** Probably fine, but a single-digit typo in the sixth
  decimal of VIEW_MIN_LON could clip an edge of the city.

#### S04. Zero-area polygons dropped (at wrong resolution)
- **Where:** `extract_blocks_from_shapefile()` ~line 265,
  `extract_buildings_from_shapefile()` ~line 318.
- **What it does:** `scanline_fill(&ring, 10000, 10000).is_empty()` — if
  the polygon produces zero tiles on a 10000×10000 grid, it's dropped.
- **What is lost:** Polygons too small to rasterize at 10K resolution.
  But the actual grid is ~6364×4809. A polygon that barely fills at 10K
  might not fill at 6364. Conversely, a polygon that doesn't fill at 10K
  (because coordinates land between pixels differently) might fill at
  6364. The filter resolution doesn't match the rasterization resolution.
- **Affected records:** Likely very few — most buildings are larger than
  one tile. But BATI=3 minor features (fountains) could be <1m² and
  affected.
- **Consequence:** Small polygons may be inconsistently included/excluded
  depending on how their vertices align with the pixel grid at 10K vs
  the real resolution.

### Rasterization Phase: RON → Tiles

#### S05. BATI=2 non-gardens fully skipped (THE KNOWN BUG)
- **Where:** `rasterize_paris()` ~line 478, the `2 =>` match arm.
- **What it does:** If `nom_bati` does not contain "jardin" or "parc",
  the polygon is skipped — no `scanline_fill`, no terrain write.
- **What is lost:** 19,080 BATI=2 polygons representing courtyards, rear
  yards, passages, and other non-built open spaces within blocks. Their
  geometry is preserved in the RON file but never applied to tiles.
- **Consequence:** When a BATI=2 courtyard polygon overlaps a BATI=1
  building polygon (the courtyard is inside the building footprint), the
  courtyard is not carved out. The building appears solid. This is the
  Halle au Blé regression. Fix: rasterize ALL BATI=2 AFTER BATI=1 so
  they carve courtyards (Courtyard terrain) and clear building_id.

#### S06. BATI=3 fully skipped
- **Where:** `rasterize_paris()` ~line 495, the `3 =>` match arm.
- **What it does:** Increments `skipped_bati3` counter, does nothing else.
- **What is lost:** 199 BATI=3 polygons. These are minor features:
  fountains, wells, kiosks, ambiguous structures. Their geometry is in
  the RON file but never applied to tiles. No registry entry, no terrain
  effect.
- **Consequence:** Currently harmless — tiles stay Courtyard from the
  block pass. But if any BATI=3 polygon overlaps a BATI=1 polygon, the
  feature is buried under building. If BATI=3 features should be visible
  (e.g. a fountain in a courtyard), they aren't.

#### S07. Garden detection is naive string matching
- **Where:** `rasterize_paris()` ~line 480.
- **What it does:** `lower.contains("jardin") || lower.contains("parc")`.
  Only these two substrings trigger Garden terrain.
- **What is lost:** Any garden/green space whose `nom_bati` uses a
  different French term. Possible misses: "verger" (orchard), "potager"
  (vegetable garden), "pépinière" (nursery), "bosquet" (grove),
  "promenade", "square" (public garden in French usage), "parterre"
  (flower bed), "terrain" (open ground), "cimetière" (cemetery — often
  green space in this era).
- **Affected records:** Unknown without scanning all 19,116 BATI=2
  `nom_bati` values. The 36 detected gardens may be an undercount.
- **Consequence:** Green spaces with non-matching names stay Courtyard
  instead of Garden. Cosmetic difference (both walkable) but wrong
  terrain type affects temperature targets (Garden=15°, Courtyard=16°)
  and future gameplay (farming, foraging).

#### S08. Garden polygons get no registry entry or building_id
- **Where:** `rasterize_paris()` ~line 488–493.
- **What it does:** Sets terrain to Garden, increments counters. Does NOT
  assign a building_id, does NOT create a BuildingData entry, does NOT
  add to any registry.
- **What is lost:** Runtime queryability. Given a Garden tile, there is
  no way to look up which garden it belongs to, its name, its area, its
  Identif, or any other metadata. The metadata exists in the RON file
  but is not loaded into any runtime structure.
- **Consequence:** Systems that need garden metadata (e.g. "is this the
  Jardin des Tuileries?") cannot answer the query. Acceptable for now
  but blocks future features that reference named green spaces.

#### S09. Building-to-block assignment takes first tile hit
- **Where:** `rasterize_paris()` ~line 530–536.
- **What it does:** Iterates the building's rasterized cells, takes the
  `block_id` of the first cell that has one, stops.
- **What is lost:** If a building spans two blocks (straddles a block
  boundary due to digitization imprecision or genuine geography), only
  one block gets the building in its `buildings` Vec. The other block
  doesn't know about it.
- **Consequence:** Block-level queries ("how many buildings in this
  block?") undercount for boundary buildings. The building's `num_ilot`
  field (from the shapefile) is correct, but the runtime block linkage
  may disagree.

#### S10. Last-writer-wins on BATI=1 tile overlap
- **Where:** `rasterize_paris()` ~line 539–544.
- **What it does:** For each cell in a building's polygon, unconditionally
  sets terrain=Wall and building_id=new_id. If two BATI=1 polygons
  overlap (party walls, digitization imprecision), the second polygon's
  building_id overwrites the first's.
- **What is lost:** The first building's ownership of the overlapping
  tiles. Those tiles are still in the first building's `tiles` Vec (they
  were added during its rasterization pass), but the tile array says
  they belong to the second building.
- **Consequence:** `classify_walls_floors` iterates the first building's
  `tiles` Vec and checks cardinal neighbors against that set. The
  overlapping tiles ARE in the set, so classify sees them as interior.
  But the tile array's building_id points to a different building. This
  creates a mismatch: the first building thinks it owns tiles it doesn't,
  and may produce incorrect Wall/Floor classification at the boundary.
  The second building's tile set is correct (it was rasterized later).

#### S11. Block polygon overlap — same last-writer-wins
- **Where:** `rasterize_paris()` ~line 439–444.
- **What it does:** For each cell in a block polygon, unconditionally sets
  terrain=Courtyard, block_id, and quartier_id.
- **What is lost:** If two block polygons overlap, the first block's
  block_id/quartier_id are overwritten in the overlap zone. Buildings
  in the overlap zone get assigned to whichever block was rasterized
  last (via S09's first-tile-hit logic).
- **Consequence:** Block membership and quartier assignment are wrong
  for tiles and buildings in the overlap zone. No detection or logging
  of overlaps exists. Unknown whether the Vasserot data has block
  overlaps — cadastral data typically doesn't, but digitization errors
  could create them.

### Post-Processing Phase

#### S12. classify_walls_floors uses stale tile lists
- **Where:** `classify_walls_floors()` ~line 676.
- **What it does:** Iterates `buildings.buildings`, builds a HashSet from
  each `bdata.tiles`, classifies each tile as Wall (has a cardinal
  neighbor outside the set) or Floor (all cardinal neighbors in set).
- **What is lost:** The `tiles` Vec is populated during BATI=1
  rasterization and never updated afterward. Three things can make it
  stale:
  1. BATI=2 carving (once implemented) removes tiles from the building
     — but the Vec still contains them.
  2. Overlapping BATI=1 polygons (S10) — the first building's Vec
     contains tiles whose building_id now points to the second building.
  3. Any future operation that modifies building_id on tiles without
     updating the registry Vec.
- **Consequence:** Wall/Floor classification operates on a tile set that
  doesn't match the tile array. Carved-out courtyard tiles would be
  classified as Floor (they're interior to the stale set). Overlapping
  tiles would be classified based on the wrong building's geometry.
  After flipping pass order (S05 fix), the carving pass MUST also update
  building tile lists in the registry before classify_walls_floors runs.

## What Was Committed (50fbf55)
- Fixed BATI doc comments (1=built, 2=non-built, 3=minor)
- Added 5 missing shapefile fields to BuildingData/BlockData/RON structs
- Two-pass rasterization (WRONG ORDER — needs flipping per S05)
- 6 new rasterization tests, all 170 pass
- Preprocessor regenerated binary data (21,035 buildings, 36 gardens)

## Next Action
Flip pass order: BATI=1 first, then BATI=2 carves courtyards + clears
building_id + updates building tile lists. Update tests. Re-run
preprocessor, visually verify Halle au Blé has its donut hole back.
