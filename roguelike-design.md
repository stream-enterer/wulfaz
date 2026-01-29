# Mech Autobattler Roguelike — Design Document

**Status:** In Progress (Interview complete, ready for implementation)

---

## Core Identity

- **Genre Hybrid:** Traditional roguelike + Autobattler + Deckbuilder elements
- **Turn-based:** Yes (simultaneous resolution, cooldown-based)
- **Permadeath:** Yes
- **Complex systems:** Yes (BTA-level depth)
- **Primary Hook:** Deep build customization IS the game
- **Run Length:** 45-60 minutes target
- **Random Starts:** Yes (not full class selection)
- **Power Fantasy:** Broken builds rewarded, not balanced into blandness

### Inspirations

| Game | What to Take |
|------|--------------|
| The Bazaar | Build customization as core fun, spatial item relationships, combat width, simultaneous resolution |
| Slice & Dice | Quick runs, spatial relationships affecting gameplay, random starts |
| DCSS | Decisions matter, low randomness |
| Path of Achra | Broken builds possible and rewarded |

### Anti-Inspirations

| Game | What to Avoid |
|------|---------------|
| Battle Brothers | Over-balanced, no power fantasy |
| Dwarf Fortress | Complexity at expense of UI/playability |
| BTA Battletech | Sloggy, grindy, repetitive campaigns (keep the customization) |

---

## Setting

**Hybrid Sci-Fi/Fantasy** (WH40k-style faction diversity)

- Tech faction: BTA-adjacent mechs, vehicles, battle armor
- Other factions: Magic, bio-organic, exotic (expansion content)
- Different design spaces for different factions

---

## Combat System

### Structure

- **Style:** Autobattler with simultaneous resolution (Bazaar-style)
- **Positioning:** Single row, combat width constraint
- **Control:** Hands-off during combat — units act via AI + loadout

### Player Agency During Combat

- Pause / slow / speed controls
- Active abilities (cooldown/resource gated)
- Retreat/reserve units
- NO direct unit control

### AI Behavior

- Units don't always fire (may take other actions)
- Pilots influence decisions via traits
- Subsystems can impede actions
- Chat bubbles explain pilot reasoning

### Combat Flow

```
Setup (between fights)
    ↓
Combat begins (simultaneous resolution)
    ↓
Units act based on: Loadout + Pilot traits + AI
    ↓
Player can: Pause, use actives, retreat units
    ↓
Resolution until one side eliminated/retreated
```

---

## Positioning & Combat Width

```
YOUR TEAM (facing up)
┌─────────────────────────────────────────────┐
│  Combat Width: [████████████████████]       │
│                                             │
│   MECH A (Medium)    MECH B (Medium)        │
│      ||      ||         ||      ||          │
│    [W1][h][ ]         [ ][h][W2]            │
│       [ ][ ]             [ ][ ]             │
│                                             │
│   (2 slots)            (2 slots)            │
└─────────────────────────────────────────────┘

ENEMY TEAM (facing down)
┌─────────────────────────────────────────────┐
│   MECH (Medium)   BA×3 (Small)  VEHICLE     │
│      [ ][ ]          (\)         [  ]       │
│    [ ][h][ ]         (\)         |HV|       │
│     |     |          (\)         |__|       │
│    ▄▄▄▄▄▄▄▄▄                                │
│    |_s_s_s_|  (2H shield)                   │
│                                             │
│   (2 slots)      (1 slot)     (1 slot)      │
└─────────────────────────────────────────────┘
```

### Positioning Rules

- Single row only (width is the spatial axis)
- Adjacency matters heavily:
  - Buffs/auras between adjacent friendlies
  - Splash damage risks
  - Targeting restrictions/bonuses
- No facing mechanic (abstracted away)
- Position set between fights, possibly adjustable during

---

## Unit System

### Size Categories

| Size | Combat Width | Customization Depth |
|------|--------------|---------------------|
| Small | 1 slot | Limited slots, unique traits |
| Medium | 2 slots | Balanced |
| Large | 3 slots | Most customization depth |

### Unit Types

- **Mechs:** Primary customizable units (all sizes)
- **Vehicles:** Support units (small/medium)
- **Battle Armor:** Infantry squads (small)
- **Other:** Faction-specific (golems, bio-beasts, etc.)

### Tradeoffs

- Larger = more gear slots, fewer units on field
- Smaller = unique traits, more bodies, less individual customization
- Optimal builds mix sizes for synergies

---

## Damage Model (BTA-style)

### Damage Resolution

1. Attack hits unit
2. Roll hit location (random component)
3. Armor absorbs damage
4. Excess goes to structure
5. Structure damage can crit

### Critical Hits

| Crit Type | Effect |
|-----------|--------|
| 0 Structure | Component destroyed |
| Ammo Crit | Explosion (AOE) |
| Engine Crit ×3 | Cored (mech destroyed + AOE) |
| Other Crits | Debuffs (weapon jam, sensor damage, etc.) |

### Death States

| State | Cause | Effect |
|-------|-------|--------|
| Cored | 3 engine crits or massive damage | AOE explosion, wreckage |
| Retreated | Successful withdrawal | Unit saved for later, no wreckage |
| Legged | Mobility destroyed | Immobilized, blocks combat slot |

### Component Destruction

- Permanent loss (must replace via shop/salvage)
- Trigger effects vary by part type
- Some components explode, some just stop working

---

## Component System

> **Note:** See "Universal Composition System" section for full architectural details.
> This section describes content categories, not implementation.

### Item Categories (by tag)

- **Weapons:** `[weapon]` — Damage dealers
- **Armor/Plating:** `[armor]` — Damage absorption
- **Actuators/Mobility:** `[actuator]` — Movement, evasion
- **Reactors/Engines:** `[engine]` — Power generation, heat
- **Cockpit/Sensors:** `[sensor, cockpit]` — Targeting, pilot safety
- **Utility/Support:** `[utility]` — Special abilities
- **Ammo:** `[ammo]` — Consumable, explosion risk

### Weapon Properties (as attributes)

| Attribute | Description |
|-----------|-------------|
| `damage` | Base damage dealt |
| `cooldown` | Time between shots |
| `range` | Targeting range |
| `heat` | Heat generated per shot |
| `ammo_per_shot` | Ammo consumed |
| `size` | Mount capacity required |

### Weapon Types (by tag)

- `[energy]` — Lasers, PPCs
- `[ballistic]` — Autocannons, machine guns
- `[missile]` — LRMs, SRMs

### Chassis

- Size determines combat width (Small=1, Medium=2, Large=3)
- Each chassis template defines part layout and mount configurations
- No tonnage system (size is the constraint)
- Distinct silhouettes and configurations

---

## Pilots

### Core Design

- Swappable between units (component-like)
- Level up during run, gain traits
- Draftable resource (recruit at shops/events)
- Compatibility bonuses/maluses with unit types
- NO affinity building (pure stat matching)

### Pilot Traits

- Affect AI decisions during combat
- Influence targeting, retreat behavior, aggression
- Chat bubbles explain reasoning when traits proc

### Pilot Objectives

- Types: Kill counts, wealth, faction goals, boss kills, combinations
- Can evolve based on events during run
- Multi-fight mini-dungeon chains for character arcs
- **Decision Points:** Meet objective → cash out or push luck

---

## Persistent Enemies

- Damaged enemies can retreat mid-fight
- Retreated enemies may ambush you in later fights
- Creates recurring nemeses and consequences
- Risk/reward: finish them now vs. let them escape
- Ambush = encounter type (worse positioning? less prep?)

---

## Run Structure (Bazaar-style)

### Flow

```
Start (random loadout + fixed options)
    ↓
Multi-choice event/shop phase
    ↓
Multi-choice: select next battle
    ↓
Combat (autobattler)
    ↓
Salvage/results
    ↓
Repeat until: cash out at objective OR death
```

### Between-Fight Verbs

- **Draft:** New components/units from events
- **Shop:** Buy with credits
- **Upgrade:** Pay to level up gear
- **Salvage:** From defeated enemies
- **Refit:** Rearrange loadouts freely
- **Repair:** Button + credits
- **Sell:** Inventory management

### Win/Lose Conditions

- **Win:** Meet pilot objective → choose to cash out
- **Continue:** Push luck for bigger rewards
- **Lose:** All units destroyed

---

## Economy

### Currencies

- **Primary:** Credits
- **Special:** Non-standard costs for rare items
  - "Kill 10 pirates"
  - "Have pilot 'John' in party"
  - Objective-gated unlocks

### Scarcity: Moderate

- Sell items to reach for expensive pieces
- Never staring at unaffordable shops (cycle stores instead)
- Can pivot build 1-2 times per run, not constantly

---

## Factions

### Design Philosophy

- Entirely different build subsystems per faction
- This is **expansion content** (not MVP)

### Mixing Rules

- Start with one faction
- Can recruit/salvage others during run
- Unique cross-faction interactions
- Hidden/gated content for discoveries

---

## Meta-Progression

### Unlocks

- Starting options (more variety, not power)
- Does NOT gate content (accessibility first)

### What Persists Between Runs

- Unlocked starting options
- Knowledge (player skill)
- Nothing else (true roguelike)

---

## Interview Progress

### Completed Rounds

- [x] Round 1: Core Identity
- [x] Round 2: Play Space
- [x] Round 3: Combat Mechanics (revised for autobattler)
- [x] Round 4: Combat Resolution
- [x] Round 5: Autobattler Specifics
- [x] Round 6: Mechs & Components
- [x] Round 7: Economy & Factions
- [x] Round 8: UI, Tech & Platform
- [x] Round 9: MVP Scope & Architecture

---

## Tech Stack

- **Architecture:** TEA (The Elm Architecture) in Go
- **Reference:** See `tea-go-ruleset.md`
- **Platform:** Desktop GUI
- **Rendering:** Hybrid (2D engine + pseudoterminal layer)
  - Full 2D engine base for physics and effects
  - Pseudoterminal layered on top for ASCII aesthetic
  - Enables text deformation and visual effects

### Display & Animation (Progressive)

| Stage | Display | Animation | Sound |
|-------|---------|-----------|-------|
| MVP | Hybrid renderer | None | None |
| Stage 2 | — | Bump combat, hit flashes | SFX (critical) |
| Stage 3 | — | Projectiles, explosions | — |
| Full | — | Particles, deformation | Ambient, soundtrack |

### Save System

- JSON dump of Model (TEA-native)

### Debug & Replay: HIGH PRIORITY

- Undo turns (architectural requirement)
- Extensive debugging at every step
- Time-travel debugging capability
- Built in from ground up, not bolted on

### Architectural Constraint: Seeded RNG

**All randomness must be external to Update — passed in via Msgs.**

```go
// All random values are pre-rolled or seeded in Msg payload
type CombatTickMsg struct {
    Seed  int64  // OR
    Rolls []int  // Pre-generated values
}

func Update(model Model, msg CombatTickMsg) Model {
    // Use msg.Rolls[n] instead of rand()
    // Deterministic: same Msg = same result
}
```

This enables:
- Full replay from Msg log
- Turn-level undo via Model snapshots
- Time-travel debugging
- Deterministic test cases

---

## MVP Scope

### Content

| Area | MVP Scope |
|------|-----------|
| Factions | 1 (tech), stub second |
| Chassis | 3 (Small, Medium, Large mech) |
| Weapon Types | 3 (Energy, Ballistic, Missile) |
| Fights | 2 |
| Shop/Event Phases | 2 (between fights) |
| Pilots | Stubs (names only, assignable to mechs) |
| Unit Pools | Symmetric (player/enemy share same pool) |
| Win Condition | Simple (survive or die) |

### MVP Flow

```
Fight 1 → Shop/Event → Shop/Event → Fight 2 → Game Over/Reset
```

### Explicit Cuts (Not in MVP)

- Multiple factions
- Meta-progression
- Sound
- Persistent returning enemies
- Pilot objectives/cash-out system
- Complex win conditions
- Pilot traits (beyond stubs)

### MVP Focus

**Clean architecture first.** The MVP is the minimal emergent result of correct architecture — not a feature checklist.

---

## Universal Composition System

### Design Philosophy

**Everything is Tags, Attributes, and Triggers. No special types. No hardcoding.**

- "Hardpoint" = a Mount with `requires_all: [ballistic]`
- "Health" vs "Structure" = just different attribute names
- "Weapon" = an Item with tag `weapon`
- "Destroyed" = attribute reaches threshold, fires trigger

### Core Types

```
TAG
  - Just a string label for categorization
  - Used for filtering, matching, querying

ATTRIBUTE
  - name: string
  - base: int
  - min: int (optional, default 0)
  - max: int (optional, default unlimited)
  - [current computed at runtime from base + modifiers]

MODIFIER
  - source_id: string (what's providing this)
  - operation: "add" | "mult" | "set" | "min" | "max"
  - value: int
  - stack_group: string | null (see Modifier Stacking below)

TRIGGER
  - event: string (open, registry-validated)
  - conditions: []Condition (optional, must pass for trigger to fire)
  - effect: string (open, registry-validated)
  - params: map[string]any
  - priority: int (optional, default 0, lower = earlier in cascade)

CONDITION (supports boolean trees)
  - Leaf: { type: string, params: map }
  - AND: { AND: []Condition }
  - OR: { OR: []Condition }
  - NOT: { NOT: Condition }
  - Top-level array = implicit AND

PROVIDED_MODIFIER (modifiers an item/trait grants when active)
  - scope: "self" | "mount" | "part" | "unit" | "adjacent" | "tagged"
  - scope_filter: []Tag (optional, targets must have these)
  - attribute: string
  - operation: string
  - value: int
  - stack_group: string | null
  - conditions: []Condition (optional, modifier only active when conditions pass)

REQUIREMENT (dependencies an item needs to function)
  - scope: "unit" | "part" | "adjacent" | "mount"
  - condition: Condition
  - on_unmet: "disabled" | "cannot_mount" | "warning"

ABILITY (player-activated powers, as opposed to passive triggers)
  - id: string
  - tags: []Tag
  - conditions: []Condition (must pass to activate)
  - costs: []Cost (resources consumed on use)
  - targeting: Targeting (what can be targeted)
  - effects: []Effect (what happens when used)
  - cooldown: int (ticks until usable again, 0 = no cooldown)
  - charges: int (uses before depleted, -1 = unlimited)
  - charge_restore: string (event that restores charges, e.g., "on_turn_start")

COST (resource consumed by ability)
  - attribute: string (heat, energy, ammo, health, etc.)
  - scope: "self" | "unit" | "part"
  - amount: ValueRef (can be static or dynamic)

TARGETING (what an ability can target)
  - type: "none" | "self" | "ally" | "enemy" | "any_unit" | "part" | "item" | "position"
  - range: int (for positional targeting)
  - count: int (how many targets, default 1)
  - filter: []Tag (targets must have these tags)

EFFECT (single effect in a chain)
  - effect: string (effect type from registry)
  - params: map[string]ValueRef
  - delay: int (ticks before effect resolves, 0 = immediate)
  - conditions: []Condition (optional, effect skipped if conditions fail)

VALUE_REF (dynamic value references — the secret sauce for modder insanity)
  - Static: just a number (5, 10, -3)
  - Reference: "self.damage", "target.health", "unit.heat"
  - Event data: "event.damage_amount", "event.source_id"
  - Computed: "self.stored_damage * 2", "target.max_health - target.health"
  - Random: "random(1, 6)", "random_from(self.damage, self.damage * 2)"

  Format: { value: 5 } or { ref: "self.damage" } or { expr: "target.health * 0.5" }
```

### Structural Hierarchy

```
UNIT
├── id, template_id
├── tags: []Tag
├── attributes: map[string]Attribute
├── parts: map[string]Part
├── triggers: []Trigger
├── abilities: []Ability
└── pilot: *Pilot (optional)

PART
├── id, template_id
├── tags: []Tag
├── attributes: map[string]Attribute
├── mounts: []Mount
├── connections: map[string][]string  // relationship_type → part_ids
├── triggers: []Trigger
└── abilities: []Ability

MOUNT
├── id
├── tags: []Tag
├── accepts: MountCriteria
│   ├── requires_all: []Tag
│   ├── requires_any: []Tag
│   └── forbids: []Tag
├── capacity: int
├── capacity_attr: string (default "size")
├── max_items: int (default 1, -1 = unlimited)
├── locked: bool
└── contents: []Item

ITEM
├── id, template_id
├── tags: []Tag
├── attributes: map[string]Attribute
├── triggers: []Trigger
├── abilities: []Ability
├── provides: []ProvidedModifier
└── requires: []Requirement (optional, dependencies to function)

PILOT
├── id, name
├── tags: []Tag
├── attributes: map[string]Attribute
├── traits: []Trait
├── triggers: []Trigger
└── abilities: []Ability

TRAIT
├── id
├── tags: []Tag
├── triggers: []Trigger
├── abilities: []Ability
└── provides: []ProvidedModifier
```

### Multi-Mount Items

Items that require multiple mounts use an attribute, not a tag:

```yaml
item:
  id: graviton_artillery
  tags: [weapon, ballistic, massive]  # "massive" is descriptive only
  attributes:
    size: { base: 14 }
    mounts_required: { base: 2 }  # system checks this
```

**Mounting logic:**
1. Item has `mounts_required` attribute (default 1)
2. Find N compatible mounts on part (or linked parts)
3. Primary mount holds the item
4. Secondary mounts marked as "occupied by [item_id]"
5. If any occupied mount destroyed → item destroyed

### Modifier Stacking

**`stack_group` field controls stacking behavior.**

- Modifiers with same `stack_group` = only highest `value` applies
- Modifiers with `stack_group: null` = always stack

```yaml
# Ammo bonus (doesn't stack with other ammo bonuses)
provides:
  - attribute: damage
    operation: add
    value: 1
    stack_group: "ammo_damage"  # only one applies

# Heatsink (stacks with other heatsinks)
provides:
  - attribute: heat_dissipation
    operation: add
    value: 2
    stack_group: null  # always stacks
```

**Operation order (deterministic):**

```
1. Collect all active modifiers for attribute
2. Group by stack_group, keep highest value from each group
3. Apply in order: SET → ADD → MULT → MIN → MAX

final = base
for each SET: final = value
for each ADD: final += value
for each MULT: final *= value
for each MIN: final = max(final, value)  // floor
for each MAX: final = min(final, value)  // ceiling
```

### Condition Evaluation

Full boolean expression trees:

```yaml
# Simple (implicit AND)
conditions:
  - { type: has_tag, params: { tag: enemy } }
  - { type: attr_gte, params: { attr: health, value: 50 } }

# Complex (explicit AND/OR/NOT)
conditions:
  - AND:
      - { type: has_tag, params: { tag: enemy } }
      - OR:
          - { type: attr_gte, params: { attr: health, value: 50 } }
          - { type: has_tag, params: { tag: boss } }

# Means: enemy AND (health >= 50 OR is_boss)
```

### Target Context

Standard context variables available in conditions and effects:

| Variable | Description |
|----------|-------------|
| `self` | The entity owning the trigger/modifier (item, part, unit) |
| `target` | The entity being affected by an effect |
| `source` | The entity that caused the event (e.g., attacker) |
| `unit` | The unit containing `self` |
| `part` | The part containing `self` (if self is an item) |
| `mount` | The mount containing `self` (if self is an item) |
| `combat` | The current combat state (for cross-unit queries) |

**Cross-unit targeting:**

```yaml
# Aura that buffs adjacent friendly units (not parts — units in combat row)
provides:
  - scope: adjacent_unit  # different from "adjacent" which means adjacent parts
    scope_filter: [friendly]
    attribute: evasion
    operation: add
    value: 1
```

### Condition Type Registry

Core condition types (extensible):

| Type | Params | Description |
|------|--------|-------------|
| `has_tag` | `target`, `tag` | Target has specified tag |
| `attr_gte` | `target`, `attr`, `value` | Attribute >= value |
| `attr_lte` | `target`, `attr`, `value` | Attribute <= value |
| `attr_eq` | `target`, `attr`, `value` | Attribute == value |
| `has_item_with_tag` | `scope`, `tag` | Scope contains item with tag |
| `mount_has_item` | `mount_id` | Specific mount has item |
| `is_adjacent_to` | `target`, `tag` | Target is adjacent to entity with tag |
| `roll_under` | `target`, `attr` | Random roll < attribute value (for % chances) |
| `in_combat` | — | Currently in combat phase |

**Target parameter values:** `self`, `target`, `source`, `unit`, `part`

### Item Dependencies

Items can declare requirements to function:

```yaml
item:
  id: ac10
  tags: [weapon, ballistic, autocannon]
  attributes:
    size: { base: 7 }
    damage: { base: 10 }
  requires:
    - scope: unit
      condition: { type: has_item_with_tag, params: { scope: unit, tag: ac_ammo } }
      on_unmet: disabled  # weapon can't fire without ammo
```

**on_unmet behaviors:**
- `disabled` — Item exists but can't activate (weapon won't fire)
- `cannot_mount` — Item can't be placed in mount until requirement met
- `warning` — UI warning, but item still functions

### Conditional Modifiers

Modifiers can have activation conditions:

```yaml
# Bonus damage only when heat is low
item:
  id: cryo_cannon
  provides:
    - scope: self
      attribute: damage
      operation: add
      value: 5
      conditions:
        - { type: attr_lte, params: { target: unit, attr: heat, value: 30 } }
```

When conditions aren't met, the modifier is inactive (not applied to attribute calculation).

### Event & Effect System

**Events and effects are open strings, validated against extensible registry.**

Events:
- `on_damaged`, `on_destroyed`, `on_crit`
- `on_attribute_zero`, `on_attribute_max`
- `on_item_mounted`, `on_item_removed`
- `on_combat_start`, `on_turn_start`, `on_turn_end`
- `on_attack`, `on_attacked`
- (extensible)

**Effects generate Msgs → maintains TEA purity.**

### Event Cascade Order

When an event triggers effects that cause more events, resolution order is:

```
1. Event occurs (e.g., part structure reaches 0)
2. Collect all triggers listening for this event
3. Evaluate trigger conditions, filter to active triggers
4. Sort by priority (default 0, lower = earlier)
5. Execute effects in order, each generating Msgs
6. Process generated Msgs through Update
7. If Msgs cause new events, repeat from step 1

Cascade depth limit: 10 (prevents infinite loops)
```

**Priority field (optional):**

```yaml
triggers:
  - event: on_destroyed
    priority: -10  # runs before default triggers
    effect: eject_pilot
  - event: on_destroyed
    priority: 0    # default
    effect: cascade
    params: { target: mount_contents, event: on_destroyed }
```

**Determinism:** Same event + same model state = same cascade order (sorted by priority, then by entity ID for ties).

### System Invariants & Edge Cases

**This section defines behavior for every edge case. No undefined behavior allowed.**

#### 1. ValueRef Cycle Detection

**Problem:** `power: { ref: "self.power" }` creates infinite recursion.

**Policy:**
- Max evaluation depth: 16 references
- Cycle detection: Track visited refs, error on revisit
- On cycle/depth exceeded: Return 0, log warning, mark entity as "corrupted"

```
Evaluation of "self.power" where power refs self.damage which refs self.power:
  → self.power
  → self.damage (depth 1)
  → self.power (depth 2, CYCLE DETECTED)
  → Return 0, log: "Cycle in ValueRef: self.power → self.damage → self.power"
```

#### 2. Scope Binding: Snapshot at Trigger Fire

**Problem:** Item transfers mid-effect-chain. Does `unit` mean old or new unit?

**Policy:** All scope references are **snapshot at trigger fire**, not at effect execution.

```yaml
triggers:
  - event: on_combat_tick
    effects:
      - effect: transfer_item          # item moves to enemy
        params: { item: self, to: random_enemy }
      - effect: deal_damage
        params: { target: unit, amount: 5 }  # hits ORIGINAL unit, not new one
```

**Context object is immutable once trigger fires:**
```
TriggerContext {
  self: [snapshot of item at fire time]
  unit: [snapshot of unit at fire time]
  part: [snapshot of part at fire time]
  mount: [snapshot of mount at fire time]
  event: [event data]
  // These never change during effect chain execution
}
```

**Exception:** `ability.target` is set when ability is activated, not when trigger fires.

#### 3. Null Reference Handling

**Problem:** `{ ref: "event.killed.pilot" }` when killed unit had no pilot.

**Policy:** Null propagation with explicit fallbacks.

| Reference Result | Behavior |
|------------------|----------|
| Valid value | Use it |
| Null/missing | Use `default` if specified, else 0 for numbers, null for entities |
| Invalid path | Log warning, treat as null |

**Fallback syntax:**
```yaml
amount: { ref: "event.killed.pilot.skill", default: 0 }
```

**Effect behavior on null target:**
```yaml
- effect: deal_damage
  params:
    target: { ref: "event.killed.pilot" }  # null
    amount: 10
# Effect is SKIPPED (no-op), not error. Logged as: "Skipped deal_damage: null target"
```

#### 4. Effect Chain Semantics: Sequential Mutation

**Problem:** Do effects in a chain see intermediate state or snapshot?

**Policy:** **Sequential mutation.** Each effect mutates state, next effect sees the mutation.

```yaml
effects:
  - effect: set_attribute
    params: { target: self, attribute: temp, value: 100 }
  - effect: deal_damage
    params: { amount: { ref: "self.temp" } }  # sees 100
  - effect: set_attribute
    params: { target: self, attribute: temp, value: 0 }
  - effect: heal
    params: { amount: { ref: "self.temp" } }  # sees 0
```

**Rationale:** Sequential is more intuitive for modders. "Do A, then B, then C."

**However:** Scope context (unit, part, etc.) is still snapshotted. Only attributes mutate.

#### 5. Simultaneous Modification: Entity ID Tie-Breaker

**Problem:** Two items both SET heat to different values at same priority.

**Policy:** When same-priority effects conflict, resolve by **entity ID (alphabetical/numeric order)**.

```
item_a (id: "aaa") sets heat = 50
item_b (id: "bbb") sets heat = 0
Both at priority 0, same event.

Resolution order: aaa, then bbb.
Final heat = 0 (bbb wins because it runs second)
```

**Explicit in cascade order:**
```
4. Sort by priority (lower = earlier)
5. Within same priority, sort by entity ID (lexicographic)
6. Execute in order (last write wins for SET operations)
```

**Modder escape hatch:** Use different priorities to control order explicitly.

#### 6. Model Layers: Combat vs Run vs Meta

**Problem:** Where does between-combat state live?

**Policy:** Three distinct Model layers:

```
META_MODEL (persists across runs)
├── unlocked_starting_options: []string
├── achievements: []Achievement
└── settings: Settings

RUN_MODEL (persists within a run, reset on permadeath)
├── credits: int
├── inventory: []Item (items not on units)
├── units: []Unit (your lance)
├── pilots: []Pilot (not assigned)
├── run_flags: map[string]any (quest progress, etc.)
└── events_seen: []string

COMBAT_MODEL (exists only during combat)
├── player_units: []Unit (deployed)
├── enemy_units: []Unit
├── combat_tick: int
├── combat_log: []Event
└── combat_flags: map[string]any
```

**Attribute persistence:**
- Item/Unit attributes in `RUN_MODEL.units` persist between combats
- `combat_flags` reset each combat
- Use `run_flags` for cross-combat tracking

**Example: Grudge Keeper**
```yaml
item:
  id: grudge_keeper
  triggers:
    - event: on_kill
      effects:
        - effect: modify_attribute
          params: { target: self, attribute: enemies_killed, operation: add, amount: 1 }
        # enemies_killed persists because item is in RUN_MODEL
```

#### 7. Event Cancellation: Supported via Interception

**Problem:** Can effects prevent events from happening?

**Policy:** **Yes, via intercept events.** Some events have `on_incoming_X` variants that fire BEFORE the event resolves and can cancel it.

**Interceptable events:**
```
on_incoming_damage → fires before on_damaged
on_incoming_transfer → fires before on_item_transferred
on_incoming_death → fires before on_destroyed
```

**Cancellation effect:**
```yaml
item:
  id: damage_immunity_field
  triggers:
    - event: on_incoming_damage
      conditions:
        - { type: attr_lte, params: { target: event.source, attr: damage, value: 10 } }
      effects:
        - effect: cancel_event
        - effect: spawn_visual
          params: { type: shield_absorb }
```

**cancel_event behavior:**
- Stops the incoming event from resolving
- Stops all subsequent effects in this trigger (after cancel_event)
- Does NOT stop other triggers listening to same event (they see `event.cancelled = true`)

**Non-interceptable events:** `on_combat_start`, `on_turn_start`, `on_turn_end` (these are phase markers, not actions)

#### 8. Dynamic Ability/Trigger Creation: Scoped to Creator

**Problem:** Dynamically created abilities — whose `self` are they?

**Policy:** Dynamically created abilities/triggers inherit scope from their creator.

```yaml
item:
  id: ability_factory
  abilities:
    - id: create_power
      effects:
        - effect: add_ability
          params:
            target: unit
            ability:
              id: generated_blast
              scope_parent: { ref: "self" }  # REQUIRED: defines what "self" means
              effects:
                - effect: deal_damage
                  params: { amount: { ref: "scope_parent.power" } }  # factory's power
```

**Rules:**
- `scope_parent` is REQUIRED for dynamically created abilities
- `self` in dynamic ability = the entity it's attached to
- `scope_parent` = the entity that created it
- Dynamic abilities are removed when `scope_parent` is destroyed

#### 9. Template Immutability

**Problem:** Can effects modify templates?

**Policy:** **Templates are immutable.** Effects can only modify instances.

```yaml
# This is FORBIDDEN and will error:
- effect: modify_template  # NO SUCH EFFECT
  params: { template: "ac10", attribute: damage, value: 999 }

# This is allowed (modifies instance):
- effect: modify_base_attribute
  params: { target: self, attribute: damage, operation: add, amount: 1 }
```

**Rationale:** Template mutation would cause chaos — all future spawns would be affected. If you want "upgrade all AC10s", iterate over instances.

#### 10. Entity Type Boundaries

**Problem:** Can a Pilot become an Item? Can an Item become a Unit?

**Policy:** **Entity types are fixed.** No transmutation.

| From | To | Allowed? |
|------|----|----------|
| Item | Item | ✓ (transfer) |
| Item | Unit | ✗ |
| Unit | Item | ✗ |
| Pilot | Item | ✗ |
| Part | Unit | ✗ (use spawn_unit instead) |

**Workaround for "item becomes unit":**
```yaml
triggers:
  - event: on_some_condition
    effects:
      - effect: spawn_unit
        params:
          template: item_transformed_form
          copy_attributes_from: self  # copies relevant attributes
      - effect: destroy_item
        params: { target: self }
```

**Rationale:** Type transmutation breaks invariants. Spawn + destroy achieves same result safely.

#### 11. Maximum Limits (Anti-Abuse)

**Hard limits to prevent combinatorial explosion:**

| Limit | Value | On Exceed |
|-------|-------|-----------|
| Cascade depth | 10 | Stop cascade, log warning |
| ValueRef depth | 16 | Return 0, log warning |
| Effects per trigger | 32 | Error at load time |
| Triggers per entity | 64 | Error at load time |
| Abilities per entity | 16 | Error at load time |
| Modifiers per attribute | 128 | Oldest dropped, log warning |
| Units in combat | 32 per side | Cannot spawn more |
| Items per unit | 256 | Cannot mount more |

#### 12. Error Handling Philosophy

**Fail gracefully, log loudly, never crash.**

| Error Type | Behavior |
|------------|----------|
| Invalid ref path | Treat as null, log warning |
| Cycle detected | Return 0, log warning, mark corrupted |
| Effect on null target | Skip effect, log info |
| Missing required param | Skip effect, log error |
| Limit exceeded | Apply limit, log warning |
| Unknown effect type | Skip effect, log error |
| Unknown event type | Ignore trigger, log error |

**Corrupted entities:**
- Entities marked "corrupted" get a visual indicator in UI
- Corrupted entities still function (with fallback values)
- Player can see corruption in debug mode
- Corruption is saved (modders can see their mistakes in save files)

### Example Definitions

**Mech Left Arm:**
```yaml
part:
  id: atlas_left_arm
  tags: [bodypart, arm, left, appendage]
  attributes:
    armor: { base: 20 }
    structure: { base: 12 }
  connections:
    adjacent: [left_torso]
  mounts:
    - id: weapon_mount
      accepts: { requires_any: [ballistic, energy, physical] }
      capacity: 10
      capacity_attr: size
  triggers:
    - event: on_attribute_zero
      conditions: [{ type: attr_name_is, params: { name: structure } }]
      effect: cascade
      params: { target: mount_contents, event: on_destroyed }
```

**Mage Hand (different terminology, same system):**
```yaml
part:
  id: mage_left_hand
  tags: [bodypart, hand, left]
  attributes:
    health: { base: 5 }  # "health" not "structure"
  mounts:
    - id: grip
      accepts: { requires_any: [held, staff, wand, orb] }
      capacity: 2
      capacity_attr: hands_required
```

**Autocannon (with ammo dependency):**
```yaml
item:
  id: ac10
  tags: [weapon, ballistic, autocannon, ranged]
  attributes:
    size: { base: 7 }
    damage: { base: 10 }
    cooldown: { base: 3 }
  requires:
    - scope: unit
      condition: { type: has_item_with_tag, params: { scope: unit, tag: ac_ammo } }
      on_unmet: disabled
```

**Ammo with explosion risk:**
```yaml
item:
  id: ac10_ammo
  tags: [ammo, ac_ammo, explosive]
  attributes:
    size: { base: 1 }
    shots: { base: 10 }
  triggers:
    - event: on_crit
      effect: deal_damage
      params: { amount: 20, scope: part, splash: adjacent }
    - event: on_crit
      effect: destroy_item
```

**Heatsink (provides modifier):**
```yaml
item:
  id: double_heatsink
  tags: [equipment, heatsink, internal]
  attributes:
    size: { base: 3 }
  provides:
    - scope: unit
      attribute: heat_dissipation
      operation: add
      value: 2
      stack_group: null
```

**Targeting Computer (conditional modifier):**
```yaml
item:
  id: targeting_computer
  tags: [equipment, sensor, internal]
  attributes:
    size: { base: 2 }
  provides:
    - scope: unit
      scope_filter: [weapon]  # only affects weapons
      attribute: accuracy
      operation: add
      value: 2
      conditions:
        - { type: attr_lte, params: { target: unit, attr: heat, value: 50 } }
      # Bonus only when heat is manageable
```

**Command Module (cross-unit aura):**
```yaml
item:
  id: command_module
  tags: [equipment, command, support]
  attributes:
    size: { base: 4 }
  provides:
    - scope: adjacent_unit
      scope_filter: [friendly]
      attribute: initiative
      operation: add
      value: 1
      stack_group: command_aura  # only one command bonus per unit
```

### Active Abilities (Modder-Grade Examples)

**Damage-Storing Shield (absorb → release):**
```yaml
item:
  id: capacitor_shield
  tags: [equipment, shield, defensive]
  attributes:
    size: { base: 3 }
    stored_damage: { base: 0, max: 100 }
  triggers:
    # Passive: absorb blocked damage
    - event: on_damage_blocked
      effects:
        - effect: modify_attribute
          params:
            target: self
            attribute: stored_damage
            operation: add
            amount: { ref: "event.damage_blocked" }
  abilities:
    # Active: discharge stored damage
    - id: discharge
      tags: [attack, special]
      targeting:
        type: enemy
        range: 3
        count: 1
      costs:
        - attribute: stored_damage
          scope: self
          amount: { ref: "self.stored_damage" }  # costs all stored
      effects:
        - effect: deal_damage
          params:
            target: { ref: "ability.target" }
            amount: { ref: "self.stored_damage" }
            damage_type: energy
      cooldown: 0
      charges: -1
```

**Regeneration Symbiote (spawn new part on death):**
```yaml
item:
  id: regeneration_symbiote
  tags: [symbiote, biological, internal]
  attributes:
    size: { base: 2 }
    regen_charges: { base: 1 }
  triggers:
    - event: on_part_destroyed
      conditions:
        - { type: is_parent_part }
        - { type: attr_gte, params: { target: self, attr: regen_charges, value: 1 } }
      effects:
        - effect: modify_attribute
          params: { target: self, attribute: regen_charges, operation: add, amount: -1 }
        - effect: spawn_part
          delay: 3  # takes 3 ticks
          params:
            template: regenerated_arm
            attach_to: unit
            position: { ref: "event.destroyed_part.position" }
```

**Soul-Drinking Weapon (permanent scaling):**
```yaml
item:
  id: soul_drinker
  tags: [weapon, melee, cursed]
  attributes:
    size: { base: 4 }
    damage: { base: 8 }
    souls_consumed: { base: 0 }
  triggers:
    - event: on_kill
      conditions:
        - { type: has_tag, params: { target: event.killed, tag: organic } }
      effects:
        - effect: modify_base_attribute  # permanent, not a modifier
          params:
            target: self
            attribute: damage
            operation: add
            amount: 1
        - effect: modify_attribute
          params:
            target: self
            attribute: souls_consumed
            operation: add
            amount: 1
        - effect: spawn_visual
          params: { type: soul_absorb, at: { ref: "event.killed.position" } }
```

**Neural Parasite (jump to enemies):**
```yaml
item:
  id: neural_parasite
  tags: [equipment, parasite, biological]
  attributes:
    size: { base: 1 }
  triggers:
    # When mounted on enemy, debuff them
    - event: on_item_mounted
      conditions:
        - { type: has_tag, params: { target: unit, tag: enemy } }
      effects:
        - effect: apply_modifier
          params:
            target: unit
            attribute: accuracy
            operation: add
            amount: -3
            source: self
  abilities:
    # Active: jump to adjacent enemy
    - id: infest
      targeting:
        type: enemy
        range: 1  # adjacent only
        filter: [has_open_mount]  # must have valid mount
      conditions:
        - { type: in_combat }
      effects:
        - effect: transfer_item
          params:
            item: self
            from: { ref: "self.mount" }
            to: { ref: "ability.target" }
            mount_filter: [internal]
      cooldown: 10
      charges: -1
```

**Ejection System (emergency pilot save):**
```yaml
item:
  id: ejection_system
  tags: [equipment, cockpit, safety]
  attributes:
    size: { base: 2 }
  triggers:
    # Auto-eject when critical
    - event: on_attribute_threshold
      conditions:
        - { type: attr_lte, params: { target: unit, attr: structure_percent, value: 15 } }
      effects:
        - effect: spawn_unit
          params:
            template: escape_pod
            position: adjacent_friendly
            transfer_pilot: true
            transfer_from: { ref: "unit" }
        - effect: add_tag
          params:
            target: unit
            tag: pilot_ejected
      priority: -100  # runs before death triggers
```

**Berserker Core (damage self for power):**
```yaml
item:
  id: berserker_core
  tags: [equipment, engine, cursed]
  attributes:
    size: { base: 5 }
    rage_stacks: { base: 0, max: 10 }
  provides:
    # Damage scales with rage
    - scope: unit
      scope_filter: [weapon]
      attribute: damage
      operation: add
      value: { ref: "self.rage_stacks" }  # +1 damage per stack
  abilities:
    - id: blood_rage
      costs:
        - attribute: structure
          scope: unit
          amount: 5  # costs 5 structure
      effects:
        - effect: modify_attribute
          params:
            target: self
            attribute: rage_stacks
            operation: add
            amount: 1
      cooldown: 0
      charges: -1
      conditions:
        - { type: attr_gte, params: { target: unit, attr: structure, value: 10 } }
```

**Quantum Entangler (link two units):**
```yaml
item:
  id: quantum_entangler
  tags: [equipment, experimental, support]
  attributes:
    size: { base: 3 }
    entangled_with: { base: null }  # stores unit ID
  abilities:
    - id: entangle
      targeting:
        type: ally
        range: 5
        count: 1
      effects:
        - effect: set_attribute
          params:
            target: self
            attribute: entangled_with
            value: { ref: "ability.target.id" }
        - effect: create_link
          params:
            type: quantum_link
            from: { ref: "unit" }
            to: { ref: "ability.target" }
      cooldown: 0
      charges: 1
      charge_restore: on_combat_end
  triggers:
    # Share damage with linked unit
    - event: on_damaged
      conditions:
        - { type: attr_not_null, params: { target: self, attr: entangled_with } }
      effects:
        - effect: deal_damage
          params:
            target: { ref: "self.entangled_with" }
            amount: { expr: "event.damage_amount * 0.5" }
            damage_type: quantum
            bypass_armor: true
```

---

## Open Design Questions

### Architecture (resolved)
- ~~Parts system~~ → Universal Composition System
- ~~Modifier stacking~~ → stack_group field
- ~~Condition logic~~ → AND/OR/NOT boolean trees
- ~~Multi-mount items~~ → mounts_required attribute
- ~~Item dependencies~~ → requires field with conditions
- ~~Conditional modifiers~~ → conditions field on ProvidedModifier
- ~~Target context~~ → Standard context variables (self, target, source, unit, part, mount, combat)
- ~~Event cascade order~~ → Priority-based, deterministic, depth-limited
- ~~Active abilities~~ → First-class Ability type with costs, targeting, effects, charges
- ~~Dynamic values~~ → ValueRef system (static, reference, expression, random)
- ~~ValueRef cycles~~ → Max depth 16, cycle detection, return 0 on failure
- ~~Scope binding~~ → Snapshot at trigger fire (immutable context)
- ~~Null references~~ → Null propagation with fallbacks, skip effect on null target
- ~~Effect ordering~~ → Sequential mutation within chain
- ~~Simultaneous mods~~ → Entity ID tie-breaker (lexicographic)
- ~~Model layers~~ → Meta (cross-run) / Run (cross-combat) / Combat (per-fight)
- ~~Event cancellation~~ → on_incoming_X events + cancel_event effect
- ~~Dynamic abilities~~ → scope_parent required, inherits creator's scope
- ~~Template mutation~~ → Forbidden (templates immutable)
- ~~Entity transmutation~~ → Forbidden (spawn+destroy instead)
- ~~System limits~~ → Hard caps on cascade, refs, triggers, etc.
- ~~Error handling~~ → Fail gracefully, log loudly, never crash

### Content (deferred to implementation)
1. Exact combat width numbers (how many total slots?)
2. Specific chassis templates (part layouts, mount configs)
3. Weapon/item balance numbers
4. Faction subsystem designs (expansion)
5. Event/encounter variety and writing
6. Pilot trait list and effects (post-MVP)

### UI (deferred)
7. UI layout and information density
8. Pseudoterminal rendering details

---

## Implementation Scope

### System Invariants: MVP vs Full

| Invariant | MVP | Full | Notes |
|-----------|-----|------|-------|
| 1. ValueRef cycles | DEFER | ✓ | MVP uses static values mostly |
| 2. Scope snapshot | DEFER | ✓ | MVP has simple 1-2 effect triggers |
| 3. Null handling | SIMPLE | ✓ | Just skip + log |
| 4. Sequential effects | DEFER | ✓ | MVP has simple triggers |
| 5. Tie-breaker | DEFER | ✓ | Few items, won't collide |
| 6. Model layers | SIMPLE | ✓ | Just Combat model, no Meta/Run split |
| 7. Event cancellation | CUT | ✓ | No on_incoming_X needed |
| 8. Dynamic abilities | CUT | ✓ | No runtime ability creation |
| 9. Template immutability | TRIVIAL | ✓ | Just don't write the effect |
| 10. Entity boundaries | TRIVIAL | ✓ | Just don't write transmutation |
| 11. Limits | TRIVIAL | ✓ | Hardcode constants |
| 12. Error handling | SIMPLE | ✓ | Log, skip corruption tracking |

**MVP implements skeleton. Invariants added as complexity demands.**

---

## Directory Structure

### MVP (~15 files)

```
wulfaz/
├── cmd/
│   └── wulfaz/
│       └── main.go                 # Entry point
│
├── internal/
│   ├── tea/                        # TEA runtime
│   │   ├── runtime.go              # Main loop, Msg dispatch
│   │   ├── model.go                # Top-level Model
│   │   └── msg.go                  # Msg interface
│   │
│   ├── model/
│   │   └── combat.go               # Combat state (only layer for MVP)
│   │
│   ├── entity/                     # Core entities
│   │   ├── unit.go
│   │   ├── part.go
│   │   ├── mount.go
│   │   ├── item.go
│   │   └── pilot.go                # STUB: just name + id
│   │
│   ├── core/                       # Foundation types
│   │   ├── tag.go
│   │   ├── attribute.go            # Simple, no fancy modifiers
│   │   ├── trigger.go
│   │   ├── condition.go            # Leaf conditions only, no AND/OR
│   │   └── limits.go               # const block
│   │
│   ├── eval/
│   │   └── valueref.go             # Static values only for MVP
│   │
│   ├── event/
│   │   └── dispatch.go             # Simple trigger firing
│   │
│   ├── effect/
│   │   └── handler.go              # deal_damage, modify_attribute only
│   │
│   └── template/
│       ├── loader.go               # YAML loading
│       └── registry.go             # Template storage
│
├── data/
│   └── templates/
│       ├── units/
│       │   ├── small_mech.yaml
│       │   ├── medium_mech.yaml
│       │   └── large_mech.yaml
│       ├── items/
│       │   ├── medium_laser.yaml
│       │   ├── ac10.yaml
│       │   └── lrm5.yaml
│       └── pilots/
│           └── stub_pilot.yaml
│
├── ui/
│   └── renderer/
│       └── stub.go                 # Minimal rendering
│
├── go.mod
├── roguelike-design.md
└── tea-go-ruleset.md
```

### Full Structure (post-MVP)

```
internal/
├── tea/
│   ├── runtime.go
│   ├── model.go
│   └── msg.go
│
├── model/
│   ├── meta.go                     # POST-MVP: cross-run
│   ├── run.go                      # POST-MVP: within-run
│   └── combat.go
│
├── entity/
│   ├── unit.go
│   ├── part.go
│   ├── mount.go
│   ├── item.go
│   ├── pilot.go
│   └── trait.go                    # POST-MVP
│
├── core/
│   ├── tag.go
│   ├── attribute.go
│   ├── modifier.go                 # POST-MVP: full stacking logic
│   ├── trigger.go
│   ├── condition.go                # POST-MVP: AND/OR/NOT trees
│   ├── ability.go                  # POST-MVP
│   ├── requirement.go              # POST-MVP
│   ├── valueref.go                 # Move here from eval/
│   └── limits.go
│
├── eval/
│   ├── condition.go                # POST-MVP: bool tree evaluation
│   ├── valueref.go                 # POST-MVP: cycle detection, expressions
│   ├── modifier.go                 # POST-MVP: stack_group logic
│   └── context.go                  # POST-MVP: snapshot semantics
│
├── event/
│   ├── registry.go                 # POST-MVP
│   ├── dispatch.go
│   ├── cascade.go                  # POST-MVP: depth tracking
│   └── intercept.go                # POST-MVP: on_incoming_X
│
├── effect/
│   ├── registry.go                 # POST-MVP
│   ├── handler.go
│   ├── damage.go                   # POST-MVP: split out
│   ├── attribute.go                # POST-MVP: split out
│   ├── spawn.go                    # POST-MVP
│   ├── transfer.go                 # POST-MVP
│   ├── destroy.go                  # POST-MVP
│   └── ability.go                  # POST-MVP: add_ability
│
├── combat/
│   ├── tick.go                     # POST-MVP
│   ├── targeting.go                # POST-MVP
│   ├── ai.go                       # POST-MVP
│   └── resolution.go               # POST-MVP
│
├── template/
│   ├── loader.go
│   ├── registry.go
│   ├── instantiate.go              # POST-MVP
│   └── validate.go                 # POST-MVP
│
├── save/
│   ├── json.go                     # POST-MVP
│   ├── snapshot.go                 # POST-MVP
│   └── replay.go                   # POST-MVP
│
├── debug/
│   ├── log.go                      # POST-MVP
│   ├── corruption.go               # POST-MVP
│   └── timewarp.go                 # POST-MVP
│
└── pkg/
    └── rng/
        └── seeded.go               # POST-MVP (use stdlib for MVP)
