# Mech Autobattler Roguelike — Design Document

**Status:** Ebitengine rendering and runtime integration complete

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

## Implemented: Code Scaffold

### Package Structure

```
wulfaz/
├── cmd/wulfaz/main.go           # Entry point (Ebitengine bootstrap)
├── internal/
│   ├── app/app.go               # Ebitengine Game implementation
│   ├── core/                    # Foundation types
│   │   ├── tag.go               # Tag string type
│   │   ├── valueref.go          # Static int (MVP), expandable
│   │   ├── attribute.go         # Name, Base, Min, Max
│   │   ├── condition.go         # ConditionType + Params
│   │   ├── modifier.go          # ModifierOp, Scope, Modifier, ProvidedModifier
│   │   ├── trigger.go           # EventType + Trigger
│   │   ├── requirement.go       # OnUnmet + Requirement
│   │   ├── ability.go           # TargetType, Targeting, Cost, EffectRef, Ability
│   │   └── limits.go            # Hard caps (cascade depth, etc.)
│   ├── entity/                  # Game entities (value types, no pointers)
│   │   ├── pilot.go             # ID, Name (stub)
│   │   ├── item.go              # Full item with triggers, abilities, modifiers
│   │   ├── mount.go             # MountCriteria + Mount with Contents
│   │   ├── part.go              # Part with Mounts, Connections
│   │   └── unit.go              # Unit with Parts, Pilot (HasPilot flag)
│   ├── model/combat.go          # CombatPhase, CombatModel
│   ├── tea/                     # TEA types and Update logic
│   │   ├── msg.go               # Msg interface + concrete messages
│   │   ├── cmd.go               # Cmd type + None, Batch
│   │   ├── model.go             # GamePhase, Model, Update
│   │   ├── model_test.go        # TEA integration tests
│   │   └── runtime.go           # Runtime with Dispatch loop (test helper)
│   ├── event/                   # Event dispatch
│   │   ├── context.go           # TriggerContext, TriggerOwner, CollectedTrigger
│   │   ├── dispatch.go          # Entity traversal, condition evaluation
│   │   └── dispatch_test.go     # 11 dispatch tests
│   ├── effect/                  # Effect handling
│   │   ├── result.go            # EffectResult, FollowUpEvent
│   │   ├── handler.go           # EffectContext, 3 effect handlers
│   │   └── handler_test.go      # 13 effect tests
│   └── template/                # Template loading
│       ├── registry.go          # Registry for units/items
│       ├── loader.go            # LoadUnitsFromDir, LoadItemsFromDir
│       ├── parse.go             # KDL parsing helpers, entity parsers
│       └── loader_test.go       # 24 tests covering all parsers
├── ui/renderer/
│   ├── stub.go                  # Text-based rendering (testing)
│   └── ebiten.go                # Ebitengine rendering
├── data/templates/
│   ├── units/
│   │   ├── small_mech.kdl       # Light chassis (combat_width=1)
│   │   ├── medium_mech.kdl      # Medium chassis (combat_width=2)
│   │   └── large_mech.kdl       # Heavy chassis (combat_width=3)
│   └── items/
│       ├── medium_laser.kdl     # Energy weapon
│       ├── autocannon.kdl       # Ballistic weapon with ammo
│       └── lrm_rack.kdl         # Missile weapon with splash
├── go.mod
├── go.sum
├── CLAUDE.md                    # TEA principles and coding rules
└── DESIGN.md
```

### Key Type Names (as implemented)

| Type | Location | Purpose |
|------|----------|---------|
| `Tag` | core/tag.go | String label for categorization |
| `ValueRef` | core/valueref.go | Static int (MVP), will support refs/exprs |
| `Attribute` | core/attribute.go | Name + Base + Min/Max |
| `ConditionType` | core/condition.go | Typed constants for condition types |
| `Condition` | core/condition.go | Type + Params |
| `ModifierOp` | core/modifier.go | Add/Mult/Set/Min/Max operations |
| `Scope` | core/modifier.go | Self/Unit/Part/Adjacent/Mount |
| `Modifier` | core/modifier.go | Applied modifier with source |
| `ProvidedModifier` | core/modifier.go | Modifier template from items |
| `EventType` | core/trigger.go | Typed constants for events |
| `Trigger` | core/trigger.go | Event + Conditions + EffectName |
| `OnUnmet` | core/requirement.go | Disabled/CannotMount/Warning |
| `Requirement` | core/requirement.go | Scope + Condition + OnUnmet |
| `TargetType` | core/ability.go | None/Self/Ally/Enemy/etc. |
| `Targeting` | core/ability.go | Type + Range + Count + Filter |
| `Cost` | core/ability.go | Attribute + Scope + Amount |
| `EffectRef` | core/ability.go | EffectName + Params + Delay + Conditions |
| `Ability` | core/ability.go | Full ability definition |
| `Pilot` | entity/pilot.go | ID + Name (stub) |
| `Item` | entity/item.go | Full item with ProvidedModifiers, Requirements |
| `MountCriteria` | entity/mount.go | RequiresAll/Any, Forbids |
| `Mount` | entity/mount.go | Accepts + Capacity + Contents |
| `Part` | entity/part.go | Mounts + Connections |
| `Unit` | entity/unit.go | Parts + Pilot + HasPilot flag |
| `TriggerOwner` | event/context.go | UnitID/PartID/MountID/ItemID path |
| `TriggerContext` | event/context.go | Event + SourceUnit + AllUnits + Tick |
| `CollectedTrigger` | event/context.go | Trigger + Owner pair |
| `EffectContext` | effect/handler.go | Owner + SourceUnit + AllUnits + Rolls |
| `EffectResult` | effect/result.go | ModifiedUnits + FollowUpEvents + LogEntries |
| `FollowUpEvent` | effect/result.go | Cascading event (Event + SourceID + TargetID) |
| `TriggersCollected` | tea/msg.go | Msg after dispatch (triggers + depth) |
| `EffectsResolved` | tea/msg.go | Msg after effects (modifications + follow-ups) |

### TEA Compliance

- [x] Model is struct with no functions/channels/mutexes
- [x] All Model methods use value receivers
- [x] Msg types named as past-tense events
- [x] No pointers in Model or entity types
- [x] Value slices/maps throughout
- [x] Cmd executed only by runtime
- [x] Seeded RNG via Msg payloads
- [x] Templates immutable, instances mutable
- [x] Task Pattern for sequential effects (TriggersCollected → EffectsResolved)
- [x] Immutable slice copies before modification

### Implemented Conditions

| Type | Params | Behavior |
|------|--------|----------|
| `has_tag` | `tag: string` | Unit has tag |
| `attr_gte` | `attr: string, value: int` | Attribute >= value |
| `attr_lte` | `attr: string, value: int` | Attribute <= value |
| `attr_eq` | `attr: string, value: int` | Attribute == value |

### Implemented Effects

| Effect | Params | Behavior |
|--------|--------|----------|
| `deal_damage` | `damage: int, target: string` | Reduce health, emit on_damaged/on_destroyed |
| `consume_ammo` | `amount: int` | Reduce owning item's ammo attribute |
| `deal_splash_damage` | `damage: int, target: string` | MVP: same as deal_damage |

**Target resolution:** `"self"` → source, `"enemy"` → first enemy (alphabetical), `"ally"` → self (MVP), or unit ID

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
| Destroyed units | Fire triggers same tick | `is_alive` condition or filter |
| Ally targeting | Self | Proper ally selection |
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

## Next Steps

1. ~~Implement KDL loader (`template/loader.go`)~~ **DONE**
2. ~~Add 3 chassis templates (small/medium/large)~~ **DONE**
3. ~~Add weapon templates (laser, AC, LRM)~~ **DONE**
4. ~~Implement event dispatch (`event/dispatch.go`)~~ **DONE**
5. ~~Implement basic effects (`effect/handler.go`)~~ **DONE**
6. ~~Wire up combat tick loop~~ **DONE**
7. ~~Minimal UI rendering (Ebitengine)~~ **DONE**
8. ~~Runtime integration (tick generation, Cmd execution)~~ **DONE**

### MVP Complete — Next Phase

9. ~~Load units from templates instead of hardcoded test data~~ **DONE**
10. Implement shop/event phase between fights
11. Add win/lose conditions (health reaches 0)
12. Second fight encounter
