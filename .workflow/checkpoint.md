# Checkpoint

## Active Task
Replace Stamina system with Fatigue system

## Modified Files
- `src/components.rs` — Stamina→Fatigue struct, removed Charge/Flee from ActionId
- `src/world.rs` — staminas→fatigues property table (field, despawn, validate, init, tests)
- `src/systems/fatigue.rs` — new Phase 2 system (recovery + excess HP damage)
- `src/systems/stamina.rs` — DELETED
- `src/systems/gait_selection.rs` — DELETED
- `src/systems/decisions.rs` — StaminaRatio→FatigueRatio, removed select_flee_target, Charge/Flee
- `src/systems/wander.rs` — removed FLEE_RANGE, Flee arm, Charge from movement
- `src/systems/combat.rs` — fatigue gain per attack, fatigue stat modifiers, unconscious check
- `src/systems/mod.rs` — stamina/gait_selection→fatigue module
- `src/loading.rs` — removed max_stamina, insert Fatigue{current:0}
- `src/main.rs` — run_stamina/run_gait_selection→run_fatigue
- `data/creatures.kdl` — removed max_stamina
- `data/utility.ron` — removed Charge/Flee defs, added FatigueRatio to Attack
- `tests/invariants.rs` — stamina→fatigue in spawn_creature, run_full_tick
- `tests/determinism.rs` — stamina→fatigue in spawn_creature, run_full_tick, WorldSnapshot

## Decisions
- Fatigue starts at 0, gains 1.0 per attack
- Recovery: 0.2/tick normal, 1.0/tick if >= 100 (unconscious fast recovery)
- Stat degradation: -1 defense per 10 fatigue, -1 attack per 20 fatigue
- Unconscious at 100: can't attack, 0 effective defense
- HP damage above 200: 1 per 50 excess, remainder*2% chance of +1
- FatigueRatio in utility scorer: high fatigue reduces attack desire

## Status
All 281 tests pass (135×2 unit + 5 determinism + 6 invariants). No warnings. Ready to commit.
