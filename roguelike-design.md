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
```

### Structural Hierarchy

```
UNIT
├── id, template_id
├── tags: []Tag
├── attributes: map[string]Attribute
├── parts: map[string]Part
├── triggers: []Trigger
└── pilot: *Pilot (optional)

PART
├── id, template_id
├── tags: []Tag
├── attributes: map[string]Attribute
├── mounts: []Mount
├── connections: map[string][]string  // relationship_type → part_ids
└── triggers: []Trigger

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
├── provides: []ProvidedModifier
└── requires: []Requirement (optional, dependencies to function)

PILOT
├── id, name
├── tags: []Tag
├── attributes: map[string]Attribute
├── traits: []Trait
└── triggers: []Trigger

TRAIT
├── id
├── tags: []Tag
├── triggers: []Trigger
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
