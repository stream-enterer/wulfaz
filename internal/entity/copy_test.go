package entity

import (
	"reflect"
	"testing"

	"wulfaz/internal/core"
)

// Round-trip tests: verify all fields are copied by comparing original to copy.
// If a new field is added to a struct but not to its Copy function, these fail.

func TestCopyMountCriteria_RoundTrip(t *testing.T) {
	orig := MountCriteria{
		RequiresAll: []core.Tag{"weapon", "energy"},
		RequiresAny: []core.Tag{"small", "medium"},
		Forbids:     []core.Tag{"missile", "explosive"},
	}
	copied := CopyMountCriteria(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyMountCriteria round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyItem_RoundTrip(t *testing.T) {
	orig := Item{
		ID:         "original_id",
		TemplateID: "laser_rifle_template",
		Tags:       []core.Tag{"weapon", "energy", "rifle"},
		Attributes: map[string]core.Attribute{
			"damage": {Name: "damage", Base: 15, Min: 0, Max: 100},
			"range":  {Name: "range", Base: 8, Min: 1, Max: 20},
		},
		Triggers: []core.Trigger{
			{
				Event:            core.EventOnDamaged,
				Conditions:       []core.Condition{{Type: core.ConditionHasTag, Params: map[string]any{"tag": "active"}}},
				TargetConditions: []core.Condition{{Type: core.ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 1}}},
				EffectName:       "fire_laser",
				Params:           map[string]any{"damage": 15},
				Priority:         1,
			},
		},
		Abilities: []core.Ability{
			{ID: "overcharge", Tags: []core.Tag{"active"}, Cooldown: 3},
		},
		ProvidedModifiers: []core.ProvidedModifier{
			{Scope: core.ScopeUnit, Attribute: "accuracy", Operation: core.ModifierOpAdd, Value: 2},
		},
		Requirements: []core.Requirement{
			{Scope: core.ScopeMount, Condition: core.Condition{Type: core.ConditionHasTag, Params: map[string]any{"tag": "arm"}}},
		},
	}

	newID := "copied_id"
	copied := CopyItem(orig, newID)

	// Verify ID was changed as expected
	if copied.ID != newID {
		t.Errorf("expected copied ID %q, got %q", newID, copied.ID)
	}

	// Compare all other fields by temporarily setting ID
	copied.ID = orig.ID
	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyItem round-trip failed (excluding ID)\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyMount_RoundTrip(t *testing.T) {
	orig := Mount{
		ID:   "arm_mount",
		Tags: []core.Tag{"arm", "weapon_mount"},
		Accepts: MountCriteria{
			RequiresAll: []core.Tag{"weapon"},
			RequiresAny: []core.Tag{"energy", "ballistic"},
			Forbids:     []core.Tag{"heavy"},
		},
		Capacity:          10,
		CapacityAttribute: "weight",
		MaxItems:          2,
		Locked:            true,
		Contents: []Item{
			{ID: "laser1", TemplateID: "laser", Tags: []core.Tag{"weapon", "energy"}},
		},
	}
	copied := CopyMount(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyMount round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyPart_RoundTrip(t *testing.T) {
	orig := Part{
		ID:         "left_arm",
		TemplateID: "standard_arm",
		Tags:       []core.Tag{"arm", "left", "limb"},
		Attributes: map[string]core.Attribute{
			"armor":  {Name: "armor", Base: 20, Min: 0, Max: 50},
			"health": {Name: "health", Base: 50, Min: 0, Max: 100},
		},
		Mounts: []Mount{
			{ID: "hand", Capacity: 5, Tags: []core.Tag{"hand"}},
		},
		Connections: map[string][]string{
			"torso": {"center_torso"},
			"other": {"shoulder"},
		},
		Triggers: []core.Trigger{
			{Event: core.EventOnDamaged, EffectName: "spark_effect", Priority: 2},
		},
		Abilities: []core.Ability{
			{ID: "punch", Tags: []core.Tag{"melee"}, Cooldown: 1},
		},
	}
	copied := CopyPart(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyPart round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyPilot_RoundTrip(t *testing.T) {
	orig := Pilot{
		ID:   "pilot_001",
		Name: "Commander Rex",
	}
	copied := CopyPilot(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyPilot round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyDie_RoundTrip(t *testing.T) {
	orig := Die{
		Faces: []DieFace{
			{Type: DieDamage, Value: 2},
			{Type: DieDamage, Value: 2},
			{Type: DieDamage, Value: 3},
			{Type: DieDamage, Value: 4},
			{Type: DieBlank, Value: 0},
			{Type: DieBlank, Value: 0},
		},
	}
	copied := CopyDie(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyDie round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyDie_Independence(t *testing.T) {
	orig := Die{
		Faces: []DieFace{
			{Type: DieDamage, Value: 2},
			{Type: DieDamage, Value: 2},
			{Type: DieDamage, Value: 3},
			{Type: DieDamage, Value: 4},
			{Type: DieBlank, Value: 0},
			{Type: DieBlank, Value: 0},
		},
	}
	copied := CopyDie(orig)

	copied.Faces[0] = DieFace{Type: DieDamage, Value: 999}

	if orig.Faces[0].Value != 2 {
		t.Error("original Faces was mutated")
	}
}

func TestCopyDice_Nil(t *testing.T) {
	result := CopyDice(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyDie_NilFaces(t *testing.T) {
	orig := Die{}
	copied := CopyDie(orig)

	if copied.Faces != nil {
		t.Error("expected nil Faces")
	}
}

func TestCopyUnit_RoundTrip(t *testing.T) {
	orig := Unit{
		ID:         "unit_original",
		TemplateID: "assault_mech",
		Tags:       []core.Tag{"mech", "assault", "heavy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 200, Min: 0, Max: 500},
			"speed":  {Name: "speed", Base: 4, Min: 1, Max: 10},
		},
		Parts: map[string]Part{
			"torso": {ID: "center_torso", Tags: []core.Tag{"torso", "center"}},
			"arm":   {ID: "left_arm", Tags: []core.Tag{"arm", "left"}},
		},
		Triggers: []core.Trigger{
			{Event: core.EventOnTurnStart, EffectName: "cooldown_vent", Priority: 0},
		},
		Abilities: []core.Ability{
			{ID: "jump_jets", Tags: []core.Tag{"movement"}, Charges: 2},
		},
		Dice: []Die{
			{Faces: []DieFace{
				{Type: DieDamage, Value: 2},
				{Type: DieDamage, Value: 2},
				{Type: DieDamage, Value: 3},
				{Type: DieDamage, Value: 4},
				{Type: DieBlank, Value: 0},
				{Type: DieBlank, Value: 0},
			}},
			{Faces: []DieFace{
				{Type: DieShield, Value: 1},
				{Type: DieShield, Value: 1},
				{Type: DieShield, Value: 2},
				{Type: DieShield, Value: 3},
				{Type: DieBlank, Value: 0},
				{Type: DieBlank, Value: 0},
			}},
		},
		Pilot:    Pilot{ID: "pilot_001", Name: "Commander Rex"},
		HasPilot: true,
	}

	newID := "unit_copied"
	copied := CopyUnit(orig, newID)

	// Verify ID was changed as expected
	if copied.ID != newID {
		t.Errorf("expected copied ID %q, got %q", newID, copied.ID)
	}

	// Compare all other fields by temporarily setting ID
	copied.ID = orig.ID
	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyUnit round-trip failed (excluding ID)\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyConnections_RoundTrip(t *testing.T) {
	orig := map[string][]string{
		"left":   {"torso", "shoulder"},
		"right":  {"torso"},
		"center": {},
	}
	copied := CopyConnections(orig)

	if !reflect.DeepEqual(orig, copied) {
		t.Errorf("CopyConnections round-trip failed\norig:   %+v\ncopied: %+v", orig, copied)
	}
}

func TestCopyMountCriteria_Independence(t *testing.T) {
	orig := MountCriteria{
		RequiresAll: []core.Tag{"weapon"},
		RequiresAny: []core.Tag{"energy", "ballistic"},
		Forbids:     []core.Tag{"missile"},
	}
	copied := CopyMountCriteria(orig)

	copied.RequiresAll[0] = "modified"
	copied.RequiresAny[0] = "modified"
	copied.Forbids[0] = "modified"

	if orig.RequiresAll[0] != "weapon" {
		t.Error("original RequiresAll was mutated")
	}
	if orig.RequiresAny[0] != "energy" {
		t.Error("original RequiresAny was mutated")
	}
	if orig.Forbids[0] != "missile" {
		t.Error("original Forbids was mutated")
	}
}

func TestCopyItem_Independence(t *testing.T) {
	orig := Item{
		ID:         "test_item",
		TemplateID: "template",
		Tags:       []core.Tag{"weapon"},
		Attributes: map[string]core.Attribute{
			"damage": {Name: "damage", Base: 10},
		},
		Triggers: []core.Trigger{
			{Event: core.EventOnDamaged, EffectName: "fire", Params: map[string]any{"dmg": 5}},
		},
		ProvidedModifiers: []core.ProvidedModifier{
			{Scope: core.ScopeUnit, Attribute: "speed", Value: 2},
		},
		Requirements: []core.Requirement{
			{Scope: core.ScopeUnit, Condition: core.Condition{Type: core.ConditionHasTag, Params: map[string]any{"tag": "mech"}}},
		},
	}
	copied := CopyItem(orig, "new_id")

	// Verify new ID
	if copied.ID != "new_id" {
		t.Errorf("expected ID 'new_id', got %q", copied.ID)
	}

	// Mutate copied values
	copied.Tags[0] = "modified"
	copied.Attributes["damage"] = core.Attribute{Name: "damage", Base: 999}
	copied.Triggers[0].Params["dmg"] = 999
	copied.Requirements[0].Condition.Params["tag"] = "modified"

	if orig.Tags[0] != "weapon" {
		t.Error("original Tags was mutated")
	}
	if orig.Attributes["damage"].Base != 10 {
		t.Error("original Attributes was mutated")
	}
	if orig.Triggers[0].Params["dmg"] != 5 {
		t.Error("original Triggers was mutated")
	}
	if orig.Requirements[0].Condition.Params["tag"] != "mech" {
		t.Error("original Requirements was mutated")
	}
}

func TestCopyItem_NilSlices(t *testing.T) {
	orig := Item{
		ID:         "minimal",
		TemplateID: "template",
		Tags:       nil,
		Attributes: nil,
		Triggers:   nil,
	}
	copied := CopyItem(orig, "new_id")

	if copied.Tags != nil {
		t.Error("expected nil Tags")
	}
	if copied.Attributes != nil {
		t.Error("expected nil Attributes")
	}
	if copied.Triggers != nil {
		t.Error("expected nil Triggers")
	}
}

func TestCopyMount_Contents(t *testing.T) {
	orig := Mount{
		ID:       "test_mount",
		Tags:     []core.Tag{"arm"},
		Capacity: 5,
		Accepts: MountCriteria{
			RequiresAny: []core.Tag{"weapon"},
		},
		Contents: []Item{
			{ID: "item1", Tags: []core.Tag{"weapon"}},
		},
	}
	copied := CopyMount(orig)

	// Mutate copied values
	copied.Tags[0] = "modified"
	copied.Accepts.RequiresAny[0] = "modified"
	copied.Contents[0].Tags[0] = "modified"

	if orig.Tags[0] != "arm" {
		t.Error("original Tags was mutated")
	}
	if orig.Accepts.RequiresAny[0] != "weapon" {
		t.Error("original Accepts was mutated")
	}
	if orig.Contents[0].Tags[0] != "weapon" {
		t.Error("original Contents was mutated")
	}
}

func TestCopyMounts_Nil(t *testing.T) {
	result := CopyMounts(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyConnections_Independence(t *testing.T) {
	orig := map[string][]string{
		"left":  {"torso", "head"},
		"right": {"torso"},
	}
	copied := CopyConnections(orig)

	copied["left"][0] = "modified"

	if orig["left"][0] != "torso" {
		t.Error("original was mutated")
	}
}

func TestCopyConnections_Nil(t *testing.T) {
	result := CopyConnections(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyPart_AllFields(t *testing.T) {
	orig := Part{
		ID:         "test_part",
		TemplateID: "template",
		Tags:       []core.Tag{"arm"},
		Attributes: map[string]core.Attribute{
			"armor": {Name: "armor", Base: 20},
		},
		Mounts: []Mount{
			{ID: "hand", Capacity: 2, Contents: []Item{{ID: "laser"}}},
		},
		Connections: map[string][]string{
			"torso": {"center"},
		},
		Triggers: []core.Trigger{
			{Event: core.EventOnDamaged, EffectName: "spark"},
		},
	}
	copied := CopyPart(orig)

	// Mutate copied values
	copied.Tags[0] = "modified"
	copied.Attributes["armor"] = core.Attribute{Name: "armor", Base: 999}
	copied.Mounts[0].Contents[0].ID = "modified"
	copied.Connections["torso"][0] = "modified"

	if orig.Tags[0] != "arm" {
		t.Error("original Tags was mutated")
	}
	if orig.Attributes["armor"].Base != 20 {
		t.Error("original Attributes was mutated")
	}
	if orig.Mounts[0].Contents[0].ID != "laser" {
		t.Error("original Mounts was mutated")
	}
	if orig.Connections["torso"][0] != "center" {
		t.Error("original Connections was mutated")
	}
}

func TestCopyParts_Nil(t *testing.T) {
	result := CopyParts(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyUnit_Independence(t *testing.T) {
	orig := Unit{
		ID:         "test",
		TemplateID: "template",
		Tags:       []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100},
		},
		Parts: map[string]Part{
			"torso": {ID: "torso", Tags: []core.Tag{"center"}},
		},
		Triggers: []core.Trigger{
			{Event: core.EventOnTurnStart, EffectName: "regen"},
		},
		Dice: []Die{
			{Faces: []DieFace{
				{Type: DieDamage, Value: 1},
				{Type: DieDamage, Value: 2},
				{Type: DieDamage, Value: 3},
			}},
		},
		Pilot:    Pilot{ID: "pilot1", Name: "Test Pilot"},
		HasPilot: true,
	}
	copied := CopyUnit(orig, "copy")

	// Verify new ID
	if copied.ID != "copy" {
		t.Errorf("expected ID 'copy', got %q", copied.ID)
	}

	// Mutate copied values
	copied.Tags[0] = "modified"
	copied.Attributes["health"] = core.Attribute{Name: "health", Base: 999}
	copiedPart := copied.Parts["torso"]
	copiedPart.Tags[0] = "modified"
	copied.Parts["torso"] = copiedPart
	copied.Dice[0].Faces[0] = DieFace{Type: DieDamage, Value: 999}

	if orig.Tags[0] != "mech" {
		t.Error("original Tags was mutated")
	}
	if orig.Attributes["health"].Base != 100 {
		t.Error("original Attributes was mutated")
	}
	if orig.Parts["torso"].Tags[0] != "center" {
		t.Error("original Parts was mutated")
	}
	if orig.Dice[0].Faces[0].Value != 1 {
		t.Error("original Dice was mutated")
	}
}

func TestCopyUnit_NilSlices(t *testing.T) {
	orig := Unit{
		ID:         "minimal",
		TemplateID: "template",
		Tags:       nil,
		Attributes: nil,
		Parts:      nil,
		Triggers:   nil,
		Dice:       nil,
	}
	copied := CopyUnit(orig, "new_id")

	if copied.Tags != nil {
		t.Error("expected nil Tags")
	}
	if copied.Attributes != nil {
		t.Error("expected nil Attributes")
	}
	if copied.Parts != nil {
		t.Error("expected nil Parts")
	}
	if copied.Triggers != nil {
		t.Error("expected nil Triggers")
	}
	if copied.Dice != nil {
		t.Error("expected nil Dice")
	}
}
