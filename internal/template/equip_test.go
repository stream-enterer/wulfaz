package template

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

func makeTestUnit() entity.Unit {
	return entity.Unit{
		ID:         "test_unit",
		TemplateID: "test_template",
		Tags:       []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100},
		},
		Parts: map[string]entity.Part{
			"right_arm": {
				ID:   "right_arm",
				Tags: []core.Tag{"arm", "right"},
				Mounts: []entity.Mount{
					{
						ID:       "hand",
						Capacity: 3,
						MaxItems: -1,
						Accepts: entity.MountCriteria{
							RequiresAny: []core.Tag{"weapon"},
						},
					},
				},
			},
			"torso": {
				ID:   "torso",
				Tags: []core.Tag{"torso"},
				Mounts: []entity.Mount{
					{
						ID:       "core",
						Capacity: 4,
						MaxItems: 2,
						Accepts: entity.MountCriteria{
							RequiresAny: []core.Tag{"equipment"},
						},
					},
				},
			},
		},
	}
}

func makeTestWeapon() entity.Item {
	return entity.Item{
		ID:         "laser",
		TemplateID: "medium_laser",
		Tags:       []core.Tag{"weapon", "energy"},
		Attributes: map[string]core.Attribute{
			"size":   {Name: "size", Base: 1},
			"damage": {Name: "damage", Base: 5},
		},
	}
}

func TestEquipItem_Success(t *testing.T) {
	unit := makeTestUnit()
	weapon := makeTestWeapon()

	newUnit, err := EquipItem(unit, "right_arm", 0, weapon)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Verify item was equipped
	mount := newUnit.Parts["right_arm"].Mounts[0]
	if len(mount.Contents) != 1 {
		t.Fatalf("expected 1 item in mount, got %d", len(mount.Contents))
	}
	if mount.Contents[0].ID != "laser" {
		t.Errorf("expected item ID 'laser', got %q", mount.Contents[0].ID)
	}

	// Verify original unit was not mutated
	origMount := unit.Parts["right_arm"].Mounts[0]
	if len(origMount.Contents) != 0 {
		t.Error("original unit was mutated")
	}
}

func TestEquipItem_PartNotFound(t *testing.T) {
	unit := makeTestUnit()
	weapon := makeTestWeapon()

	_, err := EquipItem(unit, "nonexistent", 0, weapon)
	if err == nil {
		t.Fatal("expected error for nonexistent part")
	}
}

func TestEquipItem_MountIndexOutOfRange(t *testing.T) {
	unit := makeTestUnit()
	weapon := makeTestWeapon()

	_, err := EquipItem(unit, "right_arm", 5, weapon)
	if err == nil {
		t.Fatal("expected error for invalid mount index")
	}
}

func TestEquipItem_MountLocked(t *testing.T) {
	unit := makeTestUnit()
	// Lock the mount
	part := unit.Parts["right_arm"]
	part.Mounts[0].Locked = true
	unit.Parts["right_arm"] = part

	weapon := makeTestWeapon()

	_, err := EquipItem(unit, "right_arm", 0, weapon)
	if err == nil {
		t.Fatal("expected error for locked mount")
	}
}

func TestEquipItem_CriteriaFail(t *testing.T) {
	unit := makeTestUnit()
	// Try to equip an equipment item to a weapon-only mount
	equipment := entity.Item{
		ID:   "reactor",
		Tags: []core.Tag{"equipment"}, // Not a weapon
		Attributes: map[string]core.Attribute{
			"size": {Name: "size", Base: 1},
		},
	}

	_, err := EquipItem(unit, "right_arm", 0, equipment)
	if err == nil {
		t.Fatal("expected error for criteria mismatch")
	}
}

func TestEquipItem_CapacityExceeded(t *testing.T) {
	unit := makeTestUnit()
	// Create a weapon that exceeds capacity
	bigWeapon := entity.Item{
		ID:   "huge_cannon",
		Tags: []core.Tag{"weapon"},
		Attributes: map[string]core.Attribute{
			"size": {Name: "size", Base: 10}, // Exceeds capacity of 3
		},
	}

	_, err := EquipItem(unit, "right_arm", 0, bigWeapon)
	if err == nil {
		t.Fatal("expected error for capacity exceeded")
	}
}

func TestEquipItem_MaxItemsExceeded(t *testing.T) {
	unit := makeTestUnit()
	equip1 := entity.Item{
		ID:         "equip1",
		Tags:       []core.Tag{"equipment"},
		Attributes: map[string]core.Attribute{"size": {Name: "size", Base: 1}},
	}
	equip2 := entity.Item{
		ID:         "equip2",
		Tags:       []core.Tag{"equipment"},
		Attributes: map[string]core.Attribute{"size": {Name: "size", Base: 1}},
	}
	equip3 := entity.Item{
		ID:         "equip3",
		Tags:       []core.Tag{"equipment"},
		Attributes: map[string]core.Attribute{"size": {Name: "size", Base: 1}},
	}

	// Equip two items (max is 2)
	var err error
	unit, err = EquipItem(unit, "torso", 0, equip1)
	if err != nil {
		t.Fatalf("failed to equip equip1: %v", err)
	}
	unit, err = EquipItem(unit, "torso", 0, equip2)
	if err != nil {
		t.Fatalf("failed to equip equip2: %v", err)
	}

	// Third item should fail
	_, err = EquipItem(unit, "torso", 0, equip3)
	if err == nil {
		t.Fatal("expected error for max items exceeded")
	}
}

func TestCanMount_RequiresAll(t *testing.T) {
	mount := entity.Mount{
		Accepts: entity.MountCriteria{
			RequiresAll: []core.Tag{"weapon", "energy"},
		},
	}

	// Has both tags
	item1 := entity.Item{Tags: []core.Tag{"weapon", "energy", "laser"}}
	if !CanMount(mount, item1) {
		t.Error("expected item with all required tags to be mountable")
	}

	// Missing one tag
	item2 := entity.Item{Tags: []core.Tag{"weapon"}}
	if CanMount(mount, item2) {
		t.Error("expected item missing required tag to not be mountable")
	}
}

func TestCanMount_RequiresAny(t *testing.T) {
	mount := entity.Mount{
		Accepts: entity.MountCriteria{
			RequiresAny: []core.Tag{"weapon", "equipment"},
		},
	}

	// Has one of the tags
	item1 := entity.Item{Tags: []core.Tag{"weapon"}}
	if !CanMount(mount, item1) {
		t.Error("expected item with one required tag to be mountable")
	}

	// Has none of the tags
	item2 := entity.Item{Tags: []core.Tag{"consumable"}}
	if CanMount(mount, item2) {
		t.Error("expected item with no required tags to not be mountable")
	}
}

func TestCanMount_Forbids(t *testing.T) {
	mount := entity.Mount{
		Accepts: entity.MountCriteria{
			Forbids: []core.Tag{"missile"},
		},
	}

	// Has forbidden tag
	item1 := entity.Item{Tags: []core.Tag{"weapon", "missile"}}
	if CanMount(mount, item1) {
		t.Error("expected item with forbidden tag to not be mountable")
	}

	// No forbidden tag
	item2 := entity.Item{Tags: []core.Tag{"weapon", "energy"}}
	if !CanMount(mount, item2) {
		t.Error("expected item without forbidden tag to be mountable")
	}
}

func TestCanMount_EmptyCriteria(t *testing.T) {
	mount := entity.Mount{
		Accepts: entity.MountCriteria{}, // Empty criteria
	}

	item := entity.Item{Tags: []core.Tag{"anything"}}
	if !CanMount(mount, item) {
		t.Error("expected any item to be mountable with empty criteria")
	}
}

func TestCanMount_CombinedCriteria(t *testing.T) {
	mount := entity.Mount{
		Accepts: entity.MountCriteria{
			RequiresAll: []core.Tag{"weapon"},
			RequiresAny: []core.Tag{"energy", "ballistic"},
			Forbids:     []core.Tag{"experimental"},
		},
	}

	// Good: weapon + energy, no experimental
	item1 := entity.Item{Tags: []core.Tag{"weapon", "energy"}}
	if !CanMount(mount, item1) {
		t.Error("expected valid item to be mountable")
	}

	// Bad: has experimental
	item2 := entity.Item{Tags: []core.Tag{"weapon", "energy", "experimental"}}
	if CanMount(mount, item2) {
		t.Error("expected item with forbidden tag to not be mountable")
	}

	// Bad: missing weapon
	item3 := entity.Item{Tags: []core.Tag{"energy"}}
	if CanMount(mount, item3) {
		t.Error("expected item missing required tag to not be mountable")
	}

	// Bad: wrong type (neither energy nor ballistic)
	item4 := entity.Item{Tags: []core.Tag{"weapon", "missile"}}
	if CanMount(mount, item4) {
		t.Error("expected item without any RequiresAny tag to not be mountable")
	}
}

func TestEquipItem_Immutability(t *testing.T) {
	unit := makeTestUnit()
	weapon1 := makeTestWeapon()
	weapon1.ID = "weapon1"
	weapon2 := makeTestWeapon()
	weapon2.ID = "weapon2"

	// Equip first weapon
	unit1, err := EquipItem(unit, "right_arm", 0, weapon1)
	if err != nil {
		t.Fatalf("failed to equip weapon1: %v", err)
	}

	// Equip second weapon to the result
	unit2, err := EquipItem(unit1, "right_arm", 0, weapon2)
	if err != nil {
		t.Fatalf("failed to equip weapon2: %v", err)
	}

	// Verify each unit has the correct items
	if len(unit.Parts["right_arm"].Mounts[0].Contents) != 0 {
		t.Error("original unit was mutated")
	}
	if len(unit1.Parts["right_arm"].Mounts[0].Contents) != 1 {
		t.Errorf("unit1 should have 1 item, got %d", len(unit1.Parts["right_arm"].Mounts[0].Contents))
	}
	if len(unit2.Parts["right_arm"].Mounts[0].Contents) != 2 {
		t.Errorf("unit2 should have 2 items, got %d", len(unit2.Parts["right_arm"].Mounts[0].Contents))
	}
}
