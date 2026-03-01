# Survival System Migration Spec

> Consolidated analysis of what is pinned down, what needs decisions, and what
> is known-but-accepted for integrating SURVIVAL_SPEC_B0_B1 into Wulfaz.

---

## 1. PINNED DOWN (Ready to Implement)

### 1.1 Game-Time Model

**1 engine tick = 0.01 game-minutes = 0.6 game-seconds.**

```
const GAME_MINUTES_PER_TICK: f64 = 1.0 / 100.0;
```

Derivation: `1.0 / SIM_TICKS_PER_SEC`. At speed 1, 100 ticks/sec yields
1 game-minute per real second. This is pinned by three constraints:

- Speed 1-2 (ant farm) needs DF-like visual pacing for entity observation.
- Survival tick (5 game-minutes) fires every 500 engine ticks = 5 real
  seconds at speed 1, which is frequent enough for smooth simulation but
  infrequent enough to be cheap.
- 1 real-sec = 1 game-min at speed 1 is trivially debuggable.

### 1.1b Speed Settings (Non-Linear, LOD-Coupled)

Speeds 1-2 are entity-level (ant farm / roguelike). Speeds 3-5 force all
zones to Statistical and run district aggregate equations instead of
per-entity simulation. This is what makes grand strategy timescales
possible — the cost per tick drops from O(4K entities) to O(36 districts).

| Speed | Mult | LOD Mode | Sim cost/tick | Day (real) | Year (real) | Mode |
|-------|------|----------|--------------|-----------|-------------|------|
| 1 | 1× | Camera zones | O(entities) | 24 min | — | Ant farm / combat |
| 2 | 3× | Camera zones | O(entities) | 8 min | — | Daily rhythm |
| 3 | 30× | All Statistical | O(districts) | 48 sec | ~5 hrs | Seasonal |
| 4 | 120× | All Statistical | O(districts) | 12 sec | ~1.2 hrs | Grand strategy |
| 5 | max | All Statistical | O(districts) | AFAP | AFAP | Fast-forward |

Speeds 3-5 don't render per-entity movement. The visual is time-lapse:
the calendar advances, district stats update, map overlays shift. Like
Paradox games at high speed — no army animation, just day jumps.

**Implementation:** `sim_speed` is no longer a simple dt multiplier. It
maps to a `(time_mult, lod_override)` pair:

```rust
struct SpeedConfig {
    time_mult: f64,           // dt multiplier
    force_statistical: bool,  // true = all zones Statistical
}

const SPEED_CONFIGS: [SpeedConfig; 5] = [
    SpeedConfig { time_mult: 1.0,   force_statistical: false },  // Speed 1
    SpeedConfig { time_mult: 3.0,   force_statistical: false },  // Speed 2
    SpeedConfig { time_mult: 30.0,  force_statistical: true },   // Speed 3
    SpeedConfig { time_mult: 120.0, force_statistical: true },   // Speed 4
    SpeedConfig { time_mult: 0.0,   force_statistical: true },   // Speed 5 (uncapped)
];
```

Speed 5 (`time_mult: 0.0`) means "run as many ticks as the CPU allows
per frame" — the accumulator is bypassed and ticks run until a frame
time budget is exhausted.

**Transition on speed change:** Speeding up (1→3) triggers dehydration
(D02): collapse active entities to district averages. Slowing down (3→1)
triggers hydration (D01): spawn entities from distributions. Same
mechanisms already planned for camera-driven zone transitions.

**MAX_TICKS_PER_FRAME scaling:** At speeds 3-5 with statistical-only
simulation, each tick costs microseconds (36 districts × arithmetic).
`MAX_TICKS_PER_FRAME` can be raised to 200+ without risk. At speeds 1-2
with entity simulation, keep the current cap of 5.

```rust
fn max_ticks_for_speed(speed: &SpeedConfig) -> u32 {
    if speed.force_statistical { 500 } else { 5 }
}
```

### 1.2 Survival Clock

Survival fires on a global modulo of `world.tick`, not a per-entity counter.
This matches CDDA's architecture (modulo on the turn counter).

```
const SURVIVAL_TICK_INTERVAL: u64 = 500;   // 5 game-minutes
const SLOW_CYCLE_INTERVAL: u64 = 3000;     // 30 game-minutes

fn is_survival_tick(tick: Tick) -> bool {
    tick.0 % SURVIVAL_TICK_INTERVAL == 0
}

fn is_slow_cycle(tick: Tick) -> bool {
    tick.0 % SLOW_CYCLE_INTERVAL == 0
}
```

**Tick monotonicity is guaranteed.** `world.tick` increments by exactly 1 in
`run_one_tick` (main.rs:273). No mechanism exists to skip tick numbers. The
modulo check is safe.

### 1.3 Survival Is Wall-Clock, Not Action-Clock

Per CDDA's design: survival rate is independent of movement speed. A fast
character does not starve faster. `ActivityLevel` (idle/active/etc.) modifies
BMR, not speed. The `tick.0 % 500` check ensures this — it fires at fixed
game-time intervals regardless of what actions occurred during those ticks.

In roguelike mode, player actions advance `1 + cooldown` engine ticks per
move. Each tick = 0.01 game-minutes. Walk (10 ticks/move) and Sprint
(4 ticks/move) both accumulate game-time at the same rate per tick. The
sprinter covers more ground in the same game-time. Survival fires identically
for both.

### 1.4 Spec Constants: What Stays, What Scales

**Capacities (unchanged):**

| Constant             | Value     | Reason                    |
|----------------------|-----------|---------------------------|
| STOMACH_CAPACITY     | 2500 mL   | Physical volume           |
| GUT_CAPACITY         | 24000 mL  | Physical volume           |
| THIRST_MIN/MAX       | -100/1200 | Unitless bounds           |
| STIM_OVERDOSE/LETHAL | 250/-200  | Unitless bounds           |
| KCAL_PER_KG          | 7716      | Physics constant          |
| Weight category ratios | per spec | Unitless thresholds      |

**Per-survival-tick rates (unchanged):** The survival tick always represents
exactly 5 game-minutes. Since we fire it at the correct game-time interval,
spec rates are used as-is:

| Constant           | Spec value       | Unit               |
|--------------------|------------------|--------------------|
| STOMACH_WATER_RATE | 250              | mL / survival tick |
| GUT_WATER_RATE     | 250              | mL / survival tick |
| THIRST_BASE_RATE   | 1.0              | per survival tick  |
| STIM_DECAY_RATE    | 1                | per survival tick  |

**Per-slow-cycle rates (unchanged):** The slow cycle fires every 30
game-minutes (every 3000 engine ticks, every 6th survival tick):

| Constant           | Spec value | Unit                |
|--------------------|------------|---------------------|
| STOMACH_CAL_RATE   | 0.167      | fraction / 30 min   |
| STOMACH_CAL_FLOOR  | 5          | kcal / 30 min       |
| STOMACH_VIT_FLOOR  | 1          | units / 30 min      |
| STOMACH_VIT_RATE   | 0.167      | fraction / 30 min   |

**BMR burn per survival tick:**
```
burn_kcal = base_bmr * activity_mult / TICKS_PER_DAY
```
where `TICKS_PER_DAY = 288` (survival ticks per game-day). This is survival
ticks, not engine ticks. Unchanged from spec.

### 1.5 Existing Systems Are Per-Engine-Tick, Not Per-Game-Minute

Fatigue and temperature rates are tuned for 100Hz engine ticks. Their
constants have comments confirming this:

- Fatigue: `RECOVERY_RATE = 0.2` → "20/sec at 100 tps" (fatigue.rs)
- Temperature: `0.1°C/tick` → "10°C/sec drift rate" (temperature.rs)

These are gameplay-feel rates measured in real-time, not calendar time. The
game-time multiplier does not touch them. They run every engine tick as before.
Survival systems run only on `tick.0 % 500 == 0`.

**At speeds 3-5 (all Statistical), none of these per-entity systems run.**
Fatigue, temperature, movement, combat — all skip because no entities are
hydrated. District aggregate equations model their effects statistically.

### 1.6 Phase Placement

New Phase 2 ordering:

```
Phase 2 (Needs):
    run_hunger()       →  DELETE (replaced by survival)
    run_survival()     →  NEW (fires only on survival tick boundary)
    run_fatigue()      →  KEEP (per-engine-tick, unchanged)
```

`run_survival()` is the single entry point. Internally:

```
fn run_survival(world: &mut World, tick: Tick) {
    if !is_survival_tick(tick) { return; }

    let is_slow = is_slow_cycle(tick);

    // Sort entities for determinism
    let mut entities: Vec<Entity> = world.body.stomachs.keys().copied().collect();
    entities.sort_by_key(|e| e.0);

    for entity in entities {
        if world.pending_deaths.contains(&entity) { continue; }

        // 1. Digestion: stomach → guts → body
        run_digestion(world, entity, is_slow);

        // 2. Thirst: passive gain + death check
        run_thirst(world, entity);

        // 3. Vitamins: passive decay, effects
        run_vitamins(world, entity);

        // 4. Stimulants: decay, lethal checks
        run_stimulants(world, entity);
    }
}
```

This is a single system, one file (`src/systems/survival.rs`), one driving
table (`stomachs`). Sub-functions are private helpers within the file, not
separate systems. This avoids 4× entity sorting overhead.

**LOD interaction:** `run_survival()` only runs for entities in Active and
Nearby zones (per C03: zone-aware system filtering). At speeds 3-5, all
zones are forced Statistical — `run_survival()` early-returns because no
entities have stomachs (they've been dehydrated to district aggregates).
District-level survival is handled by `run_district_stats()` (C04), which
models food consumption / hunger / death as aggregate equations per
district, not per entity.

### 1.7 Data Structures to Add

**components.rs:**

```rust
// -- Food item data (static, on food entities) --
enum ItemType { Food, Drink, Med }

struct FoodData {
    calories: i32,
    volume_ml: i32,
    water_ml: i32,
    vitamins: HashMap<VitaminId, i32>,
    fun: i32,            // future: Morale bucket
    stim: i32,           // Bucket 1
    healthy: i32,        // future: Lifestyle bucket
    spoils_in: u64,      // ticks until spoilage, 0 = never
    item_type: ItemType,
}

// -- Digestion compartments (per-entity, mutable) --
struct NutrientPacket {
    solids_ml: f32,
    water_ml: f32,
    calories: f32,
    vitamins: HashMap<VitaminId, f32>,
}

struct Stomach {
    contents: Vec<NutrientPacket>,
    capacity_ml: i32,
    last_meal_tick: u64,  // engine tick of last ingest()
}

struct Guts {
    contents: Vec<NutrientPacket>,
    capacity_ml: i32,
}

// -- Body composition (per-entity, mutable) --
struct BodyComposition {
    stored_kcal: f32,
    healthy_kcal: f32,
    height_cm: i32,
    base_bmr: i32,       // kcal/day
}

// -- Activity level (per-entity, set by movement/combat/idle systems) --
enum ActivityLevel {
    Sleep, Idle, Light, Moderate, Brisk, Active, Extreme,
}

// -- Thirst (per-entity) --
struct Thirst {
    value: i32,
}

// -- Vitamins (per-entity) --
struct VitaminLevels {
    levels: HashMap<VitaminId, i32>,
}

// -- Stimulants (per-entity) --
struct StimState {
    stim: i32,
}

// -- Vitamin definitions (global, loaded from KDL) --
enum VitaminType { Vitamin, Toxin, Drug, Counter }

struct VitaminDef {
    id: VitaminId,
    vit_type: VitaminType,
    min: i32,
    max: i32,
    rate_ticks: u64,     // passive change: 1 unit per this many survival ticks
    rate_direction: i32, // -1 or +1
    // Effects: see "Needs Decision" section
}

// -- Derived display enums (not stored, computed on read) --
enum WeightCategory {
    Emaciated, Underweight, Normal, Overweight,
    Obese, VeryObese, MorbidlyObese,
}

enum HungerState {
    Engorged, Full, Satisfied, None,
    Hungry, VeryHungry, Famished, NearStarving, Starving,
}
```

### 1.8 World Structure Changes

**BodyTables — add:**
- `stomachs: HashMap<Entity, Stomach>`
- `guts: HashMap<Entity, Guts>`
- `body_compositions: HashMap<Entity, BodyComposition>`
- `food_data: HashMap<Entity, FoodData>` (on food item entities)

**MindTables — add:**
- `activity_levels: HashMap<Entity, ActivityLevel>`
- `thirsts: HashMap<Entity, Thirst>`
- `vitamin_levels: HashMap<Entity, VitaminLevels>`
- `stim_states: HashMap<Entity, StimState>`

**MindTables — remove:**
- `hungers: HashMap<Entity, Hunger>` (replaced by derived HungerState)
- `nutritions: HashMap<Entity, Nutrition>` (replaced by FoodData)

**World (infrastructure) — add:**
- `vitamin_registry: HashMap<VitaminId, VitaminDef>` (loaded from KDL)

Each new table requires the 5-step property table checklist:
1. Struct in components.rs
2. HashMap field in sub-struct
3. `.remove(&entity)` in sub-struct's `remove()` method
4. Alive-check in `validate_world()`
5. `HashMap::new()` in sub-struct's `new()`

### 1.9 GameDate::from_tick Update

```rust
impl GameDate {
    pub fn from_tick(tick: Tick, start: &StartDate) -> Self {
        let total_minutes = (tick.0 as f64 * GAME_MINUTES_PER_TICK) as u64;
        // ... rest unchanged (calendar math on total_minutes)
    }
}
```

All 7 existing GameDate tests need tick values multiplied by 100. Mechanical
change, atomic with the constant addition.

### 1.10 Eating System Rewrite

Current `run_eating`: food at same tile → instant hunger reduction → food
despawned.

New `run_eating`: food at same tile → call `ingest()` → build NutrientPacket
from FoodData → push into Stomach → vomit check → apply stim → record
`last_meal_tick` → food entity marked `pending_deaths`.

The food entity still dies on consumption. Nutrients enter the stomach pipeline
instead of instantly satisfying hunger.

### 1.11 Decision System Adaptation

`InputAxis::HungerRatio` currently reads `hunger.current / hunger.max`. Change
to read `body_composition.stored_kcal / body_composition.healthy_kcal` (the
calorie ratio). Scoring curves may need retuning but the architecture holds.
`select_eat_target()` changes from checking `nutritions` table to checking
`food_data` table.

### 1.12 Events

| Event             | Action                         |
|-------------------|--------------------------------|
| `HungerChanged`   | Remove (hunger is now derived) |
| `Ate`             | Keep (entity, food, tick)      |
| New: `Vomited`    | Entity vomited                 |
| New: `Dehydrated` | Thirst death                   |
| New: `Overdosed`  | Stim lethal threshold          |

### 1.13 Data Files

**New `data/foods.kdl`:** All FoodData definitions. Required fields (hard
error at load): `calories`, `volume_ml`, `water_ml`, `item_type`. Optional
future-bucket fields (`fun`, `healthy`) default to 0.

**New `data/vitamins.kdl`:** All VitaminDef definitions. Required fields:
`id`, `type`, `min`, `max`, `rate`, `direction`.

**Modified `data/archetypes.kdl`:** Replace `max_hunger` with `healthy_kcal`,
`height_cm`, `base_bmr`, `stomach_capacity`, `gut_capacity`.

### 1.14 Atomic Migration

The time model constant, GameDate change, survival system, eating rewrite, and
hunger/nutrition removal must land atomically. There is no valid intermediate
state where the old hunger system runs under the new time model (rates would
be 100x too fast). Ship as one changeset.

### 1.15 Corrected Digestion Formulas (from CDDA Source)

The original spec's guts-to-body formula was wrong. Verified against CDDA
`stomach.cpp:419-472`. The real formulas:

#### Stomach → Guts (per 30-min slow cycle)

Same as original spec. No correction needed:
```
solids_budget = stomach.capacity_ml / 6
cal_budget    = max(STOMACH_CAL_FLOOR, stomach_cal * STOMACH_CAL_RATE)
                // STOMACH_CAL_FLOOR = 5 kcal, STOMACH_CAL_RATE = 1/6
vit_budget    = max(STOMACH_VIT_FLOOR, stomach_vit * STOMACH_VIT_RATE)
                // STOMACH_VIT_FLOOR = 1 unit, STOMACH_VIT_RATE = 1/6
```

Stomach is the fast valve: big fraction (1/6), tiny floor.

#### Guts → Body (per 30-min slow cycle)

**Corrected formula — clamp(floor, percentage, ceiling):**
```
GUT_CAL_RATE    = 0.05    // 5% of gut contents per cycle
GUT_CAL_FLOOR   = base_bmr / 24.0  // ~104 kcal at BMR 2500

desired  = gut_calories * GUT_CAL_RATE * hunger_mod
floor    = GUT_CAL_FLOOR * hunger_mod
ceiling  = gut_calories  // never absorb more than exists

absorbed = clamp(floor, desired, ceiling)
```

In plain language: absorb whichever is larger — 5% of gut contents or
BMR/24 — but never more than what's there. Guts are the slow steady
absorber: small fraction (5%), big floor (BMR-scaled).

**Concrete numbers at BMR 2500, hunger_mod 1.0:**

| Gut contents | 5% desired | Floor (BMR/24) | Absorbed    |
|-------------|-----------|----------------|-------------|
| 500 kcal    | 25 kcal   | 104 kcal       | 104 (floor) |
| 2000 kcal   | 100 kcal  | 104 kcal       | 104 (floor) |
| 3000 kcal   | 150 kcal  | 104 kcal       | 150 (5%)    |
| 5000 kcal   | 250 kcal  | 104 kcal       | 250 (5%)    |

The floor dominates for gut contents < ~2083 kcal (= BMR/24 / 0.05).
Above that, 5% scales up. Either way, guts empty via exponential decay,
never instantly.

#### Hunger Modifier — REVERSED from Original Spec

**The spec got this backwards.** The real `metabolic_rate()` from
`consumption.cpp:785-799`:

```
// effective_hunger combines hunger + starvation, scaled by speed
// Linearly interpolated between thresholds:
//   well-fed  (300)  → 1.0
//   moderate  (2000) → 0.8
//   severe    (5000) → 0.6
//   critical  (8000) → 0.5

fn hunger_mod(body: &BodyComposition) -> f32 {
    let ratio = body.calorie_ratio();
    if ratio >= 1.0 { return 1.0; }       // well-fed: full speed
    // Starving: SLOWER absorption (down to 0.5x)
    // Lerp from 1.0 at ratio=1.0 to 0.5 at ratio=0.0
    0.5 + ratio * 0.5
}
```

**Gameplay consequence:** This creates a starvation death spiral. Starving
→ slower absorption → fewer calories reach body → more starving. The
original spec claimed deficit → faster absorption, which would be a
stabilizing negative feedback loop. The real system is destabilizing.

**Design note:** We may want to choose which behavior we prefer for our
game rather than blindly copying CDDA. The death spiral makes survival
more punishing. A stabilizing loop makes it more forgiving. This is a
gameplay feel decision, not a technical one. For now, implement the CDDA
version (death spiral) and tune later.

#### Updated Tuning Constants

Add to the tuning table:

| Name            | Default | Unit                | Notes                          |
|-----------------|---------|---------------------|--------------------------------|
| `GUT_CAL_RATE`  | 0.05    | fraction / 30 min   | 5% of gut contents per cycle   |
| `GUT_CAL_FLOOR` | derived | kcal / 30 min       | `base_bmr / 24.0` (~104 kcal)  |

Remove from tuning table: the original spec's `KCAL_DAY_TO_TICK` (does
not exist; was a spec error).

---

## 2. NEEDS DECISION

### 2.1 ~~MAX_TICKS_PER_FRAME vs Speed Settings~~ — RESOLVED

Resolved by LOD-coupled speed model (section 1.1b). Speeds 3-5 force
Statistical mode, making ticks cheap enough to raise
`MAX_TICKS_PER_FRAME` to 500 without frame hitches. Speeds 1-2 keep
the current cap of 5.

### 2.2 SurvivalTick Newtype

**Problem:** Engine ticks and survival ticks are both `u64`. A developer
computing `base_bmr / world.tick.0` instead of `base_bmr / TICKS_PER_DAY`
gets a silent 500x error. The existing `Tick` newtype covers engine ticks
only.

**Options:**

A. **Add `SurvivalTick(u64)` newtype.** The survival system receives it;
   engine systems never see it. Compile-time safety. Small cost: one more
   newtype, conversion at the boundary.

B. **No newtype, rely on naming convention.** All survival-tick variables
   named `stick` or `survival_tick_counter`. Relies on discipline.

C. **Don't track survival ticks at all.** The survival tick counter is only
   used for `% 6` (slow cycle). Replace with `is_slow_cycle(engine_tick)`,
   which checks `engine_tick % 3000 == 0` directly. No survival tick counter
   needed. The `TICKS_PER_DAY = 288` constant is used only in the BMR formula
   and can be documented as "survival ticks per day" without needing a type.

**Recommendation:** Option C. The survival tick counter adds no value when
the engine tick modulo handles both clocks. `TICKS_PER_DAY = 288` is a
constant in the BMR formula, not a runtime value.

### 2.3 Survival Tick Spike

**Problem:** `tick.0 % 500 == 0` fires for all entities simultaneously. With
~4K active entities, this creates a periodic CPU spike every 500 ticks.

**Context:** This only matters at speeds 1-2 (entity-level simulation).
At speeds 3-5, all zones are Statistical and per-entity survival doesn't
run — district aggregate equations handle it instead.

**Options:**

A. **Accept and budget for it.** Survival per entity is Phase 2 work: a few
   HashMap lookups, arithmetic, no spatial queries. At ~1-2μs per entity, 4K
   entities = 4-8ms. Under the 10ms tick budget, but tight. Profile first.

B. **Stagger by entity ID.** `(tick.0 + entity.0 % 500) % 500 == 0`. Spreads
   survival across all 500 ticks. But: breaks deterministic replay if entity
   IDs aren't stable across runs. Also: some entities digest while others
   don't on the same tick, which may produce subtle cross-entity inconsistency
   (e.g., food item despawned but consumer hasn't digested yet).

C. **Amortize across frames.** Process N entities per tick instead of all at
   once. Requires tracking "which entities have been processed this survival
   cycle." Adds state and complexity.

**Recommendation:** Option A. Profile after implementation. The survival
system is O(n) with no spatial queries, no pathfinding, no inner loops.
If the spike exceeds budget, revisit with profiling data.

### 2.4 ActivityLevel Writer

The spec says `ActivityLevel` is "set by gameplay systems (movement, combat,
crafting, idling). Bucket 0 reads it, does not write it."

**Currently no system writes ActivityLevel.** Needs a decision:

A. **Derive from ActionId.** Map Idle→Idle, Wander→Light, Eat→Idle,
   Attack→Active. Set in Phase 3 (decisions) or Phase 4 (actions).

B. **Derive from movement state.** Moving = Light, stationary = Idle,
   in combat = Active. Set after Phase 4.

C. **Stub at Idle for B0/B1.** Defer until the Weariness bucket (B2), which
   is the system that actually needs fine-grained activity tracking.

**Recommendation:** Option A as a minimal implementation. The mapping is
trivial and gives survival a calorie-burn signal that varies with behavior.
Write it in `run_decisions` alongside intention.

### 2.5 Vitamin Effect System

The spec references `EffectID`, `apply_effect()`, `deficiency_effect`,
`excess_effect`, and `DiseaseTier`. None of these exist in the engine.

**Options:**

A. **Implement a minimal Effect system.** Effects are stat modifiers (speed
   penalty, stat penalty, HP drain) stored as an enum. Applied/removed based
   on vitamin thresholds. Requires new component, new system.

B. **Stub effects as no-ops for B1.** Load VitaminDefs, track levels, skip
   effect application. Vitamins accumulate and deplete but have no gameplay
   impact until the Effect system is built.

C. **Hardcode effects per vitamin.** No generic Effect system. Each vitamin's
   threshold check directly applies its specific penalty (e.g., iron < -12000
   → speed -= 10). Simple but doesn't scale.

**Recommendation:** Option B for initial implementation. Vitamin tracking is
useful even without effects (it validates the digestion pipeline). The Effect
system is a future bucket concern.

### 2.6 ~~Food Spawning Model~~ — DECIDED: Stub

Keep random scatter. Replace `Nutrition` with `FoodData` on spawned food
entities. `select_eat_target()` checks `food_data` table instead of
`nutritions`. Food source model (buildings, markets, inventory) is a
separate future design question.

### 2.7 `last_meal_tick` Time Comparisons

The spec's hunger display uses:

```
fn just_ate(stomach) → bool:
    return (current_time - stomach.last_meal_time) < 15 minutes

fn recently_ate(stomach, window) → bool:
    return (current_time - stomach.last_meal_time) < window
```

"15 minutes" and "3 hours" need to be expressed in engine ticks:

```
15 game-minutes = 15 / 0.01 = 1,500 engine ticks
3 game-hours = 180 / 0.01 = 18,000 engine ticks
```

**Decision:** Store `last_meal_tick` as raw `Tick` (engine tick). Compare
using engine tick arithmetic. Define named constants:

```rust
const JUST_ATE_WINDOW: u64 = 1_500;     // 15 game-minutes
const RECENTLY_ATE_WINDOW: u64 = 18_000; // 3 game-hours
```

This is straightforward but must be documented clearly so nobody interprets
these as game-minutes.

---

## 3. AMBIGUOUS / UNDERSPECIFIED

### 3.1 ~~Guts-to-Body Calorie Absorption Formula~~ — RESOLVED

**Moved to section 1.15.** The original spec formula was wrong. The real
CDDA formula (from `stomach.cpp:419-472`) is a clamp(floor, 5%, ceiling)
pattern, not a BMR multiplier on gut contents. See section 1.15 for the
corrected formula and hunger_mod reversal.

### 3.2 NutrientPacket Proportional Draining

The spec says transfer helpers "drain from source packets proportionally
across all packets." This is underspecified:

- Proportional to what? Volume? Calories? Equal share?
- If a packet's solids_ml reaches 0 but it still has calories, what happens?
- When do empty packets get removed from the Vec?

Best guess: proportional to each packet's share of the total being drained
(e.g., if draining 100 mL solids and packet A has 60% of total solids, drain
60 mL from A). Empty packets (solids + water + calories all ≈ 0) are pruned.

### 3.3 Vomit RNG Semantics

```
roll = random(stomach.capacity_ml / 2, new_volume)
if roll > stomach.capacity_ml: vomit
```

"random" — uniform integer? Inclusive/exclusive bounds? In Wulfaz, all
randomness goes through `world.rng` (seeded StdRng). The range semantics
need to be explicit for deterministic replay.

Probable intent: `world.rng.gen_range(cap/2..=new_volume)`. If the result
exceeds `cap`, vomit.

### 3.4 Vitamin `rate` Field Semantics

The spec says `rate: Duration` — "one unit lost/gained per this interval."
Example: iron has `rate: 15 min/unit`.

In the tick function:
```
ticks_per_unit = def.rate / 5 minutes
if tick_counter % ticks_per_unit == 0:
    self.levels[id] += def.rate_direction
```

At 15 min/unit with 5 min/tick: `ticks_per_unit = 3`. Every 3rd survival
tick, the level changes by 1. But the spec uses a `tick_counter` variable
that is ambiguous — is this the survival tick counter (incremented per
`survival_tick()` call) or the engine tick? Since this runs inside
`survival_tick()`, it should be a survival-tick counter, but the spec doesn't
explicitly manage one.

**Resolution approach:** Use `(tick.0 / SURVIVAL_TICK_INTERVAL) %
ticks_per_unit == 0`. This derives a survival-tick-count from the engine tick
without a separate counter.

### 3.5 Ethanol Rate = "1 sec/unit"

The spec says ethanol decays at "1 sec/unit." Since the survival tick is
5 game-minutes (300 game-seconds), that's 300 units per survival tick. This
seems intentionally fast (getting drunk wears off quickly). But it means
the modulo approach (`ticks_per_unit = 1 sec / 5 min = 0.003`) produces a
value < 1, which breaks integer modulo.

**Resolution:** For sub-tick rates (where decay is faster than 1 unit per
survival tick), compute `units_per_tick = tick_duration / rate_duration` and
apply that many units per tick. Ethanol: `300 sec / 1 sec = 300 units per
survival tick`.

---

## 4. STUBS (Known Placeholders)

### 4.1 Multiplier Stubs ([MULT])

Several rates accept a `rate_multiplier` (float, default 1.0). These are
connection points for future buckets:

| Stub                | Default | Future source        |
|---------------------|---------|----------------------|
| `rate_mult`         | 1.0     | Sleep (0.5x), traits |
| `is_sleeping`       | false   | Sleep bucket (B2)    |
| `food.fun`          | ignored | Morale bucket (B3)   |
| `food.healthy`      | ignored | Lifestyle bucket (B3)|

Implement as literal `1.0` / `false` constants. No configuration, no
component. When the source bucket is built, replace the literal with a
table lookup.

### 4.2 Hunger State Speed Effects

The spec lists speed penalties for hunger states (Engorged: -10, Full: -2,
Famished: -5, etc.). The engine has no generic speed modifier system — gait
is selected per-entity from `GaitProfile`, not modified by arithmetic.

**Stub:** Compute `HungerState` for display. Ignore speed/stat effects until
a modifier system exists. This is the same situation as vitamin effects
(section 2.5).

### 4.3 Thirst Speed Effects

Same as hunger: thirst > 40 imposes a speed penalty. No modifier system to
apply it to. Stub as display-only.

### 4.4 Pain / Morale on Vomit

The spec says vomiting applies "pain, morale penalty (future buckets)."
Neither system exists. Stub vomiting as: clear stomach contents, emit
`Vomited` event, apply -1 thirst. Ignore pain/morale.

---

## 5. KNOWN-BUT-ACCEPTED ISSUES

### 5.1 Movement Speed Compression

Walk gait = 9 ticks/tile. At 0.01 game-minutes/tick: 5.4 game-seconds per
meter = 0.19 m/s. Real walking = 1.4 m/s. The game-time speed is ~7.5x
slower than reality.

**Accepted because:** Every simulation game (DF, Rimworld, Kenshi) has this
mismatch. Gait cooldowns are tuned for 100Hz visual feel, not physical
realism. Players perceive movement visually, not mathematically. Game-time is
a calendar convention, not a physics clock. CDDA can afford 1:1 walk speed
because it has no real-time rendering.

### 5.2 Tick 0 Fires Survival

`Tick(0) % 500 == 0` is true. Every entity gets a survival tick at spawn. The
digestion pipeline is designed to handle empty stomachs gracefully (no
contents = nothing transfers). The BMR burn will deduct a tiny amount of
`stored_kcal` on the first tick. Negligible and not worth special-casing.

### 5.3 Deterministic Vec Ordering in NutrientPackets

`Stomach.contents` and `Guts.contents` are `Vec<NutrientPacket>`. Vec
ordering is insertion-order, which is deterministic as long as `ingest()`
always pushes to the end. Proportional draining preserves Vec order (drain
in-place, don't shuffle). This is deterministic without additional sorting.

### 5.4 Entity Table Bloat

The survival system adds 7-8 new HashMap tables per entity (Stomach, Guts,
BodyComposition, ActivityLevel, Thirst, VitaminLevels, StimState, plus
FoodData on food entities). At ~4K active entities, this is ~32K HashMap
entries added to the world. `despawn()` does 7-8 additional `.remove()` calls
per entity death.

**Accepted because:** HashMap remove is O(1) amortized. The total despawn
cost grows linearly with table count but remains microsecond-scale. The
property table checklist ensures no table is missed.

### 5.5 NutrientPacket Vitamin HashMap Allocation

Each `NutrientPacket` contains a `HashMap<VitaminId, f32>`. With 5 vitamin
types and potentially many packets per stomach, this is many small
allocations.

**Accepted for now.** If profiling shows allocation pressure, replace with a
fixed-size array (e.g., `[f32; MAX_VITAMINS]` with `VitaminId` as index).
This is an optimization, not a correctness concern.

### 5.6 Dehydration Loses Detailed Survival State

When an entity is dehydrated (Active → Sleeping), its detailed survival
state (Stomach contents, Guts contents, per-vitamin levels) is lost. Only
a compact snapshot is preserved in the `SleepingEntity` record:
`stored_kcal`, `thirst`, `health`. When re-hydrated, full component state
is rebuilt from the snapshot + district aggregates. Stomach/guts start
empty. Vitamins at midpoint.

**Entity identity IS preserved.** Entity IDs are permanent. Jean-Pierre
Moreau is always `Entity(47392)`, whether Active, Sleeping, or Dead. See
backlog D01/D02 for the sleeping entity model.

**Accepted because:** Mid-digestion detail (what's in the stomach right
now) is invisible at the scale where dehydration happens. The compact
snapshot preserves the narratively meaningful state: is this person
starving? Healthy? Dehydrated? The Vec contents are transient.

~100 bytes per sleeping entity × 1M population ≈ 100MB. Bounded, does
not grow. Full entity state with HashMap entries and Vec contents would
be GB-scale.

---

## 6. DELETION LIST

Components, systems, and data to remove atomically with the migration:

| Item                          | File               | Replacement              |
|-------------------------------|--------------------|--------------------------|
| `Hunger { current, max }`    | components.rs      | Derived `HungerState`    |
| `Nutrition { value }`        | components.rs      | `FoodData` struct        |
| `hungers: HashMap`           | world.rs (Mind)    | —                        |
| `nutritions: HashMap`        | world.rs (Mind)    | `food_data` in Body      |
| `run_hunger()` system        | systems/hunger.rs  | `run_survival()`         |
| `HungerChanged` event        | events.rs          | —                        |
| `max_hunger` archetype field | archetypes.kdl     | `healthy_kcal`, etc.     |
| `InputAxis::HungerRatio`     | decisions.rs       | Read from BodyComposition|

Remove the `.remove()` calls for `hungers` and `nutritions` in their
respective sub-struct `remove()` methods. Remove the `validate_world()`
checks for these tables.

---

## 7. IMPLEMENTATION ORDER

Suggested sequence within the atomic changeset:

1. Add `GAME_MINUTES_PER_TICK` constant, update `GameDate::from_tick`, fix
   tests. (No behavior change yet — game clock is display-only.)
2. Add all new component structs to `components.rs`.
3. Add all new HashMap fields to world sub-structs, with `remove()`,
   `validate_world()`, and `new()` entries.
4. Add `VitaminDef` and vitamin registry to World. Write `data/vitamins.kdl`
   and loading code.
5. Write `data/foods.kdl` and loading code. Add `FoodData` to food entity
   spawning.
6. Write `src/systems/survival.rs` with `run_survival()` and all sub-functions
   (digestion, thirst, vitamins, stimulants).
7. Rewrite `run_eating` to use `ingest()` instead of instant hunger reduction.
8. Update `run_decisions`: replace `HungerRatio` with calorie ratio lookup,
   `select_eat_target()` to use `food_data`.
9. Add `run_survival()` to main loop Phase 2, remove `run_hunger()`.
10. Delete `Hunger`, `Nutrition`, `hungers`, `nutritions`, `HungerChanged`,
    `run_hunger`, `max_hunger` in archetypes.
11. Update archetype KDL and loading for new body composition fields.
12. Add new events (`Vomited`, `Dehydrated`, `Overdosed`).
13. Tests: positive and negative cases for survival system, eating rewrite.
