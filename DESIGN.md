# Mech Autobattler Roguelike — Design Document

**Status:** Major redesign — dice-based combat, cooldowns removed

---

## Core Identity

- **Genre Hybrid:** Traditional roguelike + Autobattler + Deckbuilder elements
- **Turn-based:** Yes (dice-based, simultaneous resolution per position)
- **Permadeath:** Yes
- **Complex systems:** Yes (BTA-level depth)
- **Primary Hook:** Deep build customization IS the game
- **Run Length:** 45-60 minutes target
- **Random Starts:** Yes (not full class selection)
- **Power Fantasy:** Broken builds rewarded, not balanced into blandness

### Inspirations

| Game | What to Take |
|------|--------------|
| The Bazaar | Build customization as core fun, spatial item relationships, simultaneous resolution |
| Slice & Dice | Quick runs, dice mechanics, lock/reroll decisions, random starts |
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

### Core Concept

All combat is dice-based. Every unit (including command units) rolls dice each round. Size determines dice count. The player's command unit has lock/reroll mechanics for tactical decisions.

### Structure

- **Style:** Dice-based autobattler with simultaneous resolution
- **Positioning:** Single row (10 spaces per side)
- **Control:** Hands-off during execution — strategic decisions happen in dice phase

### Size = Dice

| Size | Spaces | Dice |
|------|--------|------|
| Small | 1 | 1d6 |
| Medium | 2 | 2d6 |
| Large | 3 | 3d6 |

### Round Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. PREVIEW PHASE                                            │
│    All dice roll (visually shown for enemy command unit)    │
│    Display: all unit dice + enemy command unit dice         │
│    Show targeting lines for all units                       │
│    Player command unit dice NOT shown (interactive)         │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. PLAYER COMMAND PHASE                                     │
│    Player command unit rolls its dice                       │
│    Lock/unlock individual dice                              │
│    Spend rerolls (rerolls all unlocked dice)                │
│    Activate dice one by one (choose order, targets)         │
│    Effects fire immediately (Shield/Heal before execution)  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. ENEMY COMMAND PHASE                                      │
│    Enemy command unit activates declared dice               │
│    Skips if target already dead                             │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. EXECUTION PHASE                                          │
│    Units fire left → right by board position                │
│    Both sides at same position fire simultaneously          │
│    Each unit fires once (their dice result)                 │
│    Player watches                                           │
└─────────────────────────────────────────────────────────────┘
                            ↓
                      (repeat from 1)
```

### Targeting

**Positional targeting:** Units hit enemy unit(s) at the same board position.

**Overflow (MTG-style):** Damage kills lowest HP unit first, excess carries over to next unit at that position.

**Gaps:** If no enemy unit at position, damage hits enemy command unit.

**Units only target units:** Command unit only targetable when all enemy units are dead.

**Command unit targeting:** Command unit dice can target anything (any enemy unit, enemy command unit).

### Win Condition

**Destroy the enemy command unit.** Units don't determine victory — losing all units doesn't end combat.

**Player wins ties:** If both command units die simultaneously, player wins.

---

## Positioning

```
YOUR TEAM (facing up)
┌─────────────────────────────────────────────┐
│  Board: [████████████████████] (10 spaces)  │
│                                             │
│   MECH A (Medium)    MECH B (Medium)        │
│      [2d6]              [2d6]               │
│                                             │
│   (2 spaces)           (2 spaces)           │
└─────────────────────────────────────────────┘

     [ COMMAND UNIT (Large, 3d6) ]
        (off-board, behind line)

ENEMY TEAM (facing down)
┌─────────────────────────────────────────────┐
│   MECH (Medium)   SMALL×2      VEHICLE      │
│      [2d6]        [1d6][1d6]    [2d6]       │
│                                             │
│   (2 spaces)     (1 + 1 space) (2 spaces)   │
└─────────────────────────────────────────────┘

     [ ENEMY COMMAND UNIT (Large, 3d6) ]
        (off-board, behind line)
```

### Positioning Rules

- Single row only (10 spaces total per side)
- Position determines which enemy you hit (same position)
- Command units are off-board, not part of the 10 spaces
- Position set between fights; free repositioning between fights

---

## Unit System

### Unit Types

- **Mechs:** Primary customizable units (all sizes)
- **Vehicles:** Support units (small/medium)
- **Battle Armor:** Infantry squads (small)
- **Command Unit:** Player/enemy flagship, has lock/reroll (large for MVP)
- **Other:** Faction-specific (golems, bio-beasts, etc.)

### Command Unit

The command unit is a unit with special properties:

- **Size:** Large (3 spaces, 3 dice) for MVP
- **Lock/Reroll:** Only the player's command unit has this mechanic
- **Dice Types:** Fixed Damage/Shield/Heal for MVP
- **Targeting:** Can target any enemy unit or enemy command unit
- **Off-board:** Not part of the 10 spaces, can always be targeted by command dice

---

## Damage Model

### Simple HP

Units have a single HP pool. Damage reduces HP. At 0 HP, the unit is destroyed and removed from play.

No hit locations, armor/structure layers, critical hits, or complex death states. Death is immediate removal.

### Shields

- Granted by Shield dice
- Absorb damage before HP (overflow hits HP)
- Stack additively
- Expire at end of round

---

## Dice System

### Unit Dice (MVP)

- All units roll damage dice
- No reroll — roll once, that's your result
- Face distribution (MVP): `[2, 2, 3, 4, 0, 0]` (lower values + blanks)
- Defined in unit template, data-driven

### Command Unit Dice (MVP)

- 3 dice: 1 Damage, 1 Shield, 1 Heal (fixed for MVP)
- Face distribution: `[5, 5, 8, 12, 0, 0]` (same for all types)
- Lock/reroll mechanics (player only)
- Rerolls: 2 per round (global pool)

### Dice Effects

**Damage:**
- Target: any enemy unit OR enemy command unit
- Effect: deal damage to target

**Shield:**
- Target: any friendly unit OR player command unit
- Effect: grant shields that absorb damage
- Shields expire at end of round

**Heal:**
- Target: any friendly unit only (NOT command unit for MVP)
- Effect: restore HP, capped at max HP

### Lock/Reroll (Player Command Unit Only)

1. Roll all dice
2. Toggle lock on individual dice (locked dice keep their face)
3. Spend reroll → all unlocked dice reroll
4. Repeat until satisfied or out of rerolls
5. Out of rerolls → remaining unlocked dice auto-lock
6. Activate dice one by one (choose order, targets)
7. Effects fire immediately

---

## Pilots / Crew [NOT IN MVP]

Pilots and crew are unified as items that modify units. They are out of scope for MVP.

### Full Vision

- Assignable to units (battlefield) or command unit
- Provide stat modifiers, traits, abilities
- Draftable resource (recruit at shops/events)
- Traits affect dice, targeting, behavior

---

## Run Structure (Bazaar-style)

### Flow

```
Start (random loadout + fixed options)
    ↓
Multi-choice event/shop phase (pick 2 of 3)
    ↓
Multi-choice: select next battle (pick difficulty from 3)
    ↓
Combat (dice-based)
    ↓
Salvage/results
    ↓
Repeat until: ~10 fights → final boss → win, OR death
```

Full vision includes detours and optional cash-outs.

### Between-Fight Verbs

- **Draft:** New components/units from events
- **Shop:** Buy with credits
- **Upgrade:** Pay to level up gear
- **Salvage:** From defeated enemies
- **Refit:** Rearrange loadouts freely
- **Repair:** Button + credits
- **Sell:** Inventory management

### MVP Flow

```
Fight 1 → 2 events → Fight 2 → 2 events → Fight 1 → ... (loops indefinitely until death)
```

Events: Pick 2 of 3 options. Then pick fight difficulty from 3 options. Shops and events are conceptually separate but share stubbed implementation for MVP.

---

## Tech Stack

- **Architecture:** TEA (The Elm Architecture) in Go
- **Reference:** See `CLAUDE.md` for TEA principles and rules
- **Platform:** Desktop GUI (Ebitengine)
- **Data Format:** KDL 1.0 for templates (via github.com/sblinch/kdl-go)
- **Rendering:** Ebitengine 2D (github.com/hajimehoshi/ebiten/v2)

### Architectural Constraint: Seeded RNG

**All randomness must be external to Update — passed in via Msgs.**

```go
type RoundStarted struct {
    UnitDice    map[string][]int  // Pre-rolled dice per unit
    EnemyCmdDice []int            // Enemy command unit dice
}

func (m Model) Update(msg Msg) (Model, Cmd) {
    // Use pre-rolled values from Msg
    // Deterministic: same Msg = same result
}
```

This enables: full replay, turn-level undo, time-travel debugging, deterministic tests.

---

## MVP Scope

### Content

| Area | MVP Scope |
|------|-----------|
| Factions | 1 (tech), stub second |
| Chassis | 3 (Small, Medium, Large mech) |
| Fights | 2 (alternating in infinite loop) |
| Shop/Event Phases | 2 (between fights, pick 2 of 3) |
| Pilots/Crew | Out of scope |
| Items/Mounts | Out of scope |
| Unit Pools | Symmetric (player/enemy share same pool) |
| Win Condition | None (loop until death or quit) |

### Combat MVP

| Element | MVP Value |
|---------|-----------|
| Player command unit | Large (3 dice: Damage/Shield/Heal) |
| Enemy command unit | Mirror (same as player) |
| Unit dice | All damage, distribution `[2, 2, 3, 4, 0, 0]` |
| Command dice distribution | `[5, 5, 8, 12, 0, 0]` (all types) |
| Rerolls per round | 2 |
| Unit targeting | Positional (same board position) |
| Overflow | MTG-style (lowest HP first) |
| Gaps | Hit command unit |
| Damage persistence | Carries forward between fights |
| Destroyed units | Gone forever (permadeath) |
| Simultaneous resolution | Yes (per position) |

### Explicit Cuts (Not in MVP)

- Multiple factions
- Meta-progression
- Sound
- Persistent returning enemies
- Cash-out system
- Complex win conditions
- Pilots/crew (entire system)
- Items/Mounts (entire system)
- Active abilities (separate from dice)
- Retreat mechanic
- Speed controls (pause only)
- Varied dice types per unit (all damage for MVP)

---

## Universal Composition System

**Everything is Tags, Attributes, and Dice. No special types. No hardcoding.**

### Structural Hierarchy (MVP)

```
UNIT
├── ID, TemplateID
├── Tags: []Tag
├── Attributes: map[string]Attribute
│   ├── spaces (1/2/3 for S/M/L)
│   ├── health
│   ├── max_health
│   └── shields
├── Dice: []Die
│   ├── Type (damage/shield/heal)
│   └── Faces: []int
└── Position: int (board position)
```

### Full Hierarchy (Post-MVP)

```
UNIT
├── ID, TemplateID
├── Tags: []Tag
├── Attributes: map[string]Attribute
├── Parts: map[string]Part  [NOT IN MVP]
├── Dice: []Die
└── Position: int

PART [NOT IN MVP]
├── ID, TemplateID
├── Tags: []Tag
├── Attributes: map[string]Attribute
├── Mounts: []Mount
└── Connections: map[string][]string

MOUNT [NOT IN MVP]
├── ID
├── Tags: []Tag
├── Accepts: MountCriteria
├── Capacity: int
└── Contents: []Item

ITEM [NOT IN MVP]
├── ID, TemplateID
├── Tags: []Tag
├── Attributes: map[string]Attribute
├── ProvidedModifiers: []ProvidedModifier
└── DiceModifiers: []DiceModifier
```

### Modifier Resolution

```
1. Collect all active modifiers for attribute
2. Group by StackGroup, keep highest value from each group
3. Apply in order: SET → ADD → MULT → MIN → MAX
```

---

## System Invariants & Edge Cases

| Invariant | MVP | Full |
|-----------|-----|------|
| Null handling | Skip + log | Full propagation with fallbacks |
| Tie-breaker | Player wins | Entity ID (lexicographic) |
| Model layers | Combat only | Meta/Run/Combat split |
| Error handling | Log + skip | + corruption tracking |
| Simultaneous death | Both die | Same |
| Overflow damage | MTG-style | Same |

---

## Deferred

### Architecture (Post-MVP)

| Feature | MVP Behavior | Post-MVP |
|---------|--------------|----------|
| Dice types per unit | All damage | Varied types, data-driven |
| Lock/reroll | Command unit only | Potentially other sources |
| Items/Mounts | Not implemented | Full customization system |
| Pilots/Crew | Not implemented | Modifier items for units |
| Condition logic | Leaf-only | AND/OR/NOT boolean trees |
| ValueRef | Static int | Expressions, references |

### Content (Deferred to Implementation)

1. Specific chassis templates
2. Dice balance numbers
3. Faction subsystem designs (expansion)
4. Event/encounter variety and writing
5. Pilot/crew trait list and effects

### UI (Post-MVP)

- Custom fonts and sprites (currently using debug text)
- Health bars and damage numbers
- Dice rolling animations
- Menu and shop screens

### Template Loading (Post-MVP)

- Absolute template paths (currently relative, requires running from repo root)
- Graceful template errors (currently `log.Fatalf` on missing templates)

---

## KDL Template Examples

### Unit Template

```kdl
unit id="medium_mech" {
    tags "mech" "medium"
    attributes {
        attribute name="spaces" base=2
        attribute name="health" base=50 min=0
        attribute name="max_health" base=50
    }
    dice {
        die type="damage" faces="2,2,3,4,0,0"
        die type="damage" faces="2,2,3,4,0,0"
    }
}
```

### Command Unit Template

```kdl
unit id="player_command" {
    tags "command" "large"
    attributes {
        attribute name="spaces" base=3
        attribute name="health" base=100 min=0
        attribute name="max_health" base=100
    }
    dice {
        die type="damage" faces="5,5,8,12,0,0"
        die type="shield" faces="5,5,8,12,0,0"
        die type="heal" faces="5,5,8,12,0,0"
    }
    lock_reroll rerolls=2
}
```

### Item Template (Post-MVP)

```kdl
item id="targeting_computer" {
    tags "equipment" "electronics"
    attributes {
        attribute name="size" base=1
    }
    dice_modifiers {
        modifier die_index=0 face_index=4 value=3  // Replace a 0 with 3
    }
}
```

---

## Enemy AI

### MVP Behavior

- Lock good faces (non-zero), reroll bad faces (zeros)
- Random targeting for damage dice
- Random targeting for heal/shield dice

### Display

- Player sees enemy's final dice results and targeting lines
- Enemy command unit rolling happens before preview phase (visual only)
- Lines/arrows show planned targets

### Future

- Per-commander personalities (aggressive, defensive, greedy)
- Smarter targeting priorities
- Predictable patterns players can learn

---

## Key Properties

- **Data-driven:** All dice types, face distributions defined in templates
- **Extensible:** New dice types, face effects can be added
- **Information asymmetry:** Player sees enemy's plan before acting
- **Preemptive counterplay:** Shield a unit before enemy damage hits, kill target before enemy heal lands
- **Risk/reward:** Reroll economy creates gambling moments
- **Layered agency:** Passive satisfaction (watching execution) + active decisions (command dice)
- **Position matters:** Board position determines which enemy you hit
- **Size matters:** Larger units = more dice = more damage potential
