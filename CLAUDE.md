# Wulfaz

Mech autobattler roguelike in Go using TEA (The Elm Architecture).

## Project Structure

```
Layer 4 (top):    tea
Layer 3:          model   resolve
Layer 2:          effect  template
Layer 1:          event
Layer 0 (bottom): entity
Foundation:       core
```

A package may only import from strictly lower layers or `core`.
Permitted same-layer edge: `resolve → model`.
`app` and `ui` sit above all layers and are unconstrained.
New `internal/` packages must be assigned a layer here before use.
Templates in `data/templates/` as KDL 1.0 *NOT* 2.0.

## Reference Docs

`docs/ref/` contains source references: `ebiten/` (Ebitengine), `ebitenui/` (Ebitenui source), `ebitenui.github.io/` (Ebitenui docs).

---

## TEA Principles

### P1: State is Data
Serialize it, compare it, copy it. `fmt.Printf("%+v", state)` shows everything.

### P2: Time is Events
Same initial state + same Msg sequence = same result. Always.

### P3: Effects are Described
Update returns Cmd descriptions. Runtime executes them.

### P4: No Mutation
If anyone else can change what you hold, you have a bug. Clone slices and maps before modifying.

### P5: Explicit Over Implicit
Trace any action to its effects by reading linearly.

---

## Core Types

```go
type Model struct { Version int; /* all state */ }  // Value type, serializable
type Msg interface { isMsg() }                       // Sealed, past-tense
type Cmd func() Msg                                  // Effect thunk
type Sub struct { ID string; Run func(context.Context) <-chan Msg }

func Init(flags Flags) (Model, Cmd)
func (m Model) Update(msg Msg) (Model, Cmd)          // PURE
func (m Model) View() UI                             // PURE
func (m Model) Subscriptions() []Sub
```

---

## Rules

### Model
- Struct with `Version` first field
- Value type (never pointer)
- Only: primitives, strings, slices, maps, structs
- Never: functions, channels, mutexes, pointers
- Optional fields: `HasX bool` pattern, not pointers

### Msg
- Past-tense: `DiceRolled`, `CombatStarted` — not `RollDice`
- Sealed: unexported `isMsg()` method
- Carry results: `DiceRolled{Values: []int}` not `{Count: int}`
- Serializable: no `error` type, use `Code int, Message string`
- Defined in `model` package; `tea` consumes Msgs but must not define them (would force upward imports)

### Update
- Pure: same input → same output
- Value receiver, exhaustive type switch
- Copy slices/maps before modification
- Never: IO, logging, random, global state

### Cmd
- Thunk `func() Msg`, executed only by runtime
- No ordering when batched

---

## Task Pattern

Sequential effects use intermediate Msgs:

```go
case LoadRequested:
    return m, ReadFile(path)
case FileRead:
    return m, Parse(msg.Data)
case Parsed:
    m.State = msg.Result
    return m, nil
```

Not chained in Cmd — breaks testability, visibility, replay.

---

## Verify Before Completing

- [ ] Model: no pointers, has Version, value receivers
- [ ] Msgs: past-tense, serializable, carry results
- [ ] Update: pure, exhaustive switch, copies collections
- [ ] Sequential effects: Task Pattern
- [ ] `json.Marshal(model)` works
- [ ] Imports: every `internal/` import targets a lower layer or permitted same-layer edge (see Project Structure)
