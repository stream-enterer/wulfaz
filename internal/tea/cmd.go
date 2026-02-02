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
			} else if len(rd.Faces) > 0 {
				// Reroll unlocked dice
				results[i] = rng.Intn(len(rd.Faces))
			}
		}

		return RerollRequested{UnitID: unitID, Results: results}
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

// ===== Wave 3: Combat Phase Commands =====

// ExecuteEnemyCommand runs simple enemy AI for command dice.
// Damage -> lowest HP player unit; Shield/Heal -> lowest HP enemy unit.
// Blank faces are skipped.
func ExecuteEnemyCommand(combat model.CombatModel) Cmd {
	return func() Msg {
		enemyCmd := findEnemyCommandUnit(combat)
		if enemyCmd == nil {
			return EnemyCommandResolved{Actions: nil}
		}

		rolled := combat.RolledDice[enemyCmd.ID]
		var actions []EnemyDiceAction

		for i, rd := range rolled {
			// Skip blank faces
			if rd.Type() == entity.DieBlank {
				continue
			}

			var targetID string
			switch rd.Type() {
			case entity.DieDamage:
				targetID = findLowestHPAliveUnit(combat.PlayerUnits)
			case entity.DieShield, entity.DieHeal:
				targetID = findLowestHPAliveUnit(combat.EnemyUnits)
			case entity.DieBlank:
				// Blank dice are skipped earlier, but handle for exhaustiveness
			}

			if targetID != "" {
				actions = append(actions, EnemyDiceAction{
					SourceUnitID: enemyCmd.ID,
					TargetUnitID: targetID,
					DieIndex:     i,
					Effect:       rd.Type(),
					Value:        rd.Value(),
				})
			}
		}
		return EnemyCommandResolved{Actions: actions}
	}
}

// ExecuteExecution builds firing order and starts execution.
func ExecuteExecution(combat model.CombatModel) Cmd {
	return func() Msg {
		order := buildFiringOrder(combat)
		return ExecutionStarted{FiringOrder: order}
	}
}

// ResolvePosition calculates attacks for units at one position.
// Key: Collect ALL attacks first, THEN calculate final HP (simultaneous).
func ResolvePosition(pos model.FiringPosition, combat model.CombatModel, timestamp int64) Cmd {
	return func() Msg {
		var attacks []AttackResult
		unitMap := buildUnitMap(combat)
		hpSnapshot := buildHPSnapshot(combat)

		playerCmd := FindPlayerCommandUnit(combat)
		enemyCmd := findEnemyCommandUnit(combat)

		// Player units attack enemy units
		attacks = resolveAttacks(attacks, pos.PlayerUnits, combat.EnemyUnits, enemyCmd,
			unitMap, combat.RolledDice, hpSnapshot)

		// Enemy units attack player units
		attacks = resolveAttacks(attacks, pos.EnemyUnits, combat.PlayerUnits, playerCmd,
			unitMap, combat.RolledDice, hpSnapshot)

		return PositionResolved{Position: pos.Position, Attacks: attacks, Timestamp: timestamp}
	}
}

// resolveAttacks calculates damage from attackerIDs to targets with overflow.
// Each die attack is resolved separately with MTG-style overflow.
// Gap damage only hits command if ALL enemy units are dead (F-167).
func resolveAttacks(
	attacks []AttackResult,
	attackerIDs []string,
	targets []entity.Unit,
	targetCmd *entity.Unit,
	unitMap map[string]entity.Unit,
	rolledDice map[string][]entity.RolledDie,
	hpSnapshot map[string][2]int,
) []AttackResult {
	for _, uid := range attackerIDs {
		attacker, ok := unitMap[uid]
		if !ok || !attacker.IsAlive() {
			continue
		}
		rolled := rolledDice[uid]

		for dieIdx, rd := range rolled {
			// Skip non-damage (this also skips blanks since blank != damage)
			if rd.Type() != entity.DieDamage {
				continue
			}

			// Try overflow damage to overlapping enemies
			results := ApplyDamageWithOverflow(attacker, rd.Value(), targets, hpSnapshot)

			if len(results) > 0 {
				// Convert overflow results to AttackResults
				for _, r := range results {
					attacks = append(attacks, AttackResult{
						AttackerID:     uid,
						TargetID:       r.TargetID,
						DieIndex:       dieIdx,
						Damage:         r.Damage,
						ShieldAbsorbed: r.ShieldAbsorbed,
						NewHealth:      r.NewHP,
						NewShields:     r.NewShields,
						TargetDead:     r.Killed,
					})
				}
			} else if !AnyAliveUnits(targets) && targetCmd != nil && targetCmd.IsAlive() {
				// Gap case: F-166 + F-167
				// Only hit command if ALL enemy units are dead
				hp, shields := hpSnapshot[targetCmd.ID][0], hpSnapshot[targetCmd.ID][1]
				remaining := rd.Value()
				absorbed := min(remaining, shields)
				remaining -= absorbed
				shields -= absorbed
				hp = max(0, hp-remaining)

				attacks = append(attacks, AttackResult{
					AttackerID:     uid,
					TargetID:       targetCmd.ID,
					DieIndex:       dieIdx,
					Damage:         rd.Value(),
					ShieldAbsorbed: absorbed,
					NewHealth:      hp,
					NewShields:     shields,
					TargetDead:     hp <= 0,
				})
				hpSnapshot[targetCmd.ID] = [2]int{hp, shields}
			}
			// Else: gap but units exist elsewhere - damage wasted
		}
	}
	return attacks
}

// StartNextRound wraps RollAllDice with round-based seed variation.
func StartNextRound(baseSeed int64, round int, units []entity.Unit) Cmd {
	roundSeed := baseSeed + int64(round)*7919 // Prime offset per round
	return RollAllDice(roundSeed, round, units)
}
