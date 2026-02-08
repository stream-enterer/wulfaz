package template

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

func TestInstantiateUnit_Success(t *testing.T) {
	reg := NewRegistry()
	reg.RegisterUnit("test_mech", entity.Unit{
		ID:         "test_mech",
		TemplateID: "test_mech",
		Tags:       []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100},
		},
	})

	unit, err := InstantiateUnit(reg, "test_mech", "instance_1")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if unit.ID != "instance_1" {
		t.Errorf("expected ID 'instance_1', got %q", unit.ID)
	}
	if unit.TemplateID != "test_mech" {
		t.Errorf("expected TemplateID 'test_mech', got %q", unit.TemplateID)
	}
	if len(unit.Tags) != 1 || unit.Tags[0] != "mech" {
		t.Errorf("expected Tags ['mech'], got %v", unit.Tags)
	}
}

func TestInstantiateUnit_NotFound(t *testing.T) {
	reg := NewRegistry()

	_, err := InstantiateUnit(reg, "nonexistent", "instance_1")
	if err == nil {
		t.Fatal("expected error for nonexistent template")
	}
}

func TestInstantiateUnit_Independence(t *testing.T) {
	reg := NewRegistry()
	reg.RegisterUnit("test_mech", entity.Unit{
		ID:         "test_mech",
		TemplateID: "test_mech",
		Tags:       []core.Tag{"mech"},
	})

	unit1, _ := InstantiateUnit(reg, "test_mech", "instance_1")
	unit2, _ := InstantiateUnit(reg, "test_mech", "instance_2")

	// Mutate one instance
	unit1.Tags[0] = "modified"

	// Check other instance is unchanged
	if unit2.Tags[0] != "mech" {
		t.Error("instance_2 was mutated when instance_1 was modified")
	}

	// Check template is unchanged
	tmpl, _ := reg.GetUnit("test_mech")
	if tmpl.Tags[0] != "mech" {
		t.Error("template was mutated when instance was modified")
	}
}
