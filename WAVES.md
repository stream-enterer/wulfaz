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

### Wave 6: Dice UI ✓
Display and interaction (F-223–F-226): Shield display ("HP:X SH:Y"), round toast overlay between rounds, dice boxes with pip patterns on all units (3-die pyramid for command, centered/diagonal for board units), full interaction UI (left-click select/target, right-click lock, R reroll, auto-advance when all dice activated). Command units positioned above/below boards. Combat is now playable.

---

---

## Wave 7: Combat Visualization

**Goal:** Make combat understandable — show what's happening and why.

| ID | Feature | Why Now |
|----|---------|---------|
| F-228 | Execution Phase Visual Delay | See damage as it happens |
| F-227 | Targeting Lines Display | Understand who targets whom |
| F-202 | Enemy Targeting Display (Lines) | See enemy intent in Preview |

**Deliverable:** Combat is readable. Players understand the flow and can anticipate outcomes.

---

## Wave 8: Combat Edge Cases

**Goal:** Fix gameplay bugs and edge cases that break combat logic.

| ID | Feature | Why Now |
|----|---------|---------|
| F-192 | Dead Target Skip | AI must retarget when target dies |
| F-124 | Dead Unit Gap Handling | Board display when units die |
| F-190 | Pure Command vs Command | Handle all-units-dead scenario |
| F-191 | Zero-Dice Unit Handling | Units with no dice shouldn't break |
| F-193 | Simultaneous Death Resolution | Multiple deaths in one frame |

**Deliverable:** Combat handles all edge cases correctly. No crashes or undefined behavior.

---

## Wave 9: Run Structure

**Goal:** Complete the roguelike loop — actions between fights and content variety.

| ID | Feature | Why Now |
|----|---------|---------|
| F-215 | Repair Action | Restore HP between fights |
| F-216 | Free Repositioning | Tactical depth between fights |
| F-243 | MVP Fight Encounter Templates | Multiple enemy compositions |
| F-245 | MVP Event Templates | Meaningful choice events |

**Deliverable:** Full run loop with repair, repositioning, varied encounters, and events.

---

## Wave 10: Polish & Systems

**Goal:** Entity system cleanup, AI improvements, and remaining content.

| ID | Feature | Why Now |
|----|---------|---------|
| F-113 | Size System (Spaces + Dice Count) | Proper size mechanics |
| F-115 | Unit Type Tags (Mech/Vehicle/BA) | Type-based effects |
| F-123 | Command Unit Off-Board Position | Visual distinction |
| F-200 | Enemy Dice Rolling (No Reroll) | AI dice behavior |
| F-201 | Enemy Target Heuristics | Smarter AI targeting |
| F-203 | Player-First Resolution Order | Balance tuning |
| F-241 | Command Unit Template Schema | Data cleanup |
| F-244 | Symmetric Unit Pool | Content variety |

**Deliverable:** Complete MVP with polished systems.

---

## Summary

| Wave | Features | Focus | Status |
|------|----------|-------|--------|
| 1–3 | 20 | Dice core, combat phases | ✓ |
| 4 | 8 | Targeting (positional, overflow) | ✓ |
| 5 | 8 | Death & victory (shields, permadeath, win) | ✓ |
| 6 | 4 | Dice UI (display, interact, shields, round) | ✓ |
| 7 | 3 | Combat visualization (delays, lines) | |
| 8 | 5 | Edge cases (dead targets, gaps, zero dice) | |
| 9 | 4 | Run structure (repair, reposition, content) | |
| 10 | 8 | Polish & systems (size, tags, AI, templates) | |
| **Total** | **60** | | **40 done** |

---

## Dependencies to Watch

- **F-228 (Execution Delay)** — Needed before F-227 targeting lines make sense
- **F-124 (Dead Unit Gap Handling)** — Affects board display when units die

---

## Next Steps

1. **Wave 7: Combat Visualization** — Add execution delays and targeting lines
2. Combat is now playable — iterate on feedback
