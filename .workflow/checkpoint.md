# Checkpoint

## Active Task
Charging/Fleeing AI + Stamina System

## Modified Files
- `src/components.rs` — Stamina struct, Charge/Flee in ActionId
- `src/world.rs` — staminas property table (field, despawn, validate, init, test)
- `src/systems/stamina.rs` — new Phase 2 system, drain/recover by gait
- `src/systems/gait_selection.rs` — new Phase 4 system, gait from intention+stamina
- `src/systems/decisions.rs` — StaminaRatio axis, select_flee_target(), Charge/Flee targets
- `src/systems/wander.rs` — Flee away-movement, Charge toward-movement
- `src/systems/combat.rs` — Charge recognized as attack
- `src/systems/mod.rs` — registered stamina, gait_selection modules
- `src/loading.rs` — parse max_stamina from KDL, insert Stamina at spawn
- `src/main.rs` — wired run_stamina (Phase 2), run_gait_selection (Phase 4)
- `data/creatures.kdl` — added max_stamina to all creatures
- `data/utility.ron` — added Charge and Flee ActionDefs with StaminaRatio
- `tests/invariants.rs` — stamina in spawn_creature, new systems in run_full_tick
- `tests/determinism.rs` — stamina in spawn_creature/snapshot, new systems in run_full_tick

## Decisions
- Stamina rates: Sprint -2.0, Run -1.0, Hustle -0.3, Walk +0.5, Stroll +1.0, Creep +2.0
- Charge degrades: Sprint→Run→Hustle (never slower than Hustle)
- Flee degrades: Sprint→Run→Walk (exhausted prey is vulnerable)
- select_flee_target: nearest entity with aggression > 0.2
- Flee direction: normalized (pos - threat) × FLEE_RANGE, clamped to map

## Status
All 297 tests pass (143×2 unit + 5 determinism + 6 invariants). No warnings. Ready to commit.
