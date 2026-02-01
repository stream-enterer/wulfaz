# Implementation Waves — Wulfaz MVP

**Generated:** 2026-02-01
**Source:** FEATURES.md (61 remaining features)

---

## Strategy

Build toward a **playable vertical slice** first, then expand. The dice combat system is the core gameplay loop — everything else supports it.

**Principles:**
1. Unblock dependencies early
2. Complete one wave before starting the next
3. Each wave should produce testable functionality
4. Reassess priorities after each wave

---

## Wave 1: Unlock Blockers

**Goal:** Create the two primitives that unblock the most downstream work.

| ID | Feature | Unlocks |
|----|---------|---------|
| F-114 | Command Unit Entity | F-123, F-133, F-164, F-180, F-190, F-241 |
| F-130 | Die Entity | F-131, F-132, F-133, F-136, F-137, F-138 |

**Deliverable:** Command unit type distinct from regular units. Die struct with type and faces.

**Estimated scope:** Small — two new structs in `internal/entity/` or `internal/core/`.

---

## Wave 2: Dice Core

**Goal:** Implement the dice rolling and manipulation mechanics.

| Order | ID | Feature | Depends |
|-------|-----|---------|---------|
| 1 | F-131 | Dice Face Distribution (Data-Driven) | F-130 |
| 2 | F-132 | Unit Dice Rolling | F-130 |
| 3 | F-133 | Command Unit Dice Rolling | F-130, F-114 |
| 4 | F-134 | Lock/Unlock Mechanic | F-133 |
| 5 | F-135 | Reroll Mechanic | F-134 |
| 6 | F-136 | Dice Effect — Damage | F-130 |
| 7 | F-137 | Dice Effect — Shield | F-130 |
| 8 | F-138 | Dice Effect — Heal | F-130 |
| 9 | F-139 | Dice Activation (Click-to-Target) | F-134 |

**Deliverable:** Player can roll dice, lock/reroll command dice, and activate dice for effects.

**Estimated scope:** Medium — new dice state in Model, KDL parsing for dice blocks, UI for dice display/interaction.

---

## Wave 3: Combat Phases

**Goal:** Replace tick-based combat with discrete phase structure.

| Order | ID | Feature | Depends |
|-------|-----|---------|---------|
| 1 | F-171 | Preview Phase | F-132, F-133 |
| 2 | F-172 | Player Command Phase | F-134, F-135, F-139 |
| 3 | F-173 | Enemy Command Phase | F-133 |
| 4 | F-174 | Execution Phase | F-160 |
| 5 | F-175 | Simultaneous Resolution (Per Position) | F-174 |
| 6 | F-176 | Left-to-Right Firing Order | F-174 |
| 7 | F-177 | Round End Phase | F-154 |
| 8 | F-178 | Combat Loop (Repeat Until End) | F-177 |

**Deliverable:** Combat proceeds through Preview → Player Command → Enemy Command → Execution → Round End → repeat.

**Estimated scope:** Medium — refactor `CombatPhase` enum, new phase transition logic, remove legacy tick dispatch.

**Note:** F-174 depends on F-160 (targeting), F-177 depends on F-154 (shields). May need to pull those forward or stub them.

---

## Wave 4: Targeting

**Goal:** Implement positional targeting based on board overlap.

| Order | ID | Feature | Depends |
|-------|-----|---------|---------|
| 1 | F-160 | Positional Targeting (Overlap) | — |
| 2 | F-161 | Target Selection (Lowest HP First) | F-160 |
| 3 | F-162 | Multi-Die Separate Attacks | F-132 |
| 4 | F-163 | Overflow Damage (MTG-Style) | F-161 |
| 5 | F-164 | Command Unit Targeting (Any Enemy) | F-114, F-160 |
| 6 | F-165 | Friendly Targeting (Shield/Heal) | F-137, F-138 |
| 7 | F-166 | Gap-to-Command Fallback | F-124, F-164 |
| 8 | F-167 | Units-Only-Target-Units Rule | F-160 |

**Deliverable:** Units target enemies based on board position overlap. Damage overflows. Command units can target any enemy.

**Estimated scope:** Medium — targeting resolution logic, position-based overlap calculation.

---

## Wave 5: Death & Victory

**Goal:** Complete the damage model and implement win conditions.

### 5A: Death System
| Order | ID | Feature | Depends |
|-------|-----|---------|---------|
| 1 | F-152 | Unit Death (Immediate Removal) | — |
| 2 | F-155 | Damage Persistence (Between Fights) | — |
| 3 | F-156 | Permadeath (Destroyed Units Gone) | F-152 |

### 5B: Shield System
| Order | ID | Feature | Depends |
|-------|-----|---------|---------|
| 1 | F-153 | Shield Buffer | F-137 |
| 2 | F-154 | Shield Expiration (Round End) | F-153 |

### 5C: Victory Conditions
| Order | ID | Feature | Depends |
|-------|-----|---------|---------|
| 1 | F-180 | Win Condition (Destroy Enemy Command) | F-114, F-152 |
| 2 | F-181 | Immediate Combat End | F-180 |
| 3 | F-182 | Player Wins Ties | F-180 |
| 4 | F-183 | No Victory Screen (MVP) | F-180 |

**Deliverable:** Units die and are removed. Shields absorb damage and expire. Game ends when command unit dies.

**Estimated scope:** Medium — death removal logic, shield attribute, victory check.

---

## Wave 6: Polish & Content

**Goal:** Fill remaining gaps, add content, handle edge cases.

### 6A: Entity Gaps
| ID | Feature |
|----|---------|
| F-113 | Size System (Spaces + Dice Count) |
| F-115 | Unit Type Tags (Mech/Vehicle/BattleArmor) |
| F-123 | Command Unit Off-Board Position |
| F-124 | Dead Unit Gap Handling |

### 6B: Edge Cases
| ID | Feature |
|----|---------|
| F-190 | Pure Command vs Command (All Units Dead) |
| F-191 | Zero-Dice Unit Handling |
| F-192 | Dead Target Skip |
| F-193 | Simultaneous Death Resolution |

### 6C: Enemy AI
| ID | Feature |
|----|---------|
| F-200 | Enemy Dice Rolling (No Reroll) |
| F-201 | Enemy Target Heuristics |
| F-202 | Enemy Targeting Display (Lines) |
| F-203 | Player-First Resolution Order |

### 6D: Run Structure Gaps
| ID | Feature |
|----|---------|
| F-215 | Repair Action |
| F-216 | Free Repositioning (Between Fights) |

### 6E: UI Gaps
| ID | Feature |
|----|---------|
| F-223 | Shield Display |
| F-224 | Round Number Display |
| F-225 | Dice Display (Preview Phase) |
| F-226 | Dice Interaction UI (Player Command) |
| F-227 | Targeting Lines Display |
| F-228 | Execution Phase Visual Delay |
| F-229 | Round Toast / Continue Prompt |

### 6F: Templates & Content
| ID | Feature |
|----|---------|
| F-241 | Command Unit Template Schema |
| F-243 | MVP Fight Encounter Templates |
| F-244 | Symmetric Unit Pool |
| F-245 | MVP Event Templates |

**Deliverable:** Complete MVP with all features implemented.

**Estimated scope:** Large (25 features) — but many are small/independent and can be parallelized.

---

## Summary

| Wave | Features | Focus |
|------|----------|-------|
| 1 | 2 | Unlock blockers (Command Unit, Die Entity) |
| 2 | 9 | Dice mechanics (roll, lock, reroll, effects) |
| 3 | 8 | Combat phases (replace tick-based) |
| 4 | 8 | Targeting (positional, overflow) |
| 5 | 9 | Death & victory (shields, permadeath, win) |
| 6 | 25 | Polish & content (edge cases, AI, UI, templates) |
| **Total** | **61** | |

---

## Dependencies to Watch

Some cross-wave dependencies that may require ordering adjustments:

- **F-174 (Execution Phase)** needs F-160 (Targeting) — consider doing Wave 4 before Wave 3, or stub targeting
- **F-177 (Round End Phase)** needs F-154 (Shield Expiration) — pull F-153/F-154 into Wave 3 if needed
- **F-166 (Gap-to-Command Fallback)** needs F-124 (Dead Unit Gap Handling) — may need to pull F-124 earlier

---

## Next Steps

1. **Plan Wave 1** — Define F-114 and F-130 implementation details
2. **Implement Wave 1** — Create Command Unit and Die entities
3. **Plan Wave 2** — Design dice mechanics in detail
4. **Iterate** — Complete each wave, reassess, continue
