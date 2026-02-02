# Feature Contract — Wulfaz MVP

**Generated:** 2026-02-01
**Source:** DESIGN.md
**Pass:** Index (titles + line references only)

---

## How to Use This Contract

**Status values:**
- `Not Started` — No work begun
- `In Progress` — Active development
- `Complete` — Implemented and verified
- `Blocked` — Waiting on dependency
- `Deferred` — Explicitly out of MVP scope

**Feature entry format:**
```
### F-NNN: Title
- **Status:** <status>
- **Source:** DESIGN.md:NN-MM
- **Depends:** F-XXX, F-YYY (or None)
- **Description:** <deferred to description pass>
```

**Numbering scheme:** Each category reserves a range with gaps for future additions.

---

## Category 1: Core Architecture (F-100 series)

Foundation layer — TEA runtime, model structure, RNG, data loading.

_All features in this category have been implemented._

_Reserved: F-104 – F-109_

---

## Category 2: Entity System (F-110 series)

Core data structures — tags, attributes, units, dice holders.

### F-113: Size System (Spaces + Dice Count)
- **Status:** Not Started
- **Source:** DESIGN.md:59-66
- **Depends:** None (F-112 complete)
- **Description:** _deferred_

### F-114: Command Unit Entity
- **Status:** Complete
- **Source:** DESIGN.md:202-211, 550-566
- **Depends:** None (F-112 complete)
- **Description:** Command unit with off-board position and 3-dice pool (Wave 1)

### F-115: Unit Type Tags (Mech/Vehicle/BattleArmor)
- **Status:** Not Started
- **Source:** DESIGN.md:193-200
- **Depends:** None (F-110, F-112 complete)
- **Description:** _deferred_

_Reserved: F-116 – F-119_

---

## Category 3: Positioning (F-120 series)

Board layout, unit placement, space occupation.

### F-123: Command Unit Off-Board Position
- **Status:** Not Started
- **Source:** DESIGN.md:166-167, 177-178, 186, 210
- **Depends:** F-114
- **Description:** _deferred_

### F-124: Dead Unit Gap Handling
- **Status:** Complete
- **Source:** DESIGN.md:127-128, 188
- **Depends:** None (F-120 complete)
- **Description:** Dead units rendered with grey color and red X overlay during execution (Wave 8)

_Reserved: F-125 – F-129_

---

## Category 4: Dice System (F-130 series)

Die entities, face distributions, rolling, lock/reroll mechanics.

### F-130: Die Entity
- **Status:** Complete
- **Source:** DESIGN.md:435-437
- **Depends:** None (F-101 complete)
- **Description:** Die entity with face values and locked state (Wave 1)

### F-131: Dice Face Distribution (Data-Driven)
- **Status:** Complete
- **Source:** DESIGN.md:243-244, 249, 389-390, 631-632
- **Depends:** F-130
- **Description:** KDL-driven face distributions for dice types (Wave 2)

### F-132: Unit Dice Rolling
- **Status:** Complete
- **Source:** DESIGN.md:239-244
- **Depends:** F-130
- **Description:** Units roll their dice pool at round start (Wave 2)

### F-133: Command Unit Dice Rolling
- **Status:** Complete
- **Source:** DESIGN.md:246-252
- **Depends:** F-130, F-114
- **Description:** Command unit rolls 3 dice (damage/shield/heal) at round start (Wave 2)

### F-134: Lock/Unlock Mechanic
- **Status:** Complete
- **Source:** DESIGN.md:275-284, 82-84
- **Depends:** F-133
- **Description:** Player can lock dice to preserve values across rerolls (Wave 2)

### F-135: Reroll Mechanic
- **Status:** Complete
- **Source:** DESIGN.md:275-284, 83-84, 251, 391
- **Depends:** F-134
- **Description:** Player can reroll unlocked dice (Wave 2)

### F-136: Dice Effect — Damage
- **Status:** Complete
- **Source:** DESIGN.md:260-262
- **Depends:** F-130
- **Description:** Damage dice deal damage to target (Wave 2)

### F-137: Dice Effect — Shield
- **Status:** Complete
- **Source:** DESIGN.md:264-269
- **Depends:** F-130
- **Description:** Shield dice add temporary shields to friendly target (Wave 2)

### F-138: Dice Effect — Heal
- **Status:** Complete
- **Source:** DESIGN.md:271-273
- **Depends:** F-130
- **Description:** Heal dice restore HP to friendly target (Wave 2)

### F-139: Dice Activation (Click-to-Target)
- **Status:** Complete
- **Source:** DESIGN.md:85-86, 600-602
- **Depends:** F-134
- **Description:** Player activates dice to apply effects (Wave 2)

_Reserved: F-140 – F-149_

---

## Category 5: Damage Model (F-150 series)

HP, damage application, death, shields, persistence.

### F-152: Unit Death (Immediate Removal)
- **Status:** Complete
- **Source:** DESIGN.md:104, 218-219
- **Depends:** None (F-151 complete)
- **Description:** Dead units removed at round end via removeDeadUnits in handleRoundEnded (Wave 5)

### F-153: Shield Buffer
- **Status:** Complete
- **Source:** DESIGN.md:222-228
- **Depends:** F-137
- **Description:** Shields absorb damage first in applyDiceEffectToCombat (Wave 3/4)

### F-154: Shield Expiration (Round End)
- **Status:** Complete
- **Source:** DESIGN.md:109-110, 227, 229-230, 269
- **Depends:** F-153
- **Description:** Shields expire at round end (Wave 3)

### F-155: Damage Persistence (Between Fights)
- **Status:** Complete
- **Source:** DESIGN.md:143, 395
- **Depends:** None (F-150 complete)
- **Description:** PlayerRoster persists between fights; synced via syncRosterFromCombat on victory (Wave 5)

### F-156: Permadeath (Destroyed Units Gone)
- **Status:** Complete
- **Source:** DESIGN.md:11, 233, 396
- **Depends:** F-152
- **Description:** Dead units removed from roster; only surviving units sync back (Wave 5)

_Reserved: F-157 – F-159_

---

## Category 6: Targeting (F-160 series)

Target selection, overlap rules, overflow, command targeting.

### F-160: Positional Targeting (Overlap)
- **Status:** Complete
- **Source:** DESIGN.md:117-123, 185, 392
- **Depends:** None (F-122 complete)
- **Description:** Units target enemies based on board position overlap (Wave 4)

### F-161: Target Selection (Lowest HP First)
- **Status:** Complete
- **Source:** DESIGN.md:121
- **Depends:** F-160
- **Description:** SelectTargetUnit picks lowest HP overlapping enemy, ties broken left-to-right (Wave 4)

### F-162: Multi-Die Separate Attacks
- **Status:** Complete
- **Source:** DESIGN.md:123
- **Depends:** F-132
- **Description:** Each die resolves separately in resolveAttacks loop (Wave 4)

### F-163: Overflow Damage (MTG-Style)
- **Status:** Complete
- **Source:** DESIGN.md:125-126, 393, 492
- **Depends:** F-161
- **Description:** ApplyDamageWithOverflow chains through overlapping enemies, stops at command (Wave 4)

### F-164: Command Unit Targeting (Any Enemy)
- **Status:** Complete
- **Source:** DESIGN.md:131, 209, 260-261
- **Depends:** F-114, F-160
- **Description:** Command dice target lowest HP player/enemy unit (Wave 3)

### F-165: Friendly Targeting (Shield/Heal)
- **Status:** Complete
- **Source:** DESIGN.md:255-258, 265-266, 272
- **Depends:** F-137, F-138
- **Description:** handleDiceActivated routes shield/heal to friendly units (Wave 3)

### F-166: Gap-to-Command Fallback
- **Status:** Complete
- **Source:** DESIGN.md:127-128, 394
- **Depends:** F-124, F-164
- **Description:** Gap damage hits command only when all units dead (Wave 4, constrained by F-167)

### F-167: Units-Only-Target-Units Rule
- **Status:** Complete
- **Source:** DESIGN.md:129
- **Depends:** F-160
- **Description:** AnyAliveUnits prevents command targeting while units alive (Wave 4)

_Reserved: F-168 – F-169_

---

## Category 7: Combat Flow (F-170 series)

Round structure, phases, resolution order.

### F-171: Preview Phase
- **Status:** Complete
- **Source:** DESIGN.md:70-77, 593-598
- **Depends:** F-132, F-133
- **Description:** All dice rolled at round start, player sees enemy plan (Wave 2)

### F-172: Player Command Phase
- **Status:** Complete
- **Source:** DESIGN.md:79-87
- **Depends:** F-134, F-135, F-139
- **Description:** Player locks/rerolls/activates command dice (Wave 2)

### F-173: Enemy Command Phase
- **Status:** Complete
- **Source:** DESIGN.md:89-93
- **Depends:** F-133
- **Description:** Simple AI activates enemy command dice (Wave 3)

### F-174: Execution Phase
- **Status:** Complete
- **Source:** DESIGN.md:95-106
- **Depends:** F-160
- **Description:** Units fire in position order (Wave 3, stub targeting)

### F-175: Simultaneous Resolution (Per Position)
- **Status:** Complete
- **Source:** DESIGN.md:55, 99-102, 397
- **Depends:** F-174
- **Description:** Attacks at same position calculated before applying (Wave 3)

### F-176: Left-to-Right Firing Order
- **Status:** Complete
- **Source:** DESIGN.md:97
- **Depends:** F-174
- **Description:** Positions 0-9 resolve in order (Wave 3)

### F-177: Round End Phase
- **Status:** Complete
- **Source:** DESIGN.md:108-112
- **Depends:** F-154
- **Description:** Shields expire, round cleanup (Wave 3)

### F-178: Combat Loop (Repeat Until End)
- **Status:** Complete
- **Source:** DESIGN.md:114
- **Depends:** F-177
- **Description:** Combat continues until command unit dies (Wave 3)

_Reserved: F-179_

---

## Category 8: Victory Conditions (F-180 series)

Win/loss detection, tie-breaking, combat end.

### F-180: Win Condition (Destroy Enemy Command)
- **Status:** Complete
- **Source:** DESIGN.md:133-138
- **Depends:** F-114, F-152
- **Description:** checkCombatEnd detects command death and returns victor (Wave 3)

### F-181: Immediate Combat End
- **Status:** Complete
- **Source:** DESIGN.md:137
- **Depends:** F-180
- **Description:** applyCombatEnd called after each position resolve (Wave 3)

### F-182: Player Wins Ties
- **Status:** Complete
- **Source:** DESIGN.md:139, 488
- **Depends:** F-180
- **Description:** checkCombatEnd returns VictorPlayer when both commands dead (Wave 3)

### F-183: No Victory Screen (MVP)
- **Status:** Complete
- **Source:** DESIGN.md:141
- **Depends:** F-180
- **Description:** Combat victory transitions to PhaseInterCombat, no dedicated screen (Wave 3)

_Reserved: F-184 – F-189_

---

## Category 9: Edge Cases (F-190 series)

Special combat situations requiring explicit handling.

### F-190: Pure Command vs Command (All Units Dead)
- **Status:** Complete
- **Source:** DESIGN.md:147
- **Depends:** F-114
- **Description:** Empty firing order handled gracefully; ExecutionComplete returned immediately (Wave 8)

### F-191: Zero-Dice Unit Handling
- **Status:** Complete
- **Source:** DESIGN.md:149
- **Depends:** F-132
- **Description:** Units with empty dice slice skipped gracefully in RollAllDice (Wave 8)

### F-192: Dead Target Skip
- **Status:** Complete
- **Source:** DESIGN.md:92
- **Depends:** F-152, F-173
- **Description:** Enemy command dice skip targets killed by earlier dice this phase (Wave 8)

### F-193: Simultaneous Death Resolution
- **Status:** Complete
- **Source:** DESIGN.md:491
- **Depends:** F-175
- **Description:** HP snapshot ensures units attack even if killed in same position (Wave 8)

_Reserved: F-194 – F-199_

---

## Category 10: Enemy AI (F-200 series)

Enemy behavior, targeting heuristics, display.

### F-200: Enemy Dice Rolling (No Reroll)
- **Status:** Not Started
- **Source:** DESIGN.md:612-613
- **Depends:** F-133
- **Description:** _deferred_

### F-201: Enemy Target Heuristics
- **Status:** Not Started
- **Source:** DESIGN.md:613-614
- **Depends:** F-200
- **Description:** _deferred_

### F-202: Enemy Targeting Display (Lines)
- **Status:** Not Started
- **Source:** DESIGN.md:74-75, 618-621
- **Depends:** F-201
- **Description:** _deferred_

### F-203: Player-First Resolution Order
- **Status:** Not Started
- **Source:** DESIGN.md:615
- **Depends:** F-172, F-173
- **Description:** _deferred_

_Reserved: F-204 – F-209_

---

## Category 11: Run Structure (F-210 series)

Meta-loop, fight sequencing, between-fight actions.

### F-215: Repair Action
- **Status:** Not Started
- **Source:** DESIGN.md:327
- **Depends:** None (F-150 complete)
- **Description:** _deferred_

### F-216: Free Repositioning (Between Fights)
- **Status:** Not Started
- **Source:** DESIGN.md:187
- **Depends:** None (F-121 complete)
- **Description:** _deferred_

_Reserved: F-217 – F-219_

---

## Category 12: UI / Display (F-220 series)

Visual presentation of game state.

### F-223: Shield Display
- **Status:** Complete
- **Source:** DESIGN.md:588-589
- **Depends:** F-153
- **Description:** "HP:X SH:Y" format in drawUnit() and drawCommandUnit() (Wave 6)

### F-224: Round Toast
- **Status:** Complete
- **Source:** DESIGN.md:591, 111
- **Depends:** None (F-170 complete)
- **Description:** Round toast overlay shown between rounds, click to continue (Wave 6)

### F-225: Dice Display (Preview Phase)
- **Status:** Complete
- **Source:** DESIGN.md:593-598
- **Depends:** F-171
- **Description:** Dice boxes with pip patterns on all units, command unit pyramid layout (Wave 6)

### F-226: Dice Interaction UI (Player Command)
- **Status:** Complete
- **Source:** DESIGN.md:597, 600-602
- **Depends:** F-172
- **Description:** Left-click select/target, right-click lock, R reroll, auto-advance when done (Wave 6)

### F-227: Targeting Lines Display
- **Status:** Not Started
- **Source:** DESIGN.md:74-75
- **Depends:** F-202
- **Description:** _deferred_

### F-228: Execution Phase Visual Delay
- **Status:** Not Started
- **Source:** DESIGN.md:98
- **Depends:** F-174
- **Description:** _deferred_

### F-229: Round Toast / Continue Prompt
- **Status:** Not Started
- **Source:** DESIGN.md:111
- **Depends:** F-177
- **Description:** _deferred_

_Reserved: F-230 – F-239_

---

## Category 13: Templates / Content (F-240 series)

KDL schema definitions and MVP content.

### F-241: Command Unit Template Schema
- **Status:** Not Started
- **Source:** DESIGN.md:550-566
- **Depends:** F-114
- **Description:** _deferred_

### F-243: MVP Fight Encounter Templates
- **Status:** Not Started
- **Source:** DESIGN.md:376
- **Depends:** None (F-240 complete)
- **Description:** _deferred_

### F-244: Symmetric Unit Pool
- **Status:** Not Started
- **Source:** DESIGN.md:380
- **Depends:** None (F-242 complete)
- **Description:** _deferred_

### F-245: MVP Event Templates
- **Status:** Not Started
- **Source:** DESIGN.md:377
- **Depends:** None (F-212 complete)
- **Description:** _deferred_

_Reserved: F-246 – F-249_

---

## Deferred Features (F-D series)

Explicitly out of MVP scope per DESIGN.md.

### F-D01: Pilots/Crew System
- **Status:** Deferred
- **Source:** DESIGN.md:287-297, 378, 410, 505

### F-D02: Items/Mounts System
- **Status:** Deferred
- **Source:** DESIGN.md:379, 411, 451-470, 504, 569-580

### F-D03: Parts System
- **Status:** Deferred
- **Source:** DESIGN.md:447, 451-456

### F-D04: Multiple Factions
- **Status:** Deferred
- **Source:** DESIGN.md:37-43, 405

### F-D05: Meta-Progression
- **Status:** Deferred
- **Source:** DESIGN.md:406

### F-D06: Sound
- **Status:** Deferred
- **Source:** DESIGN.md:407

### F-D07: Complex Win Conditions
- **Status:** Deferred
- **Source:** DESIGN.md:409

### F-D08: Retreat Mechanic
- **Status:** Deferred
- **Source:** DESIGN.md:413

### F-D09: Speed Controls (Beyond Pause)
- **Status:** Deferred
- **Source:** DESIGN.md:414

### F-D10: Varied Dice Types per Unit
- **Status:** Deferred
- **Source:** DESIGN.md:415, 502

### F-D11: Active Abilities (Non-Dice)
- **Status:** Deferred
- **Source:** DESIGN.md:412

### F-D12: Undo Button (Dice Activation)
- **Status:** Deferred
- **Source:** DESIGN.md:605-606

### F-D13: Complex Condition Logic (AND/OR/NOT)
- **Status:** Deferred
- **Source:** DESIGN.md:506

### F-D14: ValueRef Expressions
- **Status:** Deferred
- **Source:** DESIGN.md:507

### F-D15: Absolute Template Paths
- **Status:** Deferred
- **Source:** DESIGN.md:524-527

### F-D16: Custom Fonts/Sprites
- **Status:** Deferred
- **Source:** DESIGN.md:519

### F-D17: Animated Health Bars / Damage Numbers
- **Status:** Deferred
- **Source:** DESIGN.md:520

### F-D18: Dice Rolling Animations
- **Status:** Deferred
- **Source:** DESIGN.md:521

### F-D19: Menu and Shop Screens
- **Status:** Deferred
- **Source:** DESIGN.md:522

### F-D20: Enemy AI Personalities
- **Status:** Deferred
- **Source:** DESIGN.md:624-628

---

## Summary

| Category | ID Range | Remaining | Complete |
|----------|----------|-----------|----------|
| Core Architecture | F-100 – F-109 | 0 | 4 |
| Entity System | F-110 – F-119 | 2 | 4 |
| Positioning | F-120 – F-129 | 1 | 4 |
| Dice System | F-130 – F-149 | 0 | 10 |
| Damage Model | F-150 – F-159 | 0 | 5 |
| Targeting | F-160 – F-169 | 0 | 8 |
| Combat Flow | F-170 – F-179 | 0 | 8 |
| Victory Conditions | F-180 – F-189 | 0 | 4 |
| Edge Cases | F-190 – F-199 | 0 | 4 |
| Enemy AI | F-200 – F-209 | 4 | 0 |
| Run Structure | F-210 – F-219 | 2 | 5 |
| UI / Display | F-220 – F-239 | 3 | 4 |
| Templates / Content | F-240 – F-249 | 4 | 2 |
| **MVP Total** | | **16** | **45** |
| Deferred | F-D01 – F-D20 | 20 | — |

---

## Critical Path

```
[COMPLETE] F-100 (TEA Runtime)
 ├─► [COMPLETE] F-101 (Model) ─► [COMPLETE] F-110 (Tags) ─► [COMPLETE] F-112 (Unit)
 │                            ─► [COMPLETE] F-111 (Attr) ─► [COMPLETE] F-150 (HP)
 │                            ─► [COMPLETE] F-130 (Die) ─► [COMPLETE] F-132 (Roll)
 ├─► [COMPLETE] F-102 (RNG)  ─► [COMPLETE] F-132, F-133
 └─► [COMPLETE] F-103 (KDL)  ─► [COMPLETE] F-240 (Templates)

[COMPLETE] F-112 (Unit)
 ├─► F-113 (Size) ─► [COMPLETE] F-122 (Occupation) ─► [COMPLETE] F-160 (Targeting)
 ├─► [COMPLETE] F-114 (Cmd) ─► [COMPLETE] F-133 (Cmd Dice) ─► [COMPLETE] F-134 (Lock)
 └─► [COMPLETE] F-120 (Board)─► [COMPLETE] F-121 (Placement)

[COMPLETE] F-160 (Targeting) + [COMPLETE] F-151 (Damage) ─► [COMPLETE] F-174 (Execution)
[COMPLETE] F-174 ─► [COMPLETE] F-175 (Simultaneous) ─► [COMPLETE] F-170 (Round)
[COMPLETE] F-170 ─► [COMPLETE] F-210 (Run Loop)
```

---

## Implementation Phases

| Phase | Focus | Features | Status |
|-------|-------|----------|--------|
| 1 | Foundation | F-100, F-101, F-102, F-103 | **COMPLETE** |
| 2 | Entities | F-110 – F-115, F-130, F-131 | Partial (F-113, F-115 remain) |
| 3 | Board | F-120 – F-124 | Partial (F-123 remains) |
| 4 | Dice Mechanics | F-132 – F-139 | **COMPLETE** |
| 5 | Damage | F-150 – F-156 | **COMPLETE** |
| 6 | Targeting | F-160 – F-167 | **COMPLETE** |
| 7 | Combat Flow | F-170 – F-178 | **COMPLETE** |
| 8 | Victory + Edge | F-180 – F-183, F-190 – F-193 | **COMPLETE** |
| 9 | Enemy AI | F-200 – F-203 | Not Started |
| 10 | Run Structure | F-210 – F-216 | Partial (F-215, F-216 remain) |
| 11 | UI | F-220 – F-229 | Partial (F-227 – F-229 remain) |
| 12 | Content | F-240 – F-245 | Partial (F-241, F-243 – F-245 remain) |
