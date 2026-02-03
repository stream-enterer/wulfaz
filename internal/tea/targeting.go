package tea

import (
	"math/rand"
	"slices"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

// BoardSpaces is the number of positions on the combat board.
const BoardSpaces = 10

// getHP returns unit's current health.
func getHP(u entity.Unit) int {
	if h, ok := u.Attributes["health"]; ok {
		return h.Base
	}
	return 0
}

// getShields returns unit's current shields.
func getShields(u entity.Unit) int {
	if s, ok := u.Attributes["shields"]; ok {
		return s.Base
	}
	return 0
}

// AnyAliveUnits returns true if any non-command unit is alive.
func AnyAliveUnits(units []entity.Unit) bool {
	for _, u := range units {
		if !u.IsCommand() && u.IsAlive() {
			return true
		}
	}
	return false
}

// CanTargetUnit returns true if target can be attacked given F-167 constraints.
// Command units can only be targeted when all regular enemies are dead.
func CanTargetUnit(target entity.Unit, allEnemies []entity.Unit) bool {
	if !target.IsAlive() {
		return false
	}
	if target.IsCommand() {
		// F-167: Can only target command when all regular enemies dead
		return !AnyAliveUnits(allEnemies)
	}
	return true
}

// GetValidEnemyTargets returns alive enemies that can be targeted (respects F-167).
func GetValidEnemyTargets(enemies []entity.Unit) []entity.Unit {
	var valid []entity.Unit
	hasAliveRegular := AnyAliveUnits(enemies)

	for _, e := range enemies {
		if !e.IsAlive() {
			continue
		}
		if e.IsCommand() && hasAliveRegular {
			// F-167: Skip command while regulars alive
			continue
		}
		valid = append(valid, e)
	}
	return valid
}

// GetValidAlliedTargets returns alive allies (for shield/heal targeting).
func GetValidAlliedTargets(allies []entity.Unit) []entity.Unit {
	var valid []entity.Unit
	for _, a := range allies {
		if a.IsAlive() {
			valid = append(valid, a)
		}
	}
	return valid
}

// FilterDoomedTargets removes targets that will die from incoming damage.
// Returns remaining targets, or all targets if all would be doomed.
func FilterDoomedTargets(enemies []entity.Unit, incoming map[string]int, combat model.CombatModel) []entity.Unit {
	var remaining []entity.Unit

	for _, e := range enemies {
		incomingDmg := incoming[e.ID]
		hp := getHP(e)
		shields := getShields(e)
		effective := hp + shields

		if incomingDmg < effective {
			remaining = append(remaining, e)
		}
	}

	// If all would be doomed, return original list
	if len(remaining) == 0 {
		return enemies
	}
	return remaining
}

// SelectLowestHP returns the unit ID with lowest HP from candidates.
// Ties: first in slice order.
func SelectLowestHP(units []entity.Unit) string {
	if len(units) == 0 {
		return ""
	}

	// Sort by HP
	slices.SortFunc(units, func(a, b entity.Unit) int {
		return getHP(a) - getHP(b)
	})

	return units[0].ID
}

// SelectRandomTarget returns a random unit ID from candidates.
func SelectRandomTarget(units []entity.Unit, rng *rand.Rand) string {
	if len(units) == 0 {
		return ""
	}
	return units[rng.Intn(len(units))].ID
}
