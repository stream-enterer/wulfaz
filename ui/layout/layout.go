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
	roundText    *widget.Text
	keysText     *widget.Text
	hintText     *widget.Text
	unlockButton *widget.Button

	// Right sidebar
	logText *widget.Text

	// Callbacks (set by app)
	OnUnlockClicked func()
}

func NewGameUI(face *text.Face) *GameUI {
	g := &GameUI{}

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

	// Unlock button (hidden by default)
	buttonColor := color.RGBA{60, 60, 80, 255}
	g.unlockButton = widget.NewButton(
		widget.ButtonOpts.WidgetOpts(
			widget.WidgetOpts.MinSize(80, 24),
		),
		widget.ButtonOpts.Image(&widget.ButtonImage{
			Idle:    eimage.NewNineSliceColor(buttonColor),
			Hover:   eimage.NewNineSliceColor(color.RGBA{80, 80, 100, 255}),
			Pressed: eimage.NewNineSliceColor(color.RGBA{40, 40, 60, 255}),
		}),
		widget.ButtonOpts.Text("Unlock", face, &widget.ButtonTextColor{
			Idle: textColor,
		}),
		widget.ButtonOpts.ClickedHandler(func(args *widget.ButtonClickedEventArgs) {
			if g.OnUnlockClicked != nil {
				g.OnUnlockClicked()
			}
		}),
	)
	// Start hidden
	g.unlockButton.GetWidget().Visibility = widget.Visibility_Hide

	leftSidebar.AddChild(g.roundText)
	leftSidebar.AddChild(g.keysText)
	leftSidebar.AddChild(g.hintText)
	leftSidebar.AddChild(g.unlockButton)

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

	// Root container - 3 column grid
	root := widget.NewContainer(
		widget.ContainerOpts.Layout(widget.NewGridLayout(
			widget.GridLayoutOpts.Columns(3),
			widget.GridLayoutOpts.Stretch([]bool{false, true, false}, []bool{true}),
		)),
	)
	root.AddChild(leftSidebar)
	root.AddChild(g.centerPanel)
	root.AddChild(rightSidebar)

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

// SetUnlockButtonVisible shows or hides the unlock button
func (g *GameUI) SetUnlockButtonVisible(visible bool) {
	if visible {
		g.unlockButton.GetWidget().Visibility = widget.Visibility_Show
	} else {
		g.unlockButton.GetWidget().Visibility = widget.Visibility_Hide
	}
}
