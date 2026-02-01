package tea

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

func TestHandleRoundStarted(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{{
				ID:   "unit1",
				Dice: []entity.Die{{Type: entity.DieDamage, Faces: []int{2, 2, 3, 4, 0, 0}}},
			}},
		},
	}

	msg := RoundStarted{
		Round:     1,
		UnitRolls: map[string][]int{"unit1": {2}}, // face index 2 = value 3
	}

	newM, _ := m.Update(msg)

	if newM.Combat.Round != 1 {
		t.Errorf("Round = %d, want 1", newM.Combat.Round)
	}
	if newM.Combat.DicePhase != model.DicePhasePreview {
		t.Errorf("DicePhase = %v, want Preview", newM.Combat.DicePhase)
	}
	if newM.Combat.RerollsRemaining != 2 {
		t.Errorf("RerollsRemaining = %d, want 2", newM.Combat.RerollsRemaining)
	}

	rolled := newM.Combat.RolledDice["unit1"]
	if len(rolled) != 1 {
		t.Fatalf("expected 1 rolled die, got %d", len(rolled))
	}
	if rolled[0].Result != 3 {
		t.Errorf("rolled[0].Result = %d, want 3", rolled[0].Result)
	}
	if rolled[0].FaceIndex != 2 {
		t.Errorf("rolled[0].FaceIndex = %d, want 2", rolled[0].FaceIndex)
	}
}

func TestHandleDiceEffectApplied_DamageWithShields(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			EnemyUnits: []entity.Unit{{
				ID: "enemy",
				Attributes: map[string]core.Attribute{
					"health":  {Name: "health", Base: 50},
					"shields": {Name: "shields", Base: 8},
				},
			}},
		},
	}

	msg := DiceEffectApplied{
		SourceUnitID: "player_cmd",
		TargetUnitID: "enemy",
		Effect:       entity.DieDamage,
		Value:        12,
		NewHealth:    46, // 12 damage - 8 shields = 4 to health, 50-4=46
		NewShields:   0,
	}

	newM, _ := m.Update(msg)

	enemy := newM.Combat.EnemyUnits[0]
	if enemy.Attributes["health"].Base != 46 {
		t.Errorf("health = %d, want 46", enemy.Attributes["health"].Base)
	}
	if enemy.Attributes["shields"].Base != 0 {
		t.Errorf("shields = %d, want 0", enemy.Attributes["shields"].Base)
	}
}

func TestHandleDiceActivated_TargetValidation(t *testing.T) {
	playerCmd := entity.Unit{
		ID:   "player_cmd",
		Tags: []core.Tag{"command"},
		Dice: []entity.Die{{Type: entity.DieDamage, Faces: []int{5}}},
	}

	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			PlayerUnits:      []entity.Unit{playerCmd},
			EnemyUnits:       []entity.Unit{{ID: "enemy"}},
			DicePhase:        model.DicePhasePlayerCommand,
			SelectedUnitID:   "player_cmd",
			SelectedDieIndex: 0,
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Type: entity.DieDamage, Result: 5}},
			},
			ActivatedDice: map[string][]bool{"player_cmd": {false}},
		},
	}

	// Damage die targeting friendly = invalid, should be no-op
	msg := DiceActivated{
		SourceUnitID: "player_cmd",
		DieIndex:     0,
		TargetUnitID: "player_cmd", // friendly target
	}

	newM, cmd := m.Update(msg)

	// Should not activate (invalid target)
	if newM.Combat.ActivatedDice["player_cmd"][0] {
		t.Error("damage die should not activate on friendly target")
	}
	if cmd != nil {
		t.Error("should not return effect cmd for invalid target")
	}
}

func TestHandleDieLockToggled(t *testing.T) {
	playerCmd := entity.Unit{
		ID:   "player_cmd",
		Tags: []core.Tag{"command"},
		Dice: []entity.Die{{Type: entity.DieDamage, Faces: []int{5}}},
	}

	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerCmd},
			DicePhase:   model.DicePhasePlayerCommand,
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Type: entity.DieDamage, Result: 5, Locked: false}},
			},
		},
	}

	msg := DieLockToggled{UnitID: "player_cmd", DieIndex: 0}
	newM, _ := m.Update(msg)

	if !newM.Combat.RolledDice["player_cmd"][0].Locked {
		t.Error("die should be locked after toggle")
	}

	// Toggle again
	newM2, _ := newM.Update(msg)
	if newM2.Combat.RolledDice["player_cmd"][0].Locked {
		t.Error("die should be unlocked after second toggle")
	}
}

func TestHandlePreviewDone(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			DicePhase: model.DicePhasePreview,
		},
	}

	newM, _ := m.Update(PreviewDone{})

	if newM.Combat.DicePhase != model.DicePhasePlayerCommand {
		t.Errorf("DicePhase = %v, want PlayerCommand", newM.Combat.DicePhase)
	}
}

func TestHandlePlayerCommandDone(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			DicePhase: model.DicePhasePlayerCommand,
		},
	}

	newM, cmd := m.Update(PlayerCommandDone{})

	if newM.Combat.DicePhase != model.DicePhaseEnemyCommand {
		t.Errorf("DicePhase = %v, want EnemyCommand", newM.Combat.DicePhase)
	}
	if cmd == nil {
		t.Error("expected cmd to advance phase")
	}
}
