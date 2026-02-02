package renderer

import (
	"fmt"
	"image"
	"image/color"
	"math"
	"strings"
	"time"

	"github.com/hajimehoshi/ebiten/v2"
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

	// Combat log (top right, mirrors tick/pause text)
	logY           = 10
	logMaxLines    = 20
	lineHeight     = 14
	logWidthPixels = 260

	// Die detail rendering
	dieContentPadding         = 4 // Used in drawRedX for X positioning
	commandDiceAreaYOffset    = 16
	commandDiceTopYOffset     = 14
	diagonalDiceMargin        = 15
	diagonalDiceTopYOffset    = 20
	diagonalDiceBottomYOffset = 20

	// UI layout
	unlockButtonWidth  = 80
	unlockButtonHeight = 20
	unlockButtonTextX  = 8
	unlockButtonTextY  = 2
	uiLeftMargin       = 10
	uiHintY1           = 50
	uiHintY2           = 70
	uiUnlockButtonY    = 90

	// Text rendering
	unitIDTruncateWidth = 80
	unitIDTruncateLen   = 6
	unitStatTextYOffset = 16
	textPadding         = 4 // Common padding for text in cards

	// Overlay
	pausedOverlayAlpha = 128
)

var (
	colorBackground   = color.RGBA{30, 30, 50, 255}
	colorPlayer       = color.RGBA{60, 100, 200, 255}
	colorEnemy        = color.RGBA{200, 60, 60, 255}
	colorOrangeLock   = color.RGBA{255, 165, 0, 255} // Locked die
	colorGreenSelect  = color.RGBA{0, 255, 0, 255}   // Selected die
	colorRedUsed      = color.RGBA{255, 0, 0, 255}   // Activated/used die
	colorDieBox       = color.RGBA{40, 40, 40, 255}  // Die background
	colorGrayBlank    = color.RGBA{40, 40, 40, 255}  // Blank die border (#282828)
	colorUnlockButton = color.RGBA{60, 60, 80, 255}
	colorDeadUnit     = color.RGBA{60, 60, 60, 180} // F-124: Greyed out dead unit

	// Wave 7: Arrow colors
	colorArrowDamage = color.RGBA{255, 80, 80, 220}  // Red
	colorArrowShield = color.RGBA{80, 140, 255, 220} // Blue
	colorArrowHeal   = color.RGBA{80, 255, 140, 220} // Green
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

// getDieState returns die outline state: 0=normal, 1=locked, 2=selected, 3=activated, 4=blank
// Priority: activated > selected > locked > blank > normal
// Exception: during execution phase, blank dice always show blank state (not locked)
func getDieState(unitID string, dieIdx int, combat model.CombatModel, isPlayerCmd bool) int {
	rolled := combat.RolledDice[unitID]
	if rolled == nil || dieIdx >= len(rolled) {
		return 0
	}
	rd := rolled[dieIdx]

	// Check activated first (highest priority)
	if activated := combat.ActivatedDice[unitID]; activated != nil && dieIdx < len(activated) && activated[dieIdx] {
		return 3 // red
	}
	// Check selected (only for player command)
	if isPlayerCmd && combat.SelectedUnitID == unitID && combat.SelectedDieIndex == dieIdx {
		return 2 // green
	}
	// During execution/round-end phases, blank dice should show blank state (not locked orange)
	if (combat.DicePhase == model.DicePhaseExecution || combat.DicePhase == model.DicePhaseRoundEnd) && rd.Type() == entity.DieBlank {
		return 4 // gray for blank
	}
	// Check locked
	if rd.Locked {
		return 1 // orange
	}
	// Blank faces get gray state
	if rd.Type() == entity.DieBlank {
		return 4 // gray for blank
	}
	return 0 // white
}

// drawDieBox draws a single die box and returns its hit region rectangle.
func drawDieBox(screen *ebiten.Image, x, y float32, face entity.DieFace, state int) image.Rectangle {
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
	case 4:
		outline = colorGrayBlank
	}
	vector.StrokeRect(screen, x, y, DieBoxSize, DieBoxSize, 2, outline, false)

	// Content: pips, X for blank, or number
	if face.Type == entity.DieBlank {
		drawRedX(screen, x, y)
	} else {
		drawPips(screen, x, y, face.Value)
	}

	return image.Rect(int(x), int(y), int(x)+DieBoxSize, int(y)+DieBoxSize)
}

// drawPips draws the die value as a centered number.
func drawPips(screen *ebiten.Image, x, y float32, count int) {
	s := fmt.Sprintf("%d", count)
	textW := MeasureTextWidth(s)
	cx := int(x) + DieBoxSize/2 - textW/2
	cy := int(y) + DieBoxSize/2 - FontSize/2
	DrawText(screen, s, cx, cy)
}

// drawRedX draws an X for blank (0) die faces.
func drawRedX(screen *ebiten.Image, x, y float32) {
	vector.StrokeLine(screen, x+dieContentPadding, y+dieContentPadding, x+DieBoxSize-dieContentPadding, y+DieBoxSize-dieContentPadding, 2, colorRedUsed, false)
	vector.StrokeLine(screen, x+DieBoxSize-dieContentPadding, y+dieContentPadding, x+dieContentPadding, y+DieBoxSize-dieContentPadding, 2, colorRedUsed, false)
}

// drawCommandDice draws 3-die pyramid for command unit, returns hit regions.
func drawCommandDice(screen *ebiten.Image, unit entity.Unit, cardX, cardY float32, combat model.CombatModel, isPlayerCmd bool) []HitRegion {
	var regions []HitRegion
	rolled := combat.RolledDice[unit.ID]
	if len(rolled) == 0 {
		return regions
	}

	// Pyramid layout centered in 128x112 card
	diceAreaY := cardY + commandDiceAreaYOffset
	topDieX := cardX + (CommandUnitWidth-DieBoxSize)/2
	topDieY := diceAreaY + commandDiceTopYOffset
	bottomLeftX := cardX + (CommandUnitWidth-2*DieBoxSize-DieBoxMargin)/2
	bottomRightX := bottomLeftX + DieBoxSize + DieBoxMargin
	bottomY := topDieY + DieBoxSize + DieBoxMargin

	positions := []struct{ x, y float32 }{
		{topDieX, topDieY},      // Die 0: top
		{bottomLeftX, bottomY},  // Die 1: bottom-left
		{bottomRightX, bottomY}, // Die 2: bottom-right
	}

	for i, pos := range positions {
		if i >= len(rolled) {
			break
		}
		state := getDieState(unit.ID, i, combat, isPlayerCmd)
		rect := drawDieBox(screen, pos.x, pos.y, rolled[i].CurrentFace(), state)
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
		rect := drawDieBox(screen, dieX, dieY, rolled[0].CurrentFace(), state)
		regions = append(regions, HitRegion{Rect: rect, Type: "die", UnitID: unit.ID, DieIndex: 0})
	} else if cw >= 2 && len(rolled) >= 2 {
		// Medium+ unit: 2 dice diagonal
		margin := float32(diagonalDiceMargin)
		die1X := cardX + margin
		die1Y := cardY + diagonalDiceTopYOffset
		die2X := cardX + cardW - margin - DieBoxSize
		die2Y := cardY + SlotHeight - diagonalDiceBottomYOffset - DieBoxSize

		state0 := getDieState(unit.ID, 0, combat, false)
		rect0 := drawDieBox(screen, die1X, die1Y, rolled[0].CurrentFace(), state0)
		regions = append(regions, HitRegion{Rect: rect0, Type: "die", UnitID: unit.ID, DieIndex: 0})

		state1 := getDieState(unit.ID, 1, combat, false)
		rect1 := drawDieBox(screen, die2X, die2Y, rolled[1].CurrentFace(), state1)
		regions = append(regions, HitRegion{Rect: rect1, Type: "die", UnitID: unit.ID, DieIndex: 1})
	}

	return regions
}

// ===== Wave 7: Arrow and Flash Rendering =====

// arrowColor returns the color for an arrow based on effect type
func arrowColor(effectType entity.DieType) color.RGBA {
	switch effectType {
	case entity.DieDamage:
		return colorArrowDamage
	case entity.DieShield:
		return colorArrowShield
	case entity.DieHeal:
		return colorArrowHeal
	case entity.DieBlank:
		return color.RGBA{200, 200, 200, 220}
	}
	return color.RGBA{200, 200, 200, 220}
}

// drawTargetingArrows renders all active arrows
func drawTargetingArrows(screen *ebiten.Image, combat model.CombatModel, boardX float32) {
	for _, arrow := range combat.ActiveArrows {
		srcX, srcY := getUnitCenter(arrow.SourceUnitID, combat, boardX)
		dstX, dstY := getUnitCenter(arrow.TargetUnitID, combat, boardX)

		if srcX == 0 && srcY == 0 {
			continue // Source unit not found
		}
		if dstX == 0 && dstY == 0 {
			continue // Target unit not found
		}

		c := arrowColor(arrow.EffectType)

		if arrow.IsDashed {
			drawDashedLine(screen, srcX, srcY, dstX, dstY, c)
		} else {
			vector.StrokeLine(screen, srcX, srcY, dstX, dstY, 3, c, false)
		}
		drawArrowhead(screen, srcX, srcY, dstX, dstY, c)
	}
}

// getUnitCenter returns the center screen coordinates of a unit
func getUnitCenter(unitID string, combat model.CombatModel, boardX float32) (float32, float32) {
	// Check player units
	currentX := boardX + float32(BoardMargin)
	for _, u := range combat.PlayerUnits {
		if u.IsCommand() {
			if u.ID == unitID {
				// Player command: centered below player board
				cmdX := boardX + (float32(BoardWidth)+2*float32(BoardMargin)-float32(CommandUnitWidth))/2
				cmdY := float32(playerBoardY + SlotHeight + 2*BoardMargin + CommandGap)
				return cmdX + float32(CommandUnitWidth)/2, cmdY + float32(CommandUnitHeight)/2
			}
			continue
		}
		cw := getCombatWidth(u)
		w := calcUnitWidth(cw)
		if u.ID == unitID {
			return currentX + w/2, float32(playerBoardY+BoardMargin) + float32(SlotHeight)/2
		}
		currentX += w + float32(UnitGap)
	}

	// Check enemy units
	currentX = boardX + float32(BoardMargin)
	for _, u := range combat.EnemyUnits {
		if u.IsCommand() {
			if u.ID == unitID {
				// Enemy command: centered above enemy board
				cmdX := boardX + (float32(BoardWidth)+2*float32(BoardMargin)-float32(CommandUnitWidth))/2
				cmdY := float32(enemyBoardY - CommandGap - CommandUnitHeight)
				return cmdX + float32(CommandUnitWidth)/2, cmdY + float32(CommandUnitHeight)/2
			}
			continue
		}
		cw := getCombatWidth(u)
		w := calcUnitWidth(cw)
		if u.ID == unitID {
			return currentX + w/2, float32(enemyBoardY+BoardMargin) + float32(SlotHeight)/2
		}
		currentX += w + float32(UnitGap)
	}

	return 0, 0 // Not found
}

// drawDashedLine draws a dashed line (8px dash, 4px gap)
func drawDashedLine(screen *ebiten.Image, x1, y1, x2, y2 float32, c color.RGBA) {
	const dashLen = 8.0
	const gapLen = 4.0

	dx := x2 - x1
	dy := y2 - y1
	length := float32(math.Sqrt(float64(dx*dx + dy*dy)))
	if length == 0 {
		return
	}

	unitX := dx / length
	unitY := dy / length

	drawn := float32(0)
	for drawn < length {
		endDash := drawn + dashLen
		if endDash > length {
			endDash = length
		}

		sx := x1 + unitX*drawn
		sy := y1 + unitY*drawn
		ex := x1 + unitX*endDash
		ey := y1 + unitY*endDash

		vector.StrokeLine(screen, sx, sy, ex, ey, 2, c, false)
		drawn = endDash + gapLen
	}
}

// drawArrowhead draws a V-shaped arrowhead at the destination
func drawArrowhead(screen *ebiten.Image, x1, y1, x2, y2 float32, c color.RGBA) {
	const headLen = 10.0
	const headAngle = 0.4 // radians (~23 degrees)

	dx := x2 - x1
	dy := y2 - y1
	length := float32(math.Sqrt(float64(dx*dx + dy*dy)))
	if length == 0 {
		return
	}

	// Unit vector pointing back from destination
	ux := -dx / length
	uy := -dy / length

	// Rotate for left and right sides of arrowhead
	cos := float32(math.Cos(headAngle))
	sin := float32(math.Sin(headAngle))

	// Left arm of arrowhead
	lx := x2 + headLen*(ux*cos-uy*sin)
	ly := y2 + headLen*(ux*sin+uy*cos)

	// Right arm of arrowhead
	rx := x2 + headLen*(ux*cos+uy*sin)
	ry := y2 + headLen*(-ux*sin+uy*cos)

	vector.StrokeLine(screen, x2, y2, lx, ly, 2, c, false)
	vector.StrokeLine(screen, x2, y2, rx, ry, 2, c, false)
}

// drawUnlockButton draws the ↰ unlock all button and returns its hit rectangle.
func drawUnlockButton(screen *ebiten.Image, x, y int) image.Rectangle {
	btnW, btnH := unlockButtonWidth, unlockButtonHeight
	fx, fy := float32(x), float32(y)

	// Button background
	vector.FillRect(screen, fx, fy, float32(btnW), float32(btnH), colorUnlockButton, false)
	vector.StrokeRect(screen, fx, fy, float32(btnW), float32(btnH), 1, color.White, false)

	// Button text
	DrawText(screen, "↰ Unlock", x+unlockButtonTextX, y+unlockButtonTextY)

	return image.Rect(x, y, x+btnW, y+btnH)
}

// drawCommandUnit draws command unit card and returns its rectangle.
func drawCommandUnit(screen *ebiten.Image, unit entity.Unit, c color.RGBA, x, y float32) image.Rectangle {
	// F-124: Use grey color for dead command units
	drawColor := c
	if !unit.IsAlive() {
		drawColor = colorDeadUnit
	}
	vector.FillRect(screen, x, y, CommandUnitWidth, CommandUnitHeight, drawColor, false)
	vector.StrokeRect(screen, x, y, CommandUnitWidth, CommandUnitHeight, FrameStroke, color.White, false)

	// Unit ID at top
	DrawText(screen, unit.ID, int(x)+textPadding, int(y)+textPadding)

	// HP + Shields at bottom
	hp := getAttr(unit, "health")
	shields := getAttr(unit, "shields")
	statText := fmt.Sprintf("HP:%d", hp)
	if shields > 0 {
		statText += fmt.Sprintf(" SH:%d", shields)
	}
	DrawText(screen, statText, int(x)+textPadding, int(y)+CommandUnitHeight-unitStatTextYOffset)

	// F-124: Draw destroyed indicator for dead command units (red X)
	if !unit.IsAlive() {
		vector.StrokeLine(screen, x, y, x+CommandUnitWidth, y+CommandUnitHeight, 2, colorRedUsed, false)
		vector.StrokeLine(screen, x+CommandUnitWidth, y, x, y+CommandUnitHeight, 2, colorRedUsed, false)
	}

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
		DrawText(screen, "Unknown phase", 0, 0)
		return nil
	}
}

func renderMenu(screen *ebiten.Image) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()
	DrawTextCentered(screen, "=== WULFAZ ===", w/2, h/2-20)
	DrawTextCentered(screen, "Press SPACE to start", w/2, h/2+10)
}

func renderGameOver(screen *ebiten.Image) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()
	DrawTextCentered(screen, "=== GAME OVER ===", w/2, h/2-10)
	DrawTextCentered(screen, "Press ESC to quit", w/2, h/2+20)
}

func renderCombat(screen *ebiten.Image, combat model.CombatModel) []HitRegion {
	var regions []HitRegion
	w := screen.Bounds().Dx()
	boardX := CalcBoardX(w)

	// Header
	DrawText(screen, fmt.Sprintf("Round: %d", combat.Round), 10, 10)
	DrawText(screen, "SPACE=Pause  ESC=Quit", 10, 30)

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

	// Draw targeting arrows (Wave 7)
	if len(combat.ActiveArrows) > 0 {
		drawTargetingArrows(screen, combat, boardX)
	}

	// Draw floating texts (Wave 7)
	drawFloatingTexts(screen, combat, boardX)

	// Phase-specific UI hints
	if combat.DicePhase == model.DicePhaseExecution {
		DrawText(screen, "Click to continue...", uiLeftMargin, uiHintY1)
	}
	if combat.DicePhase == model.DicePhasePlayerCommand {
		allLocked := tea.AllCommandDiceLocked(combat)

		if !allLocked {
			// Lock phase hints
			DrawText(screen, "LClick die to lock/unlock", uiLeftMargin, uiHintY1)
			DrawText(screen, fmt.Sprintf("R - Reroll unlocked (%d/2)", combat.RerollsRemaining), uiLeftMargin, uiHintY2)
		} else {
			// Activation phase hints
			DrawText(screen, "LClick die to select, LClick target to activate", uiLeftMargin, uiHintY1)
			DrawText(screen, "RClick to cancel selection", uiLeftMargin, uiHintY2)

			// ↰ Unlock button (only if rerolls > 0)
			if combat.RerollsRemaining > 0 {
				btnRect := drawUnlockButton(screen, uiLeftMargin, uiUnlockButtonY)
				regions = append(regions, HitRegion{Rect: btnRect, Type: "unlock_button", UnitID: "", DieIndex: -1})
			}
		}
	}
	if combat.DicePhase == model.DicePhasePreview {
		DrawText(screen, "Click to continue...", uiLeftMargin, uiHintY1)
	}

	renderLog(screen, combat.Log)

	// Paused overlay
	if combat.Phase == model.CombatPaused {
		renderPausedOverlay(screen)
	}

	return regions
}

func drawUnit(screen *ebiten.Image, unit entity.Unit, c color.RGBA, x, y, width float32) {
	// F-124: Use grey color for dead units
	drawColor := c
	if !unit.IsAlive() {
		drawColor = colorDeadUnit
	}
	vector.FillRect(screen, x, y, width, SlotHeight, drawColor, false)
	vector.StrokeRect(screen, x, y, width, SlotHeight, FrameStroke, color.White, false)

	// Unit ID (truncated for narrow units)
	displayID := unit.ID
	if width < unitIDTruncateWidth && len(unit.ID) > unitIDTruncateLen {
		displayID = unit.ID[:unitIDTruncateLen]
	}
	DrawText(screen, displayID, int(x)+textPadding, int(y)+textPadding)

	// HP + Shields at bottom (F-223)
	hp := getAttr(unit, "health")
	shields := getAttr(unit, "shields")
	statText := fmt.Sprintf("HP:%d", hp)
	if shields > 0 {
		statText += fmt.Sprintf(" SH:%d", shields)
	}
	DrawText(screen, statText, int(x)+textPadding, int(y)+SlotHeight-unitStatTextYOffset)

	// F-124: Draw destroyed indicator for dead units (red X)
	if !unit.IsAlive() {
		vector.StrokeLine(screen, x, y, x+width, y+SlotHeight, 2, colorRedUsed, false)
		vector.StrokeLine(screen, x+width, y, x, y+SlotHeight, 2, colorRedUsed, false)
	}
}

func renderLog(screen *ebiten.Image, log []string) {
	w := screen.Bounds().Dx()
	logX := w - logWidthPixels - BoardMargin

	DrawText(screen, "Combat Log:", logX, logY)

	var lines []string
	for _, entry := range log {
		lines = append(lines, wrapText(entry, logWidthPixels)...)
	}

	start := 0
	if len(lines) > logMaxLines {
		start = len(lines) - logMaxLines
	}

	for i, line := range lines[start:] {
		DrawText(screen, line, logX, logY+lineHeight+i*lineHeight)
	}
}

// wrapText wraps text to fit within maxWidth pixels.
func wrapText(text string, maxWidth int) []string {
	if MeasureTextWidth(text) <= maxWidth {
		return []string{text}
	}

	var lines []string
	words := strings.Fields(text)
	currentLine := ""

	for _, word := range words {
		testLine := currentLine
		if testLine != "" {
			testLine += " "
		}
		testLine += word

		if MeasureTextWidth(testLine) <= maxWidth {
			currentLine = testLine
		} else {
			if currentLine != "" {
				lines = append(lines, currentLine)
			}
			currentLine = word
		}
	}
	if currentLine != "" {
		lines = append(lines, currentLine)
	}
	return lines
}

func renderPausedOverlay(screen *ebiten.Image) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()

	// Semi-transparent overlay
	overlay := color.RGBA{0, 0, 0, pausedOverlayAlpha}
	vector.FillRect(screen, 0, 0, float32(w), float32(h), overlay, false)

	// PAUSED text
	DrawTextCentered(screen, "=== PAUSED ===", w/2, h/2-10)
	DrawTextCentered(screen, "Press SPACE to resume", w/2, h/2+20)
}

func renderChoice(screen *ebiten.Image, ct tea.ChoiceType, choices []string) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()

	header := "Choose a reward:"
	if ct == tea.ChoiceFight {
		header = "Choose next fight:"
	}
	DrawTextCentered(screen, header, w/2, h/2-60)

	for i, c := range choices {
		line := fmt.Sprintf("[%d] %s", i+1, c)
		DrawTextCentered(screen, line, w/2, h/2-30+i*20)
	}
}

// ===== Wave 7: Floating Text Rendering =====

// drawFloatingTexts renders all active floating combat texts above units.
func drawFloatingTexts(screen *ebiten.Image, combat model.CombatModel, boardX float32) {
	now := time.Now().UnixNano()
	durationNano := int64(tea.CombatTextDuration)

	for _, ft := range combat.FloatingTexts {
		elapsed := now - ft.StartedAt
		if elapsed < 0 || elapsed > durationNano {
			continue
		}

		// Get unit bounds
		unitX, unitY, unitW, unitH := getUnitBounds(ft.UnitID, combat, boardX)
		if unitW == 0 {
			continue // Unit not found
		}

		// Progress: 0.0 to 1.0
		progress := float32(elapsed) / float32(durationNano)

		// Y position: start at 40% from top, scroll to 10% from top
		startY := unitY + unitH*0.4
		endY := unitY + unitH*0.1
		textY := startY + (endY-startY)*progress

		// Stack offset (capped at MaxTextStack)
		textY += float32(ft.YOffset) * 14

		// Center X
		textX := unitX + unitW/2

		drawCombatText(screen, ft.Text, textX, textY, ft.ColorRGBA, 1.0)
	}
}

// getUnitBounds mirrors getUnitCenter's cumulative positioning approach
// since units have variable widths based on combat_width attribute.
func getUnitBounds(unitID string, combat model.CombatModel, boardX float32) (x, y, w, h float32) {
	// Check player units - cumulative positioning like drawUnitsOnBoard
	currentX := boardX + float32(BoardMargin)
	for _, u := range combat.PlayerUnits {
		if u.IsCommand() {
			if u.ID == unitID {
				cmdX := boardX + (float32(BoardWidth)+2*float32(BoardMargin)-float32(CommandUnitWidth))/2
				cmdY := float32(playerBoardY + SlotHeight + 2*BoardMargin + CommandGap)
				return cmdX, cmdY, CommandUnitWidth, CommandUnitHeight
			}
			continue
		}
		cw := getCombatWidth(u)
		uw := calcUnitWidth(cw)
		if u.ID == unitID {
			return currentX, float32(playerBoardY + BoardMargin), uw, SlotHeight
		}
		currentX += uw + UnitGap
	}

	// Check enemy units
	currentX = boardX + float32(BoardMargin)
	for _, u := range combat.EnemyUnits {
		if u.IsCommand() {
			if u.ID == unitID {
				cmdX := boardX + (float32(BoardWidth)+2*float32(BoardMargin)-float32(CommandUnitWidth))/2
				cmdY := float32(enemyBoardY - CommandGap - CommandUnitHeight)
				return cmdX, cmdY, CommandUnitWidth, CommandUnitHeight
			}
			continue
		}
		cw := getCombatWidth(u)
		uw := calcUnitWidth(cw)
		if u.ID == unitID {
			return currentX, float32(enemyBoardY + BoardMargin), uw, SlotHeight
		}
		currentX += uw + UnitGap
	}
	return 0, 0, 0, 0
}

// drawCombatText draws centered, colored text with alpha support.
func drawCombatText(screen *ebiten.Image, s string, x, y float32, rgba uint32, alpha float32) {
	c := color.RGBA{
		R: uint8((rgba >> 24) & 0xFF),
		G: uint8((rgba >> 16) & 0xFF),
		B: uint8((rgba >> 8) & 0xFF),
		A: uint8(float32((rgba)&0xFF) * alpha),
	}
	DrawTextCenteredColor(screen, s, int(x), int(y), c)
}
