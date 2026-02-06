package layout

import (
	"image"
	"image/color"

	"github.com/ebitenui/ebitenui"
	eimage "github.com/ebitenui/ebitenui/image"
	"github.com/ebitenui/ebitenui/widget"
	"github.com/hajimehoshi/ebiten/v2"
	"github.com/hajimehoshi/ebiten/v2/text/v2"

	"wulfaz/internal/entity"
)

const (
	LeftSidebarWidth  = 200
	RightSidebarWidth = 260
)

var (
	sidebarColor = color.RGBA{25, 25, 40, 255} // #191928
	centerColor  = color.RGBA{30, 30, 50, 255} // #1E1E32
	textColor    = color.White
	hpColor      = color.RGBA{255, 80, 80, 255}   // Red for HP
	shieldColor  = color.RGBA{150, 150, 150, 255} // Grey for shields
)

type GameUI struct {
	UI          *ebitenui.UI
	CenterRect  image.Rectangle
	centerPanel *widget.Container

	// Left sidebar text widgets
	roundText *widget.Text
	keysText  *widget.Text
	hintText  *widget.Text

	// Right sidebar
	rightSidebar   *widget.Container // Parent container (for relayout calls)
	statsContainer *widget.Container // Top section: name, HP, shields, dice
	logContainer   *widget.Container // Bottom section: combat log

	// Stats display (right sidebar top)
	nameText    *widget.Text
	hpText      *widget.Text
	shieldText  *widget.Text
	diceGraphic *widget.Graphic
	diceFont    *text.Face // Cached for dice net rendering

	// Combat log (right sidebar bottom)
	logText *widget.Text

	// Header and footer
	headerText *widget.Text
	footerText *widget.Text
}

func NewGameUI(face, monoFace *text.Face) *GameUI {
	g := &GameUI{}

	// Header container
	header := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Padding(widget.NewInsetsSimple(8)),
		)),
	)
	g.headerText = widget.NewText(
		widget.TextOpts.Text("", face, textColor),
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

	leftSidebar.AddChild(g.roundText)
	leftSidebar.AddChild(g.keysText)
	leftSidebar.AddChild(g.hintText)

	// Center panel - transparent passthrough
	g.centerPanel = widget.NewContainer(
	// No background - game renders here
	)

	// Right sidebar - grid with 2 rows (stats top, log bottom)
	rightSidebar := widget.NewContainer(
		widget.ContainerOpts.BackgroundImage(eimage.NewNineSliceColor(sidebarColor)),
		widget.ContainerOpts.WidgetOpts(
			widget.WidgetOpts.MinSize(RightSidebarWidth, 0),
		),
		widget.ContainerOpts.Layout(widget.NewGridLayout(
			widget.GridLayoutOpts.Columns(1),
			widget.GridLayoutOpts.Stretch([]bool{true}, []bool{false, true}),
			widget.GridLayoutOpts.Padding(widget.NewInsetsSimple(10)),
			widget.GridLayoutOpts.Spacing(0, 10),
		)),
	)
	g.rightSidebar = rightSidebar

	// Stats container (top) - vertical stack
	g.statsContainer = widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Direction(widget.DirectionVertical),
			widget.RowLayoutOpts.Spacing(4),
		)),
	)

	// Stats widgets
	g.nameText = widget.NewText(
		widget.TextOpts.Text("", face, textColor),
		widget.TextOpts.MaxWidth(float64(RightSidebarWidth-20)),
	)
	g.hpText = widget.NewText(
		widget.TextOpts.Text("", monoFace, hpColor),
		widget.TextOpts.MaxWidth(float64(RightSidebarWidth-20)),
	)
	g.shieldText = widget.NewText(
		widget.TextOpts.Text("", monoFace, shieldColor),
		widget.TextOpts.MaxWidth(float64(RightSidebarWidth-20)),
	)
	g.diceFont = monoFace
	g.diceGraphic = widget.NewGraphic(
		widget.GraphicOpts.Image(CreateEmptyDiceNet()),
	)

	g.statsContainer.AddChild(g.nameText)
	g.statsContainer.AddChild(g.hpText)
	g.statsContainer.AddChild(g.shieldText)
	g.statsContainer.AddChild(g.diceGraphic)

	// Log container (bottom) - anchor layout with log at bottom
	g.logContainer = widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewAnchorLayout()),
	)

	g.logText = widget.NewText(
		widget.TextOpts.Text("", face, textColor),
		widget.TextOpts.MaxWidth(float64(RightSidebarWidth-20)),
		widget.TextOpts.WidgetOpts(
			widget.WidgetOpts.LayoutData(widget.AnchorLayoutData{
				VerticalPosition:   widget.AnchorLayoutPositionEnd,
				HorizontalPosition: widget.AnchorLayoutPositionStart,
			}),
		),
	)

	g.logContainer.AddChild(g.logText)

	// Add sections to right sidebar
	rightSidebar.AddChild(g.statsContainer)
	rightSidebar.AddChild(g.logContainer)

	// Footer container
	footer := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewRowLayout(
			widget.RowLayoutOpts.Padding(widget.NewInsetsSimple(8)),
		)),
	)
	g.footerText = widget.NewText(
		widget.TextOpts.Text("", face, textColor),
	)
	footer.AddChild(g.footerText)

	// Center wrapper - 1 column grid with 3 rows (header, game area, footer)
	centerWrapper := widget.NewContainer(
		widget.ContainerOpts.BackgroundImage(eimage.NewNineSliceColor(centerColor)),
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
	g.rightSidebar.RequestRelayout()
}

// SetHeaderText updates the header text
func (g *GameUI) SetHeaderText(s string) {
	g.headerText.Label = s
}

// SetFooterText updates the footer text
func (g *GameUI) SetFooterText(s string) {
	g.footerText.Label = s
}

// IsMouseInGameArea returns true if the point is in the center game area
func (g *GameUI) IsMouseInGameArea(x, y int) bool {
	return image.Pt(x, y).In(g.CenterRect)
}

// GetCenterOffset returns the top-left offset for game board rendering
func (g *GameUI) GetCenterOffset() image.Point {
	return g.CenterRect.Min
}

// SetNameText updates the unit name display
func (g *GameUI) SetNameText(s string) {
	g.nameText.Label = s
	g.rightSidebar.RequestRelayout()
}

// SetHPText updates the HP bar display
func (g *GameUI) SetHPText(s string) {
	g.hpText.Label = s
	g.rightSidebar.RequestRelayout()
}

// SetShieldText updates the shield display
func (g *GameUI) SetShieldText(s string) {
	g.shieldText.Label = s
	g.rightSidebar.RequestRelayout()
}

// SetDiceFaces updates the dice net display with the given faces.
// Pass nil or empty slice to clear the display.
func (g *GameUI) SetDiceFaces(faces []entity.DieFace) {
	if len(faces) < 6 {
		g.diceGraphic.Image = CreateEmptyDiceNet()
	} else {
		g.diceGraphic.Image = RenderDiceNet(faces, g.diceFont)
	}
	g.rightSidebar.RequestRelayout()
}
