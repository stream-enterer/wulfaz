package template

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/sblinch/kdl-go"

	"wulfaz/internal/core"
)

// Unit tests for parsing helpers (inline KDL strings)

func TestParseTags(t *testing.T) {
	kdlStr := `tags "weapon" "energy" "laser"`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	tags := parseTags(doc.Nodes[0])
	if len(tags) != 3 {
		t.Fatalf("expected 3 tags, got %d", len(tags))
	}
	if tags[0] != "weapon" || tags[1] != "energy" || tags[2] != "laser" {
		t.Errorf("unexpected tags: %v", tags)
	}
}

func TestParseAttributes(t *testing.T) {
	kdlStr := `attributes {
		attribute name="hp" base=100 min=0 max=200
		attribute name="speed" base=5
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	attrs, err := parseAttributes(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseAttributes: %v", err)
	}

	if len(attrs) != 2 {
		t.Fatalf("expected 2 attributes, got %d", len(attrs))
	}

	hp, ok := attrs["hp"]
	if !ok {
		t.Fatal("missing hp attribute")
	}
	if hp.Name != "hp" || hp.Base != 100 || hp.Min != 0 || hp.Max != 200 {
		t.Errorf("unexpected hp attribute: %+v", hp)
	}

	speed, ok := attrs["speed"]
	if !ok {
		t.Fatal("missing speed attribute")
	}
	if speed.Name != "speed" || speed.Base != 5 || speed.Min != 0 || speed.Max != 0 {
		t.Errorf("unexpected speed attribute: %+v", speed)
	}
}

func TestParseCondition(t *testing.T) {
	kdlStr := `condition type="attr_gte" attribute="ammo" value=1`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	cond, err := parseCondition(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseCondition: %v", err)
	}

	if cond.Type != core.ConditionAttrGTE {
		t.Errorf("expected attr_gte, got %v", cond.Type)
	}
	if cond.Params["attribute"] != "ammo" {
		t.Errorf("expected attribute=ammo, got %v", cond.Params["attribute"])
	}
	if cond.Params["value"] != 1 {
		t.Errorf("expected value=1, got %v", cond.Params["value"])
	}
}

func TestParseTrigger(t *testing.T) {
	kdlStr := `trigger event="on_activate" effect_name="deal_damage" priority=5 {
		params damage=10 target="enemy"
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	trigger, err := parseTrigger(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseTrigger: %v", err)
	}

	if trigger.Event != core.EventOnActivate {
		t.Errorf("expected on_activate, got %v", trigger.Event)
	}
	if trigger.EffectName != "deal_damage" {
		t.Errorf("expected deal_damage, got %v", trigger.EffectName)
	}
	if trigger.Priority != 5 {
		t.Errorf("expected priority 5, got %d", trigger.Priority)
	}
	if trigger.Params["damage"] != 10 {
		t.Errorf("expected damage=10, got %v", trigger.Params["damage"])
	}
	if trigger.Params["target"] != "enemy" {
		t.Errorf("expected target=enemy, got %v", trigger.Params["target"])
	}
}

func TestParseTrigger_WithTargetConditions(t *testing.T) {
	kdlStr := `trigger event="on_combat_tick" effect_name="deal_damage" {
		params damage=5 target="enemy"
		target_conditions {
			condition type="attr_gte" attribute="health" value=1
		}
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	trigger, err := parseTrigger(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseTrigger: %v", err)
	}

	if len(trigger.TargetConditions) != 1 {
		t.Fatalf("expected 1 target condition, got %d", len(trigger.TargetConditions))
	}
	if trigger.TargetConditions[0].Type != core.ConditionAttrGTE {
		t.Errorf("expected attr_gte, got %v", trigger.TargetConditions[0].Type)
	}
	if trigger.TargetConditions[0].Params["attribute"] != "health" {
		t.Errorf("expected attribute=health, got %v", trigger.TargetConditions[0].Params["attribute"])
	}
	if trigger.TargetConditions[0].Params["value"] != 1 {
		t.Errorf("expected value=1, got %v", trigger.TargetConditions[0].Params["value"])
	}
}

func TestParseTrigger_BothConditionTypes(t *testing.T) {
	kdlStr := `trigger event="on_combat_tick" effect_name="deal_damage" {
		params damage=5 target="enemy"
		conditions {
			condition type="attr_gte" attribute="health" value=1
		}
		target_conditions {
			condition type="attr_gte" attribute="health" value=1
			condition type="has_tag" tag="mech"
		}
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	trigger, err := parseTrigger(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseTrigger: %v", err)
	}

	if len(trigger.Conditions) != 1 {
		t.Errorf("expected 1 source condition, got %d", len(trigger.Conditions))
	}
	if len(trigger.TargetConditions) != 2 {
		t.Errorf("expected 2 target conditions, got %d", len(trigger.TargetConditions))
	}
}

func TestParseRequirement(t *testing.T) {
	kdlStr := `requirement scope="unit" on_unmet="disabled" {
		condition type="attr_gte" attribute="ammo" value=1
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	req, err := parseRequirement(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseRequirement: %v", err)
	}

	if req.Scope != "unit" {
		t.Errorf("expected scope=unit, got %v", req.Scope)
	}
	if req.OnUnmet != core.OnUnmetDisabled {
		t.Errorf("expected OnUnmetDisabled, got %v", req.OnUnmet)
	}
	if req.Condition.Type != core.ConditionAttrGTE {
		t.Errorf("expected attr_gte condition, got %v", req.Condition.Type)
	}
}

func TestParseProvidedModifier(t *testing.T) {
	kdlStr := `modifier scope="unit" attribute="damage" operation="add" value=5 stack_group="buff"`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	mod, err := parseModifier(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseModifier: %v", err)
	}

	if mod.Scope != core.ScopeUnit {
		t.Errorf("expected scope=unit, got %v", mod.Scope)
	}
	if mod.Attribute != "damage" {
		t.Errorf("expected attribute=damage, got %v", mod.Attribute)
	}
	if mod.Operation != core.ModifierOpAdd {
		t.Errorf("expected operation=add, got %v", mod.Operation)
	}
	if mod.Value != 5 {
		t.Errorf("expected value=5, got %d", mod.Value)
	}
	if mod.StackGroup != "buff" {
		t.Errorf("expected stack_group=buff, got %v", mod.StackGroup)
	}
}

func TestParseUnit(t *testing.T) {
	kdlStr := `unit id="test_mech" {
		tags "mech" "medium"
		attributes {
			attribute name="combat_width" base=2
		}
		parts {
			part id="left_arm" template_id="mech_arm" {
				tags "arm" "left"
			}
			part id="right_arm" template_id="mech_arm" {
				tags "arm" "right"
			}
		}
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	unit, err := parseUnit(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseUnit: %v", err)
	}

	if unit.ID != "test_mech" {
		t.Errorf("expected id=test_mech, got %v", unit.ID)
	}
	if len(unit.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(unit.Tags))
	}
	if len(unit.Parts) != 2 {
		t.Errorf("expected 2 parts, got %d", len(unit.Parts))
	}
	if _, ok := unit.Parts["left_arm"]; !ok {
		t.Error("missing left_arm part")
	}
	if _, ok := unit.Parts["right_arm"]; !ok {
		t.Error("missing right_arm part")
	}

	leftArm := unit.Parts["left_arm"]
	if leftArm.TemplateID != "mech_arm" {
		t.Errorf("expected template_id=mech_arm, got %v", leftArm.TemplateID)
	}
	if len(leftArm.Tags) != 2 {
		t.Errorf("expected 2 tags on left_arm, got %d", len(leftArm.Tags))
	}
}

func TestParseItem(t *testing.T) {
	kdlStr := `item id="test_weapon" {
		tags "weapon" "energy"
		attributes {
			attribute name="damage" base=10
			attribute name="size" base=1
		}
		triggers {
			trigger event="on_activate" effect_name="deal_damage" {
				params damage=10
			}
		}
		requirements {
			requirement scope="unit" on_unmet="disabled" {
				condition type="attr_gte" attribute="ammo" value=1
			}
		}
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	item, err := parseItem(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseItem: %v", err)
	}

	if item.ID != "test_weapon" {
		t.Errorf("expected id=test_weapon, got %v", item.ID)
	}
	if len(item.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(item.Tags))
	}
	if len(item.Attributes) != 2 {
		t.Errorf("expected 2 attributes, got %d", len(item.Attributes))
	}
	if len(item.Triggers) != 1 {
		t.Errorf("expected 1 trigger, got %d", len(item.Triggers))
	}
	if len(item.Requirements) != 1 {
		t.Errorf("expected 1 requirement, got %d", len(item.Requirements))
	}
}

// Enum validation tests

func TestParseEventType_Valid(t *testing.T) {
	tests := []struct {
		input    string
		expected core.EventType
	}{
		{"on_damaged", core.EventOnDamaged},
		{"on_destroyed", core.EventOnDestroyed},
		{"on_combat_tick", core.EventOnCombatTick},
		{"on_turn_start", core.EventOnTurnStart},
		{"on_turn_end", core.EventOnTurnEnd},
		{"on_activate", core.EventOnActivate},
	}

	for _, tt := range tests {
		result, err := parseEventType(tt.input)
		if err != nil {
			t.Errorf("parseEventType(%q): unexpected error: %v", tt.input, err)
			continue
		}
		if result != tt.expected {
			t.Errorf("parseEventType(%q): got %v, want %v", tt.input, result, tt.expected)
		}
	}
}

func TestParseEventType_Invalid(t *testing.T) {
	_, err := parseEventType("invalid_event")
	if err == nil {
		t.Error("expected error for invalid event type")
	}
}

func TestParseOnUnmet_Valid(t *testing.T) {
	tests := []struct {
		input    string
		expected core.OnUnmet
	}{
		{"disabled", core.OnUnmetDisabled},
		{"cannot_mount", core.OnUnmetCannotMount},
		{"warning", core.OnUnmetWarning},
	}

	for _, tt := range tests {
		result, err := parseOnUnmet(tt.input)
		if err != nil {
			t.Errorf("parseOnUnmet(%q): unexpected error: %v", tt.input, err)
			continue
		}
		if result != tt.expected {
			t.Errorf("parseOnUnmet(%q): got %v, want %v", tt.input, result, tt.expected)
		}
	}
}

func TestParseModifierOp_Valid(t *testing.T) {
	tests := []struct {
		input    string
		expected core.ModifierOp
	}{
		{"add", core.ModifierOpAdd},
		{"mult", core.ModifierOpMult},
		{"set", core.ModifierOpSet},
		{"min", core.ModifierOpMin},
		{"max", core.ModifierOpMax},
	}

	for _, tt := range tests {
		result, err := parseModifierOp(tt.input)
		if err != nil {
			t.Errorf("parseModifierOp(%q): unexpected error: %v", tt.input, err)
			continue
		}
		if result != tt.expected {
			t.Errorf("parseModifierOp(%q): got %v, want %v", tt.input, result, tt.expected)
		}
	}
}

func TestParseScope_Valid(t *testing.T) {
	tests := []struct {
		input    string
		expected core.Scope
	}{
		{"self", core.ScopeSelf},
		{"unit", core.ScopeUnit},
		{"part", core.ScopePart},
		{"adjacent", core.ScopeAdjacent},
		{"mount", core.ScopeMount},
	}

	for _, tt := range tests {
		result, err := parseScope(tt.input)
		if err != nil {
			t.Errorf("parseScope(%q): unexpected error: %v", tt.input, err)
			continue
		}
		if result != tt.expected {
			t.Errorf("parseScope(%q): got %v, want %v", tt.input, result, tt.expected)
		}
	}
}

func TestParseScope_Invalid(t *testing.T) {
	_, err := parseScope("invalid_scope")
	if err == nil {
		t.Error("expected error for invalid scope")
	}
}

func TestParseConditionType_Valid(t *testing.T) {
	tests := []struct {
		input    string
		expected core.ConditionType
	}{
		{"has_tag", core.ConditionHasTag},
		{"attr_gte", core.ConditionAttrGTE},
		{"attr_lte", core.ConditionAttrLTE},
		{"attr_eq", core.ConditionAttrEQ},
	}

	for _, tt := range tests {
		result, err := parseConditionType(tt.input)
		if err != nil {
			t.Errorf("parseConditionType(%q): unexpected error: %v", tt.input, err)
			continue
		}
		if result != tt.expected {
			t.Errorf("parseConditionType(%q): got %v, want %v", tt.input, result, tt.expected)
		}
	}
}

func TestParseConditionType_Invalid(t *testing.T) {
	_, err := parseConditionType("invalid_condition")
	if err == nil {
		t.Error("expected error for invalid condition type")
	}
}

func TestParseMount(t *testing.T) {
	kdlStr := `mount id="weapon_slot" capacity=4 max_items=2 locked=false {
		tags "weapon" "energy"
		accepts {
			requires_all "weapon"
			requires_any "energy" "ballistic"
			forbids "oversized"
		}
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	mount, err := parseMount(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseMount: %v", err)
	}

	if mount.ID != "weapon_slot" {
		t.Errorf("expected id=weapon_slot, got %v", mount.ID)
	}
	if mount.Capacity != 4 {
		t.Errorf("expected capacity=4, got %d", mount.Capacity)
	}
	if mount.MaxItems != 2 {
		t.Errorf("expected max_items=2, got %d", mount.MaxItems)
	}
	if mount.Locked != false {
		t.Errorf("expected locked=false, got %v", mount.Locked)
	}
	if len(mount.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(mount.Tags))
	}
	if len(mount.Accepts.RequiresAll) != 1 || mount.Accepts.RequiresAll[0] != "weapon" {
		t.Errorf("unexpected requires_all: %v", mount.Accepts.RequiresAll)
	}
	if len(mount.Accepts.RequiresAny) != 2 {
		t.Errorf("expected 2 requires_any, got %d", len(mount.Accepts.RequiresAny))
	}
	if len(mount.Accepts.Forbids) != 1 || mount.Accepts.Forbids[0] != "oversized" {
		t.Errorf("unexpected forbids: %v", mount.Accepts.Forbids)
	}
}

func TestParsePart(t *testing.T) {
	kdlStr := `part id="left_arm" template_id="mech_arm" {
		tags "arm" "left"
		attributes {
			attribute name="hp" base=50
		}
		mounts {
			mount id="hand" capacity=2
		}
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	part, err := parsePart(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parsePart: %v", err)
	}

	if part.ID != "left_arm" {
		t.Errorf("expected id=left_arm, got %v", part.ID)
	}
	if part.TemplateID != "mech_arm" {
		t.Errorf("expected template_id=mech_arm, got %v", part.TemplateID)
	}
	if len(part.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(part.Tags))
	}
	if len(part.Attributes) != 1 {
		t.Errorf("expected 1 attribute, got %d", len(part.Attributes))
	}
	if len(part.Mounts) != 1 {
		t.Errorf("expected 1 mount, got %d", len(part.Mounts))
	}
	if part.Mounts[0].ID != "hand" {
		t.Errorf("expected mount id=hand, got %v", part.Mounts[0].ID)
	}
}

// Integration tests

func TestLoadUnitsFromDir(t *testing.T) {
	reg := NewRegistry()
	dir := filepath.Join("..", "..", "data", "templates", "units")
	err := LoadUnitsFromDir(dir, reg)
	if err != nil {
		t.Fatalf("LoadUnitsFromDir: %v", err)
	}

	// Check small_mech loaded
	small, ok := reg.GetUnit("small_mech")
	if !ok {
		t.Error("small_mech not registered")
	} else {
		if small.ID != "small_mech" {
			t.Errorf("small_mech ID: got %q, want %q", small.ID, "small_mech")
		}
		if cw, ok := small.Attributes["combat_width"]; !ok || cw.Base != 1 {
			t.Errorf("small_mech combat_width: got %+v", small.Attributes["combat_width"])
		}
	}

	// Check medium_mech loaded
	medium, ok := reg.GetUnit("medium_mech")
	if !ok {
		t.Error("medium_mech not registered")
	} else {
		if medium.ID != "medium_mech" {
			t.Errorf("medium_mech ID: got %q, want %q", medium.ID, "medium_mech")
		}
		if cw, ok := medium.Attributes["combat_width"]; !ok || cw.Base != 2 {
			t.Errorf("medium_mech combat_width: got %+v", medium.Attributes["combat_width"])
		}
	}

	// Check large_mech loaded
	large, ok := reg.GetUnit("large_mech")
	if !ok {
		t.Error("large_mech not registered")
	} else {
		if large.ID != "large_mech" {
			t.Errorf("large_mech ID: got %q, want %q", large.ID, "large_mech")
		}
		if cw, ok := large.Attributes["combat_width"]; !ok || cw.Base != 3 {
			t.Errorf("large_mech combat_width: got %+v", large.Attributes["combat_width"])
		}
		if len(large.Parts) != 7 {
			t.Errorf("large_mech parts: got %d, want 7", len(large.Parts))
		}
	}
}

func TestLoadItemsFromDir(t *testing.T) {
	reg := NewRegistry()
	dir := filepath.Join("..", "..", "data", "templates", "items")
	err := LoadItemsFromDir(dir, reg)
	if err != nil {
		t.Fatalf("LoadItemsFromDir: %v", err)
	}

	// Check medium_laser loaded
	laser, ok := reg.GetItem("medium_laser")
	if !ok {
		t.Error("medium_laser not registered")
	} else {
		if laser.ID != "medium_laser" {
			t.Errorf("medium_laser ID: got %q, want %q", laser.ID, "medium_laser")
		}
		if dmg, ok := laser.Attributes["damage"]; !ok || dmg.Base != 5 {
			t.Errorf("medium_laser damage: got %+v", laser.Attributes["damage"])
		}
	}

	// Check autocannon loaded
	ac, ok := reg.GetItem("autocannon")
	if !ok {
		t.Error("autocannon not registered")
	} else {
		if ac.ID != "autocannon" {
			t.Errorf("autocannon ID: got %q, want %q", ac.ID, "autocannon")
		}
		if dmg, ok := ac.Attributes["damage"]; !ok || dmg.Base != 8 {
			t.Errorf("autocannon damage: got %+v", ac.Attributes["damage"])
		}
		if len(ac.Triggers) != 2 {
			t.Errorf("autocannon triggers: got %d, want 2", len(ac.Triggers))
		}
		if len(ac.Requirements) != 1 {
			t.Errorf("autocannon requirements: got %d, want 1", len(ac.Requirements))
		}
	}

	// Check lrm_rack loaded
	lrm, ok := reg.GetItem("lrm_rack")
	if !ok {
		t.Error("lrm_rack not registered")
	} else {
		if lrm.ID != "lrm_rack" {
			t.Errorf("lrm_rack ID: got %q, want %q", lrm.ID, "lrm_rack")
		}
		if splash, ok := lrm.Attributes["splash"]; !ok || splash.Base != 2 {
			t.Errorf("lrm_rack splash: got %+v", lrm.Attributes["splash"])
		}
	}
}

func TestLoadUnitsFromDir_EmptyDir(t *testing.T) {
	// Create empty temp directory
	dir, err := os.MkdirTemp("", "empty_units")
	if err != nil {
		t.Fatalf("create temp dir: %v", err)
	}
	defer os.RemoveAll(dir)

	reg := NewRegistry()
	err = LoadUnitsFromDir(dir, reg)
	if err != nil {
		t.Errorf("LoadUnitsFromDir on empty dir: %v", err)
	}
}

func TestLoadUnitsFromDir_InvalidKDL(t *testing.T) {
	// Create temp directory with invalid KDL
	dir, err := os.MkdirTemp("", "invalid_units")
	if err != nil {
		t.Fatalf("create temp dir: %v", err)
	}
	defer os.RemoveAll(dir)

	// Write invalid KDL file
	invalidKDL := `unit id="broken" { this is not valid kdl!!!`
	if err := os.WriteFile(filepath.Join(dir, "broken.kdl"), []byte(invalidKDL), 0644); err != nil {
		t.Fatalf("write invalid KDL: %v", err)
	}

	reg := NewRegistry()
	err = LoadUnitsFromDir(dir, reg)
	if err == nil {
		t.Error("expected error for invalid KDL, got nil")
	}
}

func TestLoadUnitsFromDir_MissingID(t *testing.T) {
	// Create temp directory with KDL missing ID
	dir, err := os.MkdirTemp("", "missing_id_units")
	if err != nil {
		t.Fatalf("create temp dir: %v", err)
	}
	defer os.RemoveAll(dir)

	// Write KDL file with missing ID
	missingIDKDL := `unit {
		tags "mech"
	}`
	if err := os.WriteFile(filepath.Join(dir, "missing.kdl"), []byte(missingIDKDL), 0644); err != nil {
		t.Fatalf("write missing ID KDL: %v", err)
	}

	reg := NewRegistry()
	err = LoadUnitsFromDir(dir, reg)
	if err == nil {
		t.Error("expected error for missing ID, got nil")
	}
}

func TestParseError_Format(t *testing.T) {
	tests := []struct {
		err      ParseError
		expected string
	}{
		{
			ParseError{File: "test.kdl", Node: "unit", Field: "id", Message: "missing"},
			"test.kdl: unit.id: missing",
		},
		{
			ParseError{File: "test.kdl", Node: "unit", Message: "invalid structure"},
			"test.kdl: unit: invalid structure",
		},
		{
			ParseError{File: "test.kdl", Message: "parse error"},
			"test.kdl: parse error",
		},
	}

	for _, tt := range tests {
		result := tt.err.Error()
		if result != tt.expected {
			t.Errorf("ParseError.Error(): got %q, want %q", result, tt.expected)
		}
	}
}
