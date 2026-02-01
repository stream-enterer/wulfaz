package core

import (
	"reflect"
	"testing"
)

// Round-trip tests: verify all fields are copied by comparing original to copy.
// If a new field is added to a struct but not to its Copy function, these fail.

func TestCopyCondition_RoundTrip(t *testing.T) {
	orig := Condition{
		Type:   ConditionAttrGTE,
		Params: map[string]any{"attribute": "health", "value": 50},
	}
	copied := CopyCondition(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyCondition round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyTrigger_RoundTrip(t *testing.T) {
	orig := Trigger{
		Event:            EventOnDamaged,
		Conditions:       []Condition{{Type: ConditionHasTag, Params: map[string]any{"tag": "active"}}},
		TargetConditions: []Condition{{Type: ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 1}}},
		EffectName:       "deal_damage",
		Params:           map[string]any{"damage": 10, "type": "energy"},
		Priority:         5,
	}
	copied := CopyTrigger(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyTrigger round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyEffectRef_RoundTrip(t *testing.T) {
	orig := EffectRef{
		EffectName: "apply_status",
		Params:     map[string]any{"status": "burning", "duration": 3},
		Delay:      2,
		Conditions: []Condition{{Type: ConditionHasTag, Params: map[string]any{"tag": "flammable"}}},
	}
	copied := CopyEffectRef(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyEffectRef round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyTargeting_RoundTrip(t *testing.T) {
	orig := Targeting{
		Type:   TargetEnemy,
		Range:  5,
		Count:  2,
		Filter: []Tag{"mech", "active"},
	}
	copied := CopyTargeting(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyTargeting round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyCost_RoundTrip(t *testing.T) {
	orig := Cost{
		Attribute: "energy",
		Scope:     ScopeUnit,
		Amount:    ValueRef{Value: 15},
	}
	copied := CopyCost(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyCost round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyAbility_RoundTrip(t *testing.T) {
	orig := Ability{
		ID:                 "laser_blast",
		Tags:               []Tag{"active", "weapon", "energy"},
		Conditions:         []Condition{{Type: ConditionAttrGTE, Params: map[string]any{"attribute": "heat", "value": 0}}},
		Costs:              []Cost{{Attribute: "energy", Scope: ScopeUnit, Amount: ValueRef{Value: 10}}},
		Targeting:          Targeting{Type: TargetEnemy, Range: 8, Count: 1, Filter: []Tag{"mech"}},
		Effects:            []EffectRef{{EffectName: "damage", Params: map[string]any{"amount": 25}}},
		Cooldown:           2,
		Charges:            3,
		ChargeRestoreEvent: EventOnTurnStart,
	}
	copied := CopyAbility(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyAbility round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyProvidedModifier_RoundTrip(t *testing.T) {
	orig := ProvidedModifier{
		Scope:       ScopeAdjacent,
		ScopeFilter: []Tag{"ally", "mech"},
		Attribute:   "armor",
		Operation:   ModifierOpAdd,
		Value:       5,
		StackGroup:  "armor_aura",
		Conditions:  []Condition{{Type: ConditionHasTag, Params: map[string]any{"tag": "active"}}},
	}
	copied := CopyProvidedModifier(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyProvidedModifier round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyRequirement_RoundTrip(t *testing.T) {
	orig := Requirement{
		Scope:     "part",
		Condition: Condition{Type: ConditionAttrGTE, Params: map[string]any{"attribute": "slots", "value": 2}},
		OnUnmet:   OnUnmetCannotMount,
	}
	copied := CopyRequirement(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyRequirement round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyTags_Nil(t *testing.T) {
	result := CopyTags(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyTags_Independence(t *testing.T) {
	orig := []Tag{"a", "b", "c"}
	copied := CopyTags(orig)

	copied[0] = "modified"
	if orig[0] != "a" {
		t.Error("original was mutated")
	}
}

func TestCopyCondition_NilParams(t *testing.T) {
	orig := Condition{Type: ConditionHasTag, Params: nil}
	copied := CopyCondition(orig)

	if copied.Params != nil {
		t.Error("expected nil Params for nil input")
	}
}

func TestCopyCondition_Independence(t *testing.T) {
	orig := Condition{
		Type:   ConditionHasTag,
		Params: map[string]any{"tag": "weapon"},
	}
	copied := CopyCondition(orig)

	copied.Params["tag"] = "modified"
	if orig.Params["tag"] != "weapon" {
		t.Error("original Params was mutated")
	}
}

func TestCopyConditions_Nil(t *testing.T) {
	result := CopyConditions(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyTrigger_Independence(t *testing.T) {
	orig := Trigger{
		Event:      EventOnDamaged,
		EffectName: "deal_damage",
		Params:     map[string]any{"damage": 5},
		Conditions: []Condition{
			{Type: ConditionHasTag, Params: map[string]any{"tag": "weapon"}},
		},
		Priority: 1,
	}
	copied := CopyTrigger(orig)

	// Mutate copied values
	copied.Params["damage"] = 99
	copied.Conditions[0].Params["tag"] = "modified"

	if orig.Params["damage"] != 5 {
		t.Error("original Params was mutated")
	}
	if orig.Conditions[0].Params["tag"] != "weapon" {
		t.Error("original Conditions was mutated")
	}
}

func TestCopyTriggers_Nil(t *testing.T) {
	result := CopyTriggers(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyAbility_NestedSlices(t *testing.T) {
	orig := Ability{
		ID:         "test_ability",
		Tags:       []Tag{"active", "weapon"},
		Conditions: []Condition{{Type: ConditionAttrGTE, Params: map[string]any{"attribute": "heat", "value": 10}}},
		Costs:      []Cost{{Attribute: "energy", Amount: ValueRef{Value: 5}}},
		Targeting:  Targeting{Type: TargetEnemy, Filter: []Tag{"mech"}},
		Effects:    []EffectRef{{EffectName: "damage", Params: map[string]any{"amount": 10}}},
		Cooldown:   2,
		Charges:    3,
	}
	copied := CopyAbility(orig)

	// Mutate copied values
	copied.Tags[0] = "modified"
	copied.Conditions[0].Params["attribute"] = "modified"
	copied.Targeting.Filter[0] = "modified"
	copied.Effects[0].Params["amount"] = 999

	if orig.Tags[0] != "active" {
		t.Error("original Tags was mutated")
	}
	if orig.Conditions[0].Params["attribute"] != "heat" {
		t.Error("original Conditions was mutated")
	}
	if orig.Targeting.Filter[0] != "mech" {
		t.Error("original Targeting.Filter was mutated")
	}
	if orig.Effects[0].Params["amount"] != 10 {
		t.Error("original Effects was mutated")
	}
}

func TestCopyAbilities_Nil(t *testing.T) {
	result := CopyAbilities(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyProvidedModifier_Independence(t *testing.T) {
	orig := ProvidedModifier{
		Scope:       ScopeUnit,
		ScopeFilter: []Tag{"mech"},
		Attribute:   "damage",
		Operation:   ModifierOpAdd,
		Value:       5,
		StackGroup:  "damage_boost",
		Conditions:  []Condition{{Type: ConditionHasTag, Params: map[string]any{"tag": "active"}}},
	}
	copied := CopyProvidedModifier(orig)

	copied.ScopeFilter[0] = "modified"
	copied.Conditions[0].Params["tag"] = "modified"

	if orig.ScopeFilter[0] != "mech" {
		t.Error("original ScopeFilter was mutated")
	}
	if orig.Conditions[0].Params["tag"] != "active" {
		t.Error("original Conditions was mutated")
	}
}

func TestCopyRequirement_Independence(t *testing.T) {
	orig := Requirement{
		Scope:     "unit",
		Condition: Condition{Type: ConditionHasTag, Params: map[string]any{"tag": "mech"}},
		OnUnmet:   OnUnmetDisabled,
	}
	copied := CopyRequirement(orig)

	copied.Condition.Params["tag"] = "modified"

	if orig.Condition.Params["tag"] != "mech" {
		t.Error("original Condition was mutated")
	}
}

func TestCopyAttributes_Nil(t *testing.T) {
	result := CopyAttributes(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyAttributes_Independence(t *testing.T) {
	orig := map[string]Attribute{
		"health": {Name: "health", Base: 100, Min: 0, Max: 200},
	}
	copied := CopyAttributes(orig)

	copied["health"] = Attribute{Name: "health", Base: 999}

	if orig["health"].Base != 100 {
		t.Error("original was mutated")
	}
}
