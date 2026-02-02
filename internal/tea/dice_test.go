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
				ID: "unit1",
				Dice: []entity.Die{{Faces: []entity.DieFace{
					{Type: entity.DieDamage, Value: 2},
					{Type: entity.DieDamage, Value: 2},
					{Type: entity.DieDamage, Value: 3},
					{Type: entity.DieDamage, Value: 4},
					{Type: entity.DieBlank, Value: 0},
					{Type: entity.DieBlank, Value: 0},
				}}},
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
	if newM.Combat.RerollsRemaining != model.DefaultRerollsPerRound {
		t.Errorf("RerollsRemaining = %d, want %d", newM.Combat.RerollsRemaining, model.DefaultRerollsPerRound)
	}

	rolled := newM.Combat.RolledDice["unit1"]
	if len(rolled) != 1 {
		t.Fatalf("expected 1 rolled die, got %d", len(rolled))
	}
	if rolled[0].Value() != 3 {
		t.Errorf("rolled[0].Value() = %d, want 3", rolled[0].Value())
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
		Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
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
				"player_cmd": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0}},
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
		Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerCmd},
			Phase:       model.CombatActive,
			DicePhase:   model.DicePhasePlayerCommand,
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0, Locked: false}},
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

func TestHandleDieSelected_BoundsCheck(t *testing.T) {
	playerCmd := entity.Unit{
		ID:   "player_cmd",
		Tags: []core.Tag{"command"},
		Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits:      []entity.Unit{playerCmd},
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			SelectedUnitID:   "",
			SelectedDieIndex: -1,
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string][]bool{"player_cmd": {false}},
		},
	}

	// Out of bounds index should be rejected
	msg := DieSelected{UnitID: "player_cmd", DieIndex: 99}
	newM, _ := m.Update(msg)

	if newM.Combat.SelectedDieIndex != -1 {
		t.Errorf("SelectedDieIndex = %d, want -1 (invalid index should be rejected)", newM.Combat.SelectedDieIndex)
	}

	// Negative index should be rejected
	msg2 := DieSelected{UnitID: "player_cmd", DieIndex: -1}
	newM2, _ := m.Update(msg2)

	if newM2.Combat.SelectedDieIndex != -1 {
		t.Errorf("SelectedDieIndex = %d, want -1 (negative index should be rejected)", newM2.Combat.SelectedDieIndex)
	}

	// Valid index should be accepted
	msg3 := DieSelected{UnitID: "player_cmd", DieIndex: 0}
	newM3, _ := m.Update(msg3)

	if newM3.Combat.SelectedDieIndex != 0 {
		t.Errorf("SelectedDieIndex = %d, want 0", newM3.Combat.SelectedDieIndex)
	}
}

func TestHandlePreviewDone(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
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

func TestDieLockToggled_RequiresCombatActive(t *testing.T) {
	playerCmd := entity.Unit{
		ID:   "player_cmd",
		Tags: []core.Tag{"command"},
		Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerCmd},
			Phase:       model.CombatPaused, // PAUSED
			DicePhase:   model.DicePhasePlayerCommand,
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0, Locked: false}},
			},
		},
	}

	msg := DieLockToggled{UnitID: "player_cmd", DieIndex: 0}
	newM, _ := m.Update(msg)

	// Should NOT toggle - combat is paused
	if newM.Combat.RolledDice["player_cmd"][0].Locked {
		t.Error("die should NOT be locked when combat is paused")
	}
}

func TestDieLockToggled_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseChoice, // Wrong phase
		Combat: model.CombatModel{
			DicePhase: model.DicePhasePlayerCommand,
		},
	}

	msg := DieLockToggled{UnitID: "player_cmd", DieIndex: 0}
	newM, _ := m.Update(msg)

	// Should be no-op - not in combat phase
	if newM.Combat.RolledDice != nil {
		t.Error("should not modify dice when not in PhaseCombat")
	}
}

func TestDieDeselected_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseChoice, // Wrong phase
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			SelectedUnitID:   "player_cmd",
			SelectedDieIndex: 1,
		},
	}

	newM, _ := m.Update(DieDeselected{})

	// Should be no-op - not in combat phase
	if newM.Combat.SelectedDieIndex != 1 {
		t.Error("should not deselect when not in PhaseCombat")
	}
}

func TestDieDeselected_RequiresCombatActive(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			Phase:            model.CombatPaused, // Paused
			DicePhase:        model.DicePhasePlayerCommand,
			SelectedUnitID:   "player_cmd",
			SelectedDieIndex: 1,
		},
	}

	newM, _ := m.Update(DieDeselected{})

	// Should be no-op - combat is paused
	if newM.Combat.SelectedDieIndex != 1 {
		t.Error("should not deselect when combat is paused")
	}
}

func TestPreviewDone_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseChoice, // Wrong phase
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhasePreview,
		},
	}

	newM, _ := m.Update(PreviewDone{})

	// Should be no-op - not in combat phase
	if newM.Combat.DicePhase != model.DicePhasePreview {
		t.Error("should not advance from preview when not in PhaseCombat")
	}
}

func TestPreviewDone_RequiresCombatActive(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatPaused, // Paused
			DicePhase: model.DicePhasePreview,
		},
	}

	newM, _ := m.Update(PreviewDone{})

	// Should be no-op - combat is paused
	if newM.Combat.DicePhase != model.DicePhasePreview {
		t.Error("should not advance from preview when combat is paused")
	}
}

func TestUnlockAllDice_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseChoice, // Wrong phase
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			RerollsRemaining: 1,
		},
	}

	newM, _ := m.Update(UnlockAllDice{})

	// Should be no-op - not in combat phase
	if newM.Combat.RerollsRemaining != 1 {
		t.Error("should not process unlock when not in PhaseCombat")
	}
}

// ===== F-191: Zero-Dice Unit Handling Tests =====

func TestRoundStarted_ZeroDiceUnit(t *testing.T) {
	// Setup: Create combat with a unit that has empty Dice slice
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Seed:    42,
		Combat: model.CombatModel{
			Phase: model.CombatActive,
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				{
					ID:       "zero_dice_unit",
					Position: 0,
					Dice:     []entity.Die{}, // Empty dice slice
					Attributes: map[string]core.Attribute{
						"health": {Base: 50},
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
		},
	}

	// Verify: RollAllDice doesn't panic, unit is skipped gracefully
	msg := RoundStarted{
		Round:     1,
		UnitRolls: map[string][]int{"player_cmd": {0}, "enemy_cmd": {0}}, // No rolls for zero_dice_unit
	}

	newM, _ := m.Update(msg)

	// Zero-dice unit should have no rolled dice
	if rolled, ok := newM.Combat.RolledDice["zero_dice_unit"]; ok && len(rolled) > 0 {
		t.Errorf("zero_dice_unit should have no rolled dice, got %d", len(rolled))
	}

	// Other units should have rolled dice
	if len(newM.Combat.RolledDice["player_cmd"]) != 1 {
		t.Errorf("player_cmd should have 1 rolled die, got %d", len(newM.Combat.RolledDice["player_cmd"]))
	}
}

func TestEnemyCommand_ZeroDiceEnemyCmd(t *testing.T) {
	// Setup: Enemy command unit with no dice
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseEnemyCommand,
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Dice:     []entity.Die{}, // No dice
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
			RolledDice: map[string][]entity.RolledDie{}, // No rolled dice for enemy
		},
	}

	// EnemyCommandResolved with empty actions should work fine
	msg := EnemyCommandResolved{Actions: []EnemyDiceAction{}}

	newM, cmd := m.Update(msg)

	// Should advance to execution phase without crashing
	if newM.Combat.DicePhase != model.DicePhaseExecution {
		t.Errorf("DicePhase = %v, want Execution", newM.Combat.DicePhase)
	}
	if cmd == nil {
		t.Error("expected cmd to advance to execution")
	}
}
