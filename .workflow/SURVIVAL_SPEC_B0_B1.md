# Survival Systems Spec: Buckets 0 & 1

> Complete implementation spec for the core survival loop.
> Derived from CDDA topology but written as a freestanding spec — no CDDA-specific content (mutations, bionics, etc.).
> All constants are collected in a tuning table at the end for easy balancing.

---

## Overview

**Bucket 0 (Core):** Digestion pipeline, calorie storage, body weight, hunger display.
**Bucket 1 (Plugs into Core):** Thirst, vitamins, stimulants.

These two buckets together give a character that needs to eat and drink, gains/loses weight, gets hungry/thirsty, can be poisoned or malnourished, and reacts to stimulants. No other systems are required.

### Tick Rate

All systems run on a unified **5-minute tick**. Some processes (solid digestion) only advance every 6th tick (i.e., every 30 minutes) but are still evaluated on the 5-minute clock.

### Multiplier Stub Convention

Several rates accept a `rate_multiplier` (float, default 1.0). Later buckets (sleep, weariness, temperature) will supply real values. Until then, all multipliers are 1.0. The spec marks these with **[MULT]**.

---

# BUCKET 0: Digestion + Calories + Hunger

---

## B0.1 Data Structures

### Food Item

Every consumable carries a `FoodData` record. This is static item data, not mutable state.

```
FoodData {
    calories:    int       // kcal energy content (>= 0)
    volume_ml:   int       // physical volume in mL
    water_ml:    int       // water content in mL (derived: quench × 5)
    vitamins:    map<VitaminID, int>  // micronutrient content (Bucket 1)
    fun:         int       // morale modifier when eaten (future: Morale bucket)
    stim:        int       // stimulant modifier (Bucket 1)
    healthy:     int       // lifestyle modifier (future: Lifestyle bucket)
    spoils_in:   Duration  // time until item spoils (0 = never)
    item_type:   enum { FOOD, DRINK, MED }
}
```

### Nutrient Packet

Internal transfer unit between compartments. Represents "stuff being digested."

```
NutrientPacket {
    solids_ml:   float     // solid volume remaining
    water_ml:    float     // water volume remaining
    calories:    float     // kcal remaining
    vitamins:    map<VitaminID, float>  // vitamin units remaining
}
```

### Stomach

```
Stomach {
    contents:    list<NutrientPacket>   // currently held food items
    capacity_ml: int                    // max volume (tunable, default 2500)

    // Derived (computed, not stored):
    //   total_volume()  → sum of (solids_ml + water_ml) across all packets
    //   total_calories() → sum of calories across all packets
    //   total_water()    → sum of water_ml across all packets
    //   total_vitamins() → merged map of all vitamin sums
}
```

### Guts

Same structure as Stomach with a larger capacity.

```
Guts {
    contents:    list<NutrientPacket>
    capacity_ml: int                    // max volume (tunable, default 24000)

    // Same derived helpers as Stomach
}
```

### Body (calorie/weight storage)

```
Body {
    stored_kcal:    int     // current fat reserves in kcal
    healthy_kcal:   int     // target kcal for "normal" weight (character-specific)
    height_cm:      int     // character height (fixed at creation)
    base_bmr:       int     // base metabolic rate in kcal/day (tunable, default 2500)

    // Derived:
    //   calorie_ratio()    → stored_kcal / healthy_kcal  (float)
    //   body_fat_kg()      → (stored_kcal - healthy_kcal) / KCAL_PER_KG
    //                         (clamped >= 0; this is excess/deficit fat)
    //   weight_category()  → see B0.4
}
```

### Activity Level

An enum set by gameplay systems (movement, combat, crafting, idling). Bucket 0 reads it, does not write it.

```
enum ActivityLevel {
    SLEEP        = 0,   // mult 0.85
    IDLE         = 1,   // mult 1.0
    LIGHT        = 2,   // mult 2.0
    MODERATE     = 3,   // mult 4.0
    BRISK        = 4,   // mult 6.0
    ACTIVE       = 5,   // mult 8.0
    EXTREME      = 6,   // mult 10.0
}
```

The float multiplier for each level is stored in a lookup table (see Tuning).

---

## B0.2 Ingestion (Eating)

When the character consumes a food item:

```
fn ingest(food: FoodData, stomach: &mut Stomach):

    // 1. Build the nutrient packet
    packet = NutrientPacket {
        solids_ml:  food.volume_ml,
        water_ml:   food.water_ml,
        calories:   food.calories,
        vitamins:   food.vitamins,
    }

    // 2. Check capacity — will this overfill?
    new_volume = stomach.total_volume() + food.volume_ml + food.water_ml

    // 3. Add to stomach regardless (overfilling is allowed, triggers vomit check)
    stomach.contents.push(packet)

    // 4. Vomit check if overfull
    if new_volume > stomach.capacity_ml:
        // Roll: random_in_range(stomach.capacity_ml / 2, new_volume)
        // If roll > stomach.capacity_ml → vomit
        roll = random(stomach.capacity_ml / 2, new_volume)
        if roll > stomach.capacity_ml:
            vomit(stomach)   // empties stomach, applies effects

    // 5. Apply instant effects
    body.stim += food.stim                         // Bucket 1: Stimulants
    // food.fun → Morale (future bucket, ignored for now)
    // food.healthy → Lifestyle (future bucket, ignored for now)

    // 6. Record "just_ate" timestamp for hunger display
    stomach.last_meal_time = current_time
```

### Vomiting

```
fn vomit(stomach: &mut Stomach):
    // Eject all stomach contents. Nutrients are lost.
    stomach.contents.clear()
    // Apply: -1 thirst (slight dehydration from fluid loss)  [Bucket 1]
    // Apply: pain, morale penalty (future buckets)
```

---

## B0.3 Digestion Tick

Called every 5 minutes. This is the core simulation loop for Bucket 0.

```
fn digestion_tick(stomach, guts, body, activity_level, rate_mult [MULT]):

    // ─── Phase 1: Stomach → Guts ───

    // Solids transfer: once per 30 min (every 6th tick)
    if tick_counter % 6 == 0:
        solids_budget = stomach.capacity_ml / 6     // capacity/6 per 30 min
        transfer_solids(stomach, guts, solids_budget)

    // Water transfer: every tick (fast absorption)
    water_budget = STOMACH_WATER_RATE               // 250 mL per tick
    water_absorbed = transfer_water(stomach, water_budget)
    // Water goes to Thirst system (Bucket 1)
    thirst_system.absorb_water(water_absorbed)       // → reduces thirst

    // Calorie transfer: once per 30 min
    if tick_counter % 6 == 0:
        cal_in_stomach = stomach.total_calories()
        cal_budget = max(STOMACH_CAL_FLOOR, cal_in_stomach * STOMACH_CAL_RATE)
        transfer_calories(stomach, guts, cal_budget)

    // Vitamin transfer: once per 30 min
    if tick_counter % 6 == 0:
        for each vitamin_id in stomach.total_vitamins():
            vit_amount = stomach.total_vitamin(vitamin_id)
            vit_budget = max(STOMACH_VIT_FLOOR, vit_amount * STOMACH_VIT_RATE)
            transfer_vitamin(stomach, guts, vitamin_id, vit_budget)

    // Drug-type vitamins: absorbed instantly from stomach (Bucket 1)
    for each vitamin_id in stomach.total_vitamins():
        if vitamin_registry.get(vitamin_id).vit_type == DRUG:
            amount = stomach.drain_vitamin(vitamin_id)  // take all
            vitamin_system.add(vitamin_id, amount)       // Bucket 1

    // ─── Phase 2: Guts → Body ───

    // Water from guts
    gut_water = transfer_water(guts, GUT_WATER_RATE)    // 250 mL per tick
    thirst_system.absorb_water(gut_water)

    // Calories from guts → stored_kcal
    hunger_mod = get_hunger_modifier(body)   // 1.0 normally; see below
    cal_from_guts = guts.total_calories() * (body.base_bmr / KCAL_DAY_TO_TICK)
                    * hunger_mod * rate_mult
    guts.drain_calories(cal_from_guts)
    body.stored_kcal += cal_from_guts

    // Vitamins from guts → vitamin levels (Bucket 1)
    for each vitamin_id in guts.total_vitamins():
        vit_amount = guts.total_vitamin(vitamin_id) * hunger_mod * rate_mult
        vit_absorbed = min(vit_amount, guts.total_vitamin(vitamin_id))
        guts.drain_vitamin(vitamin_id, vit_absorbed)
        vitamin_system.add(vitamin_id, vit_absorbed)    // Bucket 1

    // ─── Phase 3: Metabolic Burn ───

    burn_kcal = body.base_bmr * activity_multiplier(activity_level)
                / TICKS_PER_DAY * rate_mult
    body.stored_kcal -= burn_kcal
    body.stored_kcal = max(0, body.stored_kcal)       // can't go negative

    tick_counter += 1
```

### Transfer Helpers

These drain from source packets proportionally and either move to the destination compartment or discard (for absorption into body).

```
fn transfer_solids(src, dst, budget_ml):
    // Drain up to budget_ml of solids from src.contents
    // proportionally across all packets.
    // Move drained volume into a new NutrientPacket in dst.
    // Carry proportional calories/vitamins with the solids.

fn transfer_water(src, budget_ml) → float:
    // Drain up to budget_ml of water from src.contents.
    // Return actual mL drained.

fn transfer_calories(src, dst, budget_kcal):
    // Drain up to budget_kcal from src, add to dst.

fn transfer_vitamin(src, dst, vitamin_id, budget):
    // Same pattern for a single vitamin.
```

### Hunger Modifier

Controls how fast guts absorb based on whether the body is in deficit.

```
fn get_hunger_modifier(body) → float:
    ratio = body.calorie_ratio()
    if ratio >= 1.0:
        return 1.0          // well-fed: normal absorption
    else:
        return 1.0 + (1.0 - ratio)  // hungry: faster absorption (up to 2.0x)
```

---

## B0.4 Weight Categories

Derived from `body.calorie_ratio()`. These are abstract tiers, not real-world BMI.

```
fn weight_category(body) → WeightCategory:
    ratio = body.calorie_ratio()

    if ratio < 0.5:   return EMACIATED
    if ratio < 0.7:   return UNDERWEIGHT
    if ratio < 1.1:   return NORMAL
    if ratio < 1.4:   return OVERWEIGHT
    if ratio < 2.0:   return OBESE
    if ratio < 2.5:   return VERY_OBESE
    return MORBIDLY_OBESE
```

| Category | Ratio Range | Gameplay |
|----------|-------------|----------|
| Emaciated | < 0.5 | Starving state; severe stat penalties |
| Underweight | 0.5 – 0.7 | Near-starving; moderate penalties |
| Normal | 0.7 – 1.1 | No penalties |
| Overweight | 1.1 – 1.4 | Minor stamina penalty (future bucket) |
| Obese | 1.4 – 2.0 | Lifestyle penalty (future bucket) |
| Very Obese | 2.0 – 2.5 | Larger penalties |
| Morbidly Obese | >= 2.5 | Severe penalties |

**Conversion constant:** `KCAL_PER_KG = 7716` (kcal per kg of body fat).

---

## B0.5 Hunger Display

Hunger is a **derived display state**, not a counter. Computed fresh each time the UI needs it.

```
fn hunger_display(stomach, body) → HungerState:
    cap = stomach.capacity_ml
    vol = stomach.total_volume()
    has_deficit = body.calorie_ratio() < 1.0
    wt = weight_category(body)

    // --- Overfed states (no deficit check) ---
    if vol >= cap * 5/6:             return ENGORGED
    if vol >= cap * 3/4:             return FULL

    // --- Fed states ---
    if not has_deficit:
        if just_ate(stomach) and vol > cap / 2:
            return SATISFIED
        return NONE                  // no label shown

    // --- Deficit states ---
    if just_ate(stomach):            return HUNGRY
    if recently_ate(stomach, 3 hrs): return VERY_HUNGRY
    if wt == EMACIATED:              return STARVING
    if wt == UNDERWEIGHT:            return NEAR_STARVING
    return FAMISHED
```

### Hunger State Effects

| State | Speed | Other |
|-------|-------|-------|
| ENGORGED | -10 | Vomit chance 1/500 per tick; pain +3 |
| FULL | -2 | — |
| SATISFIED | — | — |
| NONE | — | — |
| HUNGRY | — | Mild UI warning |
| VERY_HUNGRY | — | Activity interruption (dismissable) |
| FAMISHED | -5 | Stat penalties begin |
| NEAR_STARVING | -15 | STR/DEX -2 each |
| STARVING | -30 | STR/DEX -4 each; HP drain over time |

### Time Helpers

```
fn just_ate(stomach) → bool:
    return (current_time - stomach.last_meal_time) < 15 minutes

fn recently_ate(stomach, window) → bool:
    return (current_time - stomach.last_meal_time) < window
```

---

## B0.6 Starting State (New Character)

```
stomach.contents = [ NutrientPacket { solids_ml: 475, water_ml: 0,
                                       calories: 800, vitamins: {} } ]
guts.contents    = [ NutrientPacket { solids_ml: 0, water_ml: 0,
                                       calories: 300, vitamins: {} } ]
body.stored_kcal  = body.healthy_kcal    // starts at normal weight
body.stim         = 0
thirst            = 0
all vitamin levels at midpoint of their range
```

---

## B0.7 Interfaces Exposed to Other Buckets

These are the touch-points that Bucket 1 (and later buckets) connect to.

| Interface | Direction | Used by |
|-----------|-----------|---------|
| `stomach.total_water()` | read | Thirst (instant thirst display) |
| `thirst_system.absorb_water(mL)` | call | Thirst (Bucket 1) |
| `vitamin_system.add(id, amount)` | call | Vitamins (Bucket 1) |
| `body.stim` | write | Stimulants (Bucket 1) |
| `body.calorie_ratio()` | read | Hunger display, weight category |
| `activity_level` | read | Calorie burn rate |
| `rate_mult` **[MULT]** | read | Sleep (future: 0.5x), traits, etc. |
| `body.base_bmr` | read | Weariness (future bucket) |
| `stomach.last_meal_time` | read | Hunger display |

---

# BUCKET 1: Thirst + Vitamins + Stimulants

These three systems plug into Bucket 0 via the interfaces above. They have no dependencies on each other and can be implemented in any order.

---

## B1.1 Thirst System

### Data

```
ThirstSystem {
    thirst:      int     // current thirst level
                         // range: THIRST_MIN (-100) to THIRST_MAX (1200)
}
```

### Water Absorption (called by Bucket 0)

```
fn absorb_water(self, ml: float):
    // Each mL absorbed reduces thirst by THIRST_PER_ML
    self.thirst -= ml * THIRST_PER_ML
    self.thirst = clamp(self.thirst, THIRST_MIN, THIRST_MAX)
```

### Passive Thirst Gain (per 5-min tick)

```
fn thirst_tick(self, is_sleeping: bool, rate_mult [MULT]):
    base = THIRST_BASE_RATE                     // 1.0 per tick
    if is_sleeping:
        base *= THIRST_SLEEP_MULT               // 0.5

    self.thirst += base * rate_mult
    self.thirst = clamp(self.thirst, THIRST_MIN, THIRST_MAX)

    // Death check
    if self.thirst >= THIRST_MAX:
        kill_character("dehydration")
```

### Instant Thirst (Display)

What the UI shows, accounting for water currently in the stomach that hasn't been absorbed yet.

```
fn display_thirst(self, stomach) → int:
    return self.thirst - stomach.total_water() * THIRST_PER_ML
```

### Thirst Display Tiers

| Range | Label | Color |
|-------|-------|-------|
| < -60 | Turgid | Green |
| -60 to -20 | Hydrated | Green |
| -20 to 0 | Slaked | Green |
| 0 to 40 | *(none)* | Default |
| 40 to 80 | Thirsty | Yellow |
| 80 to 240 | Very Thirsty | Yellow |
| 240 to 520 | Dehydrated | Light Red |
| > 520 | Parched | Red |

### Thirst Effects

| Threshold | Effect |
|-----------|--------|
| > 40 | Speed penalty: `-thirst / (THIRST_MAX / 75)` (linear, max -75) |
| >= 200 | All stats penalized: `-thirst / 200` |
| > 520 | Activity interruption ("dangerously dehydrated") |
| >= 1200 | **Death** |

---

## B1.2 Vitamin System

### Vitamin Definition (Data-Driven)

Each vitamin type is defined in data, not code. The engine processes them generically.

```
VitaminDef {
    id:              VitaminID          // e.g., "iron", "vitC", "ethanol"
    vit_type:        enum { VITAMIN, TOXIN, DRUG, COUNTER }
    min:             int                // lower accumulation bound
    max:             int                // upper accumulation bound
    rate:            Duration           // passive change: one unit lost/gained per this interval
    rate_direction:  int                // -1 (depletes) or +1 (accumulates); default -1 for VITAMIN
    deficiency_effect: EffectID | null  // applied when level <= min
    excess_effect:     EffectID | null  // applied when level >= max
    disease_tiers:   list<DiseaseTier>  // progressive severity for deficiency
    excess_tiers:    list<DiseaseTier>  // progressive severity for excess
    decays_into:     map<VitaminID, float> | null  // conversion on decay
}

DiseaseTier {
    threshold:  int        // level at which this tier activates
    effect:     EffectID   // effect to apply
}
```

### Vitamin Type Behavior

| Type | Absorption | Passive Rate | Purpose |
|------|-----------|--------------|---------|
| **VITAMIN** | Through guts (slow) | Depletes over time | Nutritional needs |
| **TOXIN** | Through guts (slow) | Accumulates from food | Poison, contamination |
| **DRUG** | Instant from stomach (bypasses guts) | Decays toward 0 | Alcohol, caffeine, medicine |
| **COUNTER** | However the game sets it | Per definition | Generic mechanic tracker |

### Character Vitamin State

```
VitaminSystem {
    levels: map<VitaminID, int>    // current level per vitamin
}
```

### Operations

```
fn add(self, id: VitaminID, amount: int):
    def = vitamin_registry.get(id)
    self.levels[id] = clamp(self.levels[id] + amount, def.min, def.max)

fn tick(self):    // called every 5-min tick
    for each (id, level) in self.levels:
        def = vitamin_registry.get(id)

        // Passive decay/accumulation
        ticks_per_unit = def.rate / 5 minutes    // how many ticks per 1 unit change
        if tick_counter % ticks_per_unit == 0:
            self.levels[id] += def.rate_direction
            self.levels[id] = clamp(self.levels[id], def.min, def.max)

        // Conversion (decays_into)
        if def.decays_into != null and ticks_per_unit check passes:
            for each (target_id, ratio) in def.decays_into:
                self.add(target_id, ratio * 1)

        // Effect application
        apply_deficiency_excess_effects(id, self.levels[id], def)

fn apply_deficiency_excess_effects(id, level, def):
    // Deficiency: check disease_tiers from most severe to least
    // Apply the highest-severity tier whose threshold is >= level
    // (tiers are checked against distance below min)

    // Excess: same logic, checked against distance above max

    // Simple deficiency/excess (non-tiered):
    if level <= def.min and def.deficiency_effect != null:
        apply_effect(def.deficiency_effect)
    if level >= def.max and def.excess_effect != null:
        apply_effect(def.excess_effect)
```

### Example Vitamin Roster

These are starting-point definitions. Actual roster is game-specific.

| ID | Type | Min | Max | Rate | Deficiency | Excess |
|----|------|-----|-----|------|------------|--------|
| `iron` | VITAMIN | -24000 | 3600 | 15 min/unit | Anemia (tiered) | — |
| `vitC` | VITAMIN | -5600 | 0 | 15 min/unit | Scurvy (tiered) | — |
| `calcium` | VITAMIN | -48000 | 0 | 15 min/unit | Bone weakness | — |
| `ethanol` | DRUG | 0 | 1000 | 1 sec/unit | — | Drunk |
| `caffeine` | DRUG | 0 | 500 | 30 sec/unit | — | Jittery |

### Vitamin Integration with Bucket 0

- **VITAMIN/TOXIN types:** Arrive via `guts → vitamin_system.add()` during digestion tick Phase 2.
- **DRUG types:** Arrive via `stomach → vitamin_system.add()` during digestion tick Phase 1 (instant absorption, bypass guts).
- **COUNTER types:** Set/modified directly by game logic, not by the digestion pipeline.

---

## B1.3 Stimulant System

### Data

```
StimulantSystem {
    stim: int    // current stimulant level (positive = stimulated, negative = depressed)
}
```

Stim is set during ingestion (`body.stim += food.stim`). It then decays toward 0.

### Tick (every 5-min tick)

```
fn stim_tick(self):
    // Decay toward 0
    if self.stim > 0:
        self.stim -= STIM_DECAY_RATE       // default 1 per tick
    elif self.stim < 0:
        self.stim += STIM_DECAY_RATE

    // Lethal thresholds
    if self.stim > STIM_OVERDOSE:           // 250
        trigger_overdose()                  // "heart attack" — lethal
    if self.stim < STIM_LETHAL_LOW:         // -200
        trigger_respiratory_failure()       // lethal
```

### Effects (read by other systems)

| Range | Effect | Used By |
|-------|--------|---------|
| > 0 | Stamina regen bonus: `+min(5, stim/15)` per turn | Stamina (future) |
| >= 30 | Delays forced sleep until severe deprivation | Sleep (future) |
| > STIM_OVERDOSE | Lethal overdose | Self |
| < 0 | Stamina regen penalty (proportional) | Stamina (future) |
| < STIM_LETHAL_LOW | Lethal withdrawal | Self |

### Interface

Other systems read `stim` as a plain integer. No callbacks needed.

---

# TICK ORCHESTRATION

The master tick runs every 5 in-game minutes. Order matters.

```
fn survival_tick(character):
    // 1. Digestion: stomach → guts → body
    digestion_tick(character.stomach, character.guts, character.body,
                   character.activity_level, character.rate_mult)

    // 2. Thirst: passive gain + death check
    character.thirst_system.thirst_tick(character.is_sleeping,
                                         character.rate_mult)

    // 3. Vitamins: passive decay, effect checks
    character.vitamin_system.tick()

    // 4. Stimulants: decay toward 0, lethal checks
    character.stim_system.stim_tick()

    // 5. Hunger display: recompute (no side effects, just cache for UI)
    character.hunger_state = hunger_display(character.stomach, character.body)
```

---

# TUNING TABLE

All magic numbers in one place. Adjust these to balance the game.

## Bucket 0 Constants

| Name | Default | Unit | Notes |
|------|---------|------|-------|
| `STOMACH_CAPACITY` | 2500 | mL | Can be modified by traits |
| `STOMACH_CAPACITY_MIN` | 250 | mL | Floor even with shrinking traits |
| `GUT_CAPACITY` | 24000 | mL | |
| `STOMACH_WATER_RATE` | 250 | mL / 5-min tick | |
| `STOMACH_CAL_FLOOR` | 5 | kcal / 30-min cycle | Prevents stalling on tiny amounts |
| `STOMACH_CAL_RATE` | 0.167 | fraction / 30-min cycle | 1/6 of stomach calories |
| `STOMACH_VIT_FLOOR` | 1 | units / 30-min cycle | |
| `STOMACH_VIT_RATE` | 0.167 | fraction / 30-min cycle | |
| `GUT_WATER_RATE` | 250 | mL / 5-min tick | |
| `BASE_BMR` | 2500 | kcal / day | |
| `KCAL_PER_KG` | 7716 | kcal / kg body fat | 3500 kcal/lb × 2.205 |
| `TICKS_PER_DAY` | 288 | ticks | 24 hr × 12 ticks/hr |

### Activity Level Multipliers

| Level | Multiplier |
|-------|------------|
| SLEEP | 0.85 |
| IDLE | 1.0 |
| LIGHT | 2.0 |
| MODERATE | 4.0 |
| BRISK | 6.0 |
| ACTIVE | 8.0 |
| EXTREME | 10.0 |

### Weight Category Thresholds (calorie ratio)

| Category | Ratio |
|----------|-------|
| EMACIATED | < 0.5 |
| UNDERWEIGHT | < 0.7 |
| NORMAL | < 1.1 |
| OVERWEIGHT | < 1.4 |
| OBESE | < 2.0 |
| VERY_OBESE | < 2.5 |
| MORBIDLY_OBESE | >= 2.5 |

## Bucket 1 Constants

### Thirst

| Name | Default | Unit | Notes |
|------|---------|------|-------|
| `THIRST_MIN` | -100 | | Overhydrated floor |
| `THIRST_MAX` | 1200 | | Death threshold |
| `THIRST_BASE_RATE` | 1.0 | per 5-min tick | |
| `THIRST_SLEEP_MULT` | 0.5 | | |
| `THIRST_PER_ML` | 0.2 | thirst reduced per mL | i.e., 1/5 |
| `QUENCH_TO_ML` | 5 | mL per quench unit | For item data conversion |

### Stimulants

| Name | Default | Unit | Notes |
|------|---------|------|-------|
| `STIM_DECAY_RATE` | 1 | per 5-min tick | |
| `STIM_OVERDOSE` | 250 | | Lethal upper bound |
| `STIM_LETHAL_LOW` | -200 | | Lethal lower bound |
| `STIM_SLEEP_GATE` | 30 | | Stim level that delays forced sleep |

---

# APPENDIX: What's NOT in These Buckets

These systems are intentionally excluded and will be specified in later bucket docs. Bucket 0+1 should work correctly without them — all touch-points use stub multipliers or are no-ops.

| System | Bucket | Stub behavior in B0+B1 |
|--------|--------|------------------------|
| Sleepiness / Sleep | 2 | `is_sleeping = false`, sleep multipliers = 1.0 |
| Weariness / Exertion | 2 | No weariness tracking |
| Stamina | 2 | No stamina pool |
| Body Temperature | 3 | No temperature effects |
| Lifestyle | 3 | `food.healthy` is accepted but discarded |
| Morale | 3 | `food.fun` is accepted but discarded |
