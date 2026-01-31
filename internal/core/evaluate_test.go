package core

import "testing"

func TestEvaluateConditions_Empty(t *testing.T) {
	// Empty slice returns true
	if !EvaluateConditions(nil, nil, nil) {
		t.Error("expected nil conditions to return true")
	}
	if !EvaluateConditions([]Condition{}, nil, nil) {
		t.Error("expected empty conditions to return true")
	}
}

func TestEvaluateConditions_AllPass(t *testing.T) {
	conditions := []Condition{
		{Type: ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 50}},
		{Type: ConditionAttrLTE, Params: map[string]any{"attribute": "health", "value": 150}},
	}
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 100},
	}

	if !EvaluateConditions(conditions, nil, attrs) {
		t.Error("expected all conditions to pass")
	}
}

func TestEvaluateConditions_OneFails(t *testing.T) {
	conditions := []Condition{
		{Type: ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 50}},
		{Type: ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 150}}, // fails
	}
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 100},
	}

	if EvaluateConditions(conditions, nil, attrs) {
		t.Error("expected conditions to fail when one fails")
	}
}

func TestEvaluateCondition_HasTag_Found(t *testing.T) {
	cond := Condition{
		Type:   ConditionHasTag,
		Params: map[string]any{"tag": "mech"},
	}
	tags := []Tag{"mech", "heavy"}

	if !EvaluateCondition(cond, tags, nil) {
		t.Error("expected tag to be found")
	}
}

func TestEvaluateCondition_HasTag_Missing(t *testing.T) {
	cond := Condition{
		Type:   ConditionHasTag,
		Params: map[string]any{"tag": "light"},
	}
	tags := []Tag{"mech", "heavy"}

	if EvaluateCondition(cond, tags, nil) {
		t.Error("expected tag to not be found")
	}
}

func TestEvaluateCondition_AttrGTE_Pass(t *testing.T) {
	cond := Condition{
		Type:   ConditionAttrGTE,
		Params: map[string]any{"attribute": "health", "value": 50},
	}
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 100},
	}

	if !EvaluateCondition(cond, nil, attrs) {
		t.Error("expected attr_gte to pass (100 >= 50)")
	}
}

func TestEvaluateCondition_AttrGTE_Fail(t *testing.T) {
	cond := Condition{
		Type:   ConditionAttrGTE,
		Params: map[string]any{"attribute": "health", "value": 150},
	}
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 100},
	}

	if EvaluateCondition(cond, nil, attrs) {
		t.Error("expected attr_gte to fail (100 < 150)")
	}
}

func TestEvaluateCondition_AttrLTE(t *testing.T) {
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 50},
	}

	// Pass case
	cond := Condition{
		Type:   ConditionAttrLTE,
		Params: map[string]any{"attribute": "health", "value": 50},
	}
	if !EvaluateCondition(cond, nil, attrs) {
		t.Error("expected attr_lte to pass (50 <= 50)")
	}

	// Fail case
	cond = Condition{
		Type:   ConditionAttrLTE,
		Params: map[string]any{"attribute": "health", "value": 49},
	}
	if EvaluateCondition(cond, nil, attrs) {
		t.Error("expected attr_lte to fail (50 > 49)")
	}
}

func TestEvaluateCondition_AttrEQ(t *testing.T) {
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 50},
	}

	// Pass case
	cond := Condition{
		Type:   ConditionAttrEQ,
		Params: map[string]any{"attribute": "health", "value": 50},
	}
	if !EvaluateCondition(cond, nil, attrs) {
		t.Error("expected attr_eq to pass (50 == 50)")
	}

	// Fail case
	cond = Condition{
		Type:   ConditionAttrEQ,
		Params: map[string]any{"attribute": "health", "value": 51},
	}
	if EvaluateCondition(cond, nil, attrs) {
		t.Error("expected attr_eq to fail (50 != 51)")
	}
}

func TestEvaluateCondition_MissingAttr(t *testing.T) {
	cond := Condition{
		Type:   ConditionAttrGTE,
		Params: map[string]any{"attribute": "nonexistent", "value": 50},
	}
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 100},
	}

	if EvaluateCondition(cond, nil, attrs) {
		t.Error("expected condition to fail when attribute missing")
	}
}

func TestEvaluateCondition_UnknownType(t *testing.T) {
	cond := Condition{
		Type:   ConditionType("unknown"),
		Params: map[string]any{},
	}

	if EvaluateCondition(cond, nil, nil) {
		t.Error("expected unknown condition type to return false")
	}
}

func TestEvaluateCondition_FloatValue(t *testing.T) {
	// JSON deserialization gives float64, should still work
	cond := Condition{
		Type:   ConditionAttrGTE,
		Params: map[string]any{"attribute": "health", "value": float64(50)},
	}
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 100},
	}

	if !EvaluateCondition(cond, nil, attrs) {
		t.Error("expected float64 value to work correctly")
	}
}

func TestEvaluateCondition_NilParams(t *testing.T) {
	// Condition with nil Params should return false, not panic
	cond := Condition{
		Type:   ConditionAttrGTE,
		Params: nil,
	}
	attrs := map[string]Attribute{
		"health": {Name: "health", Base: 100},
	}

	if EvaluateCondition(cond, nil, attrs) {
		t.Error("expected nil params to return false")
	}

	// Same for has_tag
	cond = Condition{
		Type:   ConditionHasTag,
		Params: nil,
	}
	tags := []Tag{"mech"}

	if EvaluateCondition(cond, tags, nil) {
		t.Error("expected nil params on has_tag to return false")
	}
}

func TestHasTag(t *testing.T) {
	tags := []Tag{"mech", "heavy", "assault"}

	if !HasTag(tags, Tag("mech")) {
		t.Error("expected to find 'mech' tag")
	}
	if !HasTag(tags, Tag("heavy")) {
		t.Error("expected to find 'heavy' tag")
	}
	if HasTag(tags, Tag("light")) {
		t.Error("expected to not find 'light' tag")
	}
	if HasTag(nil, Tag("mech")) {
		t.Error("expected nil tags to return false")
	}
}
