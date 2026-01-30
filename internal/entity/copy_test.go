package entity

import (
	"testing"

	"wulfaz/internal/core"
)

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
			{Event: core.EventOnCombatTick, EffectName: "fire", Params: map[string]any{"dmg": 5}},
		},
		ProvidedModifiers: []core.ProvidedModifier{
			{Scope: core.ScopeUnit, Attribute: "speed", Value: 2},
		},
		Requirements: []core.Requirement{
			{Scope: "unit", Condition: core.Condition{Type: core.ConditionHasTag, Params: map[string]any{"tag": "mech"}}},
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

	if orig.Tags[0] != "mech" {
		t.Error("original Tags was mutated")
	}
	if orig.Attributes["health"].Base != 100 {
		t.Error("original Attributes was mutated")
	}
	if orig.Parts["torso"].Tags[0] != "center" {
		t.Error("original Parts was mutated")
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
}
