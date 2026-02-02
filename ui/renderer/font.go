package renderer

import (
	"bytes"
	_ "embed"
	"image/color"
	"log"

	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/text/v2"
)

//go:embed fonts/ark-pixel-12px-proportional-latin.otf
var fontData []byte

var arkPixelFace *text.GoTextFace

// FontSize is the pixel height of the font (exported for layout calculations).
const FontSize = 12

func init() {
	src, err := text.NewGoTextFaceSource(bytes.NewReader(fontData))
	if err != nil {
		log.Fatalf("font load failed: %v", err)
	}
	arkPixelFace = &text.GoTextFace{Source: src, Size: FontSize}
}

// DrawText renders white text at (x, y) where y is the TOP of the text.
func DrawText(screen *ebiten.Image, s string, x, y int) {
	DrawTextColor(screen, s, x, y, color.White)
}

// DrawTextColor renders text at (x, y) with specified color.
// text/v2 uses top-left as origin by default (unlike v1 which used baseline).
func DrawTextColor(screen *ebiten.Image, s string, x, y int, c color.Color) {
	op := &text.DrawOptions{}
	op.GeoM.Translate(float64(x), float64(y))
	op.ColorScale.ScaleWithColor(c)
	text.Draw(screen, s, arkPixelFace, op)
}

// DrawTextCentered renders text horizontally centered at centerX.
func DrawTextCentered(screen *ebiten.Image, s string, centerX, y int) {
	w := MeasureTextWidth(s)
	DrawText(screen, s, centerX-w/2, y)
}

// DrawTextCenteredColor renders centered text with specified color.
func DrawTextCenteredColor(screen *ebiten.Image, s string, centerX, y int, c color.Color) {
	w := MeasureTextWidth(s)
	DrawTextColor(screen, s, centerX-w/2, y, c)
}

// MeasureTextWidth returns the pixel width of the string.
func MeasureTextWidth(s string) int {
	w, _ := text.Measure(s, arkPixelFace, 0)
	return int(w)
}
