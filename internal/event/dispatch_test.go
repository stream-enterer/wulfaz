package event

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

func TestDispatch_UnitLevelTrigger(t *testing.T) {
	unit := entity.Unit{
		ID: "unit1",
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnCombatTick,
				EffectName: "deal_damage",
				Params:     map[string]any{"damage": 10},
				Priority:   1,
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 1 {
		t.Fatalf("expected 1 trigger, got %d", len(collected))
	}
	if collected[0].Trigger.EffectName != "deal_damage" {
		t.Errorf("expected effect name 'deal_damage', got %s", collected[0].Trigger.EffectName)
	}
	if collected[0].Owner.UnitID != "unit1" {
		t.Errorf("expected owner unit 'unit1', got %s", collected[0].Owner.UnitID)
	}
	if collected[0].Owner.PartID != "" {
		t.Errorf("expected empty part ID for unit-level trigger, got %s", collected[0].Owner.PartID)
	}
}

func TestDispatch_WrongEvent(t *testing.T) {
	unit := entity.Unit{
		ID: "unit1",
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnDamaged,
				EffectName: "deal_damage",
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 0 {
		t.Fatalf("expected 0 triggers for wrong event, got %d", len(collected))
	}
}

func TestDispatch_FailingCondition(t *testing.T) {
	unit := entity.Unit{
		ID: "unit1",
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 50},
		},
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnCombatTick,
				EffectName: "deal_damage",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrGTE, Params: map[string]any{"attr": "health", "value": 100}},
				},
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 0 {
		t.Fatalf("expected 0 triggers with failing condition, got %d", len(collected))
	}
}

func TestDispatch_PassingCondition(t *testing.T) {
	unit := entity.Unit{
		ID: "unit1",
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100},
		},
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnCombatTick,
				EffectName: "deal_damage",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrGTE, Params: map[string]any{"attr": "health", "value": 50}},
				},
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 1 {
		t.Fatalf("expected 1 trigger with passing condition, got %d", len(collected))
	}
}

func TestDispatch_HasTagCondition(t *testing.T) {
	unit := entity.Unit{
		ID:   "unit1",
		Tags: []core.Tag{"mech", "heavy"},
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnCombatTick,
				EffectName: "effect1",
				Conditions: []core.Condition{
					{Type: core.ConditionHasTag, Params: map[string]any{"tag": "heavy"}},
				},
			},
			{
				Event:      core.EventOnCombatTick,
				EffectName: "effect2",
				Conditions: []core.Condition{
					{Type: core.ConditionHasTag, Params: map[string]any{"tag": "light"}},
				},
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 1 {
		t.Fatalf("expected 1 trigger, got %d", len(collected))
	}
	if collected[0].Trigger.EffectName != "effect1" {
		t.Errorf("expected effect1, got %s", collected[0].Trigger.EffectName)
	}
}

func TestDispatch_PrioritySorting(t *testing.T) {
	unit := entity.Unit{
		ID: "unit1",
		Triggers: []core.Trigger{
			{Event: core.EventOnCombatTick, EffectName: "effect3", Priority: 3},
			{Event: core.EventOnCombatTick, EffectName: "effect1", Priority: 1},
			{Event: core.EventOnCombatTick, EffectName: "effect2", Priority: 2},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 3 {
		t.Fatalf("expected 3 triggers, got %d", len(collected))
	}
	if collected[0].Trigger.EffectName != "effect1" {
		t.Errorf("expected effect1 first, got %s", collected[0].Trigger.EffectName)
	}
	if collected[1].Trigger.EffectName != "effect2" {
		t.Errorf("expected effect2 second, got %s", collected[1].Trigger.EffectName)
	}
	if collected[2].Trigger.EffectName != "effect3" {
		t.Errorf("expected effect3 third, got %s", collected[2].Trigger.EffectName)
	}
}

func TestDispatch_PartLevelTrigger(t *testing.T) {
	unit := entity.Unit{
		ID: "unit1",
		Parts: map[string]entity.Part{
			"left_arm": {
				ID: "left_arm",
				Triggers: []core.Trigger{
					{Event: core.EventOnCombatTick, EffectName: "arm_effect"},
				},
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 1 {
		t.Fatalf("expected 1 trigger, got %d", len(collected))
	}
	if collected[0].Owner.PartID != "left_arm" {
		t.Errorf("expected part ID 'left_arm', got %s", collected[0].Owner.PartID)
	}
}

func TestDispatch_ItemLevelTrigger(t *testing.T) {
	unit := entity.Unit{
		ID: "unit1",
		Parts: map[string]entity.Part{
			"torso": {
				ID: "torso",
				Mounts: []entity.Mount{
					{
						ID: "weapon_mount",
						Contents: []entity.Item{
							{
								ID: "laser",
								Triggers: []core.Trigger{
									{Event: core.EventOnCombatTick, EffectName: "fire_laser"},
								},
							},
						},
					},
				},
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 1 {
		t.Fatalf("expected 1 trigger, got %d", len(collected))
	}
	if collected[0].Owner.PartID != "torso" {
		t.Errorf("expected part ID 'torso', got %s", collected[0].Owner.PartID)
	}
	if collected[0].Owner.MountID != "weapon_mount" {
		t.Errorf("expected mount ID 'weapon_mount', got %s", collected[0].Owner.MountID)
	}
	if collected[0].Owner.ItemID != "laser" {
		t.Errorf("expected item ID 'laser', got %s", collected[0].Owner.ItemID)
	}
}

func TestDispatch_AttrConditions(t *testing.T) {
	unit := entity.Unit{
		ID: "unit1",
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 50},
		},
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnCombatTick,
				EffectName: "gte_pass",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrGTE, Params: map[string]any{"attr": "health", "value": 50}},
				},
			},
			{
				Event:      core.EventOnCombatTick,
				EffectName: "gte_fail",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrGTE, Params: map[string]any{"attr": "health", "value": 51}},
				},
			},
			{
				Event:      core.EventOnCombatTick,
				EffectName: "lte_pass",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrLTE, Params: map[string]any{"attr": "health", "value": 50}},
				},
			},
			{
				Event:      core.EventOnCombatTick,
				EffectName: "lte_fail",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrLTE, Params: map[string]any{"attr": "health", "value": 49}},
				},
			},
			{
				Event:      core.EventOnCombatTick,
				EffectName: "eq_pass",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrEQ, Params: map[string]any{"attr": "health", "value": 50}},
				},
			},
			{
				Event:      core.EventOnCombatTick,
				EffectName: "eq_fail",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrEQ, Params: map[string]any{"attr": "health", "value": 51}},
				},
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	effectNames := make(map[string]bool)
	for _, c := range collected {
		effectNames[c.Trigger.EffectName] = true
	}

	if !effectNames["gte_pass"] {
		t.Error("expected gte_pass to be collected")
	}
	if effectNames["gte_fail"] {
		t.Error("expected gte_fail to not be collected")
	}
	if !effectNames["lte_pass"] {
		t.Error("expected lte_pass to be collected")
	}
	if effectNames["lte_fail"] {
		t.Error("expected lte_fail to not be collected")
	}
	if !effectNames["eq_pass"] {
		t.Error("expected eq_pass to be collected")
	}
	if effectNames["eq_fail"] {
		t.Error("expected eq_fail to not be collected")
	}
}

func TestDispatch_MissingAttribute(t *testing.T) {
	unit := entity.Unit{
		ID:         "unit1",
		Attributes: map[string]core.Attribute{},
		Triggers: []core.Trigger{
			{
				Event:      core.EventOnCombatTick,
				EffectName: "effect1",
				Conditions: []core.Condition{
					{Type: core.ConditionAttrGTE, Params: map[string]any{"attr": "nonexistent", "value": 50}},
				},
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	collected := Dispatch(ctx)

	if len(collected) != 0 {
		t.Errorf("expected 0 triggers when attribute missing, got %d", len(collected))
	}
}

func TestDispatch_DeterministicPartOrder(t *testing.T) {
	// Parts are stored in a map, so order must be enforced via sorting
	unit := entity.Unit{
		ID: "unit1",
		Parts: map[string]entity.Part{
			"zebra_part": {
				ID: "zebra_part",
				Triggers: []core.Trigger{
					{Event: core.EventOnCombatTick, EffectName: "zebra_effect", Priority: 1},
				},
			},
			"alpha_part": {
				ID: "alpha_part",
				Triggers: []core.Trigger{
					{Event: core.EventOnCombatTick, EffectName: "alpha_effect", Priority: 1},
				},
			},
			"middle_part": {
				ID: "middle_part",
				Triggers: []core.Trigger{
					{Event: core.EventOnCombatTick, EffectName: "middle_effect", Priority: 1},
				},
			},
		},
	}

	ctx := TriggerContext{
		Event:      core.EventOnCombatTick,
		SourceUnit: unit,
	}

	// Run multiple times to verify determinism
	var firstOrder []string
	for i := 0; i < 10; i++ {
		collected := Dispatch(ctx)

		if len(collected) != 3 {
			t.Fatalf("expected 3 triggers, got %d", len(collected))
		}

		order := make([]string, 3)
		for j, c := range collected {
			order[j] = c.Owner.PartID
		}

		if i == 0 {
			firstOrder = order
			// Verify alphabetical order (since same priority)
			if order[0] != "alpha_part" || order[1] != "middle_part" || order[2] != "zebra_part" {
				t.Errorf("expected alphabetical order, got %v", order)
			}
		} else {
			// Verify same order on subsequent runs
			for j := range order {
				if order[j] != firstOrder[j] {
					t.Errorf("iteration %d: order not deterministic, expected %v, got %v", i, firstOrder, order)
					break
				}
			}
		}
	}
}
