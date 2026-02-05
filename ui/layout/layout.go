package layout

import (
	"image"
	"image/color"

	"github.com/ebitenui/ebitenui"
	eimage "github.com/ebitenui/ebitenui/image"
	"github.com/ebitenui/ebitenui/widget"
	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/text/v2"
)

const (
	LeftSidebarWidth  = 200
	RightSidebarWidth = 260
)

var (
	sidebarColor = color.RGBA{25, 25, 40, 255}
	textColor    = color.White
)

type GameUI struct {
	UI          *ebitenui.UI
	CenterRect  image.Rectangle
	centerPanel *widget.Container

	// Left sidebar text widgets
	leftSidebar *widget.Container
	roundText   *widget.Text
	keysText    *widget.Text
	hintText    *widget.Text

	// Right sidebar
	logText *widget.Text

	// Header and footer
	headerText *widget.Text
	footerText *widget.Text
}

func NewGameUI(face *text.Face) *GameUI {
	g := &GameUI{}

	// Header container
	header := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Padding(widget.NewInsetsSimple(8)),
		)),
	)
	g.headerText = widget.NewText(
		widget.TextOpts.Text("Header test", face, textColor),
	)
	header.AddChild(g.headerText)

	// Left sidebar container
	leftSidebar := widget.NewContainer(
		widget.ContainerOpts.BackgroundImage(eimage.NewNineSliceColor(sidebarColor)),
		widget.ContainerOpts.WidgetOpts(
			widget.WidgetOpts.MinSize(LeftSidebarWidth, 0),
			widget.WidgetOpts.LayoutData(widget.GridLayoutData{
				VerticalPosition: widget.GridLayoutPositionStart,
			}),
		),
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Direction(widget.DirectionVertical),
			widget.RowLayoutOpts.Padding(widget.NewInsetsSimple(10)),
			widget.RowLayoutOpts.Spacing(8),
		)),
	)

	// Create text widgets for left sidebar
	g.roundText = widget.NewText(
		widget.TextOpts.Text("", face, textColor),
	)
	g.keysText = widget.NewText(
		widget.TextOpts.Text("", face, textColor),
	)
	g.hintText = widget.NewText(
		widget.TextOpts.Text("", face, textColor),
		widget.TextOpts.MaxWidth(float64(LeftSidebarWidth-20)),
	)

	g.leftSidebar = leftSidebar

	leftSidebar.AddChild(g.roundText)
	leftSidebar.AddChild(g.keysText)
	leftSidebar.AddChild(g.hintText)

	// Center panel - transparent passthrough
	g.centerPanel = widget.NewContainer(
	// No background - game renders here
	)

	// Right sidebar container
	rightSidebar := widget.NewContainer(
		widget.ContainerOpts.BackgroundImage(eimage.NewNineSliceColor(sidebarColor)),
		widget.ContainerOpts.WidgetOpts(
			widget.WidgetOpts.MinSize(RightSidebarWidth, 0),
		),
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Direction(widget.DirectionVertical),
			widget.RowLayoutOpts.Padding(widget.NewInsetsSimple(10)),
		)),
	)

	g.logText = widget.NewText(
		widget.TextOpts.Text("", face, textColor),
		widget.TextOpts.MaxWidth(float64(RightSidebarWidth-20)),
	)
	rightSidebar.AddChild(g.logText)

	// Footer container
	footer := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Padding(widget.NewInsetsSimple(8)),
		)),
	)
	g.footerText = widget.NewText(
		widget.TextOpts.Text("Footer test", face, textColor),
	)
	footer.AddChild(g.footerText)

	// Center wrapper - 1 column grid with 3 rows (header, game area, footer)
	centerWrapper := widget.NewContainer(
		widget.ContainerOpts.BackgroundImage(eimage.NewNineSliceColor(color.RGBA{30, 30, 50, 255})), // #1E1E32
		widget.ContainerOpts.Layout(widget.NewGridLayout(
			widget.GridLayoutOpts.Columns(1),
			widget.GridLayoutOpts.Stretch([]bool{true}, []bool{false, true, false}),
		)),
	)
	centerWrapper.AddChild(header)        // Row 0: doesn't stretch vertically
	centerWrapper.AddChild(g.centerPanel) // Row 1: stretches to fill
	centerWrapper.AddChild(footer)        // Row 2: doesn't stretch vertically

	// Root container - 3 column grid (sidebars + center wrapper)
	root := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewGridLayout(
			widget.GridLayoutOpts.Columns(3),
			widget.GridLayoutOpts.Stretch([]bool{false, true, false}, []bool{true}),
		)),
	)
	root.AddChild(leftSidebar)   // Column 0: fixed width
	root.AddChild(centerWrapper) // Column 1: stretches horizontally, contains header/center/footer
	root.AddChild(rightSidebar)  // Column 2: fixed width

	g.UI = &ebitenui.UI{Container: root}

	return g
}

func (g *GameUI) Update() {
	g.UI.Update()
	// Cache center rect after layout
	g.CenterRect = g.centerPanel.GetWidget().Rect
}

func (g *GameUI) Draw(screen *ebiten.Image) {
	g.UI.Draw(screen)
}

// SetRoundText updates the round display
func (g *GameUI) SetRoundText(s string) {
	g.roundText.Label = s
}

// SetKeysText updates the keybinding hints
func (g *GameUI) SetKeysText(s string) {
	g.keysText.Label = s
}

// SetHintText updates the phase-specific hints
func (g *GameUI) SetHintText(s string) {
	g.hintText.Label = s
}

// SetLogText updates the combat log (newline-separated entries)
func (g *GameUI) SetLogText(s string) {
	g.logText.Label = s
}

// IsMouseInGameArea returns true if the point is in the center game area
func (g *GameUI) IsMouseInGameArea(x, y int) bool {
	return image.Pt(x, y).In(g.CenterRect)
}

// GetCenterOffset returns the top-left offset for game board rendering
func (g *GameUI) GetCenterOffset() image.Point {
	return g.CenterRect.Min
}
