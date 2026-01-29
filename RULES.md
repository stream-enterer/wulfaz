# The Elm Architecture in Go: Explicit Rule Sheet

A prescriptive, copy-pasteable specification for implementing TEA in Go. No ambiguity. No "it depends." Follow these rules.

---

## 1. Core Types

### 1.1 Model

```go
// Model is a struct containing ALL application state.
// It MUST be a value type (not a pointer).
type Model struct {
    // All fields that represent application state
}
```

**Rules:**
- Model is a struct. Always.
- Model is passed and returned by value, not pointer.
- Model contains no functions, channels, mutexes, or other runtime handles.
- Model must be serializable (for debugging, time-travel, persistence).
- There is exactly ONE Model for the entire application (nested Models for composition are stored as fields within it).

**Violations:**
```go
// WRONG: pointer to model
type Model *AppState

// WRONG: function in model
type Model struct {
    OnClick func()  // NO
}

// WRONG: channel in model
type Model struct {
    Updates chan string  // NO
}
```

---

### 1.2 Msg

```go
// Msg is any type. In Go, use an empty interface or a sealed interface.
type Msg interface{}
```

**Rules:**
- Msg represents "what happened," not "what to do."
- Each distinct event gets its own concrete type.
- Msg types are typically small structs or type aliases.
- Name Msg types as past-tense events: `ClickedSubmit`, `ReceivedResponse`, `TickOccurred`.

**Preferred pattern — sealed interface with unexported method:**
```go
type Msg interface {
    isMsg()  // unexported = only this package can implement
}

type ClickedButton struct{}
func (ClickedButton) isMsg() {}

type ReceivedData struct {
    Data []byte
    Err  error
}
func (ReceivedData) isMsg() {}
```

**Acceptable alternative — open interface (Bubbletea style):**
```go
type Msg = interface{}  // any type is a Msg

type ClickedButton struct{}
type ReceivedData struct {
    Data []byte
    Err  error
}
```

**Violations:**
```go
// WRONG: imperative names
type DoFetch struct{}      // NO — use FetchRequested or UserClickedFetch
type SetLoading struct{}   // NO — this is an action, not an event

// WRONG: god message
type Action struct {
    Type string
    Payload interface{}
}  // NO — use concrete types, match exhaustively
```

---

### 1.3 Cmd

```go
// Cmd is a function that performs a side effect and returns a Msg.
// It takes no arguments.
type Cmd func() Msg
```

**Rules:**
- Cmd is a thunk: a deferred, described effect.
- Cmd performs IO (HTTP, file, time, random) and returns a Msg with the result.
- Cmd is executed by the runtime, NEVER by user code.
- Cmd returns exactly one Msg (or nil for fire-and-forget effects).
- Cmds have NO ordering guarantees when batched.

**Standard Cmd constructors:**
```go
// None returns a nil Cmd (no effect).
func None() Cmd {
    return nil
}

// Batch combines multiple Cmds. Execution order is NOT guaranteed.
func Batch(cmds ...Cmd) Cmd {
    return func() Msg {
        var wg sync.WaitGroup
        msgs := make(chan Msg, len(cmds))
        for _, cmd := range cmds {
            if cmd == nil {
                continue
            }
            wg.Add(1)
            go func(c Cmd) {
                defer wg.Done()
                if m := c(); m != nil {
                    msgs <- m
                }
            }(cmd)
        }
        wg.Wait()
        close(msgs)
        // Return first msg or nil; runtime handles the rest
        for m := range msgs {
            return m  // simplified; real impl collects all
        }
        return nil
    }
}
```

**Effect factories (examples):**
```go
// Fetch creates a Cmd that performs an HTTP GET.
func Fetch(url string) Cmd {
    return func() Msg {
        resp, err := http.Get(url)
        if err != nil {
            return FetchFailed{Err: err}
        }
        defer resp.Body.Close()
        body, _ := io.ReadAll(resp.Body)
        return FetchSucceeded{Data: body}
    }
}

// After creates a Cmd that waits, then sends a Msg.
func After(d time.Duration, msg Msg) Cmd {
    return func() Msg {
        time.Sleep(d)
        return msg
    }
}
```

**Violations:**
```go
// WRONG: executing effect in Update
func (m Model) Update(msg Msg) (Model, Cmd) {
    resp, _ := http.Get(url)  // NO — side effect in pure function
    ...
}

// WRONG: Cmd that takes arguments at call time
type Cmd func(url string) Msg  // NO — Cmd is a thunk, no args

// WRONG: assuming Cmd order
cmd := Batch(saveToDb(), sendEmail())  // Order NOT guaranteed
```

---

### 1.4 Sub (Subscriptions)

```go
// Sub is a function that returns a channel of Msgs.
// The runtime reads from this channel and dispatches Msgs to Update.
type Sub func(ctx context.Context) <-chan Msg
```

**Rules:**
- Sub represents continuous event sources (time, keyboard, websocket).
- Sub is declared based on current Model state.
- The runtime manages Sub lifecycle (start/stop based on diff).
- Sub must respect context cancellation.

**Subscription factory (example):**
```go
// Every creates a Sub that emits a Msg at regular intervals.
func Every(d time.Duration, msgFn func(time.Time) Msg) Sub {
    return func(ctx context.Context) <-chan Msg {
        ch := make(chan Msg)
        go func() {
            defer close(ch)
            ticker := time.NewTicker(d)
            defer ticker.Stop()
            for {
                select {
                case <-ctx.Done():
                    return
                case t := <-ticker.C:
                    ch <- msgFn(t)
                }
            }
        }()
        return ch
    }
}
```

---

### 1.5 Effect Chains (Task Pattern)

When effects must execute sequentially (e.g., load file → parse → initialize), use intermediate Msgs to chain them. Each step is a Msg; the state machine is explicit in Update.

**Pattern:**
```go
// Step 1: User initiates
case LoadGameRequested:
    m.Loading = true
    return m, LoadSaveFile(msg.Path)

// Step 2: File loaded, now parse
case SaveFileLoaded:
    return m, ParseSaveData(msg.Data)

// Step 3: Parsed, now initialize game state
case SaveDataParsed:
    m.Loading = false
    m.GameState = msg.GameState
    return m, InitializeSystems(m.GameState)

// Step 4: Ready
case SystemsInitialized:
    m.Ready = true
    return m, nil

// Error at any step
case LoadFailed:
    m.Loading = false
    m.Error = msg.Err
    return m, nil
```

**Rules:**
- Each step returns a Cmd that produces the next Msg.
- Each intermediate state is inspectable (debugging, save/load).
- Errors are just another Msg type — handle explicitly.
- No hidden continuations or callbacks.

**Game example — Attack resolution:**
```go
// Player declares attack
case AttackDeclared:
    m.Phase = PhaseResolvingAttack
    m.PendingAttack = msg
    return m, RollToHit(msg.Attacker, msg.Target, msg.Weapon)

// To-hit resolved
case ToHitRolled:
    if !msg.Hit {
        m.Phase = PhaseAwaitingInput
        m.Log = append(m.Log, "Attack missed")
        return m, nil
    }
    return m, RollHitLocation(msg.Target)

// Location determined
case HitLocationRolled:
    return m, ApplyDamage(msg.Target, msg.Location, m.PendingAttack.Weapon.Damage)

// Damage applied, check criticals
case DamageApplied:
    m.Mechs = msg.UpdatedMechs
    if msg.CriticalsPossible {
        return m, RollCriticals(msg.Target, msg.Location)
    }
    m.Phase = PhaseAwaitingInput
    return m, nil

// And so on...
```

**Benefits:**
- Full replay: save the Msg sequence, replay for debugging.
- Time-travel: step backward/forward through attack resolution.
- Testable: unit test each transition independently.
- Interruptible: user can save mid-resolution if needed.

**Violations:**
```go
// WRONG: Chaining via callbacks in Cmd
func AttackCmd(attacker, target, weapon) Cmd {
    return func() Msg {
        hit := rollToHit(...)
        if hit {
            loc := rollLocation(...)  // Hidden state machine
            dmg := applyDamage(...)   // No intermediate Msgs
            ...
        }
        return AttackComplete{...}  // Loses all intermediate state
    }
}
```

---

## 2. Core Functions

### 2.1 Init

```go
func Init(flags Flags) (Model, Cmd)
```

**Rules:**
- Init is called once at program start.
- Init returns initial Model and optional startup Cmd.
- Flags contains external configuration (CLI args, env vars, etc.).
- If no startup effect needed, return `nil` for Cmd.

**Example:**
```go
type Flags struct {
    APIEndpoint string
}

func Init(flags Flags) (Model, Cmd) {
    return Model{
        Endpoint: flags.APIEndpoint,
        Loading:  true,
    }, Fetch(flags.APIEndpoint + "/init")
}
```

---

### 2.2 Update

```go
func Update(msg Msg, model Model) (Model, Cmd)
```

**Or as a method (Bubbletea style):**
```go
func (m Model) Update(msg Msg) (Model, Cmd)
```

**Rules:**
- Update is a PURE function: same inputs → same outputs, no side effects.
- Update handles ALL Msg variants exhaustively (use type switch).
- Update returns a NEW Model; never mutate the input.
- Update returns Cmd for any needed effects; return `nil` if none.
- Update is the ONLY place state transitions occur.

**Value receiver mandate:**
```go
// CORRECT: value receiver, returns new model
func (m Model) Update(msg Msg) (Model, Cmd) {
    switch msg := msg.(type) {
    case ClickedIncrement:
        m.Count++  // modifying copy, not original
        return m, nil
    case ReceivedData:
        m.Data = msg.Data
        m.Loading = false
        return m, nil
    default:
        return m, nil
    }
}
```

**Violations:**
```go
// WRONG: pointer receiver
func (m *Model) Update(msg Msg) (Model, Cmd) { ... }

// WRONG: side effect in Update
func (m Model) Update(msg Msg) (Model, Cmd) {
    log.Println("updating")  // NO — side effect
    ...
}

// WRONG: not handling all cases
func (m Model) Update(msg Msg) (Model, Cmd) {
    if msg, ok := msg.(ClickedIncrement); ok {
        ...
    }
    // Missing default — silent failures
}
```

---

### 2.3 View

```go
func View(model Model) UI
```

**Or as a method:**
```go
func (m Model) View() string  // for TUI
func (m Model) View() Template  // for web
```

**Rules:**
- View is a PURE function: same Model → same output.
- View NEVER triggers side effects.
- View returns a DESCRIPTION of UI, not imperative commands.
- View receives the Model; it does not access global state.

**Example (TUI):**
```go
func (m Model) View() string {
    if m.Loading {
        return "Loading..."
    }
    return fmt.Sprintf("Count: %d\n\nPress q to quit.", m.Count)
}
```

**Violations:**
```go
// WRONG: accessing global state
var globalCounter int
func (m Model) View() string {
    return fmt.Sprintf("Count: %d", globalCounter)  // NO
}

// WRONG: side effect in View
func (m Model) View() string {
    log.Println("rendering")  // NO
    ...
}
```

---

### 2.4 Subscriptions

```go
func Subscriptions(model Model) Sub
```

**Or as a method:**
```go
func (m Model) Subscriptions() Sub
```

**Rules:**
- Returns Sub based on current Model state.
- Called after every Update.
- Runtime diffs old vs new Sub and manages lifecycle.
- Return `nil` or `Sub.None()` when no subscriptions needed.

**Example:**
```go
func (m Model) Subscriptions() Sub {
    if m.TimerRunning {
        return Every(time.Second, func(t time.Time) Msg {
            return TickOccurred{Time: t}
        })
    }
    return nil
}
```

---

## 3. The Runtime Contract

The runtime (you write this, or use Bubbletea) owns:

| Responsibility | User Code | Runtime |
|---------------|-----------|---------|
| Holds current Model | ✗ | ✓ |
| Calls Init | ✗ | ✓ |
| Calls Update | ✗ | ✓ |
| Calls View | ✗ | ✓ |
| Executes Cmds | ✗ | ✓ |
| Manages Subs | ✗ | ✓ |
| Renders UI | ✗ | ✓ |
| Dispatches Msgs | ✗ | ✓ |

**User code is purely functional. Runtime is imperative.**

**Minimal runtime loop:**
```go
func Run(init func(Flags) (Model, Cmd), update func(Msg, Model) (Model, Cmd), view func(Model) string, flags Flags) {
    model, cmd := init(flags)
    msgs := make(chan Msg, 256)
    
    // Execute initial command
    if cmd != nil {
        go func() { msgs <- cmd() }()
    }
    
    // Main loop
    for {
        render(view(model))
        
        select {
        case msg := <-msgs:
            var newCmd Cmd
            model, newCmd = update(msg, model)
            if newCmd != nil {
                go func(c Cmd) { msgs <- c() }(newCmd)
            }
        case msg := <-inputChannel:
            // Handle user input
            var newCmd Cmd
            model, newCmd = update(msg, model)
            if newCmd != nil {
                go func(c Cmd) { msgs <- c() }(newCmd)
            }
        }
    }
}
```

---

## 4. Game Loop Integration

TEA's runtime must integrate with a game loop. For turn-based games, use an event-driven approach. For real-time elements (animations, transitions), use a hybrid model.

### 4.1 Event-Driven Loop (Turn-Based)

For turn-based games like Battletech, the loop blocks until player input:

```go
func Run(model Model) {
    for {
        render(model.View())

        msg := waitForInput()  // Blocks until player acts
        var cmd Cmd
        model, cmd = model.Update(msg)

        // Execute command, feed resulting Msgs back
        for cmd != nil {
            resultMsg := cmd()
            if resultMsg == nil {
                break
            }
            model, cmd = model.Update(resultMsg)
        }
    }
}
```

**Rules:**
- Game logic advances ONLY on player input or Cmd completion.
- No continuous ticking. No delta time.
- Each Msg produces a new Model; chain until quiescent.

### 4.2 Hybrid Loop (Animations + Turn-Based Logic)

When you need smooth animations but discrete game logic:

```go
func Run(model Model) {
    msgs := make(chan Msg, 256)

    // Animation ticker (variable rate, cosmetic only)
    go func() {
        for range time.Tick(16 * time.Millisecond) {  // ~60fps
            msgs <- AnimationTickMsg{}
        }
    }()

    for {
        render(model.View())

        msg := <-msgs
        var cmd Cmd
        model, cmd = model.Update(msg)

        if cmd != nil {
            go func(c Cmd) {
                if m := c(); m != nil {
                    msgs <- m
                }
            }(cmd)
        }
    }
}
```

**Rules:**
- `AnimationTickMsg` updates cosmetic state (sprite positions, particles).
- Core game logic still advances only on player actions.
- Animation state lives in Model but doesn't affect game rules.

### 4.3 Fixed Timestep (Real-Time Simulation)

For games with physics or real-time simulation:

```go
const tickRate = 60
const tickDuration = time.Second / tickRate

func Run(model Model) {
    msgs := make(chan Msg, 256)
    var accumulator time.Duration
    lastTime := time.Now()

    // Fixed logic tick
    go func() {
        for range time.Tick(tickDuration) {
            msgs <- TickMsg{}
        }
    }()

    for {
        // Interpolated render (alpha = accumulator / tickDuration)
        render(model.View())

        msg := <-msgs
        model, _ = model.Update(msg)
    }
}
```

**Rules:**
- `TickMsg` arrives at fixed rate (e.g., 60Hz).
- View may interpolate between states for smooth rendering.
- Deterministic: same Msg sequence → same result.

### 4.4 Choosing Your Loop

| Game Type | Loop Style | Tick Source |
|-----------|------------|-------------|
| Turn-based (Battletech, Chess) | Event-driven | Player input only |
| Turn-based + animations | Hybrid | Input + animation ticker |
| Real-time (platformer, RTS) | Fixed timestep | Fixed timer + input |
| Physics-heavy | Fixed timestep | Fixed timer (120Hz+) |

---

## 5. Composition Rules

### 5.1 The Anti-Component Rule

**DO NOT create nested Model/Update/Msg triplets by default.**

TEA is not OOP. There are no "components" with encapsulated state and methods.

**Default approach — pure view functions:**
```go
// Good: stateless view helper
func viewButton(label string, onClick Msg) string {
    return fmt.Sprintf("[%s]", label)
}

// Good: view function that takes data, not its own Model
func viewUserCard(user User) string {
    return fmt.Sprintf("Name: %s\nEmail: %s", user.Name, user.Email)
}
```

### 5.2 When Nested TEA Is Acceptable

Only when a widget has **internal view state that no other part of the app cares about**:
- Date picker with open/closed state
- Autocomplete with suggestion visibility
- Sortable table with current sort column

**Pattern for nested components:**
```go
// In widget package
type Model struct { /* widget-internal state */ }
type Msg interface { isWidgetMsg() }

func Init() Model { ... }
func (m Model) Update(msg Msg) (Model, Cmd) { ... }
func (m Model) View() string { ... }

// In parent
type AppModel struct {
    DatePicker datepicker.Model  // nested
    // ... other app state
}

func (m AppModel) Update(msg Msg) (AppModel, Cmd) {
    switch msg := msg.(type) {
    case datepicker.Msg:
        var cmd Cmd
        m.DatePicker, cmd = m.DatePicker.Update(msg)
        return m, wrapCmd(cmd)  // map widget Cmd to app Cmd
    // ... handle app-level msgs
    }
}
```

### 5.3 Opaque State Pattern

For reusable widgets, expose opaque types:
```go
// Package: textinput

// Model is opaque — fields unexported
type Model struct {
    value   string
    cursor  int
    focused bool
}

// Exported operations
func New() Model { return Model{} }
func (m Model) Value() string { return m.value }
func (m Model) SetValue(s string) Model { m.value = s; return m }
func (m Model) Focus() Model { m.focused = true; return m }
func (m Model) Update(msg Msg) (Model, Cmd) { ... }
func (m Model) View() string { ... }
```

---

## 6. Immutability Discipline

Go does not enforce immutability. You must. Strict immutability is not optional — it enables the entire TEA value proposition.

**What strict immutability gives you:**
- **Time-travel debugging**: Step backward/forward through any game state.
- **Undo/redo for free**: Previous Model is still valid; just swap it back.
- **Replay**: Save a Msg sequence, replay to reproduce any bug.
- **Save/load trivial**: `json.Marshal(model)` — done.
- **Fearless refactoring**: No hidden mutation means no hidden bugs.

### 6.1 Value Receivers Only for Model Methods

```go
// CORRECT
func (m Model) Update(msg Msg) (Model, Cmd) { ... }
func (m Model) View() string { ... }

// WRONG
func (m *Model) Update(msg Msg) (Model, Cmd) { ... }
```

**Why:** Value receiver copies the Model. Modifications affect the copy, preserving the original. This makes Update naturally pure.

### 6.2 Return Modified Copies

```go
func (m Model) Update(msg Msg) (Model, Cmd) {
    switch msg := msg.(type) {
    case AddItem:
        // Create new slice, don't append in place
        newItems := make([]Item, len(m.Items)+1)
        copy(newItems, m.Items)
        newItems[len(m.Items)] = msg.Item
        m.Items = newItems
        return m, nil
    }
}
```

### 6.3 Deep Copy Reference Types

Slices, maps, and pointers in Model require explicit copying:
```go
func (m Model) Update(msg Msg) (Model, Cmd) {
    switch msg := msg.(type) {
    case UpdateTags:
        // WRONG: modifies shared backing array
        // m.Tags = append(m.Tags, msg.Tag)
        
        // CORRECT: copy first
        newTags := make([]string, len(m.Tags), len(m.Tags)+1)
        copy(newTags, m.Tags)
        m.Tags = append(newTags, msg.Tag)
        return m, nil
    }
}
```

### 6.4 No Pointers in Model

```go
// CORRECT: value types
type Model struct {
    User   User      // value
    Mechs  []Mech    // slice of values
    Map    HexMap    // value
}

// WRONG: pointers break immutability guarantees
type Model struct {
    User  *User      // NO — shared state risk
    World *ecs.World // NO — mutation hidden from TEA
}
```

**No exceptions for turn-based games.** With entity counts in the dozens (not thousands), copying is trivially cheap. The debugging benefits far outweigh any micro-optimization.

**If you think you need pointers**, you're either:
1. Prematurely optimizing (profile first).
2. Building a real-time simulation (different ruleset needed).
3. Fighting the architecture (reconsider your data layout).

---

## 7. Package Organization

### 7.1 Single Module for Small Apps

```
myapp/
├── main.go      // runtime, Init call
├── model.go     // Model type
├── update.go    // Update function, all Msg types
├── view.go      // View function
├── cmd.go       // Cmd constructors (effects)
└── sub.go       // Sub constructors (subscriptions)
```

### 7.2 Feature Modules for Large Apps

```
myapp/
├── main.go
├── app/
│   ├── model.go
│   ├── update.go
│   ├── view.go
│   └── msg.go
├── user/           // domain module
│   ├── user.go     // User type + operations
│   └── api.go      // Cmd factories for user API
├── components/     // only if truly needed
│   └── datepicker/
│       ├── model.go
│       ├── update.go
│       └── view.go
└── effects/
    ├── http.go
    └── time.go
```

### 7.3 Rules

- **DO NOT** create `models/`, `views/`, `updates/` packages.
- **DO** organize by domain concept, not by TEA role.
- **DO** keep Msg types in the same file as Update.
- **DO** delay extraction until you feel pain.

---

## 8. Testing

### 8.1 Update is Trivially Testable

```go
func TestUpdate_Increment(t *testing.T) {
    initial := Model{Count: 0}
    msg := ClickedIncrement{}
    
    got, cmd := initial.Update(msg)
    
    if got.Count != 1 {
        t.Errorf("Count = %d, want 1", got.Count)
    }
    if cmd != nil {
        t.Error("expected no command")
    }
}
```

### 8.2 Test Cmd Factories Separately

```go
func TestFetch_Success(t *testing.T) {
    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        w.Write([]byte("ok"))
    }))
    defer server.Close()
    
    cmd := Fetch(server.URL)
    msg := cmd()
    
    got, ok := msg.(FetchSucceeded)
    if !ok {
        t.Fatalf("got %T, want FetchSucceeded", msg)
    }
    if string(got.Data) != "ok" {
        t.Errorf("Data = %q, want %q", got.Data, "ok")
    }
}
```

### 8.3 Test Sequences

```go
func TestCounterFlow(t *testing.T) {
    m := Model{Count: 0}
    
    // Simulate user clicking increment 3 times
    for i := 0; i < 3; i++ {
        m, _ = m.Update(ClickedIncrement{})
    }
    
    if m.Count != 3 {
        t.Errorf("Count = %d, want 3", m.Count)
    }
}
```

### 8.4 Property-Based Testing

Because Update is pure, it's ideal for property testing:
```go
func TestUpdate_NeverPanics(t *testing.T) {
    rapid.Check(t, func(t *rapid.T) {
        m := generateModel(t)
        msg := generateMsg(t)
        
        // Should never panic
        _, _ = m.Update(msg)
    })
}
```

---

## 9. Checklist

### 9.1 Core TEA

Before shipping, verify:

- [ ] Model is a struct with no functions, channels, or mutexes
- [ ] All Model methods use value receivers
- [ ] Update handles all Msg types (exhaustive type switch)
- [ ] Update performs no side effects
- [ ] View performs no side effects
- [ ] Cmds are executed only by the runtime
- [ ] No global mutable state
- [ ] Slices/maps in Model are copied before modification
- [ ] Msg types are named as past-tense events
- [ ] Nested TEA used only for isolated view-state widgets

### 9.2 Strict Immutability

- [ ] No pointers in Model (no `*T` fields)
- [ ] No ECS world or mutable subsystems in Model
- [ ] `json.Marshal(model)` works without custom serializers
- [ ] Undo/redo works by swapping Model values
- [ ] Replay works by replaying Msg sequence from Init

### 9.3 Game Loop

- [ ] Game logic advances only on explicit Msgs (player input, Cmd result)
- [ ] No continuous ticking unless required (animations, real-time)
- [ ] Loop style matches game type (event-driven for turn-based)

### 9.4 Effect Chains

- [ ] Sequential effects use Task Pattern (intermediate Msgs)
- [ ] No hidden state machines inside Cmds
- [ ] Each effect step is a separate Msg type
- [ ] Error cases are explicit Msg types (not panics)
- [ ] Any multi-step sequence can be interrupted/resumed

---

## 10. Quick Reference

```go
// === TYPES ===
type Model struct { ... }      // App state (value type)
type Msg interface{}           // Events  
type Cmd func() Msg            // Deferred effects
type Sub func(context.Context) <-chan Msg  // Continuous events

// === FUNCTIONS ===
func Init(flags Flags) (Model, Cmd)
func (m Model) Update(msg Msg) (Model, Cmd)  // PURE, value receiver
func (m Model) View() string                  // PURE
func (m Model) Subscriptions() Sub

// === CMD HELPERS ===
func None() Cmd                    // No effect
func Batch(cmds ...Cmd) Cmd        // Combine (unordered!)

// === EFFECT CHAINS (Task Pattern) ===
// Step 1: User triggers action
// case ActionRequested: return m, DoFirstThing()
//
// Step 2: First thing done, do second
// case FirstThingDone: return m, DoSecondThing(msg.Result)
//
// Step 3: Complete
// case SecondThingDone: m.Done = true; return m, nil

// === TURN-BASED LOOP (runtime owns this) ===
// model, _ := Init(flags)
// for {
//     render(model.View())
//     msg := waitForInput()  // BLOCKS
//     model, cmd = model.Update(msg)
//     for cmd != nil {
//         msg = cmd()
//         model, cmd = model.Update(msg)
//     }
// }
```

---

## 11. Bubbletea Compatibility

If using [Bubbletea](https://github.com/charmbracelet/bubbletea), the interface is:

```go
type Model interface {
    Init() Cmd
    Update(Msg) (Model, Cmd)
    View() string
}
```

Key differences from this spec:
- `Init` is a method, not a standalone function
- `Init` returns only `Cmd`, not `(Model, Cmd)` — initial state is the receiver
- `Msg` is `interface{}` (any type)
- `Cmd` is `func() Msg`

All other rules in this document apply directly.

---

*This rule sheet is opinionated and prescriptive. Deviate only with explicit justification.*
