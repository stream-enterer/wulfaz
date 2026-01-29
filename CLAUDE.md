# Wulfaz

Mech autobattler roguelike in Go using TEA (The Elm Architecture).

## Key Files

- `RULES.md` — TEA architecture rules (MUST follow)
- `DESIGN.md` — Game design, types, deferred items

## Architecture

TEA pattern: `Model` → `Update(Msg)` → `(Model, Cmd)` → `View`

All randomness in Msg payloads (seeded RNG for replay/undo).

## Code Rules

1. **Value types only** — No pointers in Model or entity types
2. **Value receivers** — All Model methods use value receivers
3. **Past-tense Msgs** — `CombatStarted`, `AbilityActivated`, not `StartCombat`
4. **Optional fields** — Use `HasX bool` pattern, not pointers

## Package Dependencies

```
core ← entity ← model ← tea
         ↑
       event ← effect
         ↑
      template
```

No cycles allowed.

## Data

Templates in `data/templates/` as KDL 1.0 files.
