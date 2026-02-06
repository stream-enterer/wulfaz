package layout

import (
	"fmt"
	"image/color"

	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/text/v2"
	"github.com/hajimehoshi/ebiten/v2/vector"

	"wulfaz/internal/entity"
)

// Dice net rendering constants
const (
	diceNetCellSize = 60
	diceNetCols     = 4
	diceNetRows     = 3
	diceNetWidth    = diceNetCols * diceNetCellSize // 240px
	diceNetHeight   = diceNetRows * diceNetCellSize // 180px
	diceNetStroke   = 2
	diceNetInset    = diceNetStroke / 2 // Inset so edge strokes aren't clipped
)

// Colors for dice net rendering
var (
	colorDiceNetBg    = color.RGBA{25, 25, 40, 255}    // #191928 sidebar color
	colorDiceNetCell  = color.RGBA{40, 40, 40, 255}    // #282828 colorDieBox
	colorDiceNetGrid  = color.RGBA{80, 80, 106, 255}   // #50506A grid lines
	colorDiceNetDmg   = color.RGBA{255, 80, 80, 255}   // #FF5050 red damage
	colorDiceNetHeal  = color.RGBA{80, 255, 140, 255}  // #50FF8C green heal
	colorDiceNetShld  = color.RGBA{80, 140, 255, 255}  // #508CFF blue shield
	colorDiceNetBlank = color.RGBA{100, 100, 100, 255} // #646464 gray blank
)

// Cell positions for the cross-shaped layout
// Layout mapping (sideways cross):
//
//	        col0 col1 col2 col3
//	Row 0:       [0]              <- face 0 at col 1
//	Row 1:  [1]  [2]  [3]  [4]    <- faces 1-4 full width
//	Row 2:       [5]              <- face 5 at col 1
var diceNetCells = []struct {
	col, row int
}{
	{1, 0}, // Face 0: top center
	{0, 1}, // Face 1: middle left
	{1, 1}, // Face 2: middle center-left
	{2, 1}, // Face 3: middle center-right
	{3, 1}, // Face 4: middle right
	{1, 2}, // Face 5: bottom center
}

// RenderDiceNet creates an image of the dice net for the given faces.
// Returns nil if faces slice has fewer than 6 elements.
func RenderDiceNet(faces []entity.DieFace, font *text.Face) *ebiten.Image {
	if len(faces) < 6 {
		return nil
	}

	img := ebiten.NewImage(diceNetWidth, diceNetHeight)
	img.Fill(colorDiceNetBg)

	// Draw cell backgrounds first (inset to leave room for edge strokes)
	for i := 0; i < 6; i++ {
		cell := diceNetCells[i]
		x := float32(cell.col*diceNetCellSize + diceNetInset)
		y := float32(cell.row*diceNetCellSize + diceNetInset)
		vector.FillRect(img, x, y, diceNetCellSize-2*diceNetInset, diceNetCellSize-2*diceNetInset, colorDiceNetCell, false)
	}

	// Draw grid lines (each line drawn once for uniform thickness)
	drawDiceNetGridLines(img)

	// Draw face labels
	for i := 0; i < 6; i++ {
		cell := diceNetCells[i]
		x := float32(cell.col * diceNetCellSize)
		y := float32(cell.row * diceNetCellSize)
		label := formatDieFaceLabel(faces[i])
		c := getDieFaceColor(faces[i].Type)
		drawCenteredText(img, label, x, y, diceNetCellSize, diceNetCellSize, c, font)
	}

	return img
}

// drawDiceNetGridLines draws all grid lines for the cross-shaped layout.
// Each line is drawn exactly once to ensure uniform thickness.
// All coordinates are inset by diceNetInset so edge strokes aren't clipped.
func drawDiceNetGridLines(img *ebiten.Image) {
	cellSz := diceNetCellSize
	inset := float32(diceNetInset)
	stroke := float32(diceNetStroke)
	gridColor := colorDiceNetGrid

	// Helper to convert grid position to inset coordinate
	// Edge positions (0 and max) get inset, interior positions stay at cell boundaries
	gx := func(gridCol int) float32 {
		if gridCol == 0 {
			return inset
		}
		if gridCol == diceNetCols {
			return float32(diceNetWidth) - inset
		}
		return float32(gridCol * cellSz)
	}
	gy := func(gridRow int) float32 {
		if gridRow == 0 {
			return inset
		}
		if gridRow == diceNetRows {
			return float32(diceNetHeight) - inset
		}
		return float32(gridRow * cellSz)
	}

	// Horizontal lines
	// Top of face 0 (row 0, cols 1-2)
	vector.StrokeLine(img, gx(1), gy(0), gx(2), gy(0), stroke, gridColor, false)
	// Top of middle row (row 1, full width)
	vector.StrokeLine(img, gx(0), gy(1), gx(4), gy(1), stroke, gridColor, false)
	// Bottom of middle row (row 2, full width)
	vector.StrokeLine(img, gx(0), gy(2), gx(4), gy(2), stroke, gridColor, false)
	// Bottom of face 5 (row 3, cols 1-2)
	vector.StrokeLine(img, gx(1), gy(3), gx(2), gy(3), stroke, gridColor, false)

	// Vertical lines
	// Left of face 0 (col 1, rows 0-1)
	vector.StrokeLine(img, gx(1), gy(0), gx(1), gy(1), stroke, gridColor, false)
	// Right of face 0 (col 2, rows 0-1)
	vector.StrokeLine(img, gx(2), gy(0), gx(2), gy(1), stroke, gridColor, false)
	// Left edge of middle row (col 0, rows 1-2)
	vector.StrokeLine(img, gx(0), gy(1), gx(0), gy(2), stroke, gridColor, false)
	// Between faces 1-2 (col 1, rows 1-2)
	vector.StrokeLine(img, gx(1), gy(1), gx(1), gy(2), stroke, gridColor, false)
	// Between faces 2-3 (col 2, rows 1-2)
	vector.StrokeLine(img, gx(2), gy(1), gx(2), gy(2), stroke, gridColor, false)
	// Between faces 3-4 (col 3, rows 1-2)
	vector.StrokeLine(img, gx(3), gy(1), gx(3), gy(2), stroke, gridColor, false)
	// Right edge of middle row (col 4, rows 1-2)
	vector.StrokeLine(img, gx(4), gy(1), gx(4), gy(2), stroke, gridColor, false)
	// Left of face 5 (col 1, rows 2-3)
	vector.StrokeLine(img, gx(1), gy(2), gx(1), gy(3), stroke, gridColor, false)
	// Right of face 5 (col 2, rows 2-3)
	vector.StrokeLine(img, gx(2), gy(2), gx(2), gy(3), stroke, gridColor, false)
}

// formatDieFaceLabel returns the display string for a die face.
// Examples: "D1", "H2", "S1", "x"
func formatDieFaceLabel(face entity.DieFace) string {
	switch face.Type {
	case entity.DieDamage:
		return fmt.Sprintf("D%d", face.Value)
	case entity.DieHeal:
		return fmt.Sprintf("H%d", face.Value)
	case entity.DieShield:
		return fmt.Sprintf("S%d", face.Value)
	case entity.DieBlank:
		return "x"
	default:
		return "?"
	}
}

// getDieFaceColor returns the appropriate color for a die face type.
func getDieFaceColor(dieType entity.DieType) color.Color {
	switch dieType {
	case entity.DieDamage:
		return colorDiceNetDmg
	case entity.DieHeal:
		return colorDiceNetHeal
	case entity.DieShield:
		return colorDiceNetShld
	case entity.DieBlank:
		return colorDiceNetBlank
	default:
		return colorDiceNetBlank
	}
}

// drawCenteredText draws text centered within a cell.
func drawCenteredText(img *ebiten.Image, s string, cellX, cellY, cellW, cellH float32, c color.Color, font *text.Face) {
	if font == nil {
		return
	}

	// Measure text dimensions
	w, h := text.Measure(s, *font, 0)

	// Calculate centered position
	x := cellX + (cellW-float32(w))/2
	y := cellY + (cellH-float32(h))/2

	op := &text.DrawOptions{}
	op.GeoM.Translate(float64(x), float64(y))
	op.ColorScale.ScaleWithColor(c)
	text.Draw(img, s, *font, op)
}

// CreateEmptyDiceNet returns a blank image of the correct size for the dice net.
// Used for initial widget creation before any unit is hovered.
func CreateEmptyDiceNet() *ebiten.Image {
	img := ebiten.NewImage(diceNetWidth, diceNetHeight)
	img.Fill(color.Transparent)
	return img
}
