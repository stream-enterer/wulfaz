package tea

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

// ===== SelectTargetUnit Tests =====

func TestSelectTargetUnit_LowestHP(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 3}, // Covers 0, 1, 2
			"health":       {Base: 50},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy_30hp",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30},
			},
		},
		{
			ID:       "enemy_50hp",
			Position: 1,
			Attributes: map[string]core.Attribute{
				"health": {Base: 50},
			},
		},
		{
			ID:       "enemy_20hp",
			Position: 2,
			Attributes: map[string]core.Attribute{
				"health": {Base: 20},
			},
		},
	}

	result := SelectTargetUnit(attacker, enemies)

	if result != "enemy_20hp" {
		t.Errorf("SelectTargetUnit() = %s, want enemy_20hp (lowest HP)", result)
	}
}

func TestSelectTargetUnit_TiebreakLeftToRight(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 10}, // Covers all positions
			"health":       {Base: 50},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy_pos5",
			Position: 5,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30}, // Same HP
			},
		},
		{
			ID:       "enemy_pos3",
			Position: 3,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30}, // Same HP
			},
		},
		{
			ID:       "enemy_pos7",
			Position: 7,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30}, // Same HP
			},
		},
	}

	result := SelectTargetUnit(attacker, enemies)

	if result != "enemy_pos3" {
		t.Errorf("SelectTargetUnit() = %s, want enemy_pos3 (leftmost position)", result)
	}
}

func TestSelectTargetUnit_NoOverlap(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
			"health":       {Base: 50},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy_far",
			Position: 5,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30},
			},
		},
	}

	result := SelectTargetUnit(attacker, enemies)

	if result != "" {
		t.Errorf("SelectTargetUnit() = %s, want empty (no overlap)", result)
	}
}

func TestSelectTargetUnit_SkipsDeadUnits(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 2},
			"health":       {Base: 50},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "dead_enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 0}, // Dead - lowest HP but should be skipped
			},
		},
		{
			ID:       "alive_enemy",
			Position: 1,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30},
			},
		},
	}

	result := SelectTargetUnit(attacker, enemies)

	if result != "alive_enemy" {
		t.Errorf("SelectTargetUnit() = %s, want alive_enemy (dead units skipped)", result)
	}
}

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

// ===== ApplyDamageWithOverflow Tests =====

func TestApplyDamageWithOverflow_NoOverflow(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 50},
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"enemy": {50, 0},
	}

	results := ApplyDamageWithOverflow(attacker, 10, enemies, hpSnapshot)

	if len(results) != 1 {
		t.Fatalf("expected 1 result, got %d", len(results))
	}
	if results[0].TargetID != "enemy" {
		t.Errorf("TargetID = %s, want enemy", results[0].TargetID)
	}
	if results[0].Damage != 10 {
		t.Errorf("Damage = %d, want 10", results[0].Damage)
	}
	if results[0].NewHP != 40 {
		t.Errorf("NewHP = %d, want 40", results[0].NewHP)
	}
	if results[0].Killed {
		t.Error("Killed = true, want false")
	}
}

func TestApplyDamageWithOverflow_ExactKill(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30},
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"enemy": {30, 0},
	}

	results := ApplyDamageWithOverflow(attacker, 30, enemies, hpSnapshot)

	if len(results) != 1 {
		t.Fatalf("expected 1 result, got %d", len(results))
	}
	if results[0].NewHP != 0 {
		t.Errorf("NewHP = %d, want 0", results[0].NewHP)
	}
	if !results[0].Killed {
		t.Error("Killed = false, want true")
	}
}

func TestApplyDamageWithOverflow_SingleOverflow(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 2},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy1",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30},
			},
		},
		{
			ID:       "enemy2",
			Position: 1,
			Attributes: map[string]core.Attribute{
				"health": {Base: 40},
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"enemy1": {30, 0},
		"enemy2": {40, 0},
	}

	// 50 damage: kills enemy1 (30 HP), 20 overflow to enemy2
	results := ApplyDamageWithOverflow(attacker, 50, enemies, hpSnapshot)

	if len(results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(results))
	}

	// First target (lowest HP) should be enemy1
	if results[0].TargetID != "enemy1" {
		t.Errorf("results[0].TargetID = %s, want enemy1", results[0].TargetID)
	}
	if results[0].Damage != 30 {
		t.Errorf("results[0].Damage = %d, want 30", results[0].Damage)
	}
	if !results[0].Killed {
		t.Error("results[0].Killed = false, want true")
	}

	// Second target gets overflow
	if results[1].TargetID != "enemy2" {
		t.Errorf("results[1].TargetID = %s, want enemy2", results[1].TargetID)
	}
	if results[1].Damage != 20 {
		t.Errorf("results[1].Damage = %d, want 20", results[1].Damage)
	}
	if results[1].NewHP != 20 {
		t.Errorf("results[1].NewHP = %d, want 20", results[1].NewHP)
	}
}

func TestApplyDamageWithOverflow_MultiOverflow(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 3},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy1",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 20},
			},
		},
		{
			ID:       "enemy2",
			Position: 1,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30},
			},
		},
		{
			ID:       "enemy3",
			Position: 2,
			Attributes: map[string]core.Attribute{
				"health": {Base: 40},
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"enemy1": {20, 0},
		"enemy2": {30, 0},
		"enemy3": {40, 0},
	}

	// 100 damage chains through all: 20 + 30 + 40 = 90, 10 wasted
	results := ApplyDamageWithOverflow(attacker, 100, enemies, hpSnapshot)

	if len(results) != 3 {
		t.Fatalf("expected 3 results, got %d", len(results))
	}

	// All should be killed
	for i, r := range results {
		if !r.Killed {
			t.Errorf("results[%d].Killed = false, want true", i)
		}
	}

	// Verify snapshot updated
	if hpSnapshot["enemy1"][0] != 0 {
		t.Errorf("hpSnapshot[enemy1] = %d, want 0", hpSnapshot["enemy1"][0])
	}
	if hpSnapshot["enemy2"][0] != 0 {
		t.Errorf("hpSnapshot[enemy2] = %d, want 0", hpSnapshot["enemy2"][0])
	}
	if hpSnapshot["enemy3"][0] != 0 {
		t.Errorf("hpSnapshot[enemy3] = %d, want 0", hpSnapshot["enemy3"][0])
	}
}

func TestApplyDamageWithOverflow_ShieldsAbsorb(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health":  {Base: 30},
				"shields": {Base: 10},
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"enemy": {30, 10}, // 30 HP, 10 shields
	}

	// 20 damage: 10 absorbed by shields, 10 to HP
	results := ApplyDamageWithOverflow(attacker, 20, enemies, hpSnapshot)

	if len(results) != 1 {
		t.Fatalf("expected 1 result, got %d", len(results))
	}
	if results[0].Damage != 20 {
		t.Errorf("Damage = %d, want 20", results[0].Damage)
	}
	if results[0].NewShields != 0 {
		t.Errorf("NewShields = %d, want 0", results[0].NewShields)
	}
	if results[0].NewHP != 20 {
		t.Errorf("NewHP = %d, want 20", results[0].NewHP)
	}
}

func TestApplyDamageWithOverflow_ExcessWasted(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 50},
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"enemy": {50, 0},
	}

	// 100 damage, only 50 HP - excess wasted
	results := ApplyDamageWithOverflow(attacker, 100, enemies, hpSnapshot)

	if len(results) != 1 {
		t.Fatalf("expected 1 result, got %d", len(results))
	}
	// Damage should be capped at what was actually dealt
	if results[0].Damage != 50 {
		t.Errorf("Damage = %d, want 50 (excess wasted)", results[0].Damage)
	}
	if results[0].NewHP != 0 {
		t.Errorf("NewHP = %d, want 0", results[0].NewHP)
	}
}

func TestApplyDamageWithOverflow_AllOverlappingDead(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "dead_enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 0}, // Already dead
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"dead_enemy": {0, 0},
	}

	results := ApplyDamageWithOverflow(attacker, 50, enemies, hpSnapshot)

	if len(results) != 0 {
		t.Errorf("expected 0 results for dead enemies, got %d", len(results))
	}
}

func TestApplyDamageWithOverflow_ZeroDamage(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 50},
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"enemy": {50, 0},
	}

	results := ApplyDamageWithOverflow(attacker, 0, enemies, hpSnapshot)

	if len(results) != 0 {
		t.Errorf("expected 0 results for zero damage, got %d", len(results))
	}
}

func TestApplyDamageWithOverflow_NoOverlap(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy_far",
			Position: 5, // No overlap
			Attributes: map[string]core.Attribute{
				"health": {Base: 50},
			},
		},
	}

	hpSnapshot := map[string][2]int{
		"enemy_far": {50, 0},
	}

	results := ApplyDamageWithOverflow(attacker, 50, enemies, hpSnapshot)

	if len(results) != 0 {
		t.Errorf("expected 0 results for no overlap, got %d", len(results))
	}
}

// ===== SelectTarget Tests (Updated Behavior) =====

func TestSelectTarget_GapWithLiveUnits_DamageWasted(t *testing.T) {
	// Attacker has gap (no overlap), but live units exist elsewhere
	// Per F-167: damage should be wasted, not hit command
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 9, // Far right
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
			"health":       {Base: 50},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy_alive",
			Position: 0, // No overlap with position 9
			Attributes: map[string]core.Attribute{
				"health": {Base: 30}, // Alive!
			},
		},
	}

	enemyCmd := &entity.Unit{
		ID:       "enemy_cmd",
		Position: -1,
		Tags:     []core.Tag{"command"},
		Attributes: map[string]core.Attribute{
			"health": {Base: 100},
		},
	}

	result := SelectTarget(attacker, enemies, enemyCmd)

	// Should return empty string (no valid target) because:
	// - No overlapping enemies
	// - Live units exist, so command can't be targeted
	if result != "" {
		t.Errorf("SelectTarget() = %s, want empty (gap with live units = damage wasted)", result)
	}
}

func TestSelectTarget_GapAllUnitsDead_HitsCommand(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 9,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
			"health":       {Base: 50},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "dead_enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 0}, // Dead
			},
		},
	}

	enemyCmd := &entity.Unit{
		ID:       "enemy_cmd",
		Position: -1,
		Tags:     []core.Tag{"command"},
		Attributes: map[string]core.Attribute{
			"health": {Base: 100},
		},
	}

	result := SelectTarget(attacker, enemies, enemyCmd)

	if result != "enemy_cmd" {
		t.Errorf("SelectTarget() = %s, want enemy_cmd (all units dead -> hit command)", result)
	}
}

func TestSelectTarget_UsesLowestHP(t *testing.T) {
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 0,
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 2},
			"health":       {Base: 50},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "high_hp",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 100},
			},
		},
		{
			ID:       "low_hp",
			Position: 1,
			Attributes: map[string]core.Attribute{
				"health": {Base: 20},
			},
		},
	}

	result := SelectTarget(attacker, enemies, nil)

	if result != "low_hp" {
		t.Errorf("SelectTarget() = %s, want low_hp (lowest HP priority)", result)
	}
}
