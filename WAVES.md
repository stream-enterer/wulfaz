# Implementation Waves — Wulfaz MVP

**Source:** FEATURES.md (61 features)

---

## Strategy

Build toward a **playable vertical slice** first, then expand. The dice combat system is the core gameplay loop — everything else supports it.

**Principles:**
1. Unblock dependencies early
2. Complete one wave before starting the next
3. Each wave should produce testable functionality
4. Reassess priorities after each wave

---

## Completed Waves

### Wave 1: Unlock Blockers ✓
Command unit entity (F-114) and die entity (F-130) — the primitives that unblock downstream work.

### Wave 2: Dice Core ✓
Dice rolling (F-131–F-133), lock/reroll mechanics (F-134–F-135), and effects for damage/shield/heal (F-136–F-139).

### Legacy Cleanup ✓
Removed tick-based autocombat system to enable phase-based dice combat.

### Wave 3: Combat Phases ✓
Phase structure (F-171–F-178): Preview → Player Command → Enemy Command → Execution → Round End → repeat. Includes stub targeting, simultaneous resolution, left-to-right firing order, and shield expiration (F-154).

### Wave 4: Targeting ✓
Positional targeting with lowest HP priority (F-160–F-161), MTG-style overflow damage (F-163), units-only-target-units rule (F-167). Command unit targeting (F-164–F-165) done in Wave 3. Gap-to-command constrained by F-167: hits command only when all enemy units dead.

### Wave 5: Death & Victory ✓
Death system (F-152, F-155, F-156): Dead units removed at round end, damage persists between fights via PlayerRoster, permadeath through roster sync. Shield buffer (F-153) absorbs damage first. Victory conditions (F-180–F-183) implemented in Wave 3: checkCombatEnd detects command death, immediate combat end, player wins ties.

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

| Wave | Features | Focus | Status |
|------|----------|-------|--------|
| 1–3 | 20 | Dice core, combat phases | ✓ |
| 4 | 8 | Targeting (positional, overflow) | ✓ |
| 5 | 8 | Death & victory (shields, permadeath, win) | ✓ |
| 6 | 25 | Polish & content (edge cases, AI, UI, templates) | |
| **Total** | **61** | | **36 done** |

---

## Dependencies to Watch

- **F-124 (Dead Unit Gap Handling)** affects board display when units die — dead units are now removed at round end

---

## Next Steps

1. **Plan Wave 6** — Polish & content (edge cases, AI, UI, templates)
2. **Iterate** — Complete each wave, reassess, continue
