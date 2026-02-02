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

// Shadow settings (matches ebitenutil debug font)
var shadowColor = color.RGBA{0, 0, 0, 128} // 50% black
const shadowOffsetX = 1
const shadowOffsetY = 1

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

// DrawTextColor renders text at (x, y) with specified color and drop shadow.
// text/v2 uses top-left as origin by default (unlike v1 which used baseline).
func DrawTextColor(screen *ebiten.Image, s string, x, y int, c color.Color) {
	// Compute shadow alpha proportional to text alpha
	_, _, _, a := c.RGBA()
	shadowAlpha := uint8((uint32(shadowColor.A) * (a >> 8)) >> 8)
	shadow := color.RGBA{0, 0, 0, shadowAlpha}

	// Draw shadow first (1px right, 1px down)
	op := &text.DrawOptions{}
	op.GeoM.Translate(float64(x+shadowOffsetX), float64(y+shadowOffsetY))
	op.ColorScale.ScaleWithColor(shadow)
	text.Draw(screen, s, arkPixelFace, op)

	// Draw text on top
	op.GeoM.Reset()
	op.GeoM.Translate(float64(x), float64(y))
	op.ColorScale.Reset()
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
