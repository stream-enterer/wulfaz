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
				Die: entity.Die{Faces: []entity.DieFace{
					{Type: entity.DieDamage, Value: 2},
					{Type: entity.DieDamage, Value: 2},
					{Type: entity.DieDamage, Value: 3},
					{Type: entity.DieDamage, Value: 4},
					{Type: entity.DieBlank, Value: 0},
					{Type: entity.DieBlank, Value: 0},
				}},
				HasDie: true,
			}},
		},
	}

	msg := RoundStarted{
		Round:     1,
		UnitRolls: map[string]int{"unit1": 2}, // face index 2 = value 3
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
	if rolled.Value() != 3 {
		t.Errorf("rolled.Value() = %d, want 3", rolled.Value())
	}
	if rolled.FaceIndex != 2 {
		t.Errorf("rolled.FaceIndex = %d, want 2", rolled.FaceIndex)
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
		ID:     "player_cmd",
		Tags:   []core.Tag{"command"},
		Die:    entity.Die{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}},
		HasDie: true,
	}

	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			PlayerUnits:    []entity.Unit{playerCmd},
			EnemyUnits:     []entity.Unit{{ID: "enemy"}},
			DicePhase:      model.DicePhasePlayerCommand,
			SelectedUnitID: "player_cmd",
			RolledDice: map[string]entity.RolledDie{
				"player_cmd": {Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0},
			},
			ActivatedDice: map[string]bool{"player_cmd": false},
		},
	}

	// Damage die targeting friendly = invalid, should be no-op
	msg := DiceActivated{
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
		ID:     "player_cmd",
		Tags:   []core.Tag{"command"},
		Die:    entity.Die{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}},
		HasDie: true,
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerCmd},
			Phase:       model.CombatActive,
			DicePhase:   model.DicePhasePlayerCommand,
			RolledDice: map[string]entity.RolledDie{
				"player_cmd": {Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0, Locked: false},
			},
		},
	}

	msg := DieLockToggled{UnitID: "player_cmd"}
	newM, _ := m.Update(msg)

	if !newM.Combat.RolledDice["player_cmd"].Locked {
		t.Error("die should be locked after toggle")
	}

	// Toggle again
	newM2, _ := newM.Update(msg)
	if newM2.Combat.RolledDice["player_cmd"].Locked {
		t.Error("die should be unlocked after second toggle")
	}
}

func TestHandleDieSelected(t *testing.T) {
	playerCmd := entity.Unit{
		ID:     "player_cmd",
		Tags:   []core.Tag{"command"},
		Die:    entity.Die{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}},
		HasDie: true,
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits:    []entity.Unit{playerCmd},
			Phase:          model.CombatActive,
			DicePhase:      model.DicePhasePlayerCommand,
			SelectedUnitID: "",
			RolledDice: map[string]entity.RolledDie{
				"player_cmd": {Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0},
			},
			ActivatedDice: map[string]bool{"player_cmd": false},
		},
	}

	// Non-existent unit should be rejected
	msg := DieSelected{UnitID: "nonexistent"}
	newM, _ := m.Update(msg)

	if newM.Combat.SelectedUnitID != "" {
		t.Errorf("SelectedUnitID = %q, want empty (invalid unit should be rejected)", newM.Combat.SelectedUnitID)
	}

	// Valid unit should be accepted
	msg2 := DieSelected{UnitID: "player_cmd"}
	newM2, _ := m.Update(msg2)

	if newM2.Combat.SelectedUnitID != "player_cmd" {
		t.Errorf("SelectedUnitID = %q, want player_cmd", newM2.Combat.SelectedUnitID)
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
		ID:     "player_cmd",
		Tags:   []core.Tag{"command"},
		Die:    entity.Die{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}},
		HasDie: true,
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerCmd},
			Phase:       model.CombatPaused, // PAUSED
			DicePhase:   model.DicePhasePlayerCommand,
			RolledDice: map[string]entity.RolledDie{
				"player_cmd": {Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0, Locked: false},
			},
		},
	}

	msg := DieLockToggled{UnitID: "player_cmd"}
	newM, _ := m.Update(msg)

	// Should NOT toggle - combat is paused
	if newM.Combat.RolledDice["player_cmd"].Locked {
		t.Error("die should NOT be locked when combat is paused")
	}
}

func TestDieLockToggled_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseInterCombat, // Wrong phase
		Combat: model.CombatModel{
			DicePhase: model.DicePhasePlayerCommand,
		},
	}

	msg := DieLockToggled{UnitID: "player_cmd"}
	newM, _ := m.Update(msg)

	// Should be no-op - not in combat phase
	if newM.Combat.RolledDice != nil {
		t.Error("should not modify dice when not in PhaseCombat")
	}
}

func TestDieDeselected_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseInterCombat, // Wrong phase
		Combat: model.CombatModel{
			Phase:          model.CombatActive,
			DicePhase:      model.DicePhasePlayerCommand,
			SelectedUnitID: "player_cmd",
		},
	}

	newM, _ := m.Update(DieDeselected{})

	// Should be no-op - not in combat phase
	if newM.Combat.SelectedUnitID != "player_cmd" {
		t.Error("should not deselect when not in PhaseCombat")
	}
}

func TestDieDeselected_RequiresCombatActive(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			Phase:          model.CombatPaused, // Paused
			DicePhase:      model.DicePhasePlayerCommand,
			SelectedUnitID: "player_cmd",
		},
	}

	newM, _ := m.Update(DieDeselected{})

	// Should be no-op - combat is paused
	if newM.Combat.SelectedUnitID != "player_cmd" {
		t.Error("should not deselect when combat is paused")
	}
}

func TestPreviewDone_RequiresPhaseCombat(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseInterCombat, // Wrong phase
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
		Phase:   PhaseInterCombat, // Wrong phase
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

func TestRoundStarted_NoDiceUnit(t *testing.T) {
	// Setup: Create combat with a unit that has HasDie=false
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
					Die:      entity.Die{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}},
					HasDie:   true,
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				{
					ID:       "no_dice_unit",
					Position: 0,
					HasDie:   false, // No die
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
					Die:      entity.Die{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}},
					HasDie:   true,
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
		},
	}

	// Verify: RoundStarted doesn't panic, unit is skipped gracefully
	msg := RoundStarted{
		Round:     1,
		UnitRolls: map[string]int{"player_cmd": 0, "enemy_cmd": 0}, // No rolls for no_dice_unit
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
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			Phase:       model.CombatActive,
			DicePhase:   model.DicePhasePlayerCommand,
			PlayerUnits: []entity.Unit{playerCmd},
			EnemyUnits:  []entity.Unit{enemyCmd},
		},
	}

	// Dice effect kills enemy command (health -> 0)
	msg := DiceEffectApplied{
		SourceUnitID: "player_cmd",
		TargetUnitID: "enemy_cmd",
		Effect:       entity.DieDamage,
		Value:        10,
		NewHealth:    0,
		NewShields:   0,
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
	if _, ok := result.(CombatEnded); !ok {
		t.Errorf("Cmd returned %T, want CombatEnded", result)
	}
}
