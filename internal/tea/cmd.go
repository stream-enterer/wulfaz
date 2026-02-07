package tea

import (
	"math/rand"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

// RollAllDice creates a Cmd that rolls each die for each unit.
// RNG happens here (in Cmd), results passed via Msg (TEA compliance).
func RollAllDice(seed int64, round int, allUnits []entity.Unit) model.Cmd {
	return func() model.Msg {
		rng := rand.New(rand.NewSource(seed))
		rolls := make(map[string][]int)

		for _, unit := range allUnits {
			if len(unit.Dice) == 0 {
				continue
			}
			unitRolls := make([]int, len(unit.Dice))
			for i, die := range unit.Dice {
				if len(die.Faces) == 0 {
					continue
				}
				unitRolls[i] = rng.Intn(len(die.Faces))
			}
			rolls[unit.ID] = unitRolls
		}

		return model.RoundStarted{Round: round, UnitRolls: rolls}
	}
}

// RerollAllUnlockedDice creates a Cmd that rerolls all unlocked player dice.
func RerollAllUnlockedDice(seed int64, combat model.CombatModel) model.Cmd {
	return func() model.Msg {
		rng := rand.New(rand.NewSource(seed))
		results := make(map[string][]int)

		for _, unit := range combat.PlayerUnits {
			if len(unit.Dice) == 0 {
				continue
			}

			rolledDice, exists := combat.RolledDice[unit.ID]
			if !exists {
				continue
			}

			if entity.IsUnitLocked(rolledDice) {
				continue
			}

			// Reroll each die independently
			unitResults := make([]int, len(rolledDice))
			for i, rd := range rolledDice {
				if len(rd.Faces) == 0 {
					continue
				}
				unitResults[i] = rng.Intn(len(rd.Faces))
			}
			results[unit.ID] = unitResults
		}

		return model.RerollRequested{Results: results}
	}
}

// ApplyCompatibleDiceEffects creates a Cmd that computes results for all compatible dice.
// targetIsEnemy: true -> process only unfired damage dice; false -> process only unfired shield/heal dice.
func ApplyCompatibleDiceEffects(sourceID, targetID string, rolledDice []entity.RolledDie, targetIsEnemy bool, combat model.CombatModel, timestamp int64) model.Cmd {
	return func() model.Msg {
		// Find target unit
		var target entity.Unit
		var found bool
		for _, u := range combat.PlayerUnits {
			if u.ID == targetID {
				target = u
				found = true
				break
			}
		}
		if !found {
			for _, u := range combat.EnemyUnits {
				if u.ID == targetID {
					target = u
					break
				}
			}
		}

		health := 0
		if h, ok := target.Attributes["health"]; ok {
			health = h.Base
		}
		maxHealth := health
		if mh, ok := target.Attributes["max_health"]; ok {
			maxHealth = mh.Base
		}
		shields := 0
		if s, ok := target.Attributes["shields"]; ok {
			shields = s.Base
		}

		var results []model.DiceEffectResult
		newHealth, newShields := health, shields

		for _, rd := range rolledDice {
			if rd.Fired {
				continue
			}
			face := rd.CurrentFace()
			if face.Type == entity.DieBlank {
				continue
			}

			compatible := false
			if targetIsEnemy && face.Type == entity.DieDamage {
				compatible = true
			}
			if !targetIsEnemy && (face.Type == entity.DieShield || face.Type == entity.DieHeal) {
				compatible = true
			}
			if !compatible {
				continue
			}

			shieldAbsorbed := 0
			switch face.Type {
			case entity.DieDamage:
				remaining := face.Value
				if remaining > 0 && newShields > 0 {
					shieldAbsorbed = min(remaining, newShields)
					remaining -= shieldAbsorbed
					newShields -= shieldAbsorbed
				}
				newHealth = max(0, newHealth-remaining)
			case entity.DieShield:
				newShields += face.Value
			case entity.DieHeal:
				newHealth = min(newHealth+face.Value, maxHealth)
			case entity.DieBlank:
				// Already filtered above
			}

			results = append(results, model.DiceEffectResult{
				TargetUnitID:   targetID,
				Effect:         face.Type,
				Value:          face.Value,
				NewHealth:      newHealth,
				NewShields:     newShields,
				ShieldAbsorbed: shieldAbsorbed,
			})
		}

		return model.UnitDiceEffectsApplied{
			SourceUnitID: sourceID,
			Results:      results,
			Timestamp:    timestamp,
		}
	}
}

// AdvanceDicePhase creates a Cmd that advances to next phase.
func AdvanceDicePhase(next model.DicePhase) model.Cmd {
	return func() model.Msg {
		return model.DicePhaseAdvanced{NewPhase: next}
	}
}

// ComputeAITargets computes targets for all enemy units.
// Damage dice: regular units random target (filtered for doomed), commander lowest HP.
// Shield/Heal dice: lowest HP ally -> stored in DefenseTargets.
func ComputeAITargets(combat model.CombatModel, seed int64) model.Cmd {
	return func() model.Msg {
		rng := rand.New(rand.NewSource(seed))
		targets := make(map[string]string)
		defenseTargets := make(map[string]string)

		// Track incoming damage to filter doomed targets
		incoming := make(map[string]int)

		for _, enemy := range combat.EnemyUnits {
			if !enemy.IsAlive() || len(enemy.Dice) == 0 {
				continue
			}

			rolledDice, exists := combat.RolledDice[enemy.ID]
			if !exists || !entity.HasNonBlankDie(rolledDice) {
				continue
			}

			// Process damage dice
			if entity.HasDieOfType(rolledDice, entity.DieDamage) {
				validTargets := GetValidEnemyTargets(combat.PlayerUnits)
				if len(validTargets) > 0 {
					// Filter doomed targets for regular units
					if !enemy.IsCommand() {
						validTargets = FilterDoomedTargets(validTargets, incoming, combat)
					}

					var targetID string
					if enemy.IsCommand() {
						targetID = SelectLowestHP(validTargets)
					} else {
						targetID = SelectRandomTarget(validTargets, rng)
					}

					if targetID != "" {
						targets[enemy.ID] = targetID
						// Sum ALL damage dice values for incoming tracking
						for _, rd := range rolledDice {
							if rd.CurrentFace().Type == entity.DieDamage {
								incoming[targetID] += rd.CurrentFace().Value
							}
						}
					}
				}
			}

			// Process shield/heal dice
			if entity.HasDieOfType(rolledDice, entity.DieShield) || entity.HasDieOfType(rolledDice, entity.DieHeal) {
				validAllies := GetValidAlliedTargets(combat.EnemyUnits)
				allyID := SelectLowestHP(validAllies)
				if allyID != "" {
					defenseTargets[enemy.ID] = allyID
				}
			}
		}

		return model.AITargetsComputed{Targets: targets, DefenseTargets: defenseTargets}
	}
}

// ApplyEnemyUnitEffects creates a Cmd that computes dice effects for a single enemy unit.
// Damage dice resolve against PlayerUnits (via EnemyTargets), shield/heal against EnemyUnits (via EnemyDefenseTargets).
func ApplyEnemyUnitEffects(unitID string, combat model.CombatModel, timestamp int64) model.Cmd {
	return func() model.Msg {
		rolledDice := combat.RolledDice[unitID]
		var results []model.DiceEffectResult

		// Damage dice → target from EnemyTargets
		if targetID, ok := combat.EnemyTargets[unitID]; ok {
			var hp, shields int
			for _, u := range combat.PlayerUnits {
				if u.ID == targetID {
					if h, ok := u.Attributes["health"]; ok {
						hp = h.Base
					}
					if s, ok := u.Attributes["shields"]; ok {
						shields = s.Base
					}
					break
				}
			}
			newHP, newShields := hp, shields
			for _, rd := range rolledDice {
				face := rd.CurrentFace()
				if face.Type != entity.DieDamage {
					continue
				}
				absorbed := min(face.Value, newShields)
				newShields -= absorbed
				newHP = max(0, newHP-(face.Value-absorbed))
				results = append(results, model.DiceEffectResult{
					TargetUnitID:   targetID,
					Effect:         entity.DieDamage,
					Value:          face.Value,
					NewHealth:      newHP,
					NewShields:     newShields,
					ShieldAbsorbed: absorbed,
				})
			}
		}

		// Shield/heal dice → ally from EnemyDefenseTargets
		if allyID, ok := combat.EnemyDefenseTargets[unitID]; ok {
			var hp, shields, maxHP int
			for _, u := range combat.EnemyUnits {
				if u.ID == allyID {
					if h, ok := u.Attributes["health"]; ok {
						hp = h.Base
					}
					if s, ok := u.Attributes["shields"]; ok {
						shields = s.Base
					}
					if mh, ok := u.Attributes["max_health"]; ok {
						maxHP = mh.Base
					} else {
						maxHP = hp
					}
					break
				}
			}
			newHP, newShields := hp, shields
			for _, rd := range rolledDice {
				face := rd.CurrentFace()
				switch face.Type {
				case entity.DieShield:
					newShields += face.Value
					results = append(results, model.DiceEffectResult{
						TargetUnitID: allyID,
						Effect:       entity.DieShield,
						Value:        face.Value,
						NewHealth:    newHP,
						NewShields:   newShields,
					})
				case entity.DieHeal:
					newHP = min(newHP+face.Value, maxHP)
					results = append(results, model.DiceEffectResult{
						TargetUnitID: allyID,
						Effect:       entity.DieHeal,
						Value:        face.Value,
						NewHealth:    newHP,
						NewShields:   newShields,
					})
				}
			}
		}

		return model.UnitDiceEffectsApplied{
			SourceUnitID: unitID,
			Results:      results,
			Timestamp:    timestamp,
		}
	}
}

// StartNextRound wraps RollAllDice with round-based seed variation.
func StartNextRound(baseSeed int64, round int, units []entity.Unit) model.Cmd {
	roundSeed := baseSeed + int64(round)*7919 // Prime offset per round
	return RollAllDice(roundSeed, round, units)
}
