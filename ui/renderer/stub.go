package renderer

import (
	"fmt"
	"strings"

	"wulfaz/internal/model"
	"wulfaz/internal/tea"
)

// Render renders the Model to a string for text-based output
func Render(m tea.Model) string {
	var sb strings.Builder

	switch m.Phase {
	case tea.PhaseMenu:
		sb.WriteString("=== WULFAZ ===\n")
		sb.WriteString("Menu\n")
	case tea.PhaseCombat:
		renderCombatText(&sb, m.Combat)
	case tea.PhaseGameOver:
		sb.WriteString("=== GAME OVER ===\n")
	default:
		sb.WriteString("Unknown phase\n")
	}

	return sb.String()
}

func renderCombatText(sb *strings.Builder, combat model.CombatModel) {
	sb.WriteString("=== COMBAT ===\n")
	fmt.Fprintf(sb, "Tick: %d\n", combat.Tick)

	if combat.Phase == model.CombatPaused {
		sb.WriteString("** PAUSED **\n")
	}

	sb.WriteString("\nPlayer Units:\n")
	for _, u := range combat.PlayerUnits {
		health := 0
		if attr, ok := u.Attributes["health"]; ok {
			health = attr.Base
		}
		fmt.Fprintf(sb, "  [%s] HP: %d\n", u.ID, health)
	}

	sb.WriteString("\nEnemy Units:\n")
	for _, u := range combat.EnemyUnits {
		health := 0
		if attr, ok := u.Attributes["health"]; ok {
			health = attr.Base
		}
		fmt.Fprintf(sb, "  [%s] HP: %d\n", u.ID, health)
	}

	if len(combat.Log) > 0 {
		sb.WriteString("\nCombat Log:\n")
		start := 0
		if len(combat.Log) > 10 {
			start = len(combat.Log) - 10
		}
		for _, entry := range combat.Log[start:] {
			fmt.Fprintf(sb, "  %s\n", entry)
		}
	}
}
