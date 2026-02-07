package tea

import (
	"fmt"
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
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerUnit},
			EnemyUnits:  []entity.Unit{enemyUnit},
			Phase:       model.CombatActive,
		},
	}

	// Simulate triggers collected
	triggersMsg := model.TriggersCollected{
		Event: string(core.EventOnDamaged),
		Triggers: []model.CollectedTrigger{
			{
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 20, "target": "enemy"},
				Owner:      model.TriggerOwner{UnitID: "player1"},
			},
		},
		Depth: 0,
	}

	_, cmd := m.Update(triggersMsg)

	if cmd == nil {
		t.Fatal("expected non-nil command")
	}

	msg := cmd()
	effectsMsg, ok := msg.(model.EffectsResolved)
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

	triggersMsg := model.TriggersCollected{
		Triggers: []model.CollectedTrigger{
			{EffectName: "deal_damage", Params: map[string]any{"damage": 10}},
		},
		Depth: core.MaxCascadeDepth, // At limit
	}

	newModel, cmd := m.Update(triggersMsg)

	// Should add log entry about depth limit
	if len(newModel.Combat.Log) != 1 {
		t.Errorf("expected 1 log entry, got %d", len(newModel.Combat.Log))
	}

	// Should return nil command (applyCombatEnd with CombatSetup phase returns nil)
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
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			EnemyUnits: []entity.Unit{enemyUnit},
			Log:        []string{},
		},
	}

	effectsMsg := model.EffectsResolved{
		ModifiedUnits: model.ModifiedUnitsMap{
			"enemy1": model.ModifiedUnit{
				Attributes: map[string]model.AttributeValue{
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
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			EnemyUnits: []entity.Unit{enemyUnit},
			Log:        []string{},
		},
	}

	effectsMsg := model.EffectsResolved{
		ModifiedUnits: model.ModifiedUnitsMap{},
		FollowUpEvents: []model.FollowUpEvent{
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
	triggersMsg, ok := msg.(model.TriggersCollected)
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
		Phase:   model.PhaseCombat,
	}

	newModel, cmd := m.Update(model.PlayerQuit{})

	if newModel.Phase != model.PhaseGameOver {
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
		Phase:   model.PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{playerUnit},
			EnemyUnits:  []entity.Unit{enemyUnit},
			Phase:       model.CombatActive,
		},
	}

	// Simulate batch where enemy1 attacks first, then tries to attack again
	// with a source condition requiring health >= 1.
	// The second trigger should NOT execute because enemy1's health will be 0.
	triggersMsg := model.TriggersCollected{
		Event: string(core.EventOnDamaged),
		Triggers: []model.CollectedTrigger{
			{
				// First: player kills enemy
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 100, "target": "enemy"},
				Owner:      model.TriggerOwner{UnitID: "player1"},
			},
			{
				// Second: enemy tries to attack, but should be dead
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 50, "target": "self"}, // target self to avoid "no target" issue
				Owner:      model.TriggerOwner{UnitID: "enemy1"},
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
	effectsMsg, ok := msg.(model.EffectsResolved)
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
		Phase:   model.PhaseCombat,
	}

	newModel, cmd := m.Update(model.CombatEnded{Victor: model.VictorPlayer})

	if newModel.Phase != model.PhaseInterCombat {
		t.Errorf("expected PhaseInterCombat, got %d", newModel.Phase)
	}
	if newModel.ChoiceType != model.ChoiceReward {
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
		Phase:   model.PhaseCombat,
	}

	newModel, cmd := m.Update(model.CombatEnded{Victor: model.VictorEnemy})

	if newModel.Phase != model.PhaseGameOver {
		t.Errorf("expected PhaseGameOver, got %d", newModel.Phase)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_CombatEnded_Draw(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
	}

	newModel, cmd := m.Update(model.CombatEnded{Victor: model.VictorDraw})

	if newModel.Phase != model.PhaseGameOver {
		t.Errorf("expected PhaseGameOver, got %d", newModel.Phase)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_ChoiceSelected_Reward(t *testing.T) {
	m := Model{
		Version:           1,
		Phase:             model.PhaseInterCombat,
		ChoiceType:        model.ChoiceReward,
		RewardChoicesLeft: 2,
		Choices:           []string{"A", "B", "C"},
	}

	// First reward selection
	newModel, cmd := m.Update(model.ChoiceSelected{Index: 0})

	if newModel.RewardChoicesLeft != 1 {
		t.Errorf("expected 1 reward choice left, got %d", newModel.RewardChoicesLeft)
	}
	if newModel.ChoiceType != model.ChoiceReward {
		t.Errorf("expected ChoiceReward, got %d", newModel.ChoiceType)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}

	// Second reward selection should switch to fight selection
	newModel, cmd = newModel.Update(model.ChoiceSelected{Index: 1})

	if newModel.RewardChoicesLeft != 0 {
		t.Errorf("expected 0 reward choices left, got %d", newModel.RewardChoicesLeft)
	}
	if newModel.ChoiceType != model.ChoiceFight {
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
		Phase:       model.PhaseInterCombat,
		FightNumber: 1,
	}

	combat := model.CombatModel{
		Phase: model.CombatActive,
		Log:   []string{"Fight 2 started"},
	}

	newModel, cmd := m.Update(model.CombatStarted{Combat: combat})

	if newModel.Phase != model.PhaseCombat {
		t.Errorf("expected PhaseCombat, got %d", newModel.Phase)
	}
	if newModel.FightNumber != 2 {
		t.Errorf("expected FightNumber 2, got %d", newModel.FightNumber)
	}
	if newModel.Combat.Phase != model.CombatActive {
		t.Errorf("expected combat phase CombatActive")
	}
	// Wave 3: CombatStarted now returns StartNextRound cmd
	if cmd == nil {
		t.Error("expected StartNextRound command (Wave 3)")
	}
}

func TestChoiceSelected_ValidatesIndex(t *testing.T) {
	m := Model{
		Version:    1,
		Phase:      model.PhaseInterCombat,
		ChoiceType: model.ChoiceReward,
		Choices:    []string{"A", "B", "C"},
	}

	// Out of bounds - should be no-op
	newM, _ := m.Update(model.ChoiceSelected{Index: 99})
	if newM.Choices[0] != "A" {
		t.Error("out-of-bounds index should not change state")
	}

	// Negative - should be no-op
	newM2, _ := m.Update(model.ChoiceSelected{Index: -1})
	if newM2.Choices[0] != "A" {
		t.Error("negative index should not change state")
	}
}

func TestChoiceSelected_RequiresChoicePhase(t *testing.T) {
	m := Model{
		Version:    1,
		Phase:      model.PhaseCombat, // Wrong phase
		ChoiceType: model.ChoiceReward,
		Choices:    []string{"A", "B", "C"},
	}

	newM, _ := m.Update(model.ChoiceSelected{Index: 0})

	// Should be no-op - not in choice phase
	if newM.Phase != model.PhaseCombat {
		t.Error("should not change phase when not in PhaseInterCombat")
	}
}

func TestAppendLogEntry_Bounded(t *testing.T) {
	// Create log at max capacity
	log := make([]string, model.MaxLogEntries)
	for i := range log {
		log[i] = fmt.Sprintf("entry %d", i)
	}

	newLog := appendLogEntry(log, "new entry")

	if len(newLog) != model.MaxLogEntries {
		t.Errorf("expected %d entries, got %d", model.MaxLogEntries, len(newLog))
	}
	// First entry should be pruned, last entry should be new
	if newLog[0] == "entry 0" {
		t.Error("oldest entry should have been pruned")
	}
	if newLog[len(newLog)-1] != "new entry" {
		t.Error("new entry should be at end")
	}
}

func TestAppendLogEntries_Bounded(t *testing.T) {
	log := make([]string, model.MaxLogEntries-2)
	for i := range log {
		log[i] = fmt.Sprintf("entry %d", i)
	}

	// Add 5 entries when only 2 slots available
	newLog := appendLogEntries(log, []string{"a", "b", "c", "d", "e"})

	if len(newLog) != model.MaxLogEntries {
		t.Errorf("expected %d entries, got %d", model.MaxLogEntries, len(newLog))
	}
	// Should keep most recent entries
	if newLog[len(newLog)-1] != "e" {
		t.Error("most recent entry should be 'e'")
	}
}

func TestAppendLogEntry_Immutable(t *testing.T) {
	original := []string{"a", "b", "c"}
	originalLen := len(original)

	newLog := appendLogEntry(original, "d")

	if len(original) != originalLen {
		t.Error("original slice should not be modified")
	}
	if len(newLog) != 4 {
		t.Errorf("new log should have 4 entries, got %d", len(newLog))
	}
}

func TestAppendLogEntries_Empty(t *testing.T) {
	original := []string{"a", "b"}
	newLog := appendLogEntries(original, []string{})

	// Should return same slice (optimization)
	if &original[0] != &newLog[0] {
		t.Error("empty entries should return original slice")
	}
}
