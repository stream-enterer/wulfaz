package app

import (
	"log"
	"math/rand/v2"
	"time"

	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/inpututil"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
	"wulfaz/internal/tea"
	"wulfaz/internal/template"
	"wulfaz/ui/renderer"
)

const (
	screenWidth  = 1280
	screenHeight = 720
	tickInterval = 500 * time.Millisecond
)

// App implements ebiten.Game and drives the TEA runtime
type App struct {
	model    tea.Model
	registry *template.Registry // Immutable after init; for shop/rewards later
	rng      *rand.Rand
	lastTick time.Time
}

// New creates a new App with units loaded from templates
func New(seed int64) *App {
	rng := rand.New(rand.NewPCG(uint64(seed), uint64(seed>>32)))

	// Load templates
	reg := template.NewRegistry()
	if err := template.LoadUnitsFromDir("data/templates/units", reg); err != nil {
		log.Fatalf("load unit templates: %v", err)
	}
	if err := template.LoadItemsFromDir("data/templates/items", reg); err != nil {
		log.Fatalf("load item templates: %v", err)
	}

	a := &App{
		model: tea.Model{
			Version:     1,
			Phase:       tea.PhaseCombat,
			Seed:        seed,
			FightNumber: 1,
		},
		registry: reg,
		rng:      rng,
		lastTick: time.Now(),
	}
	a.model.Combat = a.buildCombat()
	return a
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

	if a.model.Phase == tea.PhaseChoice {
		var selected int = -1
		switch {
		case inpututil.IsKeyJustPressed(ebiten.Key1):
			selected = 0
		case inpututil.IsKeyJustPressed(ebiten.Key2):
			selected = 1
		case inpututil.IsKeyJustPressed(ebiten.Key3):
			selected = 2
		}
		if selected >= 0 {
			if a.model.ChoiceType == tea.ChoiceReward {
				a.dispatch(tea.ChoiceSelected{Index: selected})
			} else {
				// Fight selection: App builds combat (has registry access)
				if a.model.FightNumber >= 2 {
					a.dispatch(tea.PlayerQuit{}) // MVP: end after fight 2
				} else {
					combat := a.buildCombat()
					a.dispatch(tea.CombatStarted{Combat: combat})
				}
			}
		}
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

	// Execute command if present
	if cmd != nil {
		result := cmd()
		if result != nil {
			a.dispatch(result)
		}
	}
}

// buildCombat creates a new combat with fresh units for the next fight.
func (a *App) buildCombat() model.CombatModel {
	// Instantiate player units
	player1, err := template.InstantiateUnit(a.registry, "medium_mech", "player_1")
	if err != nil {
		log.Fatalf("instantiate player_1: %v", err)
	}
	player2, err := template.InstantiateUnit(a.registry, "small_mech", "player_2")
	if err != nil {
		log.Fatalf("instantiate player_2: %v", err)
	}

	// Instantiate enemy units
	enemy1, err := template.InstantiateUnit(a.registry, "small_mech", "enemy_1")
	if err != nil {
		log.Fatalf("instantiate enemy_1: %v", err)
	}
	enemy2, err := template.InstantiateUnit(a.registry, "medium_mech", "enemy_2")
	if err != nil {
		log.Fatalf("instantiate enemy_2: %v", err)
	}

	// Equip player weapons
	laser1, err := template.InstantiateItem(a.registry, "medium_laser", "p1_laser_r")
	if err != nil {
		log.Fatalf("instantiate p1_laser_r: %v", err)
	}
	player1, err = template.EquipItem(player1, "right_arm", 0, laser1)
	if err != nil {
		log.Fatalf("equip player_1 right_arm: %v", err)
	}

	laser2, err := template.InstantiateItem(a.registry, "medium_laser", "p1_laser_l")
	if err != nil {
		log.Fatalf("instantiate p1_laser_l: %v", err)
	}
	player1, err = template.EquipItem(player1, "left_arm", 0, laser2)
	if err != nil {
		log.Fatalf("equip player_1 left_arm: %v", err)
	}

	laser3, err := template.InstantiateItem(a.registry, "medium_laser", "p2_laser")
	if err != nil {
		log.Fatalf("instantiate p2_laser: %v", err)
	}
	player2, err = template.EquipItem(player2, "right_arm", 0, laser3)
	if err != nil {
		log.Fatalf("equip player_2 right_arm: %v", err)
	}

	// Equip enemy weapons
	eLaser1, err := template.InstantiateItem(a.registry, "medium_laser", "e1_laser")
	if err != nil {
		log.Fatalf("instantiate e1_laser: %v", err)
	}
	enemy1, err = template.EquipItem(enemy1, "right_arm", 0, eLaser1)
	if err != nil {
		log.Fatalf("equip enemy_1: %v", err)
	}

	eLaser2, err := template.InstantiateItem(a.registry, "medium_laser", "e2_laser")
	if err != nil {
		log.Fatalf("instantiate e2_laser: %v", err)
	}
	enemy2, err = template.EquipItem(enemy2, "right_arm", 0, eLaser2)
	if err != nil {
		log.Fatalf("equip enemy_2: %v", err)
	}

	return model.CombatModel{
		Phase:       model.CombatActive,
		PlayerUnits: []entity.Unit{player1, player2},
		EnemyUnits:  []entity.Unit{enemy1, enemy2},
		Tick:        0,
		Log:         []string{"Combat started"},
	}
}
