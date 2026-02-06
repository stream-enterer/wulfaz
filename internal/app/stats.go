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
		a.gameUI.SetDiceFaces(nil)
		return
	}

	unit := a.findUnit(a.hoveredUnitID)
	if unit == nil {
		a.gameUI.SetNameText("")
		a.gameUI.SetHPText("")
		a.gameUI.SetShieldText("")
		a.gameUI.SetDiceFaces(nil)
		return
	}

	a.gameUI.SetNameText(formatTemplateName(unit.TemplateID))
	a.gameUI.SetHPText(formatHPBar(*unit))
	a.gameUI.SetShieldText(formatShields(*unit))
	a.gameUI.SetDiceFaces(getDiceFaces(*unit))
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
	case tea.PhaseMenu, tea.PhaseGameOver:
		// No units to look up in these phases
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

// getDiceFaces returns the die faces for display (first die only).
// Returns nil if unit has no dice or first die has < 6 faces.
func getDiceFaces(unit entity.Unit) []entity.DieFace {
	if len(unit.Dice) == 0 || len(unit.Dice[0].Faces) < 6 {
		return nil
	}
	return unit.Dice[0].Faces
}
