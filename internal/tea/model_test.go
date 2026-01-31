package tea

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

func TestUpdate_CombatTicked_CollectsTriggers(t *testing.T) {
	playerUnit := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100, Min: 0},
		},
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnCombatTick,
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 10, "target": "enemy"},
				Priority:   1,
			},
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
			Tick:        0,
		},
	}

	// Send CombatTicked
	newModel, cmd := m.Update(CombatTicked{Rolls: []int{5, 3, 2}})

	// Tick should be incremented
	if newModel.Combat.Tick != 1 {
		t.Errorf("expected tick 1, got %d", newModel.Combat.Tick)
	}

	// Should return command that yields TriggersCollected
	if cmd == nil {
		t.Fatal("expected non-nil command")
	}

	msg := cmd()
	triggersMsg, ok := msg.(TriggersCollected)
	if !ok {
		t.Fatalf("expected TriggersCollected, got %T", msg)
	}

	if len(triggersMsg.Triggers) != 1 {
		t.Errorf("expected 1 trigger, got %d", len(triggersMsg.Triggers))
	}

	if triggersMsg.Triggers[0].EffectName != "deal_damage" {
		t.Errorf("expected deal_damage effect, got %s", triggersMsg.Triggers[0].EffectName)
	}
}

func TestUpdate_CombatTicked_NotActive(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			Phase: model.CombatPaused, // Not active
			Tick:  5,
		},
	}

	newModel, cmd := m.Update(CombatTicked{})

	// Tick should NOT be incremented when not active
	if newModel.Combat.Tick != 5 {
		t.Errorf("expected tick to remain 5, got %d", newModel.Combat.Tick)
	}

	// Should return nil command
	if cmd != nil {
		t.Error("expected nil command when combat not active")
	}
}

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
		Event: string(core.EventOnCombatTick),
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
			Tick:       1,
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
		Event: string(core.EventOnCombatTick),
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

func TestCombat_TwoUnitsPerSide(t *testing.T) {
	// Simulate 2v2 combat to verify damage is applied correctly after unit1s die

	attackTrigger := func() core.Trigger {
		return core.Trigger{
			Event:      core.EventOnCombatTick,
			EffectName: "deal_damage",
			Params:     map[string]any{"damage": 30, "target": "enemy"},
			Priority:   1,
			Conditions: []core.Condition{
				{Type: core.ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 1}},
			},
			TargetConditions: []core.Condition{
				{Type: core.ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 1}},
			},
		}
	}

	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			PlayerUnits: []entity.Unit{
				{
					ID:         "player1",
					Attributes: map[string]core.Attribute{"health": {Name: "health", Base: 50}},
					Triggers:   []core.Trigger{attackTrigger()},
				},
				{
					ID:         "player2",
					Attributes: map[string]core.Attribute{"health": {Name: "health", Base: 50}},
					Triggers:   []core.Trigger{attackTrigger()},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:         "enemy1",
					Attributes: map[string]core.Attribute{"health": {Name: "health", Base: 50}},
					Triggers:   []core.Trigger{attackTrigger()},
				},
				{
					ID:         "enemy2",
					Attributes: map[string]core.Attribute{"health": {Name: "health", Base: 50}},
					Triggers:   []core.Trigger{attackTrigger()},
				},
			},
			Phase: model.CombatActive,
			Tick:  0,
			Log:   []string{},
		},
	}

	// Helper to run one full tick
	runTick := func(m Model) Model {
		var cmd Cmd
		m, cmd = m.Update(CombatTicked{})
		for cmd != nil {
			msg := cmd()
			m, cmd = m.Update(msg)
		}
		return m
	}

	// Helper to get health
	getHealth := func(m Model, unitID string) int {
		for _, u := range m.Combat.PlayerUnits {
			if u.ID == unitID {
				return u.Attributes["health"].Base
			}
		}
		for _, u := range m.Combat.EnemyUnits {
			if u.ID == unitID {
				return u.Attributes["health"].Base
			}
		}
		return -1
	}

	t.Logf("Initial state:")
	t.Logf("  player1: %d, player2: %d, enemy1: %d, enemy2: %d",
		getHealth(m, "player1"), getHealth(m, "player2"),
		getHealth(m, "enemy1"), getHealth(m, "enemy2"))

	// Tick 1: All units attack
	m = runTick(m)
	t.Logf("After tick 1:")
	t.Logf("  player1: %d, player2: %d, enemy1: %d, enemy2: %d",
		getHealth(m, "player1"), getHealth(m, "player2"),
		getHealth(m, "enemy1"), getHealth(m, "enemy2"))
	t.Logf("  Log: %v", m.Combat.Log)

	// Tick 2: Surviving units should attack
	m = runTick(m)
	t.Logf("After tick 2:")
	t.Logf("  player1: %d, player2: %d, enemy1: %d, enemy2: %d",
		getHealth(m, "player1"), getHealth(m, "player2"),
		getHealth(m, "enemy1"), getHealth(m, "enemy2"))
	t.Logf("  Log: %v", m.Combat.Log)

	// Tick 3: Continue if anyone alive
	m = runTick(m)
	t.Logf("After tick 3:")
	t.Logf("  player1: %d, player2: %d, enemy1: %d, enemy2: %d",
		getHealth(m, "player1"), getHealth(m, "player2"),
		getHealth(m, "enemy1"), getHealth(m, "enemy2"))
	t.Logf("  Log: %v", m.Combat.Log)

	// Verify that unit2s took damage after unit1s died
	// With 30 damage per attack and 50 health:
	// - Tick 1: enemy1 and enemy2 both attack player1 (first alphabetically) -> player1 takes 60 damage (dead)
	//           player1 and player2 both attack enemy1 (first alphabetically) -> enemy1 takes 60 damage (dead)
	// - Tick 2: enemy2 attacks player2 (only living player) -> player2 takes 30 damage (20 left)
	//           player2 attacks enemy2 (only living enemy) -> enemy2 takes 30 damage (20 left)
	// - Tick 3: Same as tick 2 -> both should be at -10 (clamped to 0)

	if getHealth(m, "player2") != 0 && getHealth(m, "enemy2") != 0 {
		// At least one of them should have taken damage by now
		if getHealth(m, "player2") == 50 && getHealth(m, "enemy2") == 50 {
			t.Error("Neither unit2 took any damage - this is the bug!")
		}
	}
}

func TestFullCombatFlow(t *testing.T) {
	// Test the full flow: CombatTicked -> TriggersCollected -> EffectsResolved

	playerUnit := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100, Min: 0},
		},
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnCombatTick,
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 25, "target": "enemy"},
				Priority:   1,
			},
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
			Tick:        0,
			Log:         []string{},
		},
	}

	// Step 1: CombatTicked
	m, cmd := m.Update(CombatTicked{})
	if cmd == nil {
		t.Fatal("step 1: expected command")
	}

	// Step 2: TriggersCollected
	msg := cmd()
	m, cmd = m.Update(msg)
	if cmd == nil {
		t.Fatal("step 2: expected command")
	}

	// Step 3: EffectsResolved
	msg = cmd()
	m, cmd = m.Update(msg)

	// Verify final state
	if m.Combat.Tick != 1 {
		t.Errorf("expected tick 1, got %d", m.Combat.Tick)
	}

	// Enemy health should be reduced
	if m.Combat.EnemyUnits[0].Attributes["health"].Base != 25 {
		t.Errorf("expected enemy health 25, got %d", m.Combat.EnemyUnits[0].Attributes["health"].Base)
	}

	// Should have log entries
	if len(m.Combat.Log) == 0 {
		t.Error("expected log entries")
	}
}
