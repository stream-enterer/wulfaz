package tea

import (
	"math/rand"
	"time"

	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

// Timer IDs
const (
	TimerRoundEnd = "round_end" // 2 second end-of-round pause
)

// StartTimer creates a Cmd that requests a timer from the runtime.
func StartTimer(id string, duration time.Duration) Cmd {
	return func() Msg {
		return StartTimerRequested{ID: id, Duration: duration}
	}
}

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

// RollAllDice creates a Cmd that rolls the single die for each unit.
// RNG happens here (in Cmd), results passed via Msg (TEA compliance).
func RollAllDice(seed int64, round int, allUnits []entity.Unit) Cmd {
	return func() Msg {
		rng := rand.New(rand.NewSource(seed))
		rolls := make(map[string]int)

		for _, unit := range allUnits {
			if !unit.HasDie || len(unit.Die.Faces) == 0 {
				continue
			}
			rolls[unit.ID] = rng.Intn(len(unit.Die.Faces))
		}

		return RoundStarted{Round: round, UnitRolls: rolls}
	}
}

// RerollAllUnlockedDice creates a Cmd that rerolls all unlocked player dice.
func RerollAllUnlockedDice(seed int64, combat model.CombatModel) Cmd {
	return func() Msg {
		rng := rand.New(rand.NewSource(seed))
		results := make(map[string]int)

		for _, unit := range combat.PlayerUnits {
			if !unit.HasDie || len(unit.Die.Faces) == 0 {
				continue
			}

			rolled, exists := combat.RolledDice[unit.ID]
			if !exists {
				continue
			}

			if rolled.Locked {
				// Skip locked dice
				continue
			}

			// Reroll unlocked die
			results[unit.ID] = rng.Intn(len(unit.Die.Faces))
		}

		return RerollRequested{Results: results}
	}
}

// ApplyDiceEffect creates a Cmd that computes effect result.
func ApplyDiceEffect(sourceID, targetID string, effect entity.DieType, value int, combat model.CombatModel, timestamp int64) Cmd {
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
		shieldAbsorbed := 0

		switch effect {
		case entity.DieDamage:
			remaining := value
			// Shields absorb first
			if remaining > 0 && newShields > 0 {
				shieldAbsorbed = min(remaining, newShields)
				remaining -= shieldAbsorbed
				newShields -= shieldAbsorbed
			}
			newHealth = max(0, health-remaining)

		case entity.DieShield:
			newShields = shields + value

		case entity.DieHeal:
			newHealth = min(health+value, maxHealth)

		case entity.DieBlank:
			// Blank dice have no effect
		}

		return DiceEffectApplied{
			SourceUnitID:   sourceID,
			TargetUnitID:   targetID,
			Effect:         effect,
			Value:          value,
			NewHealth:      newHealth,
			NewShields:     newShields,
			ShieldAbsorbed: shieldAbsorbed,
			Timestamp:      timestamp,
		}
	}
}

// AdvanceDicePhase creates a Cmd that advances to next phase.
func AdvanceDicePhase(next model.DicePhase) Cmd {
	return func() Msg {
		return DicePhaseAdvanced{NewPhase: next}
	}
}

// ComputeAITargets computes targets for all enemy units.
// Regular units: random valid target (filtered for doomed)
// Commander: lowest HP target
// Shield/Heal: lowest HP ally
func ComputeAITargets(combat model.CombatModel, seed int64) Cmd {
	return func() Msg {
		rng := rand.New(rand.NewSource(seed))
		targets := make(map[string]string)

		// Track incoming damage to filter doomed targets
		incoming := make(map[string]int)

		for _, enemy := range combat.EnemyUnits {
			if !enemy.IsAlive() || !enemy.HasDie {
				continue
			}

			rolled, exists := combat.RolledDice[enemy.ID]
			if !exists {
				continue
			}

			face := rolled.CurrentFace()
			if face.Type == entity.DieBlank {
				continue
			}

			var targetID string

			switch face.Type {
			case entity.DieDamage:
				validTargets := GetValidEnemyTargets(combat.PlayerUnits)
				if len(validTargets) == 0 {
					continue
				}

				// Filter doomed targets for regular units
				if !enemy.IsCommand() {
					validTargets = FilterDoomedTargets(validTargets, incoming, combat)
				}

				// Commander uses lowest HP, regular units use random
				if enemy.IsCommand() {
					targetID = SelectLowestHP(validTargets)
				} else {
					targetID = SelectRandomTarget(validTargets, rng)
				}

				// Track incoming damage
				if targetID != "" {
					incoming[targetID] += face.Value
				}

			case entity.DieShield, entity.DieHeal:
				// Target lowest HP ally
				validAllies := GetValidAlliedTargets(combat.EnemyUnits)
				targetID = SelectLowestHP(validAllies)
			case entity.DieBlank:
				// Already handled by continue above, but listed for exhaustiveness
			}

			if targetID != "" {
				targets[enemy.ID] = targetID
			}
		}

		return AITargetsComputed{Targets: targets}
	}
}

// ExecuteAllAttacks resolves all attacks simultaneously from HP snapshot.
func ExecuteAllAttacks(combat model.CombatModel, timestamp int64) Cmd {
	return func() Msg {
		var attacks []AttackResult
		hpSnapshot := buildHPSnapshot(combat)

		// Collect all damage from enemy units
		for _, unit := range combat.EnemyUnits {
			if !unit.IsAlive() || !unit.HasDie {
				continue
			}

			targetID, hasTarget := combat.EnemyTargets[unit.ID]
			if !hasTarget {
				continue
			}

			rolled, exists := combat.RolledDice[unit.ID]
			if !exists {
				continue
			}

			face := rolled.CurrentFace()
			if face.Type != entity.DieDamage {
				continue
			}

			attacks = resolveDamage(attacks, unit.ID, targetID, face.Value, hpSnapshot)
		}

		return AllAttacksResolved{Attacks: attacks, Timestamp: timestamp}
	}
}

// resolveDamage applies damage from snapshot and records result.
func resolveDamage(attacks []AttackResult, attackerID, targetID string, damage int, hpSnapshot map[string][2]int) []AttackResult {
	hp, shields := hpSnapshot[targetID][0], hpSnapshot[targetID][1]

	// Shields absorb first
	absorbed := min(damage, shields)
	remaining := damage - absorbed
	shields -= absorbed

	// Then HP
	hp = max(0, hp-remaining)

	attacks = append(attacks, AttackResult{
		AttackerID:     attackerID,
		TargetID:       targetID,
		Damage:         damage,
		ShieldAbsorbed: absorbed,
		NewHealth:      hp,
		NewShields:     shields,
		TargetDead:     hp <= 0,
	})

	// Update snapshot
	hpSnapshot[targetID] = [2]int{hp, shields}

	return attacks
}

// StartNextRound wraps RollAllDice with round-based seed variation.
func StartNextRound(baseSeed int64, round int, units []entity.Unit) Cmd {
	roundSeed := baseSeed + int64(round)*7919 // Prime offset per round
	return RollAllDice(roundSeed, round, units)
}
