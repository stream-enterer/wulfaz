package renderer

import (
	"fmt"
	"image"
	"image/color"

	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/ebitenutil"
	"github.com/hajimehoshi/ebiten/v2/vector"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
	"wulfaz/internal/tea"
)

const (
	// Board layout
	BoardSlots  = 10
	SlotWidth   = 64
	SlotHeight  = 112
	BoardMargin = 10
	UnitGap     = BoardMargin
	BoardWidth  = BoardSlots * SlotWidth
	FrameStroke = 2

	// Y positions (adjusted for command units above/below boards)
	enemyBoardY  = 150 // Room for enemy command above (150-20-112 = 18 top margin)
	playerBoardY = 430 // Room for player command below

	// Command unit dimensions
	CommandUnitWidth  = 2 * SlotWidth // 128px (medium size)
	CommandUnitHeight = SlotHeight    // 112px (same as board units)
	CommandGap        = 20            // Gap between board and command unit

	// Dice box rendering
	DieBoxSize   = 24
	DieBoxMargin = 4
	DiePipRadius = 2

	// Combat log (top right, mirrors tick/pause text)
	logY        = 10
	logChars    = 35
	logMaxLines = 20
	charWidth   = 7  // approx width of debug font char
	lineHeight  = 15
)

var (
	colorBackground  = color.RGBA{30, 30, 50, 255}
	colorPlayer      = color.RGBA{60, 100, 200, 255}
	colorEnemy       = color.RGBA{200, 60, 60, 255}
	colorOrangeLock  = color.RGBA{255, 165, 0, 255}  // Locked die
	colorGreenSelect = color.RGBA{0, 255, 0, 255}    // Selected die
	colorRedUsed     = color.RGBA{255, 0, 0, 255}    // Activated/used die
	colorDieBox      = color.RGBA{40, 40, 40, 255}   // Die background
)

// HitRegion represents a clickable area on screen for input handling.
type HitRegion struct {
	Rect     image.Rectangle
	Type     string // "die" or "unit"
	UnitID   string
	DieIndex int // -1 for unit regions
}

func getAttr(unit entity.Unit, name string) int {
	if attr, ok := unit.Attributes[name]; ok {
		return attr.Base
	}
	return 0
}

func getCombatWidth(unit entity.Unit) int {
	if cw := getAttr(unit, "combat_width"); cw > 0 {
		return cw
	}
	return 1
}

// calcUnitWidth returns pixel width for a given combat_width
func calcUnitWidth(combatWidth int) float32 {
	return float32(combatWidth*SlotWidth - UnitGap)
}

// CalcBoardX returns the X position to center the board frame (exported for app.go)
func CalcBoardX(screenWidth int) float32 {
	frameWidth := BoardWidth + 2*BoardMargin
	return float32(screenWidth-frameWidth) / 2
}

// separateCommandUnit splits command unit from board units.
func separateCommandUnit(units []entity.Unit) (*entity.Unit, []entity.Unit) {
	var cmd *entity.Unit
	var board []entity.Unit
	for i := range units {
		if units[i].IsCommand() {
			cmd = &units[i]
		} else {
			board = append(board, units[i])
		}
	}
	return cmd, board
}

// getDieState returns die outline state: 0=normal, 1=locked, 2=selected, 3=activated
// Priority: activated > selected > locked > normal
func getDieState(unitID string, dieIdx int, combat model.CombatModel, isPlayerCmd bool) int {
	// Check activated first (highest priority)
	if activated := combat.ActivatedDice[unitID]; activated != nil && dieIdx < len(activated) && activated[dieIdx] {
		return 3 // red
	}
	// Check selected (only for player command)
	if isPlayerCmd && combat.SelectedUnitID == unitID && combat.SelectedDieIndex == dieIdx {
		return 2 // green
	}
	// Check locked
	if rolled := combat.RolledDice[unitID]; rolled != nil && dieIdx < len(rolled) && rolled[dieIdx].Locked {
		return 1 // orange
	}
	return 0 // white
}

// drawDieBox draws a single die box and returns its hit region rectangle.
func drawDieBox(screen *ebiten.Image, x, y float32, result int, state int) image.Rectangle {
	// Background
	vector.FillRect(screen, x, y, DieBoxSize, DieBoxSize, colorDieBox, false)

	// Outline based on state
	var outline color.Color = color.White
	switch state {
	case 1:
		outline = colorOrangeLock
	case 2:
		outline = colorGreenSelect
	case 3:
		outline = colorRedUsed
	}
	vector.StrokeRect(screen, x, y, DieBoxSize, DieBoxSize, 2, outline, false)

	// Content: pips or X
	if result == 0 {
		drawRedX(screen, x, y)
	} else {
		drawPips(screen, x, y, result)
	}

	return image.Rect(int(x), int(y), int(x)+DieBoxSize, int(y)+DieBoxSize)
}

// drawPips draws standard die pip pattern.
func drawPips(screen *ebiten.Image, x, y float32, count int) {
	cx, cy := x+DieBoxSize/2, y+DieBoxSize/2
	o := float32(6) // offset from center

	if count > 6 {
		// Show number for values > 6
		ebitenutil.DebugPrintAt(screen, fmt.Sprintf("%d", count), int(cx)-4, int(cy)-6)
		return
	}

	pip := func(px, py float32) {
		vector.FillCircle(screen, px, py, DiePipRadius, color.White, false)
	}

	switch count {
	case 1:
		pip(cx, cy)
	case 2:
		pip(cx-o, cy-o)
		pip(cx+o, cy+o)
	case 3:
		pip(cx-o, cy-o)
		pip(cx, cy)
		pip(cx+o, cy+o)
	case 4:
		pip(cx-o, cy-o)
		pip(cx+o, cy-o)
		pip(cx-o, cy+o)
		pip(cx+o, cy+o)
	case 5:
		pip(cx-o, cy-o)
		pip(cx+o, cy-o)
		pip(cx, cy)
		pip(cx-o, cy+o)
		pip(cx+o, cy+o)
	case 6:
		pip(cx-o, cy-o)
		pip(cx+o, cy-o)
		pip(cx-o, cy)
		pip(cx+o, cy)
		pip(cx-o, cy+o)
		pip(cx+o, cy+o)
	}
}

// drawRedX draws an X for blank (0) die faces.
func drawRedX(screen *ebiten.Image, x, y float32) {
	vector.StrokeLine(screen, x+4, y+4, x+DieBoxSize-4, y+DieBoxSize-4, 2, colorRedUsed, false)
	vector.StrokeLine(screen, x+DieBoxSize-4, y+4, x+4, y+DieBoxSize-4, 2, colorRedUsed, false)
}

// drawCommandDice draws 3-die pyramid for command unit, returns hit regions.
func drawCommandDice(screen *ebiten.Image, unit entity.Unit, cardX, cardY float32, combat model.CombatModel, isPlayerCmd bool) []HitRegion {
	var regions []HitRegion
	rolled := combat.RolledDice[unit.ID]
	if len(rolled) == 0 {
		return regions
	}

	// Pyramid layout centered in 128x112 card
	diceAreaY := cardY + 16
	topDieX := cardX + (CommandUnitWidth-DieBoxSize)/2
	topDieY := diceAreaY + 14
	bottomLeftX := cardX + (CommandUnitWidth-2*DieBoxSize-DieBoxMargin)/2
	bottomRightX := bottomLeftX + DieBoxSize + DieBoxMargin
	bottomY := topDieY + DieBoxSize + DieBoxMargin

	positions := []struct{ x, y float32 }{
		{topDieX, topDieY},       // Die 0: top
		{bottomLeftX, bottomY},   // Die 1: bottom-left
		{bottomRightX, bottomY},  // Die 2: bottom-right
	}

	for i, pos := range positions {
		if i >= len(rolled) {
			break
		}
		state := getDieState(unit.ID, i, combat, isPlayerCmd)
		rect := drawDieBox(screen, pos.x, pos.y, rolled[i].Result, state)
		regions = append(regions, HitRegion{Rect: rect, Type: "die", UnitID: unit.ID, DieIndex: i})
	}

	return regions
}

// drawUnitDice draws dice for non-command units (small=1, medium=2).
func drawUnitDice(screen *ebiten.Image, unit entity.Unit, cardX, cardY, cardW float32, combat model.CombatModel) []HitRegion {
	var regions []HitRegion
	rolled := combat.RolledDice[unit.ID]
	if len(rolled) == 0 {
		return regions
	}

	cw := getCombatWidth(unit)

	if cw == 1 && len(rolled) >= 1 {
		// Small unit: 1 die centered
		dieX := cardX + (cardW-DieBoxSize)/2
		dieY := cardY + (SlotHeight-DieBoxSize)/2
		state := getDieState(unit.ID, 0, combat, false)
		rect := drawDieBox(screen, dieX, dieY, rolled[0].Result, state)
		regions = append(regions, HitRegion{Rect: rect, Type: "die", UnitID: unit.ID, DieIndex: 0})
	} else if cw >= 2 && len(rolled) >= 2 {
		// Medium+ unit: 2 dice diagonal
		margin := float32(15)
		die1X := cardX + margin
		die1Y := cardY + 20
		die2X := cardX + cardW - margin - DieBoxSize
		die2Y := cardY + SlotHeight - 20 - DieBoxSize

		state0 := getDieState(unit.ID, 0, combat, false)
		rect0 := drawDieBox(screen, die1X, die1Y, rolled[0].Result, state0)
		regions = append(regions, HitRegion{Rect: rect0, Type: "die", UnitID: unit.ID, DieIndex: 0})

		state1 := getDieState(unit.ID, 1, combat, false)
		rect1 := drawDieBox(screen, die2X, die2Y, rolled[1].Result, state1)
		regions = append(regions, HitRegion{Rect: rect1, Type: "die", UnitID: unit.ID, DieIndex: 1})
	}

	return regions
}

// allCommandDiceLockedRenderer checks if all player command dice are locked.
func allCommandDiceLockedRenderer(combat model.CombatModel) bool {
	var cmdID string
	for _, u := range combat.PlayerUnits {
		if u.IsCommand() {
			cmdID = u.ID
			break
		}
	}
	if cmdID == "" {
		return true
	}
	rolled := combat.RolledDice[cmdID]
	if len(rolled) == 0 {
		return true
	}
	for _, rd := range rolled {
		if !rd.Locked {
			return false
		}
	}
	return true
}

// drawUnlockButton draws the ↰ unlock all button and returns its hit rectangle.
func drawUnlockButton(screen *ebiten.Image, x, y int) image.Rectangle {
	btnW, btnH := 80, 20
	fx, fy := float32(x), float32(y)

	// Button background
	vector.FillRect(screen, fx, fy, float32(btnW), float32(btnH), color.RGBA{60, 60, 80, 255}, false)
	vector.StrokeRect(screen, fx, fy, float32(btnW), float32(btnH), 1, color.White, false)

	// Button text
	ebitenutil.DebugPrintAt(screen, "↰ Unlock", x+8, y+2)

	return image.Rect(x, y, x+btnW, y+btnH)
}

// drawCommandUnit draws command unit card and returns its rectangle.
func drawCommandUnit(screen *ebiten.Image, unit entity.Unit, c color.RGBA, x, y float32) image.Rectangle {
	// Draw card background
	vector.FillRect(screen, x, y, CommandUnitWidth, CommandUnitHeight, c, false)
	vector.StrokeRect(screen, x, y, CommandUnitWidth, CommandUnitHeight, FrameStroke, color.White, false)

	// Unit ID at top
	ebitenutil.DebugPrintAt(screen, unit.ID, int(x)+4, int(y)+4)

	// HP + Shields at bottom
	hp := getAttr(unit, "health")
	shields := getAttr(unit, "shields")
	statText := fmt.Sprintf("HP:%d", hp)
	if shields > 0 {
		statText += fmt.Sprintf(" SH:%d", shields)
	}
	ebitenutil.DebugPrintAt(screen, statText, int(x)+4, int(y)+CommandUnitHeight-16)

	return image.Rect(int(x), int(y), int(x)+CommandUnitWidth, int(y)+CommandUnitHeight)
}

// drawBoardFrame draws the board outline
func drawBoardFrame(screen *ebiten.Image, x, y float32) {
	frameW := float32(BoardWidth + 2*BoardMargin)
	frameH := float32(SlotHeight + 2*BoardMargin)
	vector.StrokeRect(screen, x, y, frameW, frameH, FrameStroke, color.White, false)
}

// drawUnitsOnBoard renders units left-aligned within the board and returns hit regions.
func drawUnitsOnBoard(screen *ebiten.Image, units []entity.Unit, boardX, boardY float32, c color.RGBA, combat model.CombatModel) []HitRegion {
	var regions []HitRegion
	currentX := boardX + float32(BoardMargin)
	unitY := boardY + float32(BoardMargin)

	for _, unit := range units {
		cw := getCombatWidth(unit)
		w := calcUnitWidth(cw)

		// Draw unit card
		drawUnit(screen, unit, c, currentX, unitY, w)
		unitRect := image.Rect(int(currentX), int(unitY), int(currentX+w), int(unitY)+SlotHeight)
		regions = append(regions, HitRegion{Rect: unitRect, Type: "unit", UnitID: unit.ID, DieIndex: -1})

		// Draw dice on unit
		diceRegions := drawUnitDice(screen, unit, currentX, unitY, w, combat)
		regions = append(regions, diceRegions...)

		currentX += w + UnitGap
	}

	return regions
}

// RenderEbiten renders the Model to an Ebitengine screen and returns hit regions.
func RenderEbiten(screen *ebiten.Image, m tea.Model) []HitRegion {
	screen.Fill(colorBackground)

	switch m.Phase {
	case tea.PhaseMenu:
		renderMenu(screen)
		return nil
	case tea.PhaseCombat:
		return renderCombat(screen, m.Combat)
	case tea.PhaseChoice:
		renderChoice(screen, m.ChoiceType, m.Choices)
		return nil
	case tea.PhaseGameOver:
		renderGameOver(screen)
		return nil
	default:
		ebitenutil.DebugPrint(screen, "Unknown phase")
		return nil
	}
}

func renderMenu(screen *ebiten.Image) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()
	ebitenutil.DebugPrintAt(screen, "=== WULFAZ ===", w/2-50, h/2-20)
	ebitenutil.DebugPrintAt(screen, "Press SPACE to start", w/2-70, h/2+10)
}

func renderGameOver(screen *ebiten.Image) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()
	ebitenutil.DebugPrintAt(screen, "=== GAME OVER ===", w/2-60, h/2-10)
	ebitenutil.DebugPrintAt(screen, "Press ESC to quit", w/2-55, h/2+20)
}

func renderCombat(screen *ebiten.Image, combat model.CombatModel) []HitRegion {
	var regions []HitRegion
	w := screen.Bounds().Dx()
	boardX := CalcBoardX(w)

	// Header
	ebitenutil.DebugPrintAt(screen, fmt.Sprintf("Round: %d", combat.Round), 10, 10)
	ebitenutil.DebugPrintAt(screen, "SPACE=Pause  ESC=Quit", 10, 30)

	// Separate command units from board units
	enemyCmd, enemyBoard := separateCommandUnit(combat.EnemyUnits)
	playerCmd, playerBoard := separateCommandUnit(combat.PlayerUnits)

	// Enemy command unit ABOVE enemy board
	if enemyCmd != nil {
		cmdX := boardX + (BoardWidth+2*BoardMargin-CommandUnitWidth)/2
		cmdY := float32(enemyBoardY - CommandGap - CommandUnitHeight)
		rect := drawCommandUnit(screen, *enemyCmd, colorEnemy, cmdX, cmdY)
		regions = append(regions, HitRegion{Rect: rect, Type: "unit", UnitID: enemyCmd.ID, DieIndex: -1})
		diceRegions := drawCommandDice(screen, *enemyCmd, cmdX, cmdY, combat, false)
		regions = append(regions, diceRegions...)
	}

	// Enemy board
	drawBoardFrame(screen, boardX, enemyBoardY)
	boardRegions := drawUnitsOnBoard(screen, enemyBoard, boardX, enemyBoardY, colorEnemy, combat)
	regions = append(regions, boardRegions...)

	// Player board
	drawBoardFrame(screen, boardX, playerBoardY)
	boardRegions = drawUnitsOnBoard(screen, playerBoard, boardX, playerBoardY, colorPlayer, combat)
	regions = append(regions, boardRegions...)

	// Player command unit BELOW player board
	if playerCmd != nil {
		cmdX := boardX + (BoardWidth+2*BoardMargin-CommandUnitWidth)/2
		cmdY := float32(playerBoardY + SlotHeight + 2*BoardMargin + CommandGap)
		rect := drawCommandUnit(screen, *playerCmd, colorPlayer, cmdX, cmdY)
		regions = append(regions, HitRegion{Rect: rect, Type: "unit", UnitID: playerCmd.ID, DieIndex: -1})
		diceRegions := drawCommandDice(screen, *playerCmd, cmdX, cmdY, combat, true)
		regions = append(regions, diceRegions...) // Die regions added AFTER unit region
	}

	// Phase-specific UI hints
	if combat.DicePhase == model.DicePhasePlayerCommand {
		allLocked := allCommandDiceLockedRenderer(combat)

		if !allLocked {
			// Lock phase hints
			ebitenutil.DebugPrintAt(screen, "LClick die to lock/unlock", 10, 50)
			ebitenutil.DebugPrintAt(screen, fmt.Sprintf("R - Reroll unlocked (%d/2)", combat.RerollsRemaining), 10, 70)
		} else {
			// Activation phase hints
			ebitenutil.DebugPrintAt(screen, "LClick die to select, LClick target to activate", 10, 50)
			ebitenutil.DebugPrintAt(screen, "RClick to cancel selection", 10, 70)

			// ↰ Unlock button (only if rerolls > 0)
			if combat.RerollsRemaining > 0 {
				btnRect := drawUnlockButton(screen, 10, 90)
				regions = append(regions, HitRegion{Rect: btnRect, Type: "unlock_button", UnitID: "", DieIndex: -1})
			}
		}
	}
	if combat.DicePhase == model.DicePhasePreview {
		ebitenutil.DebugPrintAt(screen, "Click to continue...", 10, 50)
	}

	renderLog(screen, combat.Log)

	// Paused overlay
	if combat.Phase == model.CombatPaused {
		renderPausedOverlay(screen)
	}

	// Round toast overlay (F-224)
	if combat.ShowRoundToast {
		renderRoundToast(screen, combat.Round+1) // Show UPCOMING round number
	}

	return regions
}

func drawUnit(screen *ebiten.Image, unit entity.Unit, c color.RGBA, x, y, width float32) {
	vector.FillRect(screen, x, y, width, SlotHeight, c, false)
	vector.StrokeRect(screen, x, y, width, SlotHeight, FrameStroke, color.White, false)

	// Unit ID (truncated for narrow units)
	displayID := unit.ID
	if width < 80 && len(unit.ID) > 6 {
		displayID = unit.ID[:6]
	}
	ebitenutil.DebugPrintAt(screen, displayID, int(x)+4, int(y)+4)

	// HP + Shields at bottom (F-223)
	hp := getAttr(unit, "health")
	shields := getAttr(unit, "shields")
	statText := fmt.Sprintf("HP:%d", hp)
	if shields > 0 {
		statText += fmt.Sprintf(" SH:%d", shields)
	}
	ebitenutil.DebugPrintAt(screen, statText, int(x)+4, int(y)+SlotHeight-16)
}

func renderLog(screen *ebiten.Image, log []string) {
	w := screen.Bounds().Dx()
	logX := w - logChars*charWidth - BoardMargin

	ebitenutil.DebugPrintAt(screen, "Combat Log:", logX, logY)

	var lines []string
	for _, entry := range log {
		lines = append(lines, wrapText(entry, logChars)...)
	}

	start := 0
	if len(lines) > logMaxLines {
		start = len(lines) - logMaxLines
	}

	for i, line := range lines[start:] {
		ebitenutil.DebugPrintAt(screen, line, logX, logY+lineHeight+i*lineHeight)
	}
}

func wrapText(text string, maxChars int) []string {
	if len(text) <= maxChars {
		return []string{text}
	}

	var lines []string
	for len(text) > maxChars {
		breakAt := maxChars
		for i := maxChars; i > 0; i-- {
			if text[i] == ' ' {
				breakAt = i
				break
			}
		}
		lines = append(lines, text[:breakAt])
		text = text[breakAt:]
		if len(text) > 0 && text[0] == ' ' {
			text = text[1:]
		}
	}
	if len(text) > 0 {
		lines = append(lines, text)
	}
	return lines
}

func renderPausedOverlay(screen *ebiten.Image) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()

	// Semi-transparent overlay
	overlay := color.RGBA{0, 0, 0, 128}
	vector.FillRect(screen, 0, 0, float32(w), float32(h), overlay, false)

	// PAUSED text
	ebitenutil.DebugPrintAt(screen, "=== PAUSED ===", w/2-50, h/2-10)
	ebitenutil.DebugPrintAt(screen, "Press SPACE to resume", w/2-70, h/2+20)
}

func renderRoundToast(screen *ebiten.Image, round int) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()

	// Semi-transparent overlay
	vector.FillRect(screen, 0, 0, float32(w), float32(h), color.RGBA{0, 0, 0, 180}, false)

	// Toast box
	boxW, boxH := float32(160), float32(60)
	boxX := (float32(w) - boxW) / 2
	boxY := (float32(h) - boxH) / 2
	vector.FillRect(screen, boxX, boxY, boxW, boxH, color.RGBA{50, 50, 70, 255}, false)
	vector.StrokeRect(screen, boxX, boxY, boxW, boxH, 2, color.White, false)

	// "Round N" centered
	text := fmt.Sprintf("Round %d", round)
	ebitenutil.DebugPrintAt(screen, text, int(boxX)+55, int(boxY)+22)
}

func renderChoice(screen *ebiten.Image, ct tea.ChoiceType, choices []string) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()

	header := "Choose a reward:"
	if ct == tea.ChoiceFight {
		header = "Choose next fight:"
	}
	ebitenutil.DebugPrintAt(screen, header, w/2-60, h/2-60)

	for i, c := range choices {
		line := fmt.Sprintf("[%d] %s", i+1, c)
		ebitenutil.DebugPrintAt(screen, line, w/2-60, h/2-30+i*20)
	}
}
