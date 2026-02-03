package app

import (
	"fmt"
	"image"
	"log"
	"slices"
	"strings"
	"time"

	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/inpututil"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
	"wulfaz/internal/tea"
	"wulfaz/internal/template"
	"wulfaz/ui/layout"
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
	registry      *template.Registry   // Immutable after init; for shop/rewards later
	hitRegions    []renderer.HitRegion // Updated each frame for input handling
	pendingTimers []pendingTimer       // Timers requested by commands
	gameUI        *layout.GameUI       // 3-column UI layout
}

// New creates a new App with units loaded from templates
func New(seed int64) *App {
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
		gameUI:   layout.NewGameUI(renderer.GetFace()),
	}

	// Set up unlock button callback
	a.gameUI.OnUnlockClicked = func() {
		a.dispatch(tea.UnlockAllDice{})
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
	// Sync UI state from model
	a.syncUIState()

	// Update ebitenui
	a.gameUI.Update()

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
	// Draw ebitenui frame first (sidebars)
	a.gameUI.Draw(screen)

	// Then render game content in center area
	a.hitRegions = renderer.RenderEbiten(screen, a.model, a.gameUI.CenterRect)
}

// Layout returns the game's screen size (implements ebiten.Game)
func (a *App) Layout(outsideWidth, outsideHeight int) (int, int) {
	return screenWidth, screenHeight
}

// syncUIState updates the sidebar UI widgets from the current model state
func (a *App) syncUIState() {
	// Default: hide unlock button
	a.gameUI.SetUnlockButtonVisible(false)

	switch a.model.Phase {
	case tea.PhaseMenu:
		a.gameUI.SetRoundText("WULFAZ")
		a.gameUI.SetKeysText("SPACE=Start  ESC=Quit")
		a.gameUI.SetHintText("")
		a.gameUI.SetLogText("")

	case tea.PhaseInterCombat:
		a.gameUI.SetRoundText(fmt.Sprintf("Fight %d Complete", a.model.FightNumber))
		a.gameUI.SetKeysText("1/2/3=Select  ESC=Quit")

		// Inter-combat instructions with choice display
		hint := ""
		if len(a.model.Choices) >= 3 {
			hint = fmt.Sprintf("[1] %s\n[2] %s\n[3] %s\n\n",
				a.model.Choices[0], a.model.Choices[1], a.model.Choices[2])
		}
		if a.model.ChoiceType == tea.ChoiceReward {
			hint += fmt.Sprintf("Rewards left: %d\n", a.model.RewardChoicesLeft)
		}
		// Add drag instructions
		if !a.model.DragState.IsDragging {
			hint += "\nDrag units to reposition"
		} else {
			hint += "\nRelease to drop\nRClick or ESC to cancel"
		}
		a.gameUI.SetHintText(hint)
		a.gameUI.SetLogText("")

	case tea.PhaseGameOver:
		a.gameUI.SetRoundText("GAME OVER")
		a.gameUI.SetKeysText("ESC=Quit")
		a.gameUI.SetHintText("")
		a.gameUI.SetLogText("")

	case tea.PhaseCombat:
		combat := a.model.Combat
		a.gameUI.SetRoundText(fmt.Sprintf("Round: %d", combat.Round))
		a.gameUI.SetKeysText("SPACE=Pause  ESC=Quit")

		// Phase-specific hints and unlock button
		hint := ""
		switch combat.DicePhase {
		case model.DicePhaseExecution:
			hint = "Click to continue..."
		case model.DicePhasePreview:
			hint = "Click to continue..."
		case model.DicePhasePlayerCommand:
			allLocked := tea.AllPlayerDiceLocked(combat)
			if !allLocked {
				hint = fmt.Sprintf("LClick die to lock/unlock\nR - Reroll unlocked (%d/2)", combat.RerollsRemaining)
			} else {
				hint = "LClick die to select\nLClick target to activate\nRClick to cancel"
				// Show unlock button if rerolls remaining
				if combat.RerollsRemaining > 0 {
					a.gameUI.SetUnlockButtonVisible(true)
				}
			}
		case model.DicePhaseNone, model.DicePhaseRoundEnd:
			// No hint during these phases
		}
		a.gameUI.SetHintText(hint)

		// Combat log - show last N lines that fit
		logLines := combat.Log
		maxLines := 25
		if len(logLines) > maxLines {
			logLines = logLines[len(logLines)-maxLines:]
		}
		a.gameUI.SetLogText("Combat Log:\n" + strings.Join(logLines, "\n"))
	}
}

// pollInput checks for player input and dispatches appropriate messages
func (a *App) pollInput() {
	// ESC handling - cancel drag first, then quit
	if inpututil.IsKeyJustPressed(ebiten.KeyEscape) {
		if a.model.Phase == tea.PhaseInterCombat && a.model.DragState.IsDragging {
			a.dispatch(tea.UnitDragCanceled{})
			return
		}
		a.dispatch(tea.PlayerQuit{})
		return
	}

	switch a.model.Phase {
	case tea.PhaseMenu:
		// Menu input handled elsewhere
	case tea.PhaseCombat:
		if a.model.Combat.Phase == model.CombatActive {
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
	case tea.PhaseInterCombat:
		a.pollInterCombatInput()
	case tea.PhaseGameOver:
		// ESC handled above
	}
}

// pollInterCombatInput handles inter-combat phase input (rewards, fights, drag-drop)
func (a *App) pollInterCombatInput() {
	mx, my := ebiten.CursorPosition()

	// Active drag handling
	if a.model.DragState.IsDragging {
		a.dispatch(tea.UnitDragMoved{CurrentX: mx, CurrentY: my})

		if inpututil.IsMouseButtonJustReleased(ebiten.MouseButtonLeft) {
			idx := a.computeInsertionIndex(mx)
			a.dispatch(tea.UnitDragEnded{InsertionIndex: idx})
		}
		if inpututil.IsMouseButtonJustPressed(ebiten.MouseButtonRight) {
			a.dispatch(tea.UnitDragCanceled{})
		}
		return
	}

	// Keyboard: reward/fight selection (1/2/3)
	for i, key := range []ebiten.Key{ebiten.Key1, ebiten.Key2, ebiten.Key3} {
		if inpututil.IsKeyJustPressed(key) {
			// Check if this is fight selection BEFORE dispatching
			isFightSelection := a.model.ChoiceType == tea.ChoiceFight

			a.dispatch(tea.ChoiceSelected{Index: i})

			// If fight was selected, start combat (or end MVP game)
			if isFightSelection {
				// MVP: end game after fight 2
				if a.model.FightNumber >= 2 {
					a.dispatch(tea.PlayerQuit{})
				} else {
					combat := a.buildCombatFromRoster()
					a.dispatch(tea.CombatStarted{Combat: combat})
				}
			}
			return
		}
	}

	// Mouse: start drag on unit click (only in game area)
	if inpututil.IsMouseButtonJustPressed(ebiten.MouseButtonLeft) {
		if !a.gameUI.IsMouseInGameArea(mx, my) {
			return // Let ebitenui handle sidebar clicks
		}
		pt := image.Point{mx, my}
		for _, region := range a.hitRegions {
			if pt.In(region.Rect) && region.Type == "unit" && region.DieIndex == -1 {
				// Find roster index (excluding command unit)
				idx := a.findRosterIndex(region.UnitID)
				if idx >= 0 {
					a.dispatch(tea.UnitDragStarted{
						UnitID: region.UnitID, OriginalIndex: idx,
						StartX: mx, StartY: my,
					})
				}
				return
			}
		}
	}
}

// computeInsertionIndex determines insertion point from mouse X position
func (a *App) computeInsertionIndex(mouseX int) int {
	// Account for sidebar offset
	centerRect := a.gameUI.CenterRect
	boardX := renderer.CalcBoardX(centerRect.Dx()) + float32(centerRect.Min.X)
	currentX := boardX + float32(renderer.BoardMargin)

	// Get board units (exclude command)
	boardIdx := 0
	for _, unit := range a.model.PlayerRoster {
		if unit.IsCommand() {
			continue
		}
		// Skip the dragged unit
		if unit.ID == a.model.DragState.DraggedUnitID {
			continue
		}
		cw := renderer.GetCombatWidth(unit)
		unitW := renderer.CalcUnitWidth(cw)
		midPoint := currentX + unitW/2
		if float32(mouseX) < midPoint {
			return boardIdx
		}
		currentX += unitW + float32(renderer.UnitGap)
		boardIdx++
	}
	return boardIdx // Insert at end
}

// findRosterIndex returns board unit index (excluding command), -1 if not found
func (a *App) findRosterIndex(unitID string) int {
	boardIdx := 0
	for _, unit := range a.model.PlayerRoster {
		if unit.IsCommand() {
			continue
		}
		if unit.ID == unitID {
			return boardIdx
		}
		boardIdx++
	}
	return -1 // Not found or was command unit
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
		// R key = reroll all unlocked player dice
		if inpututil.IsKeyJustPressed(ebiten.KeyR) && combat.RerollsRemaining > 0 {
			if !tea.AllPlayerDiceLocked(combat) {
				cmd := tea.RerollAllUnlockedDice(a.model.Seed+int64(combat.Round)*100, combat)
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
	// Ignore clicks outside game area (ebitenui handles sidebar clicks including unlock button)
	if !a.gameUI.IsMouseInGameArea(mx, my) {
		return
	}

	combat := a.model.Combat
	pt := image.Point{mx, my}
	allLocked := tea.AllPlayerDiceLocked(combat)

	// Check dice - now any player unit die is interactive
	for _, region := range a.hitRegions {
		if region.Type != "die" || !pt.In(region.Rect) {
			continue
		}
		// Only player unit dice are interactive
		if !combat.IsPlayerUnit(region.UnitID) {
			continue
		}
		// Check if already activated
		if combat.ActivatedDice[region.UnitID] {
			continue // Can't interact with used dice
		}

		if !allLocked {
			// LOCK PHASE: toggle lock
			a.dispatch(tea.DieLockToggled{UnitID: region.UnitID})
		} else {
			// ACTIVATION PHASE: check for blank before allowing selection
			rolled, exists := combat.RolledDice[region.UnitID]
			if !exists || rolled.Type() == entity.DieBlank {
				continue // Can't activate blank - skip this die
			}
			// Toggle selection
			if combat.SelectedUnitID == region.UnitID {
				a.dispatch(tea.DieDeselected{})
			} else {
				a.dispatch(tea.DieSelected{UnitID: region.UnitID})
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
			rolled, exists := combat.RolledDice[combat.SelectedUnitID]
			if !exists {
				continue
			}
			// Validate target based on die type
			switch rolled.Type() {
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
				TargetUnitID: region.UnitID,
				Value:        rolled.Value(),
				Effect:       rolled.Type(),
				Timestamp:    time.Now().UnixNano(),
			})
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
