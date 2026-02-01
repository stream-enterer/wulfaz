package tea

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

func TestUpdate_TriggersCollected_ExecutesEffects(t *testing.T) {
	playerUnit := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}

	enemyUnit := entity.Unit{
		ID:   "enemy1",
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 50, Min: 0},
		},
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerUnit},
			EnemyUnits:  []entity.Unit{enemyUnit},
			Phase:       model.CombatActive,
		},
	}

	// Simulate triggers collected
	triggersMsg := TriggersCollected{
		Event: string(core.EventOnDamaged),
		Triggers: []CollectedTrigger{
			{
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 20, "target": "enemy"},
				Owner:      TriggerOwner{UnitID: "player1"},
			},
		},
		Depth: 0,
	}

	_, cmd := m.Update(triggersMsg)

	if cmd == nil {
		t.Fatal("expected non-nil command")
	}

	msg := cmd()
	effectsMsg, ok := msg.(EffectsResolved)
	if !ok {
		t.Fatalf("expected EffectsResolved, got %T", msg)
	}

	// Should have modified enemy1
	if _, ok := effectsMsg.ModifiedUnits["enemy1"]; !ok {
		t.Error("expected enemy1 in modified units")
	}

	// Should have on_damaged follow-up
	if len(effectsMsg.FollowUpEvents) != 1 {
		t.Errorf("expected 1 follow-up event, got %d", len(effectsMsg.FollowUpEvents))
	}

	if effectsMsg.FollowUpEvents[0].Event != string(core.EventOnDamaged) {
		t.Errorf("expected on_damaged, got %s", effectsMsg.FollowUpEvents[0].Event)
	}
}

func TestUpdate_TriggersCollected_CascadeDepthLimit(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Log: []string{},
		},
	}

	triggersMsg := TriggersCollected{
		Triggers: []CollectedTrigger{
			{EffectName: "deal_damage", Params: map[string]any{"damage": 10}},
		},
		Depth: core.MaxCascadeDepth, // At limit
	}

	newModel, cmd := m.Update(triggersMsg)

	// Should add log entry about depth limit
	if len(newModel.Combat.Log) != 1 {
		t.Errorf("expected 1 log entry, got %d", len(newModel.Combat.Log))
	}

	// Should return nil command
	if cmd != nil {
		t.Error("expected nil command at cascade depth limit")
	}
}

func TestUpdate_EffectsResolved_AppliesModifications(t *testing.T) {
	enemyUnit := entity.Unit{
		ID:   "enemy1",
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 50, Min: 0},
		},
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			EnemyUnits: []entity.Unit{enemyUnit},
			Log:        []string{},
		},
	}

	effectsMsg := EffectsResolved{
		ModifiedUnits: ModifiedUnitsMap{
			"enemy1": ModifiedUnit{
				Attributes: map[string]AttributeValue{
					"health": {Base: 30, Min: 0},
				},
			},
		},
		LogEntries: []string{"player1 dealt 20 damage to enemy1"},
		Depth:      0,
	}

	newModel, cmd := m.Update(effectsMsg)

	// Check health was updated
	if newModel.Combat.EnemyUnits[0].Attributes["health"].Base != 30 {
		t.Errorf("expected health 30, got %d", newModel.Combat.EnemyUnits[0].Attributes["health"].Base)
	}

	// Check log was updated
	if len(newModel.Combat.Log) != 1 {
		t.Errorf("expected 1 log entry, got %d", len(newModel.Combat.Log))
	}

	// No follow-ups, should return nil
	if cmd != nil {
		t.Error("expected nil command with no follow-ups")
	}
}

func TestUpdate_EffectsResolved_DispatchesFollowUps(t *testing.T) {
	// Unit with on_damaged trigger
	enemyUnit := entity.Unit{
		ID:   "enemy1",
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 30, Min: 0},
		},
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnDamaged,
				EffectName: "counter_attack",
				Priority:   1,
			},
		},
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			EnemyUnits: []entity.Unit{enemyUnit},
			Log:        []string{},
		},
	}

	effectsMsg := EffectsResolved{
		ModifiedUnits: ModifiedUnitsMap{},
		FollowUpEvents: []FollowUpEvent{
			{Event: string(core.EventOnDamaged), SourceID: "player1", TargetID: "enemy1"},
		},
		LogEntries: []string{},
		Depth:      0,
	}

	_, cmd := m.Update(effectsMsg)

	if cmd == nil {
		t.Fatal("expected non-nil command for follow-up dispatch")
	}

	msg := cmd()
	triggersMsg, ok := msg.(TriggersCollected)
	if !ok {
		t.Fatalf("expected TriggersCollected, got %T", msg)
	}

	// Should have collected the on_damaged trigger
	if len(triggersMsg.Triggers) != 1 {
		t.Errorf("expected 1 trigger, got %d", len(triggersMsg.Triggers))
	}

	// Depth should be incremented
	if triggersMsg.Depth != 1 {
		t.Errorf("expected depth 1, got %d", triggersMsg.Depth)
	}
}

func TestUpdate_PlayerQuit(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
	}

	newModel, cmd := m.Update(PlayerQuit{})

	if newModel.Phase != PhaseGameOver {
		t.Errorf("expected PhaseGameOver, got %d", newModel.Phase)
	}

	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_TriggersCollected_ReEvaluatesSourceConditions(t *testing.T) {
	// Test that source conditions are re-evaluated at execution time
	// A unit that dies mid-batch should not execute its trigger

	playerUnit := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100, Min: 0},
		},
	}

	enemyUnit := entity.Unit{
		ID:   "enemy1",
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 50, Min: 0},
		},
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerUnit},
			EnemyUnits:  []entity.Unit{enemyUnit},
			Phase:       model.CombatActive,
		},
	}

	// Simulate batch where enemy1 attacks first, then tries to attack again
	// with a source condition requiring health >= 1.
	// The second trigger should NOT execute because enemy1's health will be 0.
	triggersMsg := TriggersCollected{
		Event: string(core.EventOnDamaged),
		Triggers: []CollectedTrigger{
			{
				// First: player kills enemy
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 100, "target": "enemy"},
				Owner:      TriggerOwner{UnitID: "player1"},
			},
			{
				// Second: enemy tries to attack, but should be dead
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 50, "target": "self"}, // target self to avoid "no target" issue
				Owner:      TriggerOwner{UnitID: "enemy1"},
				Conditions: []core.Condition{
					{Type: core.ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 1}},
				},
			},
		},
		Depth: 0,
	}

	_, cmd := m.Update(triggersMsg)
	if cmd == nil {
		t.Fatal("expected non-nil command")
	}

	msg := cmd()
	effectsMsg, ok := msg.(EffectsResolved)
	if !ok {
		t.Fatalf("expected EffectsResolved, got %T", msg)
	}

	// enemy1 should be modified (killed by player1)
	if _, ok := effectsMsg.ModifiedUnits["enemy1"]; !ok {
		t.Error("expected enemy1 in modified units")
	}

	// enemy1's health should be 0
	if effectsMsg.ModifiedUnits["enemy1"].Attributes["health"].Base != 0 {
		t.Errorf("expected enemy1 health 0, got %d", effectsMsg.ModifiedUnits["enemy1"].Attributes["health"].Base)
	}

	// Should only have one log entry (player attacking enemy)
	// The dead enemy's trigger should NOT have executed
	if len(effectsMsg.LogEntries) != 1 {
		t.Errorf("expected 1 log entry (dead unit shouldn't attack), got %d: %v",
			len(effectsMsg.LogEntries), effectsMsg.LogEntries)
	}
}

func TestUpdate_CombatEnded_PlayerWins(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
	}

	newModel, cmd := m.Update(CombatEnded{Victor: VictorPlayer})

	if newModel.Phase != PhaseChoice {
		t.Errorf("expected PhaseChoice, got %d", newModel.Phase)
	}
	if newModel.ChoiceType != ChoiceReward {
		t.Errorf("expected ChoiceReward, got %d", newModel.ChoiceType)
	}
	if newModel.RewardChoicesLeft != 2 {
		t.Errorf("expected 2 reward choices left, got %d", newModel.RewardChoicesLeft)
	}
	if len(newModel.Choices) != 3 {
		t.Errorf("expected 3 choices, got %d", len(newModel.Choices))
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_CombatEnded_PlayerLoses(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
	}

	newModel, cmd := m.Update(CombatEnded{Victor: VictorEnemy})

	if newModel.Phase != PhaseGameOver {
		t.Errorf("expected PhaseGameOver, got %d", newModel.Phase)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_CombatEnded_Draw(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
	}

	newModel, cmd := m.Update(CombatEnded{Victor: VictorDraw})

	if newModel.Phase != PhaseGameOver {
		t.Errorf("expected PhaseGameOver, got %d", newModel.Phase)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_ChoiceSelected_Reward(t *testing.T) {
	m := Model{
		Version:           1,
		Phase:             PhaseChoice,
		ChoiceType:        ChoiceReward,
		RewardChoicesLeft: 2,
		Choices:           []string{"A", "B", "C"},
	}

	// First reward selection
	newModel, cmd := m.Update(ChoiceSelected{Index: 0})

	if newModel.RewardChoicesLeft != 1 {
		t.Errorf("expected 1 reward choice left, got %d", newModel.RewardChoicesLeft)
	}
	if newModel.ChoiceType != ChoiceReward {
		t.Errorf("expected ChoiceReward, got %d", newModel.ChoiceType)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}

	// Second reward selection should switch to fight selection
	newModel, cmd = newModel.Update(ChoiceSelected{Index: 1})

	if newModel.RewardChoicesLeft != 0 {
		t.Errorf("expected 0 reward choices left, got %d", newModel.RewardChoicesLeft)
	}
	if newModel.ChoiceType != ChoiceFight {
		t.Errorf("expected ChoiceFight, got %d", newModel.ChoiceType)
	}
	if len(newModel.Choices) != 3 {
		t.Errorf("expected 3 fight choices, got %d", len(newModel.Choices))
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_CombatStarted(t *testing.T) {
	m := Model{
		Version:     1,
		Phase:       PhaseChoice,
		FightNumber: 1,
	}

	combat := model.CombatModel{
		Phase: model.CombatActive,
		Log:   []string{"Fight 2 started"},
	}

	newModel, cmd := m.Update(CombatStarted{Combat: combat})

	if newModel.Phase != PhaseCombat {
		t.Errorf("expected PhaseCombat, got %d", newModel.Phase)
	}
	if newModel.FightNumber != 2 {
		t.Errorf("expected FightNumber 2, got %d", newModel.FightNumber)
	}
	if newModel.Combat.Phase != model.CombatActive {
		t.Errorf("expected combat phase CombatActive")
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}
