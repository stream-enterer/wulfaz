package app

import (
	"fmt"
	"image"
	"strings"

	"github.com/hajimehoshi/ebiten/v2"

	"wulfaz/internal/entity"
	"wulfaz/internal/tea"
)

// updateHoveredUnit polls mouse position and updates hoveredUnitID.
func (a *App) updateHoveredUnit() {
	mx, my := ebiten.CursorPosition()
	if !a.gameUI.IsMouseInGameArea(mx, my) {
		a.hoveredUnitID = ""
		return
	}
	pt := image.Point{mx, my}
	for _, region := range a.hitRegions {
		if region.Type == "unit" && pt.In(region.Rect) {
			a.hoveredUnitID = region.UnitID
			return
		}
	}
	a.hoveredUnitID = ""
}

// syncStatsDisplay updates all stats text widgets based on hovered unit.
func (a *App) syncStatsDisplay() {
	if a.hoveredUnitID == "" {
		a.gameUI.SetNameText("")
		a.gameUI.SetHPText("")
		a.gameUI.SetShieldText("")
		a.gameUI.SetDiceText("")
		return
	}

	unit := a.findUnit(a.hoveredUnitID)
	if unit == nil {
		a.gameUI.SetNameText("")
		a.gameUI.SetHPText("")
		a.gameUI.SetShieldText("")
		a.gameUI.SetDiceText("")
		return
	}

	a.gameUI.SetNameText(formatTemplateName(unit.TemplateID))
	a.gameUI.SetHPText(formatHPBar(*unit))
	a.gameUI.SetShieldText(formatShields(*unit))
	a.gameUI.SetDiceText(formatDiceNet(*unit))
}

// findUnit locates a unit by ID in the current phase's data.
func (a *App) findUnit(unitID string) *entity.Unit {
	switch a.model.Phase {
	case tea.PhaseCombat:
		for i := range a.model.Combat.PlayerUnits {
			if a.model.Combat.PlayerUnits[i].ID == unitID {
				return &a.model.Combat.PlayerUnits[i]
			}
		}
		for i := range a.model.Combat.EnemyUnits {
			if a.model.Combat.EnemyUnits[i].ID == unitID {
				return &a.model.Combat.EnemyUnits[i]
			}
		}
	case tea.PhaseInterCombat:
		for i := range a.model.PlayerRoster {
			if a.model.PlayerRoster[i].ID == unitID {
				return &a.model.PlayerRoster[i]
			}
		}
	}
	return nil
}

// formatTemplateName converts "small_mech" to "Small Mech".
func formatTemplateName(templateID string) string {
	words := strings.Split(templateID, "_")
	for i, word := range words {
		if len(word) > 0 {
			words[i] = strings.ToUpper(word[:1]) + word[1:]
		}
	}
	return strings.Join(words, " ")
}

// formatHPBar creates "■■■□□ 3/5" style HP display.
func formatHPBar(unit entity.Unit) string {
	hp := 0
	maxHP := 0
	if h, ok := unit.Attributes["health"]; ok {
		hp = h.Base
	}
	if m, ok := unit.Attributes["max_health"]; ok {
		maxHP = m.Base
	}
	if maxHP <= 0 {
		return ""
	}
	filled := hp
	if filled < 0 {
		filled = 0
	}
	if filled > maxHP {
		filled = maxHP
	}
	empty := maxHP - filled
	return fmt.Sprintf("%s%s %d/%d",
		strings.Repeat("■", filled),
		strings.Repeat("□", empty),
		hp, maxHP)
}

// formatShields returns shield display string.
// Returns empty string if shields <= 0.
func formatShields(unit entity.Unit) string {
	shields := 0
	if s, ok := unit.Attributes["shields"]; ok {
		shields = s.Base
	}
	if shields <= 0 {
		return ""
	}
	return fmt.Sprintf("⛨ %d", shields)
}

// formatDiceNet returns the cross-shaped dice grid.
// Returns empty string if unit has no die or < 6 faces.
func formatDiceNet(unit entity.Unit) string {
	if !unit.HasDie || len(unit.Die.Faces) < 6 {
		return ""
	}
	return buildDiceNetString(unit.Die.Faces)
}

// buildDiceNetString creates the cross-shaped dice layout using box-drawing characters.
// Layout (faces indexed 0-5):
//
//	    ┌───┐
//	    │ 0 │
//	┌───┼───┼───┬───┐
//	│ 1 │ 2 │ 3 │ 4 │
//	└───┼───┼───┴───┘
//	    │ 5 │
//	    └───┘
func buildDiceNetString(faces []entity.DieFace) string {
	if len(faces) < 6 {
		return ""
	}

	// Format each face as centered 3-char content
	f := make([]string, 6)
	for i := 0; i < 6; i++ {
		f[i] = formatDieFaceContent(faces[i])
	}

	var sb strings.Builder
	// Row 0: top of top box
	sb.WriteString("    ┌───┐\n")
	// Row 1: top box content
	sb.WriteString(fmt.Sprintf("    │%s│\n", f[0]))
	// Row 2: bottom of top box / top of middle row (shared borders)
	sb.WriteString("┌───┼───┼───┬───┐\n")
	// Row 3: middle row content
	sb.WriteString(fmt.Sprintf("│%s│%s│%s│%s│\n", f[1], f[2], f[3], f[4]))
	// Row 4: bottom of middle row / top of bottom box (shared borders)
	sb.WriteString("└───┼───┼───┴───┘\n")
	// Row 5: bottom box content
	sb.WriteString(fmt.Sprintf("    │%s│\n", f[5]))
	// Row 6: bottom of bottom box
	sb.WriteString("    └───┘")

	return sb.String()
}

// formatDieFaceContent returns 3-char centered content for a die face.
// Examples: "D1 ", "H2 ", "S1 ", " x "
func formatDieFaceContent(face entity.DieFace) string {
	switch face.Type {
	case entity.DieDamage:
		return fmt.Sprintf("D%-2d", face.Value)
	case entity.DieHeal:
		return fmt.Sprintf("H%-2d", face.Value)
	case entity.DieShield:
		return fmt.Sprintf("S%-2d", face.Value)
	case entity.DieBlank:
		return " x "
	default:
		return " ? "
	}
}
