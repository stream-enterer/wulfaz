package tea

import (
	"math/rand"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

type Cmd func() Msg

func None() Cmd { return nil }

func Batch(cmds ...Cmd) Cmd {
	if len(cmds) == 0 {
		return nil
	}
	return func() Msg {
		var msgs []Msg
		for _, cmd := range cmds {
			if cmd != nil {
				if msg := cmd(); msg != nil {
					msgs = append(msgs, msg)
				}
			}
		}
		switch len(msgs) {
		case 0:
			return nil
		case 1:
			return msgs[0]
		default:
			return BatchedMsgs{Msgs: msgs}
		}
	}
}

// RollAllDice creates a Cmd that rolls dice for all units.
// RNG happens here (in Cmd), results passed via Msg (TEA compliance).
func RollAllDice(seed int64, round int, allUnits []entity.Unit) Cmd {
	return func() Msg {
		rng := rand.New(rand.NewSource(seed))
		rolls := make(map[string][]int)

		for _, unit := range allUnits {
			if len(unit.Dice) == 0 {
				continue
			}
			unitRolls := make([]int, len(unit.Dice))
			for i, die := range unit.Dice {
				if len(die.Faces) > 0 {
					unitRolls[i] = rng.Intn(len(die.Faces))
				}
			}
			rolls[unit.ID] = unitRolls
		}

		return RoundStarted{Round: round, UnitRolls: rolls}
	}
}

// RerollUnlockedDice creates a Cmd that rerolls unlocked dice.
func RerollUnlockedDice(seed int64, unitID string, current []entity.RolledDie) Cmd {
	return func() Msg {
		rng := rand.New(rand.NewSource(seed))
		results := make([]int, len(current))

		for i, rd := range current {
			if rd.Locked {
				// Keep locked dice at same face index
				results[i] = rd.FaceIndex
			} else {
				// Reroll unlocked dice
				if len(rd.Faces) > 0 {
					results[i] = rng.Intn(len(rd.Faces))
				}
			}
		}

		return RerollRequested{UnitID: unitID, Results: results}
	}
}

// ApplyDiceEffect creates a Cmd that computes effect result.
func ApplyDiceEffect(sourceID, targetID string, effect entity.DieType, value int, combat model.CombatModel) Cmd {
	return func() Msg {
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

		newHealth, newShields := health, shields

		switch effect {
		case entity.DieDamage:
			remaining := value
			// Shields absorb first
			if remaining > 0 && newShields > 0 {
				absorbed := min(remaining, newShields)
				remaining -= absorbed
				newShields -= absorbed
			}
			newHealth = max(0, health-remaining)

		case entity.DieShield:
			newShields = shields + value

		case entity.DieHeal:
			newHealth = min(health+value, maxHealth)
		}

		return DiceEffectApplied{
			SourceUnitID: sourceID,
			TargetUnitID: targetID,
			Effect:       effect,
			Value:        value,
			NewHealth:    newHealth,
			NewShields:   newShields,
		}
	}
}

// AdvanceDicePhase creates a Cmd that advances to next phase.
func AdvanceDicePhase(next model.DicePhase) Cmd {
	return func() Msg {
		return DicePhaseAdvanced{NewPhase: next}
	}
}
