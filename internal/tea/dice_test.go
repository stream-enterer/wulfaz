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

	msg := model.RoundStarted{
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

	rolled, exists := newM.Combat.RolledDice["unit1"]
	if !exists {
		t.Fatal("expected rolled die for unit1")
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

	msg := model.UnitDiceEffectsApplied{
		SourceUnitID: "player_cmd",
		Results: []model.DiceEffectResult{{
			TargetUnitID: "enemy",
			Effect:       entity.DieDamage,
			Value:        12,
			NewHealth:    46, // 12 damage - 8 shields = 4 to health, 50-4=46
			NewShields:   0,
		}},
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
			PlayerUnits:    []entity.Unit{playerCmd},
			EnemyUnits:     []entity.Unit{{ID: "enemy"}},
			DicePhase:      model.DicePhasePlayerCommand,
			SelectedUnitID: "player_cmd",
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string]bool{"player_cmd": false},
		},
	}

	// Damage die targeting friendly = invalid, should be no-op
	msg := model.DiceActivated{
		SourceUnitID: "player_cmd",
		TargetUnitID: "player_cmd", // friendly target
	}

	newM, cmd := m.Update(msg)

	// Should not activate (invalid target)
	if newM.Combat.ActivatedDice["player_cmd"] {
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
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerCmd},
			Phase:       model.CombatActive,
			DicePhase:   model.DicePhasePlayerCommand,
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0, Locked: false}},
			},
		},
	}

	msg := model.DieLockToggled{UnitID: "player_cmd"}
	newM, _ := m.Update(msg)

	if !entity.IsUnitLocked(newM.Combat.RolledDice["player_cmd"]) {
		t.Error("die should be locked after toggle")
	}

	// Toggle again
	newM2, _ := newM.Update(msg)
	if entity.IsUnitLocked(newM2.Combat.RolledDice["player_cmd"]) {
		t.Error("die should be unlocked after second toggle")
	}
}

func TestHandleDieSelected(t *testing.T) {
	playerCmd := entity.Unit{
		ID:   "player_cmd",
		Tags: []core.Tag{"command"},
		Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
	}

	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits:    []entity.Unit{playerCmd},
			Phase:          model.CombatActive,
			DicePhase:      model.DicePhasePlayerCommand,
			SelectedUnitID: "",
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string]bool{"player_cmd": false},
		},
	}

	// Non-existent unit should be rejected
	msg := model.DieSelected{UnitID: "nonexistent"}
	newM, _ := m.Update(msg)

	if newM.Combat.SelectedUnitID != "" {
		t.Errorf("SelectedUnitID = %q, want empty (invalid unit should be rejected)", newM.Combat.SelectedUnitID)
	}

	// Valid unit should be accepted
	msg2 := model.DieSelected{UnitID: "player_cmd"}
	newM2, _ := m.Update(msg2)

	if newM2.Combat.SelectedUnitID != "player_cmd" {
		t.Errorf("SelectedUnitID = %q, want player_cmd", newM2.Combat.SelectedUnitID)
	}
}

func TestHandlePreviewDone(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhasePreview,
		},
	}

	newM, _ := m.Update(model.PreviewDone{})

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

	newM, cmd := m.Update(model.PlayerCommandDone{})

	if newM.Combat.DicePhase != model.DicePhaseExecution {
		t.Errorf("DicePhase = %v, want Execution", newM.Combat.DicePhase)
	}
	// In the new system, we wait for player click to execute attacks
	// So no cmd is returned - the UI shows the Execution phase and waits
	if cmd != nil {
		t.Error("expected nil cmd (waits for player click to execute)")
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
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerCmd},
			Phase:       model.CombatPaused, // PAUSED
			DicePhase:   model.DicePhasePlayerCommand,
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0, Locked: false}},
			},
		},
	}

	msg := model.DieLockToggled{UnitID: "player_cmd"}
	newM, _ := m.Update(msg)

	// Should NOT toggle - combat is paused
	if entity.IsUnitLocked(newM.Combat.RolledDice["player_cmd"]) {
		t.Error("die should NOT be locked when combat is paused")
	}
}

func TestDieLockToggled_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseInterCombat, // Wrong phase
		Combat: model.CombatModel{
			DicePhase: model.DicePhasePlayerCommand,
		},
	}

	msg := model.DieLockToggled{UnitID: "player_cmd"}
	newM, _ := m.Update(msg)

	// Should be no-op - not in combat phase
	if newM.Combat.RolledDice != nil {
		t.Error("should not modify dice when not in PhaseCombat")
	}
}

func TestDieDeselected_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseInterCombat, // Wrong phase
		Combat: model.CombatModel{
			Phase:          model.CombatActive,
			DicePhase:      model.DicePhasePlayerCommand,
			SelectedUnitID: "player_cmd",
		},
	}

	newM, _ := m.Update(model.DieDeselected{})

	// Should be no-op - not in combat phase
	if newM.Combat.SelectedUnitID != "player_cmd" {
		t.Error("should not deselect when not in PhaseCombat")
	}
}

func TestDieDeselected_RequiresCombatActive(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:          model.CombatPaused, // Paused
			DicePhase:      model.DicePhasePlayerCommand,
			SelectedUnitID: "player_cmd",
		},
	}

	newM, _ := m.Update(model.DieDeselected{})

	// Should be no-op - combat is paused
	if newM.Combat.SelectedUnitID != "player_cmd" {
		t.Error("should not deselect when combat is paused")
	}
}

func TestPreviewDone_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseInterCombat, // Wrong phase
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhasePreview,
		},
	}

	newM, _ := m.Update(model.PreviewDone{})

	// Should be no-op - not in combat phase
	if newM.Combat.DicePhase != model.DicePhasePreview {
		t.Error("should not advance from preview when not in PhaseCombat")
	}
}

func TestPreviewDone_RequiresCombatActive(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatPaused, // Paused
			DicePhase: model.DicePhasePreview,
		},
	}

	newM, _ := m.Update(model.PreviewDone{})

	// Should be no-op - combat is paused
	if newM.Combat.DicePhase != model.DicePhasePreview {
		t.Error("should not advance from preview when combat is paused")
	}
}

func TestUndoRequested_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseInterCombat, // Wrong phase
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			RerollsRemaining: 1,
			UndoStack: []model.UndoSnapshot{
				{RerollsRemaining: 2}, // Initial
				{RerollsRemaining: 1}, // After action
			},
		},
	}

	newM, _ := m.Update(model.UndoRequested{})

	// Should be no-op - not in combat phase
	if len(newM.Combat.UndoStack) != 2 {
		t.Error("should not process undo when not in PhaseCombat")
	}
}

func TestUndoRequested_RequiresMultipleSnapshots(t *testing.T) {
	// With only the initial snapshot, undo should be no-op
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			RerollsRemaining: 2,
			PlayerUnits: []entity.Unit{
				{ID: "p1", Position: 0},
			},
			RolledDice: map[string][]entity.RolledDie{
				"p1": {{Locked: true, Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string]bool{"p1": false},
			UndoStack: []model.UndoSnapshot{
				{RerollsRemaining: 2},
			},
		},
	}

	newM, _ := m.Update(model.UndoRequested{})

	// Should be no-op - only 1 snapshot
	if !entity.IsUnitLocked(newM.Combat.RolledDice["p1"]) {
		t.Error("undo with single snapshot should be no-op")
	}
	// Stack should remain unchanged
	if len(newM.Combat.UndoStack) != 1 {
		t.Errorf("undo stack should remain unchanged, got %d", len(newM.Combat.UndoStack))
	}
}

func TestUnlockAllDiceRequested_UnlocksAllDice(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			RerollsRemaining: 2,
			PlayerUnits: []entity.Unit{
				{ID: "p1", Position: 0, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}}}, Attributes: map[string]core.Attribute{"health": {Base: 10}}},
				{ID: "p2", Position: 1, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieShield, Value: 2}}}}, Attributes: map[string]core.Attribute{"health": {Base: 10}}},
			},
			RolledDice: map[string][]entity.RolledDie{
				"p1": {{Locked: true, Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}, FaceIndex: 0}},
				"p2": {{Locked: true, Faces: []entity.DieFace{{Type: entity.DieShield, Value: 2}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string]bool{"p1": false, "p2": false},
			UndoStack:     []model.UndoSnapshot{{RerollsRemaining: 2}}, // Initial snapshot
		},
	}

	newM, _ := m.Update(model.UnlockAllDiceRequested{})

	if entity.IsUnitLocked(newM.Combat.RolledDice["p1"]) {
		t.Error("p1 die should be unlocked")
	}
	if entity.IsUnitLocked(newM.Combat.RolledDice["p2"]) {
		t.Error("p2 die should be unlocked")
	}
	// Undo stack should be unchanged
	if len(newM.Combat.UndoStack) != 1 {
		t.Errorf("undo stack should remain unchanged, got %d", len(newM.Combat.UndoStack))
	}
}

func TestUnlockAllDiceRequested_SkipsActivatedDice(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			RerollsRemaining: 2,
			PlayerUnits: []entity.Unit{
				{ID: "p1", Position: 0, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}}}, Attributes: map[string]core.Attribute{"health": {Base: 10}}},
				{ID: "p2", Position: 1, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieShield, Value: 2}}}}, Attributes: map[string]core.Attribute{"health": {Base: 10}}},
			},
			RolledDice: map[string][]entity.RolledDie{
				"p1": {{Locked: true, Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}, FaceIndex: 0}},
				"p2": {{Locked: true, Faces: []entity.DieFace{{Type: entity.DieShield, Value: 2}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string]bool{"p1": true, "p2": false}, // p1 already activated
		},
	}

	newM, _ := m.Update(model.UnlockAllDiceRequested{})

	// p1 should remain locked (activated)
	if !entity.IsUnitLocked(newM.Combat.RolledDice["p1"]) {
		t.Error("activated die p1 should remain locked")
	}
	// p2 should be unlocked (not activated)
	if entity.IsUnitLocked(newM.Combat.RolledDice["p2"]) {
		t.Error("p2 die should be unlocked")
	}
}

func TestUnlockAllDiceRequested_RequiresRerolls(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			RerollsRemaining: 0, // No rerolls
			PlayerUnits: []entity.Unit{
				{ID: "p1", Position: 0, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}}}, Attributes: map[string]core.Attribute{"health": {Base: 10}}},
			},
			RolledDice: map[string][]entity.RolledDie{
				"p1": {{Locked: true, Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string]bool{"p1": false},
		},
	}

	newM, _ := m.Update(model.UnlockAllDiceRequested{})

	// Should be no-op - no rerolls
	if !entity.IsUnitLocked(newM.Combat.RolledDice["p1"]) {
		t.Error("should not unlock when no rerolls remaining")
	}
}

func TestUnlockAllDiceRequested_ClearsSelection(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			RerollsRemaining: 2,
			SelectedUnitID:   "p1", // Has selection
			PlayerUnits: []entity.Unit{
				{ID: "p1", Position: 0, Attributes: map[string]core.Attribute{"health": {Base: 10}}},
			},
			RolledDice: map[string][]entity.RolledDie{
				"p1": {{Locked: true, Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string]bool{"p1": false},
		},
	}

	newM, _ := m.Update(model.UnlockAllDiceRequested{})

	if newM.Combat.SelectedUnitID != "" {
		t.Error("unlock-all should clear selection")
	}
}

func TestDieUnlocked_RequiresRerolls(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			RerollsRemaining: 0, // No rerolls
			PlayerUnits: []entity.Unit{
				{ID: "p1", Position: 0},
			},
			RolledDice: map[string][]entity.RolledDie{
				"p1": {{Locked: true, Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}, FaceIndex: 0}},
			},
			ActivatedDice: map[string]bool{"p1": false},
		},
	}

	newM, _ := m.Update(model.DieUnlocked{UnitID: "p1"})

	// Should be no-op - no rerolls remaining
	if !entity.IsUnitLocked(newM.Combat.RolledDice["p1"]) {
		t.Error("should not unlock die when no rerolls remaining")
	}
}

// ===== F-191: Zero-Dice Unit Handling Tests =====

func TestRoundStarted_NoDiceUnit(t *testing.T) {
	// Setup: Create combat with a unit that has HasDie=false
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
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
					ID:       "no_dice_unit",
					Position: 0,
					// No Dice field - unit has no dice
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

	// Verify: RoundStarted doesn't panic, unit is skipped gracefully
	msg := model.RoundStarted{
		Round:     1,
		UnitRolls: map[string][]int{"player_cmd": {0}, "enemy_cmd": {0}}, // No rolls for no_dice_unit
	}

	newM, _ := m.Update(msg)

	// No-dice unit should have no rolled dice
	if _, ok := newM.Combat.RolledDice["no_dice_unit"]; ok {
		t.Error("no_dice_unit should have no rolled dice entry")
	}

	// Other units should have rolled dice
	if _, ok := newM.Combat.RolledDice["player_cmd"]; !ok {
		t.Error("player_cmd should have rolled die")
	}
}

// ===== F-167: Command Unit Targeting Tests =====

func TestF167_CommandUnitProtection(t *testing.T) {
	// When regular enemies are alive, command cannot be targeted
	enemies := []entity.Unit{
		{ID: "enemy_cmd", Tags: []core.Tag{"command"}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
		{ID: "enemy_regular", Attributes: map[string]core.Attribute{"health": {Base: 50}}},
	}

	validTargets := GetValidEnemyTargets(enemies)

	// Should only include regular enemy, not command
	if len(validTargets) != 1 {
		t.Fatalf("expected 1 valid target, got %d", len(validTargets))
	}
	if validTargets[0].ID != "enemy_regular" {
		t.Errorf("expected enemy_regular, got %s", validTargets[0].ID)
	}
}

func TestF167_CommandTargetableWhenAlone(t *testing.T) {
	// When all regular enemies are dead, command can be targeted
	enemies := []entity.Unit{
		{ID: "enemy_cmd", Tags: []core.Tag{"command"}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
		{ID: "enemy_dead", Attributes: map[string]core.Attribute{"health": {Base: 0}}}, // Dead
	}

	validTargets := GetValidEnemyTargets(enemies)

	// Should include command since all regular enemies are dead
	if len(validTargets) != 1 {
		t.Fatalf("expected 1 valid target, got %d", len(validTargets))
	}
	if validTargets[0].ID != "enemy_cmd" {
		t.Errorf("expected enemy_cmd, got %s", validTargets[0].ID)
	}
}

func TestHandleDiceEffectApplied_KillsEnemyCommand_EndsCombat(t *testing.T) {
	enemyCmd := entity.Unit{
		ID:   "enemy_cmd",
		Tags: []core.Tag{"command"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 10},
		},
	}
	playerCmd := entity.Unit{
		ID:   "player_cmd",
		Tags: []core.Tag{"command"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 50},
		},
	}

	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:       model.CombatActive,
			DicePhase:   model.DicePhasePlayerCommand,
			PlayerUnits: []entity.Unit{playerCmd},
			EnemyUnits:  []entity.Unit{enemyCmd},
		},
	}

	// Dice effect kills enemy command (health -> 0)
	msg := model.UnitDiceEffectsApplied{
		SourceUnitID: "player_cmd",
		Results: []model.DiceEffectResult{{
			TargetUnitID: "enemy_cmd",
			Effect:       entity.DieDamage,
			Value:        10,
			NewHealth:    0,
			NewShields:   0,
		}},
	}

	newM, cmd := m.Update(msg)

	// Combat should be resolved with player victory
	if newM.Combat.Phase != model.CombatResolved {
		t.Errorf("Combat.Phase = %v, want CombatResolved", newM.Combat.Phase)
	}
	if newM.Combat.Victor != "player" {
		t.Errorf("Combat.Victor = %q, want %q", newM.Combat.Victor, "player")
	}
	// Should return CombatEnded command
	if cmd == nil {
		t.Fatal("expected non-nil Cmd")
	}
	result := cmd()
	if _, ok := result.(model.CombatEnded); !ok {
		t.Errorf("Cmd returned %T, want CombatEnded", result)
	}
}

func TestHandlePreviewDone_AllBlankDice(t *testing.T) {
	playerCmd := entity.Unit{
		ID:   "player_cmd",
		Dice: []entity.Die{{}},
	}
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:       model.CombatActive,
			DicePhase:   model.DicePhasePreview,
			PlayerUnits: []entity.Unit{playerCmd},
			RolledDice: map[string][]entity.RolledDie{
				// Empty Faces slice makes CurrentFace() return DieBlank
				"player_cmd": {{Faces: nil, FaceIndex: 0}},
			},
		},
	}

	_, cmd := m.Update(model.PreviewDone{})

	// Should return PlayerCommandDone cmd to auto-advance
	if cmd == nil {
		t.Fatal("expected cmd to auto-advance, got nil")
	}
	if _, ok := cmd().(model.PlayerCommandDone); !ok {
		t.Error("expected PlayerCommandDone msg")
	}
}

func TestHandleRerollRequested_AllBlankAfterReroll(t *testing.T) {
	// Setup: 1 reroll remaining, will become 0 after this reroll
	// Die has only blank face, so reroll result is blank
	blankFaces := []entity.DieFace{{Type: entity.DieBlank}}
	playerCmd := entity.Unit{
		ID:   "player_cmd",
		Dice: []entity.Die{{}},
	}
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:            model.CombatActive,
			DicePhase:        model.DicePhasePlayerCommand,
			PlayerUnits:      []entity.Unit{playerCmd},
			RerollsRemaining: 1,
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {{Faces: blankFaces, FaceIndex: 0, Locked: false}},
			},
		},
	}

	_, cmd := m.Update(model.RerollRequested{Results: map[string][]int{"player_cmd": {0}}})

	// Should return PlayerCommandDone cmd to auto-advance
	if cmd == nil {
		t.Fatal("expected cmd to auto-advance, got nil")
	}
	if _, ok := cmd().(model.PlayerCommandDone); !ok {
		t.Error("expected PlayerCommandDone msg")
	}
}

// ===== Split-Activation Tests =====

func TestHandleDiceActivated_SplitActivation_DamageThenHeal(t *testing.T) {
	// Mixed-dice unit: 1 damage + 1 shield. First click enemy fires damage, second click ally fires shield.
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhasePlayerCommand,
			PlayerUnits: []entity.Unit{
				{ID: "player_cmd", Position: -1, Tags: []core.Tag{"command"}, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
				{ID: "p1", Position: 0, Dice: []entity.Die{
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 3}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}},
					{Faces: []entity.DieFace{{Type: entity.DieShield, Value: 3}, {Type: entity.DieShield, Value: 2}, {Type: entity.DieShield, Value: 4}, {Type: entity.DieShield, Value: 3}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}},
				}, Attributes: map[string]core.Attribute{"health": {Base: 50}}},
			},
			EnemyUnits: []entity.Unit{
				{ID: "enemy_cmd", Position: -1, Tags: []core.Tag{"command"}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
				{ID: "enemy1", Position: 0, Attributes: map[string]core.Attribute{"health": {Base: 50}}},
			},
			SelectedUnitID: "p1",
			RolledDice: map[string][]entity.RolledDie{
				"p1": {
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 3}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}, FaceIndex: 0, Locked: true, Fired: false},
					{Faces: []entity.DieFace{{Type: entity.DieShield, Value: 3}, {Type: entity.DieShield, Value: 2}, {Type: entity.DieShield, Value: 4}, {Type: entity.DieShield, Value: 3}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}, FaceIndex: 0, Locked: true, Fired: false},
				},
			},
			ActivatedDice: map[string]bool{"p1": false},
			PlayerTargets: map[string]string{},
		},
	}

	// Click 1: target enemy -> damage die fires
	msg1 := model.DiceActivated{SourceUnitID: "p1", TargetUnitID: "enemy1", Timestamp: 1000}
	m1, cmd1 := m.Update(msg1)

	// Damage die should be fired, shield die should not
	dice1 := m1.Combat.RolledDice["p1"]
	if !dice1[0].Fired {
		t.Error("damage die should be Fired after targeting enemy")
	}
	if dice1[1].Fired {
		t.Error("shield die should NOT be Fired after targeting enemy")
	}

	// Should NOT be fully activated yet
	if m1.Combat.ActivatedDice["p1"] {
		t.Error("should not be fully activated after first click")
	}

	// Should have returned a Cmd (effect application)
	if cmd1 == nil {
		t.Fatal("expected effect cmd after first activation")
	}

	// Click 2: target ally -> shield die fires
	msg2 := model.DiceActivated{SourceUnitID: "p1", TargetUnitID: "player_cmd", Timestamp: 2000}
	m2, cmd2 := m1.Update(msg2)

	dice2 := m2.Combat.RolledDice["p1"]
	if !dice2[0].Fired {
		t.Error("damage die should still be Fired")
	}
	if !dice2[1].Fired {
		t.Error("shield die should be Fired after targeting ally")
	}

	// NOW should be fully activated
	if !m2.Combat.ActivatedDice["p1"] {
		t.Error("should be fully activated after second click")
	}

	if cmd2 == nil {
		t.Fatal("expected effect cmd after second activation")
	}
}

func TestHandleDiceActivated_SplitActivation_UnitStaysSelected(t *testing.T) {
	// After first target click on mixed-dice unit, SelectedUnitID should persist.
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhasePlayerCommand,
			PlayerUnits: []entity.Unit{
				{ID: "player_cmd", Position: -1, Tags: []core.Tag{"command"}, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
				{ID: "p1", Position: 0, Dice: []entity.Die{
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 3}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}},
					{Faces: []entity.DieFace{{Type: entity.DieShield, Value: 3}, {Type: entity.DieShield, Value: 2}, {Type: entity.DieShield, Value: 4}, {Type: entity.DieShield, Value: 3}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}},
				}, Attributes: map[string]core.Attribute{"health": {Base: 50}}},
			},
			EnemyUnits: []entity.Unit{
				{ID: "enemy1", Position: 0, Attributes: map[string]core.Attribute{"health": {Base: 50}}},
			},
			SelectedUnitID: "p1",
			RolledDice: map[string][]entity.RolledDie{
				"p1": {
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 3}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}, FaceIndex: 0, Locked: true, Fired: false},
					{Faces: []entity.DieFace{{Type: entity.DieShield, Value: 3}, {Type: entity.DieShield, Value: 2}, {Type: entity.DieShield, Value: 4}, {Type: entity.DieShield, Value: 3}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}, FaceIndex: 0, Locked: true, Fired: false},
				},
			},
			ActivatedDice: map[string]bool{"p1": false},
			PlayerTargets: map[string]string{},
		},
	}

	// Click enemy -> fires damage die only
	m1, _ := m.Update(model.DiceActivated{SourceUnitID: "p1", TargetUnitID: "enemy1", Timestamp: 1000})

	if m1.Combat.SelectedUnitID != "p1" {
		t.Errorf("SelectedUnitID = %q, want p1 (unit should stay selected for second target)", m1.Combat.SelectedUnitID)
	}
}

func TestHandleDiceActivated_SplitActivation_FullyDoneAfterBothTargets(t *testing.T) {
	// After both target clicks, ActivatedDice should be true and SelectedUnitID cleared.
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhasePlayerCommand,
			PlayerUnits: []entity.Unit{
				{ID: "player_cmd", Position: -1, Tags: []core.Tag{"command"}, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
				{ID: "p1", Position: 0, Dice: []entity.Die{
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 3}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}},
					{Faces: []entity.DieFace{{Type: entity.DieShield, Value: 3}, {Type: entity.DieShield, Value: 2}, {Type: entity.DieShield, Value: 4}, {Type: entity.DieShield, Value: 3}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}},
				}, Attributes: map[string]core.Attribute{"health": {Base: 50}}},
			},
			EnemyUnits: []entity.Unit{
				{ID: "enemy1", Position: 0, Attributes: map[string]core.Attribute{"health": {Base: 50}}},
			},
			SelectedUnitID: "p1",
			RolledDice: map[string][]entity.RolledDie{
				"p1": {
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 3}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}, FaceIndex: 0, Locked: true, Fired: false},
					{Faces: []entity.DieFace{{Type: entity.DieShield, Value: 3}, {Type: entity.DieShield, Value: 2}, {Type: entity.DieShield, Value: 4}, {Type: entity.DieShield, Value: 3}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}, FaceIndex: 0, Locked: true, Fired: false},
				},
			},
			ActivatedDice: map[string]bool{"p1": false},
			PlayerTargets: map[string]string{},
		},
	}

	// Click 1: enemy
	m1, _ := m.Update(model.DiceActivated{SourceUnitID: "p1", TargetUnitID: "enemy1", Timestamp: 1000})
	// Click 2: ally
	m2, _ := m1.Update(model.DiceActivated{SourceUnitID: "p1", TargetUnitID: "player_cmd", Timestamp: 2000})

	if !m2.Combat.ActivatedDice["p1"] {
		t.Error("ActivatedDice should be true after both targets clicked")
	}
	if m2.Combat.SelectedUnitID != "" {
		t.Errorf("SelectedUnitID = %q, want empty after full activation", m2.Combat.SelectedUnitID)
	}
}

func TestHandleDiceActivated_SingleType_CompletesInOneClick(t *testing.T) {
	// Damage-only unit should fully activate after one enemy click.
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhasePlayerCommand,
			PlayerUnits: []entity.Unit{
				{ID: "player_cmd", Position: -1, Tags: []core.Tag{"command"}, Dice: []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
			},
			EnemyUnits: []entity.Unit{
				{ID: "enemy1", Position: 0, Attributes: map[string]core.Attribute{"health": {Base: 50}}},
			},
			SelectedUnitID: "player_cmd",
			RolledDice: map[string][]entity.RolledDie{
				"player_cmd": {
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 3}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}, FaceIndex: 0, Locked: true, Fired: false},
				},
			},
			ActivatedDice: map[string]bool{"player_cmd": false},
			PlayerTargets: map[string]string{},
		},
	}

	m1, cmd := m.Update(model.DiceActivated{SourceUnitID: "player_cmd", TargetUnitID: "enemy1", Timestamp: 1000})

	if !m1.Combat.ActivatedDice["player_cmd"] {
		t.Error("damage-only unit should be fully activated after one click")
	}
	if m1.Combat.SelectedUnitID != "" {
		t.Errorf("SelectedUnitID = %q, want empty", m1.Combat.SelectedUnitID)
	}
	if cmd == nil {
		t.Error("expected effect cmd")
	}
}
