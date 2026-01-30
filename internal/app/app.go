package app

import (
	"math/rand/v2"
	"time"

	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/inpututil"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
	"wulfaz/internal/tea"
	"wulfaz/ui/renderer"
)

const (
	screenWidth  = 800
	screenHeight = 600
	tickInterval = 500 * time.Millisecond
)

// App implements ebiten.Game and drives the TEA runtime
type App struct {
	model    tea.Model
	rng      *rand.Rand
	lastTick time.Time
}

// New creates a new App with test units in combat mode
func New(seed int64) *App {
	rng := rand.New(rand.NewPCG(uint64(seed), uint64(seed>>32)))

	// Create test units
	playerUnits := []entity.Unit{
		{
			ID: "player1",
			Attributes: map[string]core.Attribute{
				"health": {Name: "health", Base: 100, Min: 0, Max: 200},
			},
		},
		{
			ID: "player2",
			Attributes: map[string]core.Attribute{
				"health": {Name: "health", Base: 80, Min: 0, Max: 150},
			},
		},
	}

	enemyUnits := []entity.Unit{
		{
			ID: "enemy1",
			Attributes: map[string]core.Attribute{
				"health": {Name: "health", Base: 50, Min: 0, Max: 100},
			},
		},
		{
			ID: "enemy2",
			Attributes: map[string]core.Attribute{
				"health": {Name: "health", Base: 60, Min: 0, Max: 100},
			},
		},
	}

	return &App{
		model: tea.Model{
			Version: 1,
			Phase:   tea.PhaseCombat,
			Seed:    seed,
			Combat: model.CombatModel{
				Phase:       model.CombatActive,
				PlayerUnits: playerUnits,
				EnemyUnits:  enemyUnits,
				Tick:        0,
				Log:         []string{"Combat started"},
			},
		},
		rng:      rng,
		lastTick: time.Now(),
	}
}

// Update handles input and game logic (implements ebiten.Game)
func (a *App) Update() error {
	a.pollInput()

	if a.model.Phase == tea.PhaseGameOver {
		return ebiten.Termination
	}

	a.maybeTick()

	return nil
}

// Draw renders the game state (implements ebiten.Game)
func (a *App) Draw(screen *ebiten.Image) {
	renderer.RenderEbiten(screen, a.model)
}

// Layout returns the game's screen size (implements ebiten.Game)
func (a *App) Layout(outsideWidth, outsideHeight int) (int, int) {
	return screenWidth, screenHeight
}

// pollInput checks for player input and dispatches appropriate messages
func (a *App) pollInput() {
	if inpututil.IsKeyJustPressed(ebiten.KeyEscape) {
		a.dispatch(tea.PlayerQuit{})
		return
	}

	if inpututil.IsKeyJustPressed(ebiten.KeySpace) {
		if a.model.Combat.Phase == model.CombatActive {
			a.dispatch(tea.PlayerPaused{})
		} else if a.model.Combat.Phase == model.CombatPaused {
			a.dispatch(tea.PlayerResumed{})
		}
	}
}

// maybeTick generates a CombatTicked message if the tick interval has elapsed
func (a *App) maybeTick() {
	if a.model.Combat.Phase != model.CombatActive {
		return
	}

	if time.Since(a.lastTick) < tickInterval {
		return
	}

	a.lastTick = time.Now()

	// Generate random rolls for the tick
	rolls := make([]int, 10)
	for i := range rolls {
		rolls[i] = a.rng.IntN(100)
	}

	a.dispatch(tea.CombatTicked{Rolls: rolls})
}

// dispatch sends a message through the TEA update cycle
func (a *App) dispatch(msg tea.Msg) {
	// Handle batched messages (matches runtime.go behavior)
	if batch, ok := msg.(tea.BatchedMsgs); ok {
		for _, m := range batch.Msgs {
			a.dispatch(m)
		}
		return
	}

	var cmd tea.Cmd
	a.model, cmd = a.model.Update(msg)

	// Execute command chain
	for cmd != nil {
		result := cmd()
		if result == nil {
			break
		}
		a.dispatch(result)
	}
}
