# Checkpoint

## Active Task
None — utility scorer implementation complete.

## Completed
- Utility scorer (Phase 3 decision system) fully implemented and tested.

## Modified Files
- `Cargo.toml` — added serde + ron deps
- `src/components.rs` — ActionId, Intention, ActionState types
- `src/world.rs` — new tables, despawn, validate_world
- `src/systems/decisions.rs` — scorer system (config types, curves, run_decisions) + 18 tests
- `src/systems/mod.rs` — added decisions module
- `src/main.rs` — wired run_decisions into Phase 3, load_utility_config call
- `src/loading.rs` — load_utility_config, ActionState init in load_creatures
- `src/systems/eating.rs` — gates on Eat intention with legacy fallback
- `src/systems/combat.rs` — gates on Attack intention with legacy fallback
- `src/systems/wander.rs` — skips entities with non-Wander intention
- `data/utility.ron` — RON config with Idle/Wander/Eat/Attack curves
- `tests/determinism.rs` — updated for Phase 3 (run_decisions, ActionState, intentions snapshot)

## Decisions
- Config types live in decisions.rs, imported by world.rs (Option A)
- RON for config (separates engine tuning from KDL game content)
- Phase 4 systems use fallback logic when no intention present (backward compat)
- Rust 2024 edition requires `|&(&e, _)|` pattern in filter closures

## Next Action
All tests pass (225 total). Ready for commit.
