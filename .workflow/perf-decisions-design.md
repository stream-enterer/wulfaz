# Performance Fix: run_decisions Bottleneck

## Problem

`run_decisions` takes ~40ms/tick with 360 entities. Budget is 10ms for
the entire tick. At Phase C scale (4K active entities), the current
approach would cost ~444ms/tick — 44x over budget.

### Root Cause

Each entity evaluates 4 action definitions. Two considerations use
`entities_in_range(pos, 30)`, which does 61×61 = 3,721 HashMap lookups
per call. Additionally, `select_eat_target` and `select_attack_target`
repeat the same range scan for target selection.

Per tick: ~360 entities × 2–4 spatial queries × 3,721 lookups = 2.7–5.4M
HashMap lookups at ~15ns each = 40–80ms.

### Callers of spatial queries

| Function          | File           | Uses              |
|-------------------|----------------|-------------------|
| `entities_in_range` | decisions.rs | FoodNearby, EnemyNearby, select_eat_target, select_attack_target |
| `entities_at`     | combat.rs:123  | Exact tile lookup |
| `entities_at`     | eating.rs:48   | Exact tile lookup |

`entities_in_range` is called **only** by `decisions.rs`.
`entities_at` is called by combat and eating for exact tile lookups (fast,
single HashMap probe).

## Solution: Three Layers

### Layer 1 — Score Pruning + Consideration Reordering (decisions.rs only)

**The biggest single win.** Standard utility AI optimization.

#### 1a. Early rejection via max-possible-score bound

After evaluating k of n considerations with running product p, the
maximum possible final score is:

```
max_geo_mean = p^(1/n)           // remaining considerations all score 1.0
max_score = max_geo_mean * weight + inertia_bonus
```

If `max_score < best_score_so_far`, skip remaining considerations for
this action. This is exact — no approximation, no behavior change.

#### 1b. Evaluate cheap considerations before expensive ones

Reorder each action's considerations so table lookups come before spatial
queries. The config currently has:

| Action | Current order              | Reordered                  |
|--------|----------------------------|----------------------------|
| Eat    | [HungerRatio, FoodNearby]  | [HungerRatio, FoodNearby]  |
| Attack | [Aggression, EnemyNearby, HealthRatio] | [Aggression, HealthRatio, EnemyNearby] |

For Attack, moving HealthRatio before EnemyNearby means two cheap checks
can prune before the expensive spatial query fires.

This reordering is a **config change**, not a code change — either in the
KDL data or in `default_config()`. The scorer code just needs to support
early exit in the consideration loop.

#### 1c. Why this works so well for this workload

Most GIS-spawned entities (historical Parisians from a commercial
directory) have no `combat_stats`. For these entities:

- `Aggression` axis → `read_input` returns 0.0 (no combat_stats)
- Quadratic curve on 0.0 → 0.0
- Running product = 0.0 → max possible score = 0 + inertia
- If entity isn't currently Attacking (no inertia), Attack is pruned
  **before the EnemyNearby spatial query fires**

Similarly, entities with low hunger (ratio < 0.2):
- Logistic(12, 0.4) on 0.2 → ~0.08
- max_geo_mean = 0.08^(1/2) = 0.28, max_score = 0.28 × 1.2 = 0.34
- If Wander already scored >0.34, Eat is pruned before FoodNearby fires

**Estimated spatial queries after pruning**: ~10–50 (from ~720), depending
on how many entities are hungry or aggressive. That's a 15–70x reduction
in spatial work.

#### 1d. Determinism

No impact. Pruning is exact — it skips work that would produce a lower
score, never changes which action wins. Consideration order within an
action doesn't affect the final product (multiplication is commutative).
Entity iteration order unchanged (sorted by e.0).

### Layer 2 — Nearby-Entity Cache (decisions.rs only)

When an entity DOES need a spatial query (passes pruning), cache the
result. Currently, scoring and target selection are separate phases that
independently call `entities_in_range`:

```
Scoring:    FoodNearby  → entities_in_range(pos, 30) → count food
Scoring:    EnemyNearby → entities_in_range(pos, 30) → count enemies
Selection:  select_eat_target    → entities_in_range(pos, 30) → find best food
Selection:  select_attack_target → entities_in_range(pos, 30) → find best enemy
```

An entity that chooses Eat does 2–3 redundant range scans (FoodNearby
during scoring, possibly EnemyNearby during scoring, then
select_eat_target). All use the same center and range.

**Fix**: Collect the nearby entity list once per entity into a local
`Vec<Entity>`, reuse it for both scoring and target selection.

```rust
// Before the action-scoring loop for this entity:
let nearby: Option<Vec<Entity>> = None; // lazily populated

// In read_input for FoodNearby/EnemyNearby:
// populate `nearby` on first spatial axis, reuse on second

// In select_eat_target/select_attack_target:
// filter the cached `nearby` list instead of re-scanning
```

This eliminates 1–3 redundant `entities_in_range` calls per entity that
passes pruning. Combined with Layer 1, the total spatial queries drop
from ~720 to ~10–50 unique scans.

#### Determinism

No impact. Same data, same filters, same results. The cache is local to
each entity's evaluation within a single tick.

### Layer 3 — Coarse Spatial Grid (world.rs)

Replace the per-tile spatial index with a coarse grid for range queries.
This makes each individual spatial query ~100–500x cheaper, ensuring
the system scales to 4K+ entities even if many need spatial evaluation.

#### Current structure

```
HashMap<(i32, i32), SmallVec<[Entity; 4]>>
Key = exact tile coordinate
Range query: iterate (2*range+1)² tiles = 3,721 HashMap probes for range=30
```

#### Proposed structure

Add a **second** spatial index alongside the existing one:

```rust
// Existing — kept for entities_at() (combat, eating)
pub spatial_index: HashMap<(i32, i32), SmallVec<[Entity; 4]>>,

// New — coarse grid for range queries (decisions)
pub spatial_grid: HashMap<(i32, i32), Vec<Entity>>,
```

Cell size: 16 tiles (16 meters). Power of 2 for fast bit-shift division.

```rust
const SPATIAL_CELL_SHIFT: u32 = 4; // 2^4 = 16

fn cell_key(x: i32, y: i32) -> (i32, i32) {
    (x >> SPATIAL_CELL_SHIFT, y >> SPATIAL_CELL_SHIFT)
}
```

Range query with range=30 checks cells from `(cx-30)>>4` to `(cx+30)>>4`:
at most 5×5 = 25 cell lookups (vs 3,721 tile lookups). Each cell returns
a list of entities; filter by actual Chebyshev distance.

#### Why keep both indexes?

`entities_at(x, y)` is used by combat and eating for exact tile lookups.
It currently returns `&[Entity]` (a borrowed slice from SmallVec). With a
coarse grid, entities at a specific tile aren't stored contiguously — we'd
need to filter and collect, changing the return type.

Keeping both avoids API churn in combat/eating and is cheap: rebuilding
two indexes costs 2N HashMap inserts (N = entity count). At 4K entities,
that's ~8K inserts, well under 1ms.

#### `entities_in_range` rewrite

```rust
pub fn entities_in_range(
    &self,
    cx: i32,
    cy: i32,
    range: i32,
) -> impl Iterator<Item = Entity> + '_ {
    let min_cell_x = (cx - range) >> SPATIAL_CELL_SHIFT;
    let max_cell_x = (cx + range) >> SPATIAL_CELL_SHIFT;
    let min_cell_y = (cy - range) >> SPATIAL_CELL_SHIFT;
    let max_cell_y = (cy + range) >> SPATIAL_CELL_SHIFT;

    (min_cell_y..=max_cell_y).flat_map(move |cy_cell| {
        (min_cell_x..=max_cell_x).flat_map(move |cx_cell| {
            self.spatial_grid
                .get(&(cx_cell, cy_cell))
                .into_iter()
                .flat_map(|v| v.iter().copied())
                .filter(move |&e| {
                    if let Some(pos) = self.body.positions.get(&e) {
                        (pos.x - cx).abs().max((pos.y - cy).abs()) <= range
                    } else {
                        false
                    }
                })
        })
    })
}
```

25 HashMap probes + N distance checks (where N is entities in the 5×5
cell neighborhood) vs 3,721 HashMap probes.

#### Performance estimates

With 360 entities in ~300×200 tile area:
- ~247 cells, ~1.5 entities/cell
- 25 cells × 1.5 entities = ~38 distance checks per query
- 25 HashMap probes + 38 integer comparisons ≈ 0.5μs per query
- vs current: 3,721 probes ≈ 56μs per query → **112x speedup per query**

With 4K entities in ~600×400 tile area:
- ~950 cells, ~4.2 entities/cell
- 25 cells × 4.2 entities = ~105 distance checks per query
- ≈ 1μs per query

#### Determinism

No impact. The coarse grid returns the same set of entities as the
fine-grained scan (the distance filter is identical). Iteration order
within a cell is insertion order (from `rebuild_spatial_index` which
iterates `body.positions` — HashMap, non-deterministic). But callers
already sort or use min_by with entity-ID tiebreakers, so the output
is deterministic regardless of iteration order within the grid.

#### CLAUDE.md constraint check

> Do not replace HashMap with another data structure without profiling
> data showing >5ms per tick for that system.

Profiling shows ~40ms/tick for decisions. The spatial index is the
bottleneck within that system. The coarse grid is still a HashMap — same
data structure, different key granularity. This satisfies the constraint.

## Combined Performance Estimates

### 360 entities (current)

| Component | Before | After L1 | After L1+L2 | After L1+L2+L3 |
|-----------|--------|----------|-------------|----------------|
| Spatial queries | ~720 | ~30 | ~15 | ~15 |
| Cost per query | 56μs | 56μs | 56μs | 0.5μs |
| Total spatial | ~40ms | ~1.7ms | ~0.8ms | ~0.008ms |
| Scoring overhead | ~0.2ms | ~0.3ms | ~0.3ms | ~0.3ms |
| **Total decisions** | **~40ms** | **~2ms** | **~1.1ms** | **~0.3ms** |

### 4K entities (Phase C target)

| Component | Before | After L1 | After L1+L2 | After L1+L2+L3 |
|-----------|--------|----------|-------------|----------------|
| Spatial queries | ~8,000 | ~400 | ~200 | ~200 |
| Cost per query | 56μs | 56μs | 56μs | 1μs |
| Total spatial | ~448ms | ~22ms | ~11ms | ~0.2ms |
| Scoring overhead | ~2ms | ~3ms | ~3ms | ~3ms |
| **Total decisions** | **~450ms** | **~25ms** | **~14ms** | **~3.2ms** |

Layer 1 alone fixes the immediate problem (360 → ~2ms).
Layers 1+2+3 together scale to 4K within the 10ms total tick budget.

## Implementation Plan

### Step 1: Score pruning in decisions.rs (Layer 1)

Modify the consideration evaluation loop to track running product and
compute max-possible-score bound. Break early if bound < best_score.

Changes: `decisions.rs` only. ~15 lines of logic in the scoring loop.

No config file changes needed — the reordering (1b) is a separate
optional step. The pruning (1a) works regardless of consideration order;
reordering just makes pruning trigger earlier.

### Step 2: Nearby-entity cache in decisions.rs (Layer 2)

Restructure `run_decisions` so that `read_input` for spatial axes and
`select_*_target` share a cached nearby-entity list per entity.

Changes: `decisions.rs` only. Refactor to thread a cache through
`read_input` calls (or compute nearby list before scoring and pass it
through). ~30 lines.

### Step 3: Coarse spatial grid in world.rs (Layer 3)

Add `spatial_grid` field. Update `rebuild_spatial_index` to populate
both indexes. Rewrite `entities_in_range` to use the coarse grid.

Changes:
- `world.rs`: add field, update rebuild, rewrite `entities_in_range` (~30 lines)
- `world.rs` tests: update/add tests for coarse grid behavior
- No changes to `decisions.rs`, `combat.rs`, or `eating.rs`

### Step 4: Verify

- All existing unit tests pass (decisions, combat, eating, world)
- Property tests pass (deterministic replay, no zombie entities)
- `cargo build --release` + manual profiling confirms <2ms for decisions

## Files Changed

| File | Layer | Change |
|------|-------|--------|
| `src/systems/decisions.rs` | 1, 2 | Score pruning, nearby cache |
| `src/world.rs` | 3 | Add coarse grid, rewrite `entities_in_range` |

No changes to: `main.rs`, `combat.rs`, `eating.rs`, `components.rs`,
`CLAUDE.md`, any KDL data files.

## Risks

**Low risk.** All three layers are exact optimizations — they produce
identical results to the current code, just faster. No behavior changes,
no new state, no new dependencies.

The nearby-entity cache (Layer 2) adds a temporary `Vec<Entity>` per
entity per tick. At 4K entities with ~100 nearby each, that's ~400K
Entity values (~3.2MB) of transient allocation. Acceptable for
single-threaded, frame-scoped data.

## What About Staggered Evaluation?

Evaluating only a fraction of entities per tick (e.g., 1/5 per tick,
round-robin) was considered but is **not needed** if Layers 1–3 achieve
the target. Staggering adds latency (entities react 5 ticks late = 50ms
at 100 ticks/sec) and complexity (tracking which entities to evaluate
when, maintaining stale-but-valid intentions). Hold in reserve for
Phase D if 4K entities with Layers 1–3 still exceeds budget.

## What About Reducing SENSE_RANGE?

Reducing from 30 to 10 tiles would cut per-query cost by ~8x (441 vs
3,721 lookups). But this is a gameplay change — entities would only
detect food/enemies 10m away instead of 30m. Not recommended as a
performance fix when algorithmic improvements (pruning + coarse grid)
solve the problem without changing behavior. If gameplay design later
wants shorter sense ranges, that's a separate decision.
