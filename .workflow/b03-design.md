# B03 — GIS-Aware Entity Spawning

Spawn simulation entities from SoDUCo directory data. For each known occupant
in the target quartier, create an entity with full component set, positioned
on a floor tile of their building. Runtime operation, not preprocessor.

## Constraints

### Call site

```
main.rs, after map loading, before event loop:
  load_paris_binary()          // or load_paris_ron + apply_paris_ron
  tiles.initialize_temperatures()
  load_utility_config()
  ▶ spawn_gis_entities()       // B03 — NEW
  ... camera, app, event_loop
```

Entry point in `loading_gis.rs`:
```rust
pub fn spawn_gis_entities(world: &mut World, target_quartier: &str)
```

Call site in `main.rs` (after `load_utility_config`, before camera setup):
```rust
loading_gis::spawn_gis_entities(&mut world, "Arcis");
```

The `target_quartier` parameter is a string matching `BuildingData.quartier`.
Filtered by string comparison per building (~825 for Arcis), trivially fast
at startup.

### Data source

SoDUCo commercial directories. Each `Occupant` represents a **workplace**
listing, not a home address. However, for 1840s Paris artisans/shopkeepers
(the directory-listed population), home and workplace were the same building
in ~70-80% of cases — the shop occupied the ground floor, the household
lived above.

For B03 MVP (directory-listed people only), `HomeBuilding` and `Workplace`
are set to the same `BuildingId`. C05 (procedural population) will
differentiate them for domestiques, journaliers, blanchisseuses, etc.

### Active year

`world.gis.active_year` (currently 1839, **change to 1845** in `GisTables::new()`).
1845 has the best SoDUCo match rate (40.1%) and aligns with `StartDate::default_1845()`.

Occupant lookup uses `building.occupants_nearest(world.gis.active_year, 20)`.
Returns the nearest available snapshot within ±20 years, preferring later years.
Single year per building — this is a point-in-time simulation, not a merge of
multiple snapshots.

### Occupant name field

`Occupant.name` is often comma-separated when multiple people share one
directory entry (e.g. `"Dupont, Lefèvre"`). B03 splits on `,` and trims
whitespace, spawning one entity per name. Each entity gets the same
`activity` and `naics` from the shared entry.

### Quartier filtering

Filter buildings by string comparison: `building.quartier == target_quartier`.
~825 buildings in Arcis; comparison is once per building, not per tile.

Note: `world.gis.quartier_names` maps 0-indexed position → quartier name.
The numeric `quartier_id` (1-based, `position + 1`) is derivable for future
use in C02 zone framework, but B03 uses string comparison only.

## New components

### `HomeBuilding` and `Workplace`

```rust
// components.rs
/// The building where this entity lives.
pub struct HomeBuilding(pub BuildingId);

/// The building where this entity works.
pub struct Workplace(pub BuildingId);
```

Placement: `GisTables` (alongside `buildings: BuildingRegistry`). These link
entities to GIS data, not to body/mind state.

```rust
// world.rs — GisTables
pub home_buildings: HashMap<Entity, HomeBuilding>,
pub workplaces: HashMap<Entity, Workplace>,
```

Add to `GisTables::new()`: `HashMap::new()` for both.
**Create** `GisTables::remove(&entity)` method (does not exist yet — GisTables
currently has no entity-keyed HashMaps). Call `.remove(&entity)` on both maps.
Add `self.gis.remove(&entity)` to `World::despawn()` (currently only calls
`body.remove()` and `mind.remove()`).
Add to `validate_world()`: alive-check for both.

### `Occupation`

```rust
// components.rs
/// Professional activity from SoDUCo directory data.
pub struct Occupation {
    /// Free-text French activity string, e.g. "boulanger", "rentier".
    /// Display in entity inspector tooltip.
    pub activity: String,
    /// NAICS industry category code. Used by systems for behavior differentiation.
    pub naics: String,
}
```

Placement: `MindTables` (occupation shapes decisions — a boulanger goes to
their shop, a rentier stays home). This is the entity's identity/role, which
informs Phase 3 decision-making.

```rust
// world.rs — MindTables
pub occupations: HashMap<Entity, Occupation>,
```

Add to `MindTables::new()`: `HashMap::new()`.
Add to `MindTables::remove()`: `.remove(&entity)`.
Add to `validate_world()`: alive-check.

## Entity icon

`☻` (U+263B, BLACK SMILING FACE). All GIS-spawned person entities use this
glyph. Distinct from all existing creature/item icons.

```rust
world.body.icons.insert(e, Icon { ch: '☻' });
```

## Spawn procedure

Two-phase approach required: `world.spawn()` takes `&mut self`, which
conflicts with the iterator borrowing `world.gis.buildings.buildings`.
Collect spawn data first (immutable borrows only), then spawn (mutable).

### Phase 1 — Collect spawn data

Imperative loop (not `filter_map`) to count skip reasons for diagnostics.

```rust
let start = std::time::Instant::now();
let active_year = world.gis.active_year;
let mut spawn_data: Vec<(BuildingId, Vec<(i32, i32)>, Vec<Occupant>)> = Vec::new();

let mut buildings_in_quartier: u32 = 0;
let mut buildings_with_occupants: u32 = 0;
let mut buildings_skipped_no_floors: u32 = 0;
let mut occupant_records: u32 = 0;

for building in &world.gis.buildings.buildings {
    if building.quartier != target_quartier { continue; }
    buildings_in_quartier += 1;

    let Some((_, occupants)) = building.occupants_nearest(active_year, 20)
    else { continue; };
    buildings_with_occupants += 1;
    occupant_records += occupants.len() as u32;

    let floor_tiles: Vec<(i32, i32)> = building.tiles.iter()
        .filter(|&&(x, y)| {
            world.tiles.get_terrain(x as usize, y as usize)
                == Some(Terrain::Floor)
        })
        .copied()
        .collect();
    if floor_tiles.is_empty() {
        buildings_skipped_no_floors += 1;
        log::debug!(
            "Building {:?} has {} occupants but no floor tiles — skipped",
            building.id, occupants.len()
        );
        continue;
    }

    spawn_data.push((building.id, floor_tiles, occupants.to_vec()));
}
```

Borrows `world.gis.buildings` and `world.tiles` immutably — disjoint fields,
no conflict. `occupants.to_vec()` clones ~1,800 Occupant records (~100KB),
trivially fast at startup.

### Phase 2 — Spawn entities

```rust
let mut entities_spawned: u32 = 0;
let mut empty_names_skipped: u32 = 0;

for (building_id, floor_tiles, occupants) in &spawn_data {
    for occupant in occupants {
        for raw_name in occupant.name.split(',') {
            let name = raw_name.trim();
            if name.is_empty() { empty_names_skipped += 1; continue; }

            let e = world.spawn();

            // Position: random floor tile (deterministic via world.rng)
            let idx = world.rng.random_range(0..floor_tiles.len());
            let (x, y) = floor_tiles[idx];

            // Body tables
            world.body.names.insert(e, Name { value: name.to_string() });
            world.body.icons.insert(e, Icon { ch: '☻' });
            world.body.positions.insert(e, Position { x, y });
            world.body.healths.insert(e, Health { current: 100.0, max: 100.0 });
            world.body.fatigues.insert(e, Fatigue { current: 0.0 });
            world.body.combat_stats.insert(e, CombatStats {
                attack: 10.0, defense: 5.0, aggression: 0.0,
            });
            world.body.gait_profiles.insert(e, GaitProfile::biped());
            world.body.current_gaits.insert(e, Gait::Walk);
            world.body.move_cooldowns.insert(e, MoveCooldown { remaining: 0 });

            // Mind tables
            world.mind.hungers.insert(e, Hunger { current: 0.0, max: 100.0 });
            world.mind.action_states.insert(e, ActionState {
                current_action: None,
                ticks_in_action: 0,
                cooldowns: HashMap::new(),
            });
            world.mind.occupations.insert(e, Occupation {
                activity: occupant.activity.clone(),
                naics: occupant.naics.clone(),
            });

            // GIS tables
            world.gis.home_buildings.insert(e, HomeBuilding(*building_id));
            world.gis.workplaces.insert(e, Workplace(*building_id));

            // Event
            world.events.push(Event::Spawned { entity: e, tick: world.tick });
            entities_spawned += 1;
        }
    }
}
```

### Diagnostic summary

```rust
let ms = start.elapsed().as_millis();
if buildings_in_quartier == 0 {
    log::warn!("spawn_gis_entities: no buildings match quartier '{target_quartier}'");
}
log::info!(
    "GIS spawn '{target_quartier}': {buildings_in_quartier} buildings, \
     {buildings_with_occupants} with occupants ({occupant_records} records), \
     {buildings_skipped_no_floors} skipped (no floors), \
     {entities_spawned} entities spawned \
     ({empty_names_skipped} empty names skipped) [{ms}ms]"
);
```

Expected Arcis output: `825 buildings, 273 with occupants (1814 records),
~0-5 skipped (no floors), ~1500-2000 entities spawned`. The ratio of
`entities_spawned` to `occupant_records` shows how much the comma-split
expands (or empty names contract) the raw data. If
`buildings_skipped_no_floors` is high, the per-building `log::debug!`
lines identify which buildings to investigate in B05-POLISH.

### Iteration order

Buildings are iterated from `world.gis.buildings.buildings` (`Vec<BuildingData>`,
deterministic insertion order). Occupants within a building are iterated from
the Vec returned by `occupants_nearest` (deterministic order). Names within
an occupant entry are split left-to-right. All randomness (floor tile
selection) goes through `world.rng`. Result: fully deterministic with seeded
RNG.

### Unknown quartier

If `target_quartier` matches no buildings, `spawn_data` is empty and the
function logs "Spawned 0 entities." Consider a `log::warn!` if no buildings
matched at all (typo detection).

### Expected counts (Arcis)

From B05 diagnostic: 273 buildings with occupant data, 1,814 occupant records.
After comma-splitting names, expect ~1,500-2,000 entities (some entries have
multiple names, some have empty name fields). Exact count depends on data
quality.

This exceeds the Phase B header estimate of "~200 entities" — that was a
pre-data rough estimate. ~2,000 entities with HashMap-based systems at ~8
system passes per tick is ~16,000 hash lookups per tick (~1-2ms). Not a
performance concern for the single-threaded loop.

## Changes to existing code

### `active_year` default: 1839 → 1845

```rust
// world.rs — GisTables::new()
active_year: 1845,  // was 1839
```

### Entity inspector: show occupation

Add `occupation` field to `EntityInspectorInfo`:
```rust
pub occupation: Option<String>,
```

In `collect_inspector_info()`:
```rust
if let Some(occ) = world.mind.occupations.get(&entity) {
    info.occupation = Some(occ.activity.clone());
}
```

Display in `build_entity_inspector()` below the name, using the existing
label pattern.

## Downstream contracts

- **Existing systems:** All existing systems (hunger, fatigue, decisions,
  wander, eating, combat, death) will immediately drive spawned entities.
  Systems skip entities missing optional components via `if let Some`.
  `MoveCooldown { remaining: 0 }` means entities can move on tick 1.
- **Spatial index:** `rebuild_spatial_index()` runs at the start of each
  tick, scanning `body.positions`. No special registration needed.
- **C05 (procedural population):** Uses the same `HomeBuilding`/`Workplace`/
  `Occupation` components. Differentiates home from workplace for non-directory
  population.
- **SIM-004 (sleep):** Reads `HomeBuilding` to find the entity's return
  destination.
- **D01 (hydration):** Spawns entities from district data using the same
  pattern established here.

## Testing

Unit test in `loading_gis.rs`:

1. Construct a minimal `World` with `World::new_with_seed(42)`.
2. Set up a small TileMap with a few Floor tiles.
3. Create a `BuildingRegistry` with one building in quartier "TestQ",
   with occupants for year 1845.
4. Call `spawn_gis_entities(&mut world, "TestQ")`.
5. Assert: correct number of entities in `world.alive`.
6. Assert: each entity has Position on a Floor tile of the building.
7. Assert: each entity has HomeBuilding and Workplace matching the building ID.
8. Assert: each entity has Occupation with the expected activity/naics.
9. Assert: `validate_world()` passes (no zombie entries).

Integration test: `cargo run` with Paris data, verify entity count in
log output matches expected Arcis occupant count.

## Steps

**Resume protocol:** Run `git log --oneline | grep B03` to find completed
steps. Start from the first unchecked step. Mark each step `- [x]` in this
file after its commit succeeds.

- [x] **Step 1 — New components.** Add `HomeBuilding`, `Workplace`, `Occupation`
  to `components.rs`. Add HashMap fields to `GisTables` (home/workplace) and
  `MindTables` (occupation). Create `GisTables::remove()` method. Add
  `self.gis.remove(&entity)` to `World::despawn()`. Update `new()`,
  `validate_world()`. Change `active_year` default to 1845.
  `cargo build` + `cargo test`.
  Commit: `B03 S1: Add HomeBuilding, Workplace, Occupation components`

- [x] **Step 2 — Spawn function.** Implement `spawn_gis_entities()` in
  `loading_gis.rs`. Add call site in `main.rs` after `load_utility_config()`.
  Include diagnostic counters and summary log as specified in the Diagnostic
  summary section. `log::warn!` if quartier matches zero buildings.
  `cargo build` + `cargo test`.
  Commit: `B03 S2: Implement GIS entity spawning for Arcis`

- [x] **Step 3 — Inspector integration.** Add `occupation` to
  `EntityInspectorInfo`, populate in `collect_inspector_info()`, display
  in `build_entity_inspector()`. `cargo build` + `cargo test`.
  Commit: `B03 S3: Show occupation in entity inspector`

- [x] **Step 4 — Unit test.** Write the test described in Testing section.
  Verify deterministic replay (same seed → same entity positions).
  `cargo test`.
  Commit: `B03 S4: Add spawn_gis_entities unit test`

- [x] **Step 5 — Completion.** In `backlog.md`: delete the entire `SCALE-B03`
  entry (main bullet + all sub-bullets). Update `SCALE-B03-POLISH` dependency
  from `Needs: B03 (pending)` to `Needs: B03 (done)`.
  Commit: `B03: GIS-aware entity spawning complete`
