package renderer

import (
	"fmt"
	"image"
	"image/color"
	"math"
	"time"

	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/vector"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
	"wulfaz/internal/tea"
)

// Module-level layout info set by RenderEbiten for use by all render functions
var (
	centerOffset image.Point
	centerWidth  int
)

// offsetX returns x coordinate offset into center area
func offsetX(x float32) float32 {
	return x + float32(centerOffset.X)
}

// offsetY returns y coordinate offset into center area
func offsetY(y float32) float32 {
	return y + float32(centerOffset.Y)
}

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
	DieBoxSize = 24

	// Die detail rendering
	dieContentPadding = 4 // Used in drawRedX for X positioning

	// Text rendering
	unitIDTruncateWidth = 80
	unitIDTruncateLen   = 6
	unitStatTextYOffset = 16
	textPadding         = 4 // Common padding for text in cards

	// Overlay
	pausedOverlayAlpha = 128

	// Drag-and-drop rendering
	insertionIndicatorWidth = 4
)

var (
	colorBackground  = color.RGBA{30, 30, 50, 255}
	colorPlayer      = color.RGBA{60, 100, 200, 255}
	colorEnemy       = color.RGBA{200, 60, 60, 255}
	colorOrangeLock  = color.RGBA{255, 165, 0, 255} // Locked die
	colorGreenSelect = color.RGBA{0, 255, 0, 255}   // Selected die
	colorRedUsed     = color.RGBA{255, 0, 0, 255}   // Activated/used die
	colorDieBox      = color.RGBA{40, 40, 40, 255}  // Die background
	colorGrayBlank   = color.RGBA{40, 40, 40, 255}  // Blank die border (#282828)
	colorDeadUnit    = color.RGBA{60, 60, 60, 180}  // F-124: Greyed out dead unit

	// Wave 7: Arrow colors
	colorArrowDamage = color.RGBA{255, 80, 80, 220}  // Red
	colorArrowShield = color.RGBA{80, 140, 255, 220} // Blue
	colorArrowHeal   = color.RGBA{80, 255, 140, 220} // Green

	// Drag-and-drop colors
	colorInsertionIndicator = color.RGBA{255, 255, 0, 200} // Yellow
	colorDragHighlight      = color.RGBA{255, 255, 0, 255} // Yellow border
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

// GetCombatWidth returns the combat_width attribute of a unit, defaulting to 1.
// Exported for use by app.go for drag-drop calculations.
func GetCombatWidth(unit entity.Unit) int {
	if cw := getAttr(unit, "combat_width"); cw > 0 {
		return cw
	}
	return 1
}

// CalcUnitWidth returns pixel width for a given combat_width.
// Exported for use by app.go for drag-drop calculations.
func CalcUnitWidth(combatWidth int) float32 {
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

// getDieState returns die outline state for an individual die:
// 0=normal, 1=locked, 2=selected, 3=fired/activated, 4=blank
// Priority: fired > selected > locked > blank > normal
func getDieState(unitID string, rolled entity.RolledDie, combat model.CombatModel, isPlayerUnit bool) int {
	// Fired die (spent) — highest priority
	if rolled.Fired {
		return 3 // red
	}
	// Selected (only for player units, only unfired dice)
	if isPlayerUnit && combat.SelectedUnitID == unitID && !rolled.Fired {
		return 2 // green
	}
	// During execution/round-end phases, blank dice show blank state
	if (combat.DicePhase == model.DicePhaseExecution || combat.DicePhase == model.DicePhaseRoundEnd) && rolled.Type() == entity.DieBlank {
		return 4 // gray for blank
	}
	// Locked
	if rolled.Locked {
		return 1 // orange
	}
	// Blank faces get gray state
	if rolled.Type() == entity.DieBlank {
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

// drawUnitDice draws dice for any unit, centered horizontally in the card.
func drawUnitDice(screen *ebiten.Image, unit entity.Unit, cardX, cardY, cardW, cardH float32, combat model.CombatModel, isPlayerUnit bool) []HitRegion {
	var regions []HitRegion
	if len(unit.Dice) == 0 || !unit.IsAlive() {
		return regions
	}

	rolledDice, exists := combat.RolledDice[unit.ID]
	if !exists {
		return regions
	}

	n := len(rolledDice)
	const gap float32 = 4
	totalW := float32(n)*DieBoxSize + float32(n-1)*gap
	startX := cardX + (cardW-totalW)/2
	dieY := cardY + (cardH-DieBoxSize)/2

	for i, rd := range rolledDice {
		dieX := startX + float32(i)*(DieBoxSize+gap)
		state := getDieState(unit.ID, rd, combat, isPlayerUnit)
		rect := drawDieBox(screen, dieX, dieY, rd.CurrentFace(), state)
		regions = append(regions, HitRegion{Rect: rect, Type: "die", UnitID: unit.ID, DieIndex: i})
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
		cw := GetCombatWidth(u)
		w := CalcUnitWidth(cw)
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
		cw := GetCombatWidth(u)
		w := CalcUnitWidth(cw)
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
func drawUnitsOnBoard(screen *ebiten.Image, units []entity.Unit, boardX, boardY float32, c color.RGBA, combat model.CombatModel, isPlayerSide bool) []HitRegion {
	var regions []HitRegion
	currentX := boardX + float32(BoardMargin)
	unitY := boardY + float32(BoardMargin)

	for _, unit := range units {
		cw := GetCombatWidth(unit)
		w := CalcUnitWidth(cw)

		// Draw unit card
		drawUnit(screen, unit, c, currentX, unitY, w)
		unitRect := image.Rect(int(currentX), int(unitY), int(currentX+w), int(unitY)+SlotHeight)
		regions = append(regions, HitRegion{Rect: unitRect, Type: "unit", UnitID: unit.ID, DieIndex: -1})

		// Draw die on unit
		diceRegions := drawUnitDice(screen, unit, currentX, unitY, w, SlotHeight, combat, isPlayerSide)
		regions = append(regions, diceRegions...)

		currentX += w + UnitGap
	}

	return regions
}

// RenderEbiten renders the Model to an Ebitengine screen and returns hit regions.
// centerRect is the area where game content should be rendered (between UI sidebars).
func RenderEbiten(screen *ebiten.Image, m tea.Model, centerRect image.Rectangle) []HitRegion {
	// Skip rendering if center rect not yet laid out (first frame)
	if centerRect.Dx() == 0 || centerRect.Dy() == 0 {
		return nil
	}

	// Store layout info for all rendering functions
	centerOffset = centerRect.Min
	centerWidth = centerRect.Dx()

	// Fill only the center area (not sidebars)
	vector.FillRect(screen, float32(centerRect.Min.X), float32(centerRect.Min.Y),
		float32(centerRect.Dx()), float32(centerRect.Dy()), colorBackground, false)

	switch m.Phase {
	case model.PhaseMenu:
		renderMenu(screen)
		return nil
	case model.PhaseCombat:
		return renderCombat(screen, m.Combat)
	case model.PhaseInterCombat:
		return renderInterCombat(screen, m)
	case model.PhaseGameOver:
		renderGameOver(screen)
		return nil
	default:
		DrawText(screen, "Unknown phase", 0, 0)
		return nil
	}
}

func renderMenu(screen *ebiten.Image) {
	// Center text in the center panel area
	cx := centerOffset.X + centerWidth/2
	cy := centerOffset.Y + screen.Bounds().Dy()/2
	DrawTextCentered(screen, "=== WULFAZ ===", cx, cy-20)
	DrawTextCentered(screen, "Press SPACE to start", cx, cy+10)
}

func renderGameOver(screen *ebiten.Image) {
	// Center text in the center panel area
	cx := centerOffset.X + centerWidth/2
	cy := centerOffset.Y + screen.Bounds().Dy()/2
	DrawTextCentered(screen, "=== GAME OVER ===", cx, cy-10)
	DrawTextCentered(screen, "Press ESC to quit", cx, cy+20)
}

func renderCombat(screen *ebiten.Image, combat model.CombatModel) []HitRegion {
	var regions []HitRegion
	boardX := offsetX(CalcBoardX(centerWidth))

	// Separate command units from board units
	enemyCmd, enemyBoard := separateCommandUnit(combat.EnemyUnits)
	playerCmd, playerBoard := separateCommandUnit(combat.PlayerUnits)

	// Enemy command unit ABOVE enemy board
	if enemyCmd != nil {
		cmdX := boardX + (BoardWidth+2*BoardMargin-CommandUnitWidth)/2
		cmdY := offsetY(float32(enemyBoardY - CommandGap - CommandUnitHeight))
		rect := drawCommandUnit(screen, *enemyCmd, colorEnemy, cmdX, cmdY)
		regions = append(regions, HitRegion{Rect: rect, Type: "unit", UnitID: enemyCmd.ID, DieIndex: -1})
		diceRegions := drawUnitDice(screen, *enemyCmd, cmdX, cmdY, CommandUnitWidth, CommandUnitHeight, combat, false)
		regions = append(regions, diceRegions...)
	}

	// Enemy board
	drawBoardFrame(screen, boardX, offsetY(enemyBoardY))
	boardRegions := drawUnitsOnBoard(screen, enemyBoard, boardX, offsetY(enemyBoardY), colorEnemy, combat, false)
	regions = append(regions, boardRegions...)

	// Player board
	drawBoardFrame(screen, boardX, offsetY(playerBoardY))
	boardRegions = drawUnitsOnBoard(screen, playerBoard, boardX, offsetY(playerBoardY), colorPlayer, combat, true)
	regions = append(regions, boardRegions...)

	// Player command unit BELOW player board
	if playerCmd != nil {
		cmdX := boardX + (BoardWidth+2*BoardMargin-CommandUnitWidth)/2
		cmdY := offsetY(float32(playerBoardY + SlotHeight + 2*BoardMargin + CommandGap))
		rect := drawCommandUnit(screen, *playerCmd, colorPlayer, cmdX, cmdY)
		regions = append(regions, HitRegion{Rect: rect, Type: "unit", UnitID: playerCmd.ID, DieIndex: -1})
		diceRegions := drawUnitDice(screen, *playerCmd, cmdX, cmdY, CommandUnitWidth, CommandUnitHeight, combat, true)
		regions = append(regions, diceRegions...)
	}

	// Draw targeting arrows (Wave 7)
	if len(combat.ActiveArrows) > 0 {
		drawTargetingArrows(screen, combat, boardX)
	}

	// Draw floating texts (Wave 7)
	drawFloatingTexts(screen, combat, boardX)

	// Paused overlay (covers entire screen including sidebars)
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

func renderPausedOverlay(screen *ebiten.Image) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()

	// Semi-transparent overlay
	overlay := color.RGBA{0, 0, 0, pausedOverlayAlpha}
	vector.FillRect(screen, 0, 0, float32(w), float32(h), overlay, false)

	// PAUSED text
	DrawTextCentered(screen, "=== PAUSED ===", w/2, h/2-10)
	DrawTextCentered(screen, "Press SPACE to resume", w/2, h/2+20)
}

// renderInterCombat renders the inter-combat phase: board visible with overlay choices.
func renderInterCombat(screen *ebiten.Image, m tea.Model) []HitRegion {
	var regions []HitRegion
	boardX := offsetX(CalcBoardX(centerWidth))

	// Draw player board frame
	drawBoardFrame(screen, boardX, offsetY(playerBoardY))

	// Separate command from board units
	playerCmd, boardUnits := separateCommandUnit(m.PlayerRoster)

	// Compute insertion index from drag position (if dragging)
	insertionIdx := -1
	var draggedUnitWidth float32
	if m.DragState.IsDragging {
		insertionIdx = computeInsertionIndex(m.DragState.CurrentX, boardX, boardUnits, m.DragState.DraggedUnitID)
		// Find dragged unit width for gap
		for _, unit := range boardUnits {
			if unit.ID == m.DragState.DraggedUnitID {
				draggedUnitWidth = CalcUnitWidth(GetCombatWidth(unit))
				break
			}
		}
	}

	// Draw board units with shift for drag gap
	currentX := boardX + float32(BoardMargin)
	unitY := offsetY(float32(playerBoardY + BoardMargin))

	// Track position in non-dragged units (for insertion comparison)
	drawIdx := 0
	for _, unit := range boardUnits {
		// Skip dragged unit in normal draw
		if m.DragState.IsDragging && unit.ID == m.DragState.DraggedUnitID {
			continue // Don't increment drawIdx - dragged unit doesn't count
		}

		cw := GetCombatWidth(unit)
		unitW := CalcUnitWidth(cw)

		// Insert gap at insertion point (before this unit)
		if m.DragState.IsDragging && drawIdx == insertionIdx {
			// Draw insertion indicator
			vector.FillRect(screen, currentX, unitY-5, insertionIndicatorWidth,
				SlotHeight+10, colorInsertionIndicator, false)
			// Add gap for dragged unit
			currentX += draggedUnitWidth + UnitGap
		}

		// Draw unit
		drawUnit(screen, unit, colorPlayer, currentX, unitY, unitW)
		rect := image.Rect(int(currentX), int(unitY), int(currentX+unitW), int(unitY)+SlotHeight)
		regions = append(regions, HitRegion{Rect: rect, Type: "unit", UnitID: unit.ID, DieIndex: -1})

		currentX += unitW + UnitGap
		drawIdx++
	}

	// Insertion at end (drawIdx equals insertion point after loop)
	if m.DragState.IsDragging && drawIdx == insertionIdx {
		vector.FillRect(screen, currentX, unitY-5, insertionIndicatorWidth,
			SlotHeight+10, colorInsertionIndicator, false)
	}

	// Draw player command unit below board (not draggable)
	if playerCmd != nil {
		cmdX := boardX + (BoardWidth+2*BoardMargin-CommandUnitWidth)/2
		cmdY := offsetY(float32(playerBoardY + SlotHeight + 2*BoardMargin + CommandGap))
		drawCommandUnit(screen, *playerCmd, colorPlayer, cmdX, cmdY)
		// No hit region for command - not draggable
	}

	// Draw dragged unit at cursor (topmost layer)
	if m.DragState.IsDragging {
		for _, unit := range boardUnits {
			if unit.ID == m.DragState.DraggedUnitID {
				cw := GetCombatWidth(unit)
				unitW := CalcUnitWidth(cw)
				dragX := float32(m.DragState.CurrentX) - unitW/2
				dragY := float32(m.DragState.CurrentY) - SlotHeight/2
				drawUnit(screen, unit, colorPlayer, dragX, dragY, unitW)
				vector.StrokeRect(screen, dragX, dragY, unitW, SlotHeight, 3, colorDragHighlight, false)
				break
			}
		}
	}

	return regions
}

// computeInsertionIndex determines where dragged unit would be inserted.
// IMPORTANT: Must skip the dragged unit to match visual layout.
func computeInsertionIndex(mouseX int, boardX float32, boardUnits []entity.Unit, draggedUnitID string) int {
	currentX := boardX + float32(BoardMargin)
	insertIdx := 0
	for _, unit := range boardUnits {
		// Skip dragged unit - it's not in the visual layout
		if unit.ID == draggedUnitID {
			continue
		}
		cw := GetCombatWidth(unit)
		unitW := CalcUnitWidth(cw)
		midPoint := currentX + unitW/2
		if float32(mouseX) < midPoint {
			return insertIdx
		}
		currentX += unitW + UnitGap
		insertIdx++
	}
	return insertIdx // Insert at end
}

// ===== Wave 7: Floating Text Rendering =====

// drawFloatingTexts renders all active floating combat texts above units.
func drawFloatingTexts(screen *ebiten.Image, combat model.CombatModel, boardX float32) {
	now := time.Now().UnixNano()
	durationNano := int64(model.CombatTextDuration)

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
		cw := GetCombatWidth(u)
		uw := CalcUnitWidth(cw)
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
		cw := GetCombatWidth(u)
		uw := CalcUnitWidth(cw)
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
