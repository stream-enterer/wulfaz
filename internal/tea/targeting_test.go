package tea

import (
	"math/rand"
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

// ===== AnyAliveUnits Tests =====

func TestAnyAliveUnits_AllDead(t *testing.T) {
	units := []entity.Unit{
		{
			ID: "dead1",
			Attributes: map[string]core.Attribute{
				"health": {Base: 0},
			},
		},
		{
			ID: "dead2",
			Attributes: map[string]core.Attribute{
				"health": {Base: 0},
			},
		},
	}

	result := AnyAliveUnits(units)

	if result {
		t.Error("AnyAliveUnits() = true, want false (all dead)")
	}
}

func TestAnyAliveUnits_SomeAlive(t *testing.T) {
	units := []entity.Unit{
		{
			ID: "dead",
			Attributes: map[string]core.Attribute{
				"health": {Base: 0},
			},
		},
		{
			ID: "alive",
			Attributes: map[string]core.Attribute{
				"health": {Base: 50},
			},
		},
	}

	result := AnyAliveUnits(units)

	if !result {
		t.Error("AnyAliveUnits() = false, want true (some alive)")
	}
}

func TestAnyAliveUnits_OnlyCommandAlive(t *testing.T) {
	units := []entity.Unit{
		{
			ID:   "cmd",
			Tags: []core.Tag{"command"}, // Command unit doesn't count
			Attributes: map[string]core.Attribute{
				"health": {Base: 100},
			},
		},
		{
			ID: "dead_unit",
			Attributes: map[string]core.Attribute{
				"health": {Base: 0},
			},
		},
	}

	result := AnyAliveUnits(units)

	if result {
		t.Error("AnyAliveUnits() = true, want false (command doesn't count)")
	}
}

func TestAnyAliveUnits_Empty(t *testing.T) {
	var units []entity.Unit
	result := AnyAliveUnits(units)

	if result {
		t.Error("AnyAliveUnits(empty) = true, want false")
	}
}

// ===== CanTargetUnit Tests (F-167) =====

func TestCanTargetUnit_DeadUnit(t *testing.T) {
	target := entity.Unit{
		ID: "dead",
		Attributes: map[string]core.Attribute{
			"health": {Base: 0},
		},
	}
	allEnemies := []entity.Unit{target}

	result := CanTargetUnit(target, allEnemies)

	if result {
		t.Error("CanTargetUnit(dead) = true, want false")
	}
}

func TestCanTargetUnit_AliveRegular(t *testing.T) {
	target := entity.Unit{
		ID: "alive",
		Attributes: map[string]core.Attribute{
			"health": {Base: 50},
		},
	}
	allEnemies := []entity.Unit{target}

	result := CanTargetUnit(target, allEnemies)

	if !result {
		t.Error("CanTargetUnit(alive regular) = false, want true")
	}
}

func TestCanTargetUnit_CommandProtected(t *testing.T) {
	cmd := entity.Unit{
		ID:   "cmd",
		Tags: []core.Tag{"command"},
		Attributes: map[string]core.Attribute{
			"health": {Base: 100},
		},
	}
	regular := entity.Unit{
		ID: "regular",
		Attributes: map[string]core.Attribute{
			"health": {Base: 50},
		},
	}
	allEnemies := []entity.Unit{cmd, regular}

	result := CanTargetUnit(cmd, allEnemies)

	if result {
		t.Error("CanTargetUnit(command with live regular) = true, want false (F-167)")
	}
}

func TestCanTargetUnit_CommandTargetableWhenAlone(t *testing.T) {
	cmd := entity.Unit{
		ID:   "cmd",
		Tags: []core.Tag{"command"},
		Attributes: map[string]core.Attribute{
			"health": {Base: 100},
		},
	}
	deadRegular := entity.Unit{
		ID: "dead_regular",
		Attributes: map[string]core.Attribute{
			"health": {Base: 0}, // Dead
		},
	}
	allEnemies := []entity.Unit{cmd, deadRegular}

	result := CanTargetUnit(cmd, allEnemies)

	if !result {
		t.Error("CanTargetUnit(command with all regular dead) = false, want true")
	}
}

// ===== GetValidEnemyTargets Tests =====

func TestGetValidEnemyTargets_Basic(t *testing.T) {
	enemies := []entity.Unit{
		{ID: "alive1", Attributes: map[string]core.Attribute{"health": {Base: 50}}},
		{ID: "dead", Attributes: map[string]core.Attribute{"health": {Base: 0}}},
		{ID: "alive2", Attributes: map[string]core.Attribute{"health": {Base: 30}}},
	}

	result := GetValidEnemyTargets(enemies)

	if len(result) != 2 {
		t.Fatalf("expected 2 valid targets, got %d", len(result))
	}
}

func TestGetValidEnemyTargets_F167_ProtectsCommand(t *testing.T) {
	enemies := []entity.Unit{
		{ID: "cmd", Tags: []core.Tag{"command"}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
		{ID: "regular", Attributes: map[string]core.Attribute{"health": {Base: 50}}},
	}

	result := GetValidEnemyTargets(enemies)

	if len(result) != 1 {
		t.Fatalf("expected 1 valid target, got %d", len(result))
	}
	if result[0].ID != "regular" {
		t.Errorf("expected regular, got %s", result[0].ID)
	}
}

func TestGetValidEnemyTargets_F167_CommandTargetableWhenAlone(t *testing.T) {
	enemies := []entity.Unit{
		{ID: "cmd", Tags: []core.Tag{"command"}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
		{ID: "dead", Attributes: map[string]core.Attribute{"health": {Base: 0}}},
	}

	result := GetValidEnemyTargets(enemies)

	if len(result) != 1 {
		t.Fatalf("expected 1 valid target, got %d", len(result))
	}
	if result[0].ID != "cmd" {
		t.Errorf("expected cmd, got %s", result[0].ID)
	}
}

// ===== GetValidAlliedTargets Tests =====

func TestGetValidAlliedTargets_Basic(t *testing.T) {
	allies := []entity.Unit{
		{ID: "alive1", Attributes: map[string]core.Attribute{"health": {Base: 50}}},
		{ID: "dead", Attributes: map[string]core.Attribute{"health": {Base: 0}}},
		{ID: "alive2", Attributes: map[string]core.Attribute{"health": {Base: 30}}},
	}

	result := GetValidAlliedTargets(allies)

	if len(result) != 2 {
		t.Fatalf("expected 2 valid allies, got %d", len(result))
	}
}

func TestGetValidAlliedTargets_IncludesCommand(t *testing.T) {
	// Unlike enemy targets, allied targets for heal/shield can include command
	allies := []entity.Unit{
		{ID: "cmd", Tags: []core.Tag{"command"}, Attributes: map[string]core.Attribute{"health": {Base: 100}}},
		{ID: "regular", Attributes: map[string]core.Attribute{"health": {Base: 50}}},
	}

	result := GetValidAlliedTargets(allies)

	if len(result) != 2 {
		t.Fatalf("expected 2 valid allies (including command), got %d", len(result))
	}
}

// ===== SelectLowestHP Tests =====

func TestSelectLowestHP_Basic(t *testing.T) {
	units := []entity.Unit{
		{ID: "high", Attributes: map[string]core.Attribute{"health": {Base: 100}}},
		{ID: "low", Attributes: map[string]core.Attribute{"health": {Base: 20}}},
		{ID: "mid", Attributes: map[string]core.Attribute{"health": {Base: 50}}},
	}

	result := SelectLowestHP(units)

	if result != "low" {
		t.Errorf("SelectLowestHP() = %s, want low", result)
	}
}

func TestSelectLowestHP_TieUsesFirst(t *testing.T) {
	units := []entity.Unit{
		{ID: "first", Attributes: map[string]core.Attribute{"health": {Base: 30}}},
		{ID: "second", Attributes: map[string]core.Attribute{"health": {Base: 30}}},
	}

	result := SelectLowestHP(units)

	if result != "first" {
		t.Errorf("SelectLowestHP() = %s, want first (tie)", result)
	}
}

func TestSelectLowestHP_Empty(t *testing.T) {
	var units []entity.Unit
	result := SelectLowestHP(units)

	if result != "" {
		t.Errorf("SelectLowestHP(empty) = %s, want empty", result)
	}
}

// ===== SelectRandomTarget Tests =====

func TestSelectRandomTarget_Basic(t *testing.T) {
	units := []entity.Unit{
		{ID: "a"},
		{ID: "b"},
		{ID: "c"},
	}
	rng := rand.New(rand.NewSource(42))

	result := SelectRandomTarget(units, rng)

	// Should return one of the IDs
	valid := result == "a" || result == "b" || result == "c"
	if !valid {
		t.Errorf("SelectRandomTarget() = %s, want one of a/b/c", result)
	}
}

func TestSelectRandomTarget_Empty(t *testing.T) {
	var units []entity.Unit
	rng := rand.New(rand.NewSource(42))

	result := SelectRandomTarget(units, rng)

	if result != "" {
		t.Errorf("SelectRandomTarget(empty) = %s, want empty", result)
	}
}

// ===== FilterDoomedTargets Tests =====

func TestFilterDoomedTargets_NoIncoming(t *testing.T) {
	enemies := []entity.Unit{
		{ID: "a", Attributes: map[string]core.Attribute{"health": {Base: 50}}},
		{ID: "b", Attributes: map[string]core.Attribute{"health": {Base: 30}}},
	}
	incoming := map[string]int{}
	combat := model.CombatModel{}

	result := FilterDoomedTargets(enemies, incoming, combat)

	if len(result) != 2 {
		t.Errorf("expected 2, got %d", len(result))
	}
}

func TestFilterDoomedTargets_FiltersDoomedUnits(t *testing.T) {
	enemies := []entity.Unit{
		{ID: "doomed", Attributes: map[string]core.Attribute{"health": {Base: 30}}},
		{ID: "safe", Attributes: map[string]core.Attribute{"health": {Base: 50}}},
	}
	incoming := map[string]int{
		"doomed": 30, // Exactly lethal
	}
	combat := model.CombatModel{}

	result := FilterDoomedTargets(enemies, incoming, combat)

	if len(result) != 1 {
		t.Fatalf("expected 1, got %d", len(result))
	}
	if result[0].ID != "safe" {
		t.Errorf("expected safe, got %s", result[0].ID)
	}
}

func TestFilterDoomedTargets_ShieldsProtect(t *testing.T) {
	enemies := []entity.Unit{
		{ID: "shielded", Attributes: map[string]core.Attribute{
			"health":  {Base: 30},
			"shields": {Base: 20},
		}},
	}
	incoming := map[string]int{
		"shielded": 30, // Only 30 damage, but 50 effective HP
	}
	combat := model.CombatModel{}

	result := FilterDoomedTargets(enemies, incoming, combat)

	if len(result) != 1 {
		t.Fatalf("expected 1 (shields protect), got %d", len(result))
	}
}

func TestFilterDoomedTargets_AllDoomedReturnsOriginal(t *testing.T) {
	enemies := []entity.Unit{
		{ID: "a", Attributes: map[string]core.Attribute{"health": {Base: 10}}},
		{ID: "b", Attributes: map[string]core.Attribute{"health": {Base: 10}}},
	}
	incoming := map[string]int{
		"a": 20,
		"b": 20,
	}
	combat := model.CombatModel{}

	result := FilterDoomedTargets(enemies, incoming, combat)

	// When all are doomed, return original list
	if len(result) != 2 {
		t.Errorf("expected 2 (all doomed returns original), got %d", len(result))
	}
}
