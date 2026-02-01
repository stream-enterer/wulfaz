# Mech Autobattler Roguelike — Design Document

**Status:** MVP loop complete — Combat → Rewards → Fight Selection → Combat

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
- Adjacency matters heavily (buffs/auras, splash damage, targeting)
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

---

## Pilots

### Core Design

- Swappable between units (component-like)
- Level up during run, gain traits
- Draftable resource (recruit at shops/events)
- Compatibility bonuses/maluses with unit types
- NO affinity building (pure stat matching)

### Pilot Traits (POST-MVP)

- Affect AI decisions during combat
- Influence targeting, retreat behavior, aggression
- Chat bubbles explain reasoning when traits proc

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

### MVP Flow

```
Fight 1 → Shop/Event → Shop/Event → Fight 2 → Game Over/Reset
```

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
type CombatTicked struct {
    Rolls []int  // Pre-generated values
}

func (m Model) Update(msg Msg) (Model, Cmd) {
    // Use msg.Rolls[n] instead of rand()
    // Deterministic: same Msg = same result
}
```

This enables: full replay, turn-level undo, time-travel debugging, deterministic tests.

**Runtime dispatch:** Recursive (implicit policy) vs queue (explicit policy) — kept recursive for MVP simplicity.

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

### Explicit Cuts (Not in MVP)

- Multiple factions
- Meta-progression
- Sound
- Persistent returning enemies
- Pilot objectives/cash-out system
- Complex win conditions
- Pilot traits (beyond stubs)

---

## Universal Composition System

**Everything is Tags, Attributes, and Triggers. No special types. No hardcoding.**

### Structural Hierarchy

```
UNIT
├── ID, TemplateID
├── Tags: []Tag
├── Attributes: map[string]Attribute
├── Parts: map[string]Part
├── Triggers: []Trigger
├── Abilities: []Ability
├── Pilot: Pilot
└── HasPilot: bool

PART
├── ID, TemplateID
├── Tags: []Tag
├── Attributes: map[string]Attribute
├── Mounts: []Mount
├── Connections: map[string][]string
├── Triggers: []Trigger
└── Abilities: []Ability

MOUNT
├── ID
├── Tags: []Tag
├── Accepts: MountCriteria
├── Capacity: int
├── CapacityAttribute: string (default "size")
├── MaxItems: int (-1 = unlimited)
├── Locked: bool
└── Contents: []Item

ITEM
├── ID, TemplateID
├── Tags: []Tag
├── Attributes: map[string]Attribute
├── Triggers: []Trigger
├── Abilities: []Ability
├── ProvidedModifiers: []ProvidedModifier
└── Requirements: []Requirement
```

### Modifier Resolution

```
1. Collect all active modifiers for attribute
2. Group by StackGroup, keep highest value from each group
3. Apply in order: SET → ADD → MULT → MIN → MAX
```

### Event Cascade Order (Task Pattern)

Uses TEA Task Pattern for sequential effects via intermediate Msgs:

```
CombatTicked{Rolls}
    ↓
Update: dispatch on_combat_tick to all units
    ↓
TriggersCollected{Triggers, Depth: 0}
    ↓
Update: Handle() each effect, merge results
    ↓
EffectsResolved{ModifiedUnits, FollowUpEvents}
    ↓
Update: apply changes, dispatch follow-ups
    ↓
TriggersCollected{Depth: 1}  (if follow-ups exist)
    ↓
... repeat until no follow-ups or depth >= 10
```

**Dispatch traversal:** Unit triggers → Parts (sorted) → Mounts → Items

**Determinism:** Parts sorted alphabetically, enemies sorted by ID

---

## System Invariants & Edge Cases

Full details in previous version. Key policies:

| Invariant | MVP | Full |
|-----------|-----|------|
| ValueRef cycles | DEFER (static values) | Max depth 16, cycle detection |
| Scope snapshot | DEFER | Snapshot at trigger fire |
| Null handling | Skip + log | Full propagation with fallbacks |
| Effect ordering | DEFER | Sequential mutation |
| Tie-breaker | DEFER | Entity ID (lexicographic) |
| Model layers | Combat only | Meta/Run/Combat split |
| Event cancellation | CUT | on_incoming_X + cancel_event |
| Dynamic abilities | CUT | scope_parent required |
| Limits | Hardcode constants | Same |
| Error handling | Log + skip | + corruption tracking |

---

## Deferred

### Architecture (Post-MVP)

| Feature | MVP Behavior | Post-MVP |
|---------|--------------|----------|
| Condition logic | Leaf-only (has_tag, attr_*) | AND/OR/NOT boolean trees |
| ValueRef | Static int | Expressions, references (`self.damage`) |
| Modifier stacking | Not implemented | stack_group logic |
| Event cancellation | Not implemented | on_incoming_X + cancel_event |
| Splash damage | Same as deal_damage | Radius affects adjacent units |
| Target resolution | First enemy (alphabetical) | AI/priority-based selection |
| Damage model | Unit-level health | Per-part armor/structure (BTA) |
| Attribute merging | Last write wins | Delta accumulation |
| Ally targeting | Self | Proper ally selection |
| No-target feedback | Silent no-op (correct for error handling) | Player-facing log: "laser fired but target destroyed" |
| Model layers | Combat only | Meta/Run/Combat split |
| Error handling | Log + skip | Corruption tracking |
| Nested modifications | Unit-level attributes only | Full unit serialization (Currently item attribute changes like ammo consumption are lost after the effect chain completes) |

### Naming/Types (Post-MVP)

- `map[string]any` for Condition.Params and EffectRef.Params is loose typing; consider typed param structs when patterns emerge
- `Cost.Attribute` is a string; could become typed `AttributeName` if attribute set stabilizes

### Content (Deferred to Implementation)

1. Exact combat width numbers (how many total slots?)
2. Specific chassis templates (part layouts, mount configs)
3. Weapon/item balance numbers
4. Faction subsystem designs (expansion)
5. Event/encounter variety and writing
6. Pilot trait list and effects

### UI (Post-MVP)

- Custom fonts and sprites (currently using debug text)
- Health bars and damage numbers
- Animation system for attacks
- Menu and shop screens

### Template Loading (Post-MVP)

- Absolute template paths (currently relative, requires running from repo root)
- Graceful template errors (currently `log.Fatalf` on missing templates)
- Separate part templates (currently inline-only, `Part.TemplateID` is cosmetic)

### Runtime/Platform Integration (Post-MVP)

Currently `app/app.go` and `tea/runtime.go` are parallel implementations:
- `Runtime.Dispatch()` is a test helper with correct dispatch logic
- `App.dispatch()` is the Ebitengine driver that reimplemented dispatch

This duplication led to a bug (infinite loop from `for` vs `if`). The layering is sound:
- **Ebitengine**: platform layer (window, input, rendering, frame timing)
- **TEA**: application layer (state, pure updates, effects)

Post-MVP options:
- **Composition**: `App` embeds/owns a `Runtime` and delegates dispatch
- **Shared helper**: Extract dispatch loop into a function both use
- **Unified Runtime**: Single `Runtime` with platform hooks, backends just provide I/O

Currently acceptable for single-platform MVP; would matter if adding TUI/web backends.

---

## KDL Template Examples

### Unit Template

```kdl
unit id="medium_mech" {
    tags "mech" "medium"
    attributes {
        attribute name="combat_width" base=2
        attribute name="health" base=100 min=0
    }
    parts {
        part id="left_arm" template_id="mech_arm" {
            tags "arm" "left"
        }
        part id="right_arm" template_id="mech_arm" {
            tags "arm" "right"
        }
        part id="torso" template_id="mech_torso" {
            tags "torso" "center"
        }
    }
}
```

### Item Template

```kdl
item id="medium_laser" {
    tags "weapon" "energy" "laser"
    attributes {
        attribute name="size" base=1
        attribute name="damage" base=5
        attribute name="heat" base=3
        attribute name="cooldown" base=2
    }
    triggers {
        trigger event="on_combat_tick" effect_name="deal_damage" {
            params damage=5 target="enemy"
        }
    }
}
```

### Item with Requirements

```kdl
item id="ac10" {
    tags "weapon" "ballistic" "autocannon"
    attributes {
        attribute name="size" base=7
        attribute name="damage" base=10
    }
    requirements {
        requirement scope="unit" on_unmet="disabled" {
            condition type="has_item_with_tag" tag="ac_ammo"
        }
    }
}
```

### Item with ProvidedModifiers

```kdl
item id="double_heatsink" {
    tags "equipment" "heatsink"
    attributes {
        attribute name="size" base=3
    }
    provided_modifiers {
        modifier scope="unit" attribute="heat_dissipation" operation="add" value=2
    }
}
```

---

## Hybrid Combat System

Combines cooldown-based autobattler execution with Slice & Dice tactical dice rolling.

### Design Goals

| Source | What We Want |
|--------|--------------|
| **Cooldowns/Autobattler** | Rube Goldberg satisfaction—build a machine, watch it execute. Cascading effects, emergent combos (Peggle-like). Passive enjoyment during execution. |
| **Slice & Dice** | Tactical dice decisions—evaluate rolls, lock/reroll, risk rare outcomes. Active decision moments. The hit comes from seeing what fate gave you and choosing what to keep. |

### Two-Layer Architecture

| Layer | Build Phase | Combat Phase |
|-------|-------------|--------------|
| **Battlefield Units** | Loadout (gear, weapons, stats) | Automatic execution, timeline-driven |
| **Command Ship** | Rooms with dice, optional crew modifiers | Player rolls, locks, rerolls, activates |

**Battlefield Units:** You build their loadouts. During combat, they fight automatically. A timeline sweeps left-to-right; units are only active while the timeline is within their bounds. You watch.

**Command Ship:** You build your ship's rooms (each room has a die). Optionally staff rooms with crew for bonuses. Each round, you do the full S&D dance—roll, evaluate, lock, reroll, activate.

### Battlefield Layout

```
     |.|           ← Enemy command ship (off-field, behind line, visually x-axis centered)
|...........|         ← Enemy units (combat width 10)
|...........|         ← Player units (combat width 10)
     |.|           ← Player command ship (off-field, behind line, visually x-axis centered)
```

Command ships are visually present but not part of the combat width. They have rooms with HP but no position on the timeline.

### Round Flow

```
┌─────────────────────────────────────────────────────────┐
│ 1. ENEMY DECLARATION PHASE                              │
│    Enemy rolls dice, locks, declares activations        │
│    Shows planned targets (information advantage)        │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│ 2. PLAYER COMMAND PHASE                                 │
│    See enemy's declared plan                            │
│    Roll your ship dice                                  │
│    Lock/unlock individual dice                          │
│    Spend rerolls (rerolls all unlocked dice)            │
│    Out of rerolls → auto-lock remaining                 │
│    Activate dice one by one (choose order, targets)     │
│    Effects fire immediately on activation               │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│ 3. ENEMY EXECUTION PHASE                                │
│    Enemy activates their declared dice                  │
│    Skips if target died or room was destroyed           │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│ 4. EXECUTION PHASE (automatic)                          │
│    Timeline sweeps left → right                         │
│    Units fire on cooldown while timeline touches them   │
│    Player watches (Rube Goldberg satisfaction)          │
└─────────────────────────────────────────────────────────┘
                          ↓
                    (repeat from 1)
```

### Timeline Mechanics

The execution phase uses the existing tick system internally.

**Timeline gates activation:**
- A vertical line sweeps left-to-right across the combat width
- Units are only active while the timeline is within their bounds
- Unit size directly determines action time:
  - Small (1 width): active for 1 slot's worth of ticks
  - Medium (2 width): active for 2 slots' worth of ticks
  - Large (3 width): active for 3 slots' worth of ticks

**One shared timeline:** Both sides use the same timeline. Position determines when units activate—position 0 acts first, position 9 acts last.

**Dead units leave gaps:** If a unit dies mid-round, it leaves empty space. Timeline keeps sweeping. No repositioning mid-round.

**Ticks per slot:** 8 ticks per slot (placeholder, tune later). Combat width 10 = 80 ticks per full sweep.
- Small unit (1 width): 8 ticks of action
- Medium unit (2 width): 16 ticks of action
- Large unit (3 width): 24 ticks of action

**Cooldown behavior:**
- Cooldowns don't tick until timeline reaches the unit
- When timeline reaches unit, all cooldowns start at their base values
- Cooldowns reset each round (fresh start when timeline reaches unit again)
- Countdown first: weapon fires when cooldown reaches 0, not immediately (cooldown 4 = fires on tick 4)
- Future: some weapons can have "ready" tag to fire immediately

**Multiple weapons:**
- Each weapon has independent cooldowns (tick separately)
- All weapons on a unit target the same enemy (nearest) for MVP
- Heat system to limit simultaneous firing [NOT IN MVP]

### Command Ship Structure

```
COMMAND SHIP
├── Room 1
│   ├── HP (room health)
│   ├── Die (6 faces)
│   ├── Shields (absorb damage, expire each round)
│   └── Crew (optional modifier) [NOT IN MVP]
├── Room 2
│   ├── HP
│   ├── Die
│   ├── Shields
│   └── Crew [NOT IN MVP]
├── Room 3
│   └── ...
└── (Abilities from rooms/crew/progression) [NOT IN MVP]
```

**Rooms are the ship's HP.** Destroy all rooms = destroy the ship. No separate hull HP.

**Targeted damage:** When attacking enemy ship, choose which room to damage. Strategic choice—knock out their healer die, their damage die, etc.

**Room count:** Fixed per ship type (3 for MVP).

### Ship Combat Rules

**Ships can always attack each other** with dice. No need to clear enemy units first.

**Units can only attack enemy ship** after all enemy units are dead.

**Strategic axis:**
- Support ground troops (heals, shields) → win unit battle → then finish ship
- Nuke enemy ship directly → ignore units → race to kill ship
- Balance both based on game state

### Dice Mechanics

**Dice types (MVP):** Damage, Shield, Heal (3 types, one per room)

**Faces per die:** 6

**Face distribution (all dice, MVP):** `[5, 5, 8, 12, 0, 0]`
- Two 5s, one 8, one 12, two blanks (0 = do nothing)
- Scaled to match ground unit values (weapons deal 5-8 damage)
- Symmetric across all dice types for MVP

**Rerolls:** Global pool per round (2 for MVP). Spend one to reroll all unlocked dice.

**Lock/reroll loop:**
1. All dice roll once
2. Toggle lock on individual dice (locked dice keep their face)
3. Spend reroll → all unlocked dice reroll
4. Repeat until satisfied or out of rerolls
5. Out of rerolls → remaining unlocked dice auto-lock

**Activation:**
- After locking, activate dice one by one
- You choose activation order freely
- When activating, choose target (if face requires one)
- Effect fires immediately
- You can skip activating a die (useful for blanks or negative faces)

**Why skip activation:**
- Blank face (0)—does nothing
- No valid target (e.g., heal but no damaged allies)
- Negative face (future: some dice have bad faces—lock to stop rerolls, don't activate)
- Save mana (some powerful faces cost mana) [NOT IN MVP]

**Negative faces (future):** Full design space includes self-damage, buff enemy, resource drain. MVP only has blanks.

### Dice UI Interaction

**Two distinct phases:**

**1. Lock Phase:**
- All dice start in "rolling area"
- Click die in rolling area → moves to "fixed area" (locked)
- Click die in fixed area → moves back to rolling area (unlocked)
- "Reroll" button rerolls all dice in rolling area
- Phase ends when: all dice manually locked OR out of rerolls (remaining auto-lock)

**2. Activate Phase:**
- All dice now in fixed area
- Click die → pick target (if needed) → effect fires immediately
- Blanks (0) are auto-skipped
- Player can skip non-blank dice by not activating them
- "End Turn" button available at any time:
  - If unactivated non-blanks remain: first click shows "Are you sure?", second click confirms
  - If all dice handled: single click ends turn
- Sequential activation: one die at a time, resolve before next

**Room display:** Rooms shown in visual order (left/middle/right) but no mechanical difference in MVP.

**Future:** Certain tags will force activation or auto-skip specific dice.

### Dice Effects

**Damage:**
- Target: any enemy unit OR any specific enemy room (explicit targeting, not "the ship")
- Effect: deal damage to target

**Shield:**
- Target: any friendly unit OR any specific friendly room
- Effect: grant shields that absorb damage
- Shields stack additively
- Shields absorb damage before HP (overflow hits HP)
- Shields expire at end of round

**Heal:**
- Target: any friendly unit only (NOT rooms—room damage is permanent mid-combat)
- Effect: restore HP
- Capped at max HP (excess healing wasted)
- Future: overflow healing converts to shields

### Unit Behavior During Execution

**Targeting:** Units target nearest enemy by position (MVP). All weapons on a unit hit the same target. Smarter/per-weapon targeting later.

**When all enemies dead:** Units attack the enemy command ship directly. Ship is off-board—no positional relationship to units.

**Unit damage to ship:** Hits a random surviving room (MVP).

**When all your units die:** Execution continues. Timeline keeps sweeping. Enemy units pummel your exposed ship.

**When no units on either side:** Skip execution phase entirely. Combat becomes pure dice duel until one ship dies.

**Death timing:** Units die immediately when taking lethal damage. Removed from board, can't act, targeting updates.

**Overkill damage:** Wasted. Unit dies, excess damage disappears.

**Simultaneous damage:** If two units kill each other on the same tick, both die. True simultaneous resolution.

**Shields and simultaneous damage:** Shields consumed sequentially within a tick. First attack eats shields, subsequent attacks hit HP.

### Enemy AI

**MVP:** Simple heuristics. Lock good faces, reroll bad faces. Random targeting for both damage and heal/shield dice.

**Display:** Player sees enemy's final locked faces and planned targets only (not the rolling/locking process). Lines/arrows drawn from each enemy die to its planned target.

**Targeting scope:** Enemy can target both player units and player rooms.

**Later:** Per-commander personalities (aggressive, defensive, greedy). Smarter targeting priorities.

### Combat Start

**Dice phases happen first.** At the start of combat, both sides roll and resolve dice before any execution phase. Units may enter battle already damaged, healed, or shielded.

### End of Round

**Round boundary:** After execution phase ends (timeline reaches position 10), brief visual pause before next round.

**End-of-round effects (MVP):** Shields expire on all units and rooms.

**Future:** Additional status effects tick at round boundary.

### Win Condition

**Destroy the enemy command ship** (all rooms destroyed).

**Player wins ties:** If both ships are destroyed simultaneously, player wins. Player's ship survives with 1 HP in all rooms.

**Units don't determine victory:** Losing all units doesn't end combat. Only ship destruction matters. Combat can become pure dice duel.

Two parallel battles:
- **Ship war:** Ships trade dice fire every round (always active)
- **Ground war:** Units fight units; winner's units can also attack enemy ship

### Run Structure (MVP)

**Loop:** Fight 1 → 2 events → Fight 2 → 2 events → Fight 1 → 2 events → ... (repeats indefinitely)

**Starting loadout:** Placeholder for MVP (define later).

**Loss condition:** Ship destroyed (all rooms gone) = game over.

**No win condition:** Run loops until player loses or manually quits.

**Loss screen:** Simple "You lost" text, click to restart. Placeholder for MVP.

### Two Build Layers

Players construct two separate builds:

**1. Squad Loadout (Units)**
- Which units to field
- What gear/weapons each unit carries
- Positioning in combat width (affects timing via timeline)
- Determines automatic combat behavior

**2. Command Ship**
- Ship type determines room count and base ability [base ability NOT IN MVP]
- Each room has a die type
- Optionally staff rooms with crew for bonuses [NOT IN MVP]

### Between Fights

**Damage carries forward:** Units and ship rooms keep their damage between fights. No automatic healing. Roguelike attrition.

**Destroyed rooms:** Gone forever for MVP. High stakes—protect your ship. Repair option in future.

**Destroyed units:** Gone forever for MVP. Permadeath.

**Repositioning:** Free repositioning of units between fights. Change unit positions in combat width at will.

**Dice types:** Fixed for MVP (Damage, Shield, Heal). Future: acquire/swap dice types.

**Rewards:** Immediate rewards screen after combat, then back to run map/shop.

**Events:** Current placeholder event phases between fights.

**Future:** Repair phase between fights (costs resources).

### Resource System [NOT IN MVP]

**Mana** (future):
- Generated by Mana dice faces
- Spent on powerful face activations and commander abilities
- Usable anytime during player command phase
- Capped per turn (overflow lost)—"use it or lose it"

**Abilities** (future):
- Come from multiple sources: ship base ability, rooms, crew, progression
- Spend mana to activate
- Effects vary: direct damage, dice manipulation, buffs/debuffs

### MVP Scope

| Element | MVP Value |
|---------|-----------|
| Command ship types | 1 |
| Rooms per ship | 3 |
| Room HP | 50 each |
| Room order | Visual only, no mechanical difference |
| Dice types | 3 (Damage, Shield, Heal), fixed |
| Faces per die | 6 |
| Face distribution | `[5, 5, 8, 12, 0, 0]` (all dice) |
| Crew | None |
| Abilities | None |
| Mana | None |
| Enemy ship | Mirror (same as player) |
| Rerolls per round | 2 |
| Unit targeting | Nearest enemy (all weapons same target) |
| Unit-to-ship damage | Random room |
| Ticks per slot | 8 (placeholder) |
| Cooldown behavior | Reset each round, countdown before first fire |
| Damage persistence | Carries forward between fights |
| Destroyed rooms/units | Gone forever (permadeath) |
| Unit repositioning | Free between fights |
| Overkill damage | Wasted |
| Tie-breaker | Player wins, ship survives at 1 HP |
| Enemy AI targeting | Random (both damage and heal/shield) |
| Enemy display | Final results only, lines to targets |
| Dice activation | Sequential, one at a time |
| Skip mechanic | "End Turn" button with confirmation |
| Run structure | Fight 1 → 2 events → Fight 2 → 2 events → loop |
| Starting loadout | Placeholder |
| Win condition | None (loop until loss or quit) |
| Loss screen | "You lost" placeholder |

### Key Properties

- **Data-driven:** All dice types, face distributions, and effects defined in templates
- **Extensible:** New dice types, face effects, crew, abilities can be added
- **Information asymmetry:** Player sees enemy's declared plan before acting
- **Preemptive counterplay:** Kill their target before their heal lands, destroy their room before it activates
- **Risk/reward:** Reroll economy creates gambling moments
- **Layered agency:** Passive satisfaction (watching units) + active decisions (dice)
- **Position matters:** Unit position determines timing via timeline; larger units get more action time

### Implementation Notes

This extends the current system. The existing cooldown and trigger infrastructure powers the execution phase. The new dice system layers on top as the player decision mechanism.

**Preserved from current system:**
- Cooldown-based weapon firing
- Trigger/effect cascade system
- Unit loadout and positioning
- TEA architecture (dice rolls come in via Msgs)

**New additions:**
- Command ship entity with rooms and dice
- Timeline-gated unit activation (units only act while timeline touches them)
- Cooldowns reset each round, only tick while unit is active
- Dice rolling, locking, rerolling, activation mechanics
- Round structure: enemy declare → player command → enemy execute → execution
- Room-targeted damage and shields
- Blank dice faces (lock but don't activate)
- Damage persistence between fights
- Skip execution phase when no units (pure dice duel)
