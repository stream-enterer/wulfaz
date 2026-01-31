package renderer

import (
	"fmt"
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

	// Y positions
	enemyBoardY  = 80
	playerBoardY = 480

	// Combat log (top right, mirrors tick/pause text)
	logY        = 10
	logChars    = 35
	logMaxLines = 20
	charWidth   = 7  // approx width of debug font char
	lineHeight  = 15
)

var (
	colorBackground = color.RGBA{30, 30, 50, 255}
	colorPlayer     = color.RGBA{60, 100, 200, 255}
	colorEnemy      = color.RGBA{200, 60, 60, 255}
)

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

// calcBoardX returns the X position to center the board frame
func calcBoardX(screenWidth int) float32 {
	frameWidth := BoardWidth + 2*BoardMargin
	return float32(screenWidth-frameWidth) / 2
}

// drawBoardFrame draws the board outline
func drawBoardFrame(screen *ebiten.Image, x, y float32) {
	frameW := float32(BoardWidth + 2*BoardMargin)
	frameH := float32(SlotHeight + 2*BoardMargin)
	vector.StrokeRect(screen, x, y, frameW, frameH, FrameStroke, color.White, false)
}

// drawUnitsOnBoard renders units left-aligned within the board
func drawUnitsOnBoard(screen *ebiten.Image, units []entity.Unit, boardX, boardY float32, c color.RGBA) {
	currentX := boardX + float32(BoardMargin)
	unitY := boardY + float32(BoardMargin)

	for _, unit := range units {
		cw := getCombatWidth(unit)
		w := calcUnitWidth(cw)
		drawUnit(screen, unit.ID, getAttr(unit, "health"), c, currentX, unitY, w)
		currentX += w + UnitGap
	}
}

// RenderEbiten renders the Model to an Ebitengine screen
func RenderEbiten(screen *ebiten.Image, m tea.Model) {
	screen.Fill(colorBackground)

	switch m.Phase {
	case tea.PhaseMenu:
		renderMenu(screen)
	case tea.PhaseCombat:
		renderCombat(screen, m.Combat)
	case tea.PhaseChoice:
		renderChoice(screen, m.ChoiceType, m.Choices)
	case tea.PhaseGameOver:
		renderGameOver(screen)
	default:
		ebitenutil.DebugPrint(screen, "Unknown phase")
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

func renderCombat(screen *ebiten.Image, combat model.CombatModel) {
	w := screen.Bounds().Dx()

	// Header
	ebitenutil.DebugPrintAt(screen, fmt.Sprintf("Tick: %d", combat.Tick), 10, 10)
	ebitenutil.DebugPrintAt(screen, "SPACE=Pause  ESC=Quit", 10, 30)

	// Board X position (centered)
	boardX := calcBoardX(w)

	// Enemy board (top)
	drawBoardFrame(screen, boardX, enemyBoardY)
	drawUnitsOnBoard(screen, combat.EnemyUnits, boardX, enemyBoardY, colorEnemy)

	// Player board (bottom)
	drawBoardFrame(screen, boardX, playerBoardY)
	drawUnitsOnBoard(screen, combat.PlayerUnits, boardX, playerBoardY, colorPlayer)

	renderLog(screen, combat.Log)

	// Paused overlay
	if combat.Phase == model.CombatPaused {
		renderPausedOverlay(screen)
	}
}

func drawUnit(screen *ebiten.Image, id string, health int, c color.RGBA, x, y, width float32) {
	vector.FillRect(screen, x, y, width, SlotHeight, c, false)
	vector.StrokeRect(screen, x, y, width, SlotHeight, FrameStroke, color.White, false)

	// Truncate ID for narrow units
	displayID := id
	if width < 80 && len(id) > 6 {
		displayID = id[:6]
	}

	ebitenutil.DebugPrintAt(screen, displayID, int(x)+4, int(y)+4)
	ebitenutil.DebugPrintAt(screen, fmt.Sprintf("HP:%d", health), int(x)+4, int(y)+SlotHeight-16)
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
