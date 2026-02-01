package tea

import "wulfaz/internal/entity"

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

// SelectTarget picks first overlapping enemy, or command unit if gap.
// Stub implementation: returns first overlapping enemy (Wave 4 adds lowest HP priority).
func SelectTarget(attacker entity.Unit, enemies []entity.Unit, enemyCmd *entity.Unit) string {
	overlapping := FindOverlappingEnemies(attacker, enemies)
	if len(overlapping) > 0 {
		return overlapping[0].ID // Stub: first overlapping (Wave 4 adds lowest HP)
	}
	// Gap: target command unit
	if enemyCmd != nil && enemyCmd.IsAlive() {
		return enemyCmd.ID
	}
	return ""
}
