package entity

import (
	"reflect"
	"testing"

	"wulfaz/internal/core"
)

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
		Dice: []Die{{Faces: []DieFace{
			{Type: DieDamage, Value: 2},
			{Type: DieDamage, Value: 2},
			{Type: DieDamage, Value: 3},
			{Type: DieDamage, Value: 4},
			{Type: DieBlank, Value: 0},
			{Type: DieBlank, Value: 0},
		}}},
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

func TestCopyUnit_Independence(t *testing.T) {
	orig := Unit{
		ID:         "test",
		TemplateID: "template",
		Tags:       []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100},
		},
		Dice: []Die{{Faces: []DieFace{
			{Type: DieDamage, Value: 1},
			{Type: DieDamage, Value: 2},
			{Type: DieDamage, Value: 3},
		}}},
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
	copied.Dice[0].Faces[0] = DieFace{Type: DieDamage, Value: 999}

	if orig.Tags[0] != "mech" {
		t.Error("original Tags was mutated")
	}
	if orig.Attributes["health"].Base != 100 {
		t.Error("original Attributes was mutated")
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
		Dice:       nil,
	}
	copied := CopyUnit(orig, "new_id")

	if copied.Tags != nil {
		t.Error("expected nil Tags")
	}
	if copied.Attributes != nil {
		t.Error("expected nil Attributes")
	}
	if copied.Dice != nil {
		t.Error("expected nil Dice")
	}
}

func TestCopyRolledDiceSlice_Independence(t *testing.T) {
	orig := []RolledDie{
		{
			Faces:     []DieFace{{Type: DieDamage, Value: 3}, {Type: DieBlank, Value: 0}},
			FaceIndex: 0,
			Locked:    false,
			Fired:     false,
		},
		{
			Faces:     []DieFace{{Type: DieShield, Value: 2}, {Type: DieHeal, Value: 1}},
			FaceIndex: 1,
			Locked:    true,
			Fired:     true,
		},
	}
	copied := CopyRolledDiceSlice(orig)

	// Verify round-trip
	if len(copied) != len(orig) {
		t.Fatalf("expected %d dice, got %d", len(orig), len(copied))
	}

	// Mutate copied values
	copied[0].Fired = true
	copied[1].Fired = false
	copied[0].Faces[0] = DieFace{Type: DieDamage, Value: 999}

	if orig[0].Fired != false {
		t.Error("original Fired[0] was mutated")
	}
	if orig[1].Fired != true {
		t.Error("original Fired[1] was mutated")
	}
	if orig[0].Faces[0].Value != 3 {
		t.Error("original Faces was mutated")
	}
}
