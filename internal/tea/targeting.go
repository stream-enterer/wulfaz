package tea

import (
	"slices"

	"wulfaz/internal/entity"
)

// BoardSpaces is the number of positions on the combat board.
const BoardSpaces = 10

// GetOccupiedSpaces returns the board spaces a unit occupies [position, position+width-1].
// Returns nil for off-board units (Position < 0).
func GetOccupiedSpaces(unit entity.Unit) []int {
	if unit.Position < 0 {
		return nil
	}
	width := 1
	if w, ok := unit.Attributes["combat_width"]; ok {
		width = w.Base
	}
	spaces := make([]int, width)
	for i := 0; i < width; i++ {
		spaces[i] = unit.Position + i
	}
	return spaces
}

// FindOverlappingEnemies returns alive enemies that overlap the attacker's spaces.
func FindOverlappingEnemies(attacker entity.Unit, enemies []entity.Unit) []entity.Unit {
	attackerSpaces := GetOccupiedSpaces(attacker)
	if len(attackerSpaces) == 0 {
		return nil
	}
	spaceSet := make(map[int]bool)
	for _, s := range attackerSpaces {
		spaceSet[s] = true
	}
	var overlapping []entity.Unit
	for _, enemy := range enemies {
		if !enemy.IsAlive() || enemy.Position < 0 {
			continue
		}
		for _, es := range GetOccupiedSpaces(enemy) {
			if spaceSet[es] {
				overlapping = append(overlapping, enemy)
				break
			}
		}
	}
	return overlapping
}

// getHP returns unit's current health.
func getHP(u entity.Unit) int {
	if h, ok := u.Attributes["health"]; ok {
		return h.Base
	}
	return 0
}

// SelectTargetUnit picks lowest HP overlapping enemy.
// Ties: left-to-right by position. Returns "" if no valid target.
func SelectTargetUnit(attacker entity.Unit, enemies []entity.Unit) string {
	overlapping := FindOverlappingEnemies(attacker, enemies)
	if len(overlapping) == 0 {
		return ""
	}
	// Sort: lowest HP, then lowest position
	slices.SortFunc(overlapping, func(a, b entity.Unit) int {
		if hpA, hpB := getHP(a), getHP(b); hpA != hpB {
			return hpA - hpB
		}
		return a.Position - b.Position
	})
	return overlapping[0].ID
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

// OverflowResult tracks damage to one unit in overflow chain.
type OverflowResult struct {
	TargetID   string
	Damage     int
	NewHP      int
	NewShields int
	Killed     bool
}

// ApplyDamageWithOverflow applies damage with MTG-style overflow.
// Returns results for each unit hit, updates hpSnapshot in place.
// Does NOT overflow to command unit.
func ApplyDamageWithOverflow(
	attacker entity.Unit,
	damage int,
	enemies []entity.Unit,
	hpSnapshot map[string][2]int,
) []OverflowResult {
	if damage <= 0 {
		return nil
	}
	overlapping := FindOverlappingEnemies(attacker, enemies)
	if len(overlapping) == 0 {
		return nil
	}

	// Sort by snapshot HP, then position
	slices.SortFunc(overlapping, func(a, b entity.Unit) int {
		hpA, hpB := hpSnapshot[a.ID][0], hpSnapshot[b.ID][0]
		if hpA != hpB {
			return hpA - hpB
		}
		return a.Position - b.Position
	})

	var results []OverflowResult
	remaining := damage

	for _, target := range overlapping {
		if remaining <= 0 {
			break
		}
		hp, shields := hpSnapshot[target.ID][0], hpSnapshot[target.ID][1]
		if hp <= 0 {
			continue // Already dead
		}

		// Shields absorb first
		absorbed := min(remaining, shields)
		remaining -= absorbed
		shields -= absorbed

		// Then HP
		hpDamage := min(remaining, hp)
		remaining -= hpDamage
		hp -= hpDamage

		results = append(results, OverflowResult{
			TargetID:   target.ID,
			Damage:     absorbed + hpDamage,
			NewHP:      hp,
			NewShields: shields,
			Killed:     hp <= 0,
		})

		hpSnapshot[target.ID] = [2]int{hp, shields}
	}
	return results
}

// SelectTarget picks target for attacker (legacy wrapper).
// Uses lowest HP priority for overlapping enemies.
// Gap: only targets command if all enemy units are dead (F-167 constraint).
func SelectTarget(attacker entity.Unit, enemies []entity.Unit, enemyCmd *entity.Unit) string {
	if targetID := SelectTargetUnit(attacker, enemies); targetID != "" {
		return targetID
	}
	// Gap: only target command if all units dead
	if !AnyAliveUnits(enemies) && enemyCmd != nil && enemyCmd.IsAlive() {
		return enemyCmd.ID
	}
	return ""
}
