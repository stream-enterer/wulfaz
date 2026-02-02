package app

import (
	"image"
	"log"
	"math/rand/v2"
	"slices"
	"time"

	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/inpututil"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
	"wulfaz/internal/tea"
	"wulfaz/internal/template"
	"wulfaz/ui/renderer"
)

type pendingTimer struct {
	fireAt time.Time
	id     string
}

const (
	screenWidth  = 1280
	screenHeight = 720
)

// App implements ebiten.Game and drives the TEA runtime
type App struct {
	model         tea.Model
	registry      *template.Registry // Immutable after init; for shop/rewards later
	rng           *rand.Rand
	hitRegions    []renderer.HitRegion // Updated each frame for input handling
	pendingTimers []pendingTimer       // Timers requested by commands
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
			Phase:       tea.PhaseMenu,
			Seed:        seed,
			FightNumber: 0,
			// F-155: PlayerRoster initialized below after App created
		},
		registry: reg,
		rng:      rng,
	}

	// Build initial roster and store in model
	a.model.PlayerRoster = a.buildInitialRoster()

	// Build first combat from roster
	combat := a.buildCombatFromRoster()
	a.dispatch(tea.CombatStarted{Combat: combat})
	return a
}

// Update handles input and game logic (implements ebiten.Game)
func (a *App) Update() error {
	// Check for expired timers
	now := time.Now()
	for i := len(a.pendingTimers) - 1; i >= 0; i-- {
		if now.After(a.pendingTimers[i].fireAt) {
			id := a.pendingTimers[i].id
			a.pendingTimers = slices.Delete(a.pendingTimers, i, i+1)
			a.dispatch(tea.TimerFired{ID: id})
		}
	}

	a.pollInput()

	if a.model.Phase == tea.PhaseGameOver {
		return ebiten.Termination
	}

	return nil
}

// Draw renders the game state (implements ebiten.Game)
func (a *App) Draw(screen *ebiten.Image) {
	a.hitRegions = renderer.RenderEbiten(screen, a.model)
}

// Layout returns the game's screen size (implements ebiten.Game)
func (a *App) Layout(outsideWidth, outsideHeight int) (int, int) {
	return screenWidth, screenHeight
}

// pollInput checks for player input and dispatches appropriate messages
func (a *App) pollInput() {
	// ESC always quits
	if inpututil.IsKeyJustPressed(ebiten.KeyEscape) {
		a.dispatch(tea.PlayerQuit{})
		return
	}

	// Choice phase
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
				// Fight selection: App builds combat from persistent roster
				if a.model.FightNumber >= 2 {
					a.dispatch(tea.PlayerQuit{}) // MVP: end after fight 2
				} else {
					combat := a.buildCombatFromRoster()
					a.dispatch(tea.CombatStarted{Combat: combat})
				}
			}
		}
		return
	}

	// Combat phase input
	if a.model.Phase == tea.PhaseCombat && a.model.Combat.Phase == model.CombatActive {
		a.pollCombatInput()
	}

	// Pause/resume
	if inpututil.IsKeyJustPressed(ebiten.KeySpace) {
		if a.model.Combat.Phase == model.CombatActive {
			a.dispatch(tea.PlayerPaused{})
		} else if a.model.Combat.Phase == model.CombatPaused {
			a.dispatch(tea.PlayerResumed{})
		}
	}
}

// pollCombatInput handles dice phase interactions
func (a *App) pollCombatInput() {
	combat := a.model.Combat

	// PREVIEW PHASE: any click advances to PlayerCommand
	if combat.DicePhase == model.DicePhasePreview {
		if inpututil.IsMouseButtonJustPressed(ebiten.MouseButtonLeft) {
			a.dispatch(tea.PreviewDone{})
		}
		return
	}

	// EXECUTION PHASE: click to advance
	if combat.DicePhase == model.DicePhaseExecution {
		if inpututil.IsMouseButtonJustPressed(ebiten.MouseButtonLeft) {
			a.dispatch(tea.ExecutionAdvanceClicked{Timestamp: time.Now().UnixNano()})
		}
		return
	}

	// PLAYER COMMAND PHASE
	if combat.DicePhase == model.DicePhasePlayerCommand {
		// R key = reroll unlocked dice
		if inpututil.IsKeyJustPressed(ebiten.KeyR) && combat.RerollsRemaining > 0 {
			playerCmd := tea.FindPlayerCommandUnit(combat)
			if playerCmd != nil && !tea.AllCommandDiceLocked(combat) {
				rolled := combat.RolledDice[playerCmd.ID]
				cmd := tea.RerollUnlockedDice(a.model.Seed+int64(combat.Round)*100, playerCmd.ID, rolled)
				a.dispatch(cmd())
			}
		}

		// Left-click = lock toggle OR select/target (depending on lock state)
		if inpututil.IsMouseButtonJustPressed(ebiten.MouseButtonLeft) {
			mx, my := ebiten.CursorPosition()
			a.handleLeftClick(mx, my)
		}

		// Right-click = cancel selection (only in activation mode)
		if inpututil.IsMouseButtonJustPressed(ebiten.MouseButtonRight) {
			if combat.SelectedUnitID != "" {
				a.dispatch(tea.DieDeselected{})
			}
		}
	}
}

// handleLeftClick processes left-click for lock toggle or selection/targeting
func (a *App) handleLeftClick(mx, my int) {
	combat := a.model.Combat
	pt := image.Point{mx, my}
	playerCmd := tea.FindPlayerCommandUnit(combat)
	allLocked := tea.AllCommandDiceLocked(combat)

	// Check ↰ unlock button first (only visible when all locked AND rerolls > 0)
	for _, region := range a.hitRegions {
		if region.Type == "unlock_button" && pt.In(region.Rect) {
			a.dispatch(tea.UnlockAllDice{})
			return
		}
	}

	// Check dice
	for _, region := range a.hitRegions {
		if region.Type != "die" || !pt.In(region.Rect) {
			continue
		}
		// Only player command dice are interactive
		if playerCmd == nil || region.UnitID != playerCmd.ID {
			continue
		}
		// Check if already activated
		if activated := combat.ActivatedDice[region.UnitID]; activated != nil && region.DieIndex < len(activated) && activated[region.DieIndex] {
			continue // Can't interact with used dice
		}

		if !allLocked {
			// LOCK PHASE: toggle lock
			a.dispatch(tea.DieLockToggled{UnitID: region.UnitID, DieIndex: region.DieIndex})
		} else {
			// ACTIVATION PHASE: check for blank before allowing selection
			rolled := combat.RolledDice[region.UnitID]
			if region.DieIndex < len(rolled) && rolled[region.DieIndex].Type() == entity.DieBlank {
				continue // Can't activate blank - skip this die
			}
			// Toggle selection
			if combat.SelectedUnitID == region.UnitID && combat.SelectedDieIndex == region.DieIndex {
				a.dispatch(tea.DieDeselected{})
			} else {
				a.dispatch(tea.DieSelected{UnitID: region.UnitID, DieIndex: region.DieIndex})
			}
		}
		return
	}

	// Check units for targeting (only if die is selected AND all locked)
	if allLocked && combat.SelectedUnitID != "" {
		for _, region := range a.hitRegions {
			if region.Type != "unit" || !pt.In(region.Rect) {
				continue
			}
			rolled := combat.RolledDice[combat.SelectedUnitID]
			if combat.SelectedDieIndex < len(rolled) {
				die := rolled[combat.SelectedDieIndex]
				// Validate target based on die type
				switch die.Type() {
				case entity.DieDamage:
					if !combat.IsEnemyUnit(region.UnitID) {
						continue // Damage must target enemy
					}
				case entity.DieShield, entity.DieHeal:
					if !combat.IsPlayerUnit(region.UnitID) {
						continue // Shield/Heal must target friendly
					}
				case entity.DieBlank:
					continue // Blank dice cannot be activated
				}
				a.dispatch(tea.DiceActivated{
					SourceUnitID: combat.SelectedUnitID,
					DieIndex:     combat.SelectedDieIndex,
					TargetUnitID: region.UnitID,
					Value:        die.Value(),
					Effect:       die.Type(),
					Timestamp:    time.Now().UnixNano(),
				})
			}
			return
		}
	}

	// Clicked empty space - deselect if in activation mode
	if allLocked && combat.SelectedUnitID != "" {
		a.dispatch(tea.DieDeselected{})
	}
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

	// Intercept timer requests - don't pass to Update
	if req, ok := msg.(tea.StartTimerRequested); ok {
		a.pendingTimers = append(a.pendingTimers, pendingTimer{
			fireAt: time.Now().Add(req.Duration),
			id:     req.ID,
		})
		return
	}

	var cmd tea.Cmd
	a.model, cmd = a.model.Update(msg)

	// Clear timers when combat ends
	if _, ok := msg.(tea.CombatEnded); ok {
		a.pendingTimers = nil
	}

	// Execute command if present
	if cmd != nil {
		result := cmd()
		if result != nil {
			a.dispatch(result)
		}
	}
}

// buildInitialRoster creates the starting player roster (command + units with equipment).
func (a *App) buildInitialRoster() []entity.Unit {
	// Instantiate command unit
	playerCmd, err := template.InstantiateUnit(a.registry, "player_command", "player_cmd")
	if err != nil {
		log.Fatalf("instantiate player_cmd: %v", err)
	}
	playerCmd.Position = -1

	// Instantiate player units
	player1, err := template.InstantiateUnit(a.registry, "medium_mech", "player_1")
	if err != nil {
		log.Fatalf("instantiate player_1: %v", err)
	}
	player1.Position = 0

	player2, err := template.InstantiateUnit(a.registry, "small_mech", "player_2")
	if err != nil {
		log.Fatalf("instantiate player_2: %v", err)
	}
	player2.Position = 2

	// Equip weapons
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

	return []entity.Unit{playerCmd, player1, player2}
}

// buildCombatFromRoster creates combat using persistent roster (deep copied).
func (a *App) buildCombatFromRoster() model.CombatModel {
	// Deep copy player roster (don't mutate originals during combat)
	playerUnits := tea.DeepCopyUnits(a.model.PlayerRoster)

	// Build fresh enemy units each fight
	enemyCmd, err := template.InstantiateUnit(a.registry, "enemy_command", "enemy_cmd")
	if err != nil {
		log.Fatalf("instantiate enemy_cmd: %v", err)
	}
	enemyCmd.Position = -1

	enemy1, err := template.InstantiateUnit(a.registry, "small_mech", "enemy_1")
	if err != nil {
		log.Fatalf("instantiate enemy_1: %v", err)
	}
	enemy1.Position = 0

	enemy2, err := template.InstantiateUnit(a.registry, "medium_mech", "enemy_2")
	if err != nil {
		log.Fatalf("instantiate enemy_2: %v", err)
	}
	enemy2.Position = 1

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
		PlayerUnits: playerUnits,
		EnemyUnits:  []entity.Unit{enemyCmd, enemy1, enemy2},
		Log:         []string{"Combat started"},
	}
}
