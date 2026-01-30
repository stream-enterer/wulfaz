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
	unitWidth   = 120
	unitHeight  = 60
	unitPadding = 20

	enemyRowY  = 80
	playerRowY = 450
	logX       = 550
	logY       = 200
	logMaxLen  = 10
)

var (
	colorBackground = color.RGBA{30, 30, 50, 255}
	colorPlayer     = color.RGBA{60, 100, 200, 255}
	colorEnemy      = color.RGBA{200, 60, 60, 255}
)

// RenderEbiten renders the Model to an Ebitengine screen
func RenderEbiten(screen *ebiten.Image, m tea.Model) {
	screen.Fill(colorBackground)

	switch m.Phase {
	case tea.PhaseMenu:
		renderMenu(screen)
	case tea.PhaseCombat:
		renderCombat(screen, m.Combat)
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

	// Draw tick counter in top-left
	ebitenutil.DebugPrintAt(screen, fmt.Sprintf("Tick: %d", combat.Tick), 10, 10)

	// Draw controls hint
	ebitenutil.DebugPrintAt(screen, "SPACE=Pause  ESC=Quit", 10, 30)

	// Draw enemy units at top
	if len(combat.EnemyUnits) > 0 {
		enemyStartX := centerUnits(w, len(combat.EnemyUnits))
		for i, unit := range combat.EnemyUnits {
			x := float32(enemyStartX + i*(unitWidth+unitPadding))
			drawUnit(screen, unit.ID, getHealthFromUnit(unit), colorEnemy, x, enemyRowY)
		}
	}

	// Draw player units at bottom
	if len(combat.PlayerUnits) > 0 {
		playerStartX := centerUnits(w, len(combat.PlayerUnits))
		for i, unit := range combat.PlayerUnits {
			x := float32(playerStartX + i*(unitWidth+unitPadding))
			drawUnit(screen, unit.ID, getHealthFromUnit(unit), colorPlayer, x, playerRowY)
		}
	}

	// Draw combat log on the right side
	renderLog(screen, combat.Log)

	// Draw paused overlay
	if combat.Phase == model.CombatPaused {
		renderPausedOverlay(screen)
	}
}

func centerUnits(screenWidth, count int) int {
	totalWidth := count*unitWidth + (count-1)*unitPadding
	return (screenWidth - totalWidth) / 2
}

func drawUnit(screen *ebiten.Image, id string, health int, c color.RGBA, x, y float32) {
	// Draw unit rectangle
	vector.DrawFilledRect(screen, x, y, unitWidth, unitHeight, c, false)

	// Draw border
	vector.StrokeRect(screen, x, y, unitWidth, unitHeight, 2, color.White, false)

	// Draw ID
	ebitenutil.DebugPrintAt(screen, id, int(x)+5, int(y)+5)

	// Draw health
	healthStr := fmt.Sprintf("HP: %d", health)
	ebitenutil.DebugPrintAt(screen, healthStr, int(x)+5, int(y)+25)
}

func getHealthFromUnit(unit entity.Unit) int {
	if attr, ok := unit.Attributes["health"]; ok {
		return attr.Base
	}
	return 0
}

func renderLog(screen *ebiten.Image, log []string) {
	ebitenutil.DebugPrintAt(screen, "Combat Log:", logX, logY)

	start := 0
	if len(log) > logMaxLen {
		start = len(log) - logMaxLen
	}

	for i, entry := range log[start:] {
		if len(entry) > 30 {
			entry = entry[:27] + "..."
		}
		ebitenutil.DebugPrintAt(screen, entry, logX, logY+20+i*15)
	}
}

func renderPausedOverlay(screen *ebiten.Image) {
	w, h := screen.Bounds().Dx(), screen.Bounds().Dy()

	// Semi-transparent overlay
	overlay := color.RGBA{0, 0, 0, 128}
	vector.DrawFilledRect(screen, 0, 0, float32(w), float32(h), overlay, false)

	// PAUSED text
	ebitenutil.DebugPrintAt(screen, "=== PAUSED ===", w/2-50, h/2-10)
	ebitenutil.DebugPrintAt(screen, "Press SPACE to resume", w/2-70, h/2+20)
}
