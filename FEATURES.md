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

### F-100: TEA Runtime Shell
- **Status:** Not Started
- **Source:** DESIGN.md:340-347
- **Depends:** None
- **Description:** _deferred_

### F-101: Model Value Type
- **Status:** Not Started
- **Source:** DESIGN.md:419-438
- **Depends:** F-100
- **Description:** _deferred_

### F-102: Seeded RNG via Msgs
- **Status:** Not Started
- **Source:** DESIGN.md:348-364
- **Depends:** F-100
- **Description:** _deferred_

### F-103: KDL Template Loader
- **Status:** Not Started
- **Source:** DESIGN.md:345, 531-581
- **Depends:** F-100
- **Description:** _deferred_

_Reserved: F-104 – F-109_

---

## Category 2: Entity System (F-110 series)

Core data structures — tags, attributes, units, dice holders.

### F-110: Tag System
- **Status:** Not Started
- **Source:** DESIGN.md:419-421, 429
- **Depends:** F-101
- **Description:** _deferred_

### F-111: Attribute System
- **Status:** Not Started
- **Source:** DESIGN.md:429-434, 473-479
- **Depends:** F-101
- **Description:** _deferred_

### F-112: Unit Entity
- **Status:** Not Started
- **Source:** DESIGN.md:192-200, 424-438
- **Depends:** F-110, F-111
- **Description:** _deferred_

### F-113: Size System (Spaces + Dice Count)
- **Status:** Not Started
- **Source:** DESIGN.md:59-66
- **Depends:** F-112
- **Description:** _deferred_

### F-114: Command Unit Entity
- **Status:** Not Started
- **Source:** DESIGN.md:202-211, 550-566
- **Depends:** F-112
- **Description:** _deferred_

### F-115: Unit Type Tags (Mech/Vehicle/BattleArmor)
- **Status:** Not Started
- **Source:** DESIGN.md:193-200
- **Depends:** F-110, F-112
- **Description:** _deferred_

_Reserved: F-116 – F-119_

---

## Category 3: Positioning (F-120 series)

Board layout, unit placement, space occupation.

### F-120: Board Layout (10-Space Row)
- **Status:** Not Started
- **Source:** DESIGN.md:56, 152-189
- **Depends:** F-112
- **Description:** _deferred_

### F-121: Unit Placement on Board
- **Status:** Not Started
- **Source:** DESIGN.md:181-188
- **Depends:** F-120
- **Description:** _deferred_

### F-122: Space Occupation (Size-Based)
- **Status:** Not Started
- **Source:** DESIGN.md:119, 184
- **Depends:** F-113, F-120
- **Description:** _deferred_

### F-123: Command Unit Off-Board Position
- **Status:** Not Started
- **Source:** DESIGN.md:166-167, 177-178, 186, 210
- **Depends:** F-114, F-120
- **Description:** _deferred_

### F-124: Dead Unit Gap Handling
- **Status:** Not Started
- **Source:** DESIGN.md:127-128, 188
- **Depends:** F-120
- **Description:** _deferred_

_Reserved: F-125 – F-129_

---

## Category 4: Dice System (F-130 series)

Die entities, face distributions, rolling, lock/reroll mechanics.

### F-130: Die Entity
- **Status:** Not Started
- **Source:** DESIGN.md:435-437
- **Depends:** F-101
- **Description:** _deferred_

### F-131: Dice Face Distribution (Data-Driven)
- **Status:** Not Started
- **Source:** DESIGN.md:243-244, 249, 389-390, 631-632
- **Depends:** F-130, F-103
- **Description:** _deferred_

### F-132: Unit Dice Rolling
- **Status:** Not Started
- **Source:** DESIGN.md:239-244
- **Depends:** F-130, F-102
- **Description:** _deferred_

### F-133: Command Unit Dice Rolling
- **Status:** Not Started
- **Source:** DESIGN.md:246-252
- **Depends:** F-130, F-102, F-114
- **Description:** _deferred_

### F-134: Lock/Unlock Mechanic
- **Status:** Not Started
- **Source:** DESIGN.md:275-284, 82-84
- **Depends:** F-133
- **Description:** _deferred_

### F-135: Reroll Mechanic
- **Status:** Not Started
- **Source:** DESIGN.md:275-284, 83-84, 251, 391
- **Depends:** F-134
- **Description:** _deferred_

### F-136: Dice Effect — Damage
- **Status:** Not Started
- **Source:** DESIGN.md:260-262
- **Depends:** F-130
- **Description:** _deferred_

### F-137: Dice Effect — Shield
- **Status:** Not Started
- **Source:** DESIGN.md:264-269
- **Depends:** F-130
- **Description:** _deferred_

### F-138: Dice Effect — Heal
- **Status:** Not Started
- **Source:** DESIGN.md:271-273
- **Depends:** F-130
- **Description:** _deferred_

### F-139: Dice Activation (Click-to-Target)
- **Status:** Not Started
- **Source:** DESIGN.md:85-86, 600-602
- **Depends:** F-134
- **Description:** _deferred_

_Reserved: F-140 – F-149_

---

## Category 5: Damage Model (F-150 series)

HP, damage application, death, shields, persistence.

### F-150: HP Pool
- **Status:** Not Started
- **Source:** DESIGN.md:214-219
- **Depends:** F-111
- **Description:** _deferred_

### F-151: Damage Application
- **Status:** Not Started
- **Source:** DESIGN.md:218
- **Depends:** F-150
- **Description:** _deferred_

### F-152: Unit Death (Immediate Removal)
- **Status:** Not Started
- **Source:** DESIGN.md:104, 218-219
- **Depends:** F-151
- **Description:** _deferred_

### F-153: Shield Buffer
- **Status:** Not Started
- **Source:** DESIGN.md:222-228
- **Depends:** F-150, F-137
- **Description:** _deferred_

### F-154: Shield Expiration (Round End)
- **Status:** Not Started
- **Source:** DESIGN.md:109-110, 227, 229-230, 269
- **Depends:** F-153
- **Description:** _deferred_

### F-155: Damage Persistence (Between Fights)
- **Status:** Not Started
- **Source:** DESIGN.md:143, 395
- **Depends:** F-150
- **Description:** _deferred_

### F-156: Permadeath (Destroyed Units Gone)
- **Status:** Not Started
- **Source:** DESIGN.md:11, 233, 396
- **Depends:** F-152
- **Description:** _deferred_

_Reserved: F-157 – F-159_

---

## Category 6: Targeting (F-160 series)

Target selection, overlap rules, overflow, command targeting.

### F-160: Positional Targeting (Overlap)
- **Status:** Not Started
- **Source:** DESIGN.md:117-123, 185, 392
- **Depends:** F-122
- **Description:** _deferred_

### F-161: Target Selection (Lowest HP First)
- **Status:** Not Started
- **Source:** DESIGN.md:121
- **Depends:** F-160
- **Description:** _deferred_

### F-162: Multi-Die Separate Attacks
- **Status:** Not Started
- **Source:** DESIGN.md:123
- **Depends:** F-132
- **Description:** _deferred_

### F-163: Overflow Damage (MTG-Style)
- **Status:** Not Started
- **Source:** DESIGN.md:125-126, 393, 492
- **Depends:** F-161
- **Description:** _deferred_

### F-164: Command Unit Targeting (Any Enemy)
- **Status:** Not Started
- **Source:** DESIGN.md:131, 209, 260-261
- **Depends:** F-114, F-160
- **Description:** _deferred_

### F-165: Friendly Targeting (Shield/Heal)
- **Status:** Not Started
- **Source:** DESIGN.md:255-258, 265-266, 272
- **Depends:** F-137, F-138
- **Description:** _deferred_

### F-166: Gap-to-Command Fallback
- **Status:** Not Started
- **Source:** DESIGN.md:127-128, 394
- **Depends:** F-124, F-164
- **Description:** _deferred_

### F-167: Units-Only-Target-Units Rule
- **Status:** Not Started
- **Source:** DESIGN.md:129
- **Depends:** F-160
- **Description:** _deferred_

_Reserved: F-168 – F-169_

---

## Category 7: Combat Flow (F-170 series)

Round structure, phases, resolution order.

### F-170: Round State Machine
- **Status:** Not Started
- **Source:** DESIGN.md:68-115
- **Depends:** F-102
- **Description:** _deferred_

### F-171: Preview Phase
- **Status:** Not Started
- **Source:** DESIGN.md:70-77, 593-598
- **Depends:** F-132, F-133
- **Description:** _deferred_

### F-172: Player Command Phase
- **Status:** Not Started
- **Source:** DESIGN.md:79-87
- **Depends:** F-134, F-135, F-139
- **Description:** _deferred_

### F-173: Enemy Command Phase
- **Status:** Not Started
- **Source:** DESIGN.md:89-93
- **Depends:** F-133
- **Description:** _deferred_

### F-174: Execution Phase
- **Status:** Not Started
- **Source:** DESIGN.md:95-106
- **Depends:** F-160, F-151
- **Description:** _deferred_

### F-175: Simultaneous Resolution (Per Position)
- **Status:** Not Started
- **Source:** DESIGN.md:55, 99-102, 397
- **Depends:** F-174
- **Description:** _deferred_

### F-176: Left-to-Right Firing Order
- **Status:** Not Started
- **Source:** DESIGN.md:97
- **Depends:** F-174
- **Description:** _deferred_

### F-177: Round End Phase
- **Status:** Not Started
- **Source:** DESIGN.md:108-112
- **Depends:** F-154
- **Description:** _deferred_

### F-178: Combat Loop (Repeat Until End)
- **Status:** Not Started
- **Source:** DESIGN.md:114
- **Depends:** F-170, F-177
- **Description:** _deferred_

_Reserved: F-179_

---

## Category 8: Victory Conditions (F-180 series)

Win/loss detection, tie-breaking, combat end.

### F-180: Win Condition (Destroy Enemy Command)
- **Status:** Not Started
- **Source:** DESIGN.md:133-138
- **Depends:** F-114, F-152
- **Description:** _deferred_

### F-181: Immediate Combat End
- **Status:** Not Started
- **Source:** DESIGN.md:137
- **Depends:** F-180
- **Description:** _deferred_

### F-182: Player Wins Ties
- **Status:** Not Started
- **Source:** DESIGN.md:139, 488
- **Depends:** F-180
- **Description:** _deferred_

### F-183: No Victory Screen (MVP)
- **Status:** Not Started
- **Source:** DESIGN.md:141
- **Depends:** F-180
- **Description:** _deferred_

_Reserved: F-184 – F-189_

---

## Category 9: Edge Cases (F-190 series)

Special combat situations requiring explicit handling.

### F-190: Pure Command vs Command (All Units Dead)
- **Status:** Not Started
- **Source:** DESIGN.md:147
- **Depends:** F-170, F-114
- **Description:** _deferred_

### F-191: Zero-Dice Unit Handling
- **Status:** Not Started
- **Source:** DESIGN.md:149
- **Depends:** F-132
- **Description:** _deferred_

### F-192: Dead Target Skip
- **Status:** Not Started
- **Source:** DESIGN.md:92
- **Depends:** F-152, F-173
- **Description:** _deferred_

### F-193: Simultaneous Death Resolution
- **Status:** Not Started
- **Source:** DESIGN.md:491
- **Depends:** F-175
- **Description:** _deferred_

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

### F-210: Run Loop State Machine
- **Status:** Not Started
- **Source:** DESIGN.md:300-318, 330-336
- **Depends:** F-170
- **Description:** _deferred_

### F-211: Fight Sequence (Alternating)
- **Status:** Not Started
- **Source:** DESIGN.md:333
- **Depends:** F-210
- **Description:** _deferred_

### F-212: Event Phase (Pick 2 of 3)
- **Status:** Not Started
- **Source:** DESIGN.md:307, 336, 377
- **Depends:** F-210
- **Description:** _deferred_

### F-213: Fight Selection (Pick Difficulty)
- **Status:** Not Started
- **Source:** DESIGN.md:309, 336
- **Depends:** F-210
- **Description:** _deferred_

### F-214: Random Starting Loadout
- **Status:** Not Started
- **Source:** DESIGN.md:15, 305, 398
- **Depends:** F-102, F-112
- **Description:** _deferred_

### F-215: Repair Action
- **Status:** Not Started
- **Source:** DESIGN.md:327
- **Depends:** F-150
- **Description:** _deferred_

### F-216: Free Repositioning (Between Fights)
- **Status:** Not Started
- **Source:** DESIGN.md:187
- **Depends:** F-121
- **Description:** _deferred_

_Reserved: F-217 – F-219_

---

## Category 12: UI / Display (F-220 series)

Visual presentation of game state.

### F-220: Board Rendering
- **Status:** Not Started
- **Source:** DESIGN.md:152-179
- **Depends:** F-120
- **Description:** _deferred_

### F-221: Unit Rendering (Position + Size)
- **Status:** Not Started
- **Source:** DESIGN.md:156-175
- **Depends:** F-112, F-220
- **Description:** _deferred_

### F-222: HP Display (Units + Command)
- **Status:** Not Started
- **Source:** DESIGN.md:588-590
- **Depends:** F-150
- **Description:** _deferred_

### F-223: Shield Display
- **Status:** Not Started
- **Source:** DESIGN.md:588-589
- **Depends:** F-153
- **Description:** _deferred_

### F-224: Round Number Display
- **Status:** Not Started
- **Source:** DESIGN.md:591
- **Depends:** F-170
- **Description:** _deferred_

### F-225: Dice Display (Preview Phase)
- **Status:** Not Started
- **Source:** DESIGN.md:593-598
- **Depends:** F-171
- **Description:** _deferred_

### F-226: Dice Interaction UI (Player Command)
- **Status:** Not Started
- **Source:** DESIGN.md:597, 600-602
- **Depends:** F-172
- **Description:** _deferred_

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

### F-240: Unit Template Schema
- **Status:** Not Started
- **Source:** DESIGN.md:531-547
- **Depends:** F-103, F-112
- **Description:** _deferred_

### F-241: Command Unit Template Schema
- **Status:** Not Started
- **Source:** DESIGN.md:550-566
- **Depends:** F-103, F-114
- **Description:** _deferred_

### F-242: MVP Chassis Templates (S/M/L Mech)
- **Status:** Not Started
- **Source:** DESIGN.md:375
- **Depends:** F-240
- **Description:** _deferred_

### F-243: MVP Fight Encounter Templates
- **Status:** Not Started
- **Source:** DESIGN.md:376
- **Depends:** F-240
- **Description:** _deferred_

### F-244: Symmetric Unit Pool
- **Status:** Not Started
- **Source:** DESIGN.md:380
- **Depends:** F-242
- **Description:** _deferred_

### F-245: MVP Event Templates
- **Status:** Not Started
- **Source:** DESIGN.md:377
- **Depends:** F-212
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

| Category | ID Range | Count | Reserved |
|----------|----------|-------|----------|
| Core Architecture | F-100 – F-109 | 4 | 6 |
| Entity System | F-110 – F-119 | 6 | 4 |
| Positioning | F-120 – F-129 | 5 | 5 |
| Dice System | F-130 – F-149 | 10 | 10 |
| Damage Model | F-150 – F-159 | 7 | 3 |
| Targeting | F-160 – F-169 | 8 | 2 |
| Combat Flow | F-170 – F-179 | 9 | 1 |
| Victory Conditions | F-180 – F-189 | 4 | 6 |
| Edge Cases | F-190 – F-199 | 4 | 6 |
| Enemy AI | F-200 – F-209 | 4 | 6 |
| Run Structure | F-210 – F-219 | 7 | 3 |
| UI / Display | F-220 – F-239 | 10 | 10 |
| Templates / Content | F-240 – F-249 | 6 | 4 |
| **MVP Total** | | **84** | **66** |
| Deferred | F-D01 – F-D20 | 20 | — |

---

## Critical Path

```
F-100 (TEA Runtime)
 ├─► F-101 (Model) ─► F-110 (Tags) ─► F-112 (Unit)
 │                 ─► F-111 (Attr) ─► F-150 (HP)
 │                 ─► F-130 (Die)  ─► F-132 (Roll)
 ├─► F-102 (RNG)  ─► F-132, F-133
 └─► F-103 (KDL)  ─► F-240 (Templates)

F-112 (Unit)
 ├─► F-113 (Size) ─► F-122 (Occupation) ─► F-160 (Targeting)
 ├─► F-114 (Cmd)  ─► F-133 (Cmd Dice)   ─► F-134 (Lock)
 └─► F-120 (Board)─► F-121 (Placement)

F-160 (Targeting) + F-151 (Damage) ─► F-174 (Execution)
F-174 ─► F-175 (Simultaneous) ─► F-170 (Round)
F-170 ─► F-210 (Run Loop)
```

---

## Implementation Phases

| Phase | Focus | Features |
|-------|-------|----------|
| 1 | Foundation | F-100, F-101, F-102, F-103 |
| 2 | Entities | F-110 – F-115, F-130, F-131 |
| 3 | Board | F-120 – F-124 |
| 4 | Dice Mechanics | F-132 – F-139 |
| 5 | Damage | F-150 – F-156 |
| 6 | Targeting | F-160 – F-167 |
| 7 | Combat Flow | F-170 – F-178 |
| 8 | Victory + Edge | F-180 – F-183, F-190 – F-193 |
| 9 | Enemy AI | F-200 – F-203 |
| 10 | Run Structure | F-210 – F-216 |
| 11 | UI | F-220 – F-229 |
| 12 | Content | F-240 – F-245 |
