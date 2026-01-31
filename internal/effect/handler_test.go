package effect

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/event"
)

func TestHandle_DealDamage(t *testing.T) {
	target := entity.Unit{
		ID:   "enemy1",
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100, Min: 0},
		},
	}

	source := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}

	ctx := EffectContext{
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
			"enemy1":  target,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
	}

	params := map[string]any{
		"damage": 25,
		"target": "enemy",
	}

	result := Handle("deal_damage", params, ctx)

	// Check target was modified
	modified, ok := result.ModifiedUnits["enemy1"]
	if !ok {
		t.Fatal("expected enemy1 in modified units")
	}

	newHealth := modified.Attributes["health"].Base
	if newHealth != 75 {
		t.Errorf("expected health 75, got %d", newHealth)
	}

	// Check on_damaged event emitted
	if len(result.FollowUpEvents) != 1 {
		t.Fatalf("expected 1 follow-up event, got %d", len(result.FollowUpEvents))
	}
	if result.FollowUpEvents[0].Event != core.EventOnDamaged {
		t.Errorf("expected on_damaged event, got %s", result.FollowUpEvents[0].Event)
	}

	// Check log entry
	if len(result.LogEntries) != 1 {
		t.Fatalf("expected 1 log entry, got %d", len(result.LogEntries))
	}
}

func TestHandle_DealDamage_Destroy(t *testing.T) {
	target := entity.Unit{
		ID:   "enemy1",
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 20, Min: 0},
		},
	}

	source := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}

	ctx := EffectContext{
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
			"enemy1":  target,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
	}

	params := map[string]any{
		"damage": 50,
		"target": "enemy",
	}

	result := Handle("deal_damage", params, ctx)

	// Check health clamped to 0
	modified := result.ModifiedUnits["enemy1"]
	newHealth := modified.Attributes["health"].Base
	if newHealth != 0 {
		t.Errorf("expected health 0, got %d", newHealth)
	}

	// Check both on_damaged and on_destroyed events emitted
	if len(result.FollowUpEvents) != 2 {
		t.Fatalf("expected 2 follow-up events, got %d", len(result.FollowUpEvents))
	}

	events := make(map[core.EventType]bool)
	for _, e := range result.FollowUpEvents {
		events[e.Event] = true
	}

	if !events[core.EventOnDamaged] {
		t.Error("expected on_damaged event")
	}
	if !events[core.EventOnDestroyed] {
		t.Error("expected on_destroyed event")
	}
}

func TestHandle_DealDamage_NoHealthAttribute(t *testing.T) {
	target := entity.Unit{
		ID:         "enemy1",
		Tags:       []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{},
	}

	source := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}

	ctx := EffectContext{
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
			"enemy1":  target,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
	}

	params := map[string]any{
		"damage": 25,
		"target": "enemy",
	}

	result := Handle("deal_damage", params, ctx)

	// No modifications expected
	if len(result.ModifiedUnits) != 0 {
		t.Errorf("expected no modifications, got %d", len(result.ModifiedUnits))
	}

	// Should have log entry about missing health
	if len(result.LogEntries) != 1 {
		t.Fatalf("expected 1 log entry, got %d", len(result.LogEntries))
	}
}

func TestHandle_DealDamage_NoEnemies(t *testing.T) {
	source := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}

	ctx := EffectContext{
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
	}

	params := map[string]any{
		"damage": 25,
		"target": "enemy",
	}

	result := Handle("deal_damage", params, ctx)

	// No modifications expected
	if len(result.ModifiedUnits) != 0 {
		t.Errorf("expected no modifications with empty enemy list, got %d", len(result.ModifiedUnits))
	}

	// Silent no-op when no valid target
	if len(result.LogEntries) != 0 {
		t.Errorf("expected no log entries (silent no-op), got %d", len(result.LogEntries))
	}
}

func TestHandle_DealDamage_SelfTarget(t *testing.T) {
	source := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100, Min: 0},
		},
	}

	ctx := EffectContext{
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
		},
	}

	params := map[string]any{
		"damage": 30,
		"target": "self",
	}

	result := Handle("deal_damage", params, ctx)

	modified, ok := result.ModifiedUnits["player1"]
	if !ok {
		t.Fatal("expected player1 in modified units")
	}

	newHealth := modified.Attributes["health"].Base
	if newHealth != 70 {
		t.Errorf("expected health 70, got %d", newHealth)
	}
}

func TestHandle_ConsumeAmmo(t *testing.T) {
	item := entity.Item{
		ID: "laser",
		Attributes: map[string]core.Attribute{
			"ammo": {Name: "ammo", Base: 10, Min: 0},
		},
	}

	source := entity.Unit{
		ID: "player1",
		Parts: map[string]entity.Part{
			"torso": {
				ID: "torso",
				Mounts: []entity.Mount{
					{
						ID:       "weapon_mount",
						Contents: []entity.Item{item},
					},
				},
			},
		},
	}

	ctx := EffectContext{
		Owner: event.TriggerOwner{
			UnitID:  "player1",
			PartID:  "torso",
			MountID: "weapon_mount",
			ItemID:  "laser",
		},
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
		},
	}

	params := map[string]any{
		"amount": 1,
	}

	result := Handle("consume_ammo", params, ctx)

	modified, ok := result.ModifiedUnits["player1"]
	if !ok {
		t.Fatal("expected player1 in modified units")
	}

	// Navigate to the item to check ammo
	modItem := modified.Parts["torso"].Mounts[0].Contents[0]
	newAmmo := modItem.Attributes["ammo"].Base
	if newAmmo != 9 {
		t.Errorf("expected ammo 9, got %d", newAmmo)
	}

	if len(result.LogEntries) != 1 {
		t.Fatalf("expected 1 log entry, got %d", len(result.LogEntries))
	}
}

func TestHandle_ConsumeAmmo_NotFromItem(t *testing.T) {
	source := entity.Unit{
		ID: "player1",
	}

	ctx := EffectContext{
		Owner: event.TriggerOwner{
			UnitID: "player1",
			// No item owner
		},
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
		},
	}

	params := map[string]any{
		"amount": 1,
	}

	result := Handle("consume_ammo", params, ctx)

	if len(result.ModifiedUnits) != 0 {
		t.Errorf("expected no modifications when not triggered by item, got %d", len(result.ModifiedUnits))
	}

	if len(result.LogEntries) != 1 {
		t.Fatalf("expected 1 log entry, got %d", len(result.LogEntries))
	}
}

func TestHandle_ConsumeAmmo_NoAmmoAttribute(t *testing.T) {
	item := entity.Item{
		ID:         "laser",
		Attributes: map[string]core.Attribute{},
	}

	source := entity.Unit{
		ID: "player1",
		Parts: map[string]entity.Part{
			"torso": {
				ID: "torso",
				Mounts: []entity.Mount{
					{
						ID:       "weapon_mount",
						Contents: []entity.Item{item},
					},
				},
			},
		},
	}

	ctx := EffectContext{
		Owner: event.TriggerOwner{
			UnitID:  "player1",
			PartID:  "torso",
			MountID: "weapon_mount",
			ItemID:  "laser",
		},
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
		},
	}

	params := map[string]any{
		"amount": 1,
	}

	result := Handle("consume_ammo", params, ctx)

	if len(result.ModifiedUnits) != 0 {
		t.Errorf("expected no modifications when no ammo attribute, got %d", len(result.ModifiedUnits))
	}

	if len(result.LogEntries) != 1 {
		t.Fatalf("expected 1 log entry about missing ammo, got %d", len(result.LogEntries))
	}
}

func TestHandle_DealSplashDamage(t *testing.T) {
	// MVP: same as deal_damage
	target := entity.Unit{
		ID:   "enemy1",
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100, Min: 0},
		},
	}

	source := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}

	ctx := EffectContext{
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
			"enemy1":  target,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
	}

	params := map[string]any{
		"damage":        20,
		"splash_radius": 2,
		"target":        "enemy",
	}

	result := Handle("deal_splash_damage", params, ctx)

	modified, ok := result.ModifiedUnits["enemy1"]
	if !ok {
		t.Fatal("expected enemy1 in modified units")
	}

	newHealth := modified.Attributes["health"].Base
	if newHealth != 80 {
		t.Errorf("expected health 80, got %d", newHealth)
	}
}

func TestHandle_UnknownEffect(t *testing.T) {
	ctx := EffectContext{
		SourceUnit: entity.Unit{ID: "unit1"},
		AllUnits:   map[string]entity.Unit{},
	}

	result := Handle("nonexistent_effect", nil, ctx)

	if len(result.ModifiedUnits) != 0 {
		t.Errorf("expected no modifications for unknown effect, got %d", len(result.ModifiedUnits))
	}

	if len(result.LogEntries) != 1 {
		t.Fatalf("expected 1 log entry about unknown effect, got %d", len(result.LogEntries))
	}
}

func TestHandle_DealDamage_MissingDamageParam(t *testing.T) {
	ctx := EffectContext{
		SourceUnit: entity.Unit{ID: "unit1"},
		AllUnits:   map[string]entity.Unit{},
	}

	result := Handle("deal_damage", map[string]any{}, ctx)

	if len(result.ModifiedUnits) != 0 {
		t.Errorf("expected no modifications when damage param missing, got %d", len(result.ModifiedUnits))
	}

	if len(result.LogEntries) != 1 {
		t.Fatalf("expected 1 log entry about missing param, got %d", len(result.LogEntries))
	}
}

func TestHandle_DealDamage_DeterministicEnemySelection(t *testing.T) {
	// Test that enemy selection is deterministic (alphabetical by ID)
	enemy1 := entity.Unit{
		ID:   "zebra", // Alphabetically last
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100, Min: 0},
		},
	}
	enemy2 := entity.Unit{
		ID:   "alpha", // Alphabetically first - should be targeted
		Tags: []core.Tag{"enemy"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 100, Min: 0},
		},
	}
	source := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}

	ctx := EffectContext{
		SourceUnit: source,
		AllUnits: map[string]entity.Unit{
			"player1": source,
			"zebra":   enemy1,
			"alpha":   enemy2,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
	}

	params := map[string]any{
		"damage": 10,
		"target": "enemy",
	}

	// Run multiple times to verify determinism
	for i := 0; i < 10; i++ {
		result := Handle("deal_damage", params, ctx)

		// Should always target "alpha" (alphabetically first)
		if _, ok := result.ModifiedUnits["alpha"]; !ok {
			t.Fatalf("iteration %d: expected alpha to be targeted (deterministic), got modified units: %v",
				i, result.ModifiedUnits)
		}
		if _, ok := result.ModifiedUnits["zebra"]; ok {
			t.Fatalf("iteration %d: expected zebra not to be targeted", i)
		}
	}
}

func TestGetEnemiesOf_WithTargetConditions(t *testing.T) {
	// Set up: player1 attacks, enemies have varying health
	player := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}
	aliveEnemy := entity.Unit{
		ID:   "enemy_alive",
		Tags: []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 50},
		},
	}
	deadEnemy := entity.Unit{
		ID:   "enemy_dead",
		Tags: []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 0},
		},
	}

	ctx := EffectContext{
		SourceUnit: player,
		AllUnits: map[string]entity.Unit{
			"player1":     player,
			"enemy_alive": aliveEnemy,
			"enemy_dead":  deadEnemy,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
		TargetConditions: []core.Condition{
			{Type: core.ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 1}},
		},
	}

	enemies := getEnemiesOf(player, ctx)

	if len(enemies) != 1 {
		t.Fatalf("expected 1 enemy after filtering, got %d", len(enemies))
	}
	if enemies[0].ID != "enemy_alive" {
		t.Errorf("expected enemy_alive, got %s", enemies[0].ID)
	}
}

func TestGetEnemiesOf_AllFiltered(t *testing.T) {
	// All enemies are dead
	player := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}
	deadEnemy1 := entity.Unit{
		ID:   "enemy_dead1",
		Tags: []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 0},
		},
	}
	deadEnemy2 := entity.Unit{
		ID:   "enemy_dead2",
		Tags: []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 0},
		},
	}

	ctx := EffectContext{
		SourceUnit: player,
		AllUnits: map[string]entity.Unit{
			"player1":     player,
			"enemy_dead1": deadEnemy1,
			"enemy_dead2": deadEnemy2,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
		TargetConditions: []core.Condition{
			{Type: core.ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 1}},
		},
	}

	enemies := getEnemiesOf(player, ctx)

	if len(enemies) != 0 {
		t.Errorf("expected 0 enemies when all filtered, got %d", len(enemies))
	}
}

func TestDealDamage_SkipsDeadTargets(t *testing.T) {
	// Integration test: dead targets are not attacked
	player := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}
	deadEnemy := entity.Unit{
		ID:   "enemy_dead",
		Tags: []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 0},
		},
	}

	ctx := EffectContext{
		SourceUnit: player,
		AllUnits: map[string]entity.Unit{
			"player1":    player,
			"enemy_dead": deadEnemy,
		},
		PlayerUnitIDs: map[string]bool{"player1": true},
		TargetConditions: []core.Condition{
			{Type: core.ConditionAttrGTE, Params: map[string]any{"attribute": "health", "value": 1}},
		},
	}

	params := map[string]any{
		"damage": 25,
		"target": "enemy",
	}

	result := Handle("deal_damage", params, ctx)

	// No modifications expected - all enemies are dead
	if len(result.ModifiedUnits) != 0 {
		t.Errorf("expected no modifications when targeting dead units, got %d", len(result.ModifiedUnits))
	}

	// Silent no-op when no valid target
	if len(result.LogEntries) != 0 {
		t.Errorf("expected no log entries (silent no-op), got %d", len(result.LogEntries))
	}
}

func TestGetEnemiesOf_NoTargetConditions(t *testing.T) {
	// Without target conditions, only alive enemies should be returned
	player := entity.Unit{
		ID:   "player1",
		Tags: []core.Tag{"player"},
	}
	deadEnemy := entity.Unit{
		ID:   "enemy1",
		Tags: []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 0},
		},
	}
	aliveEnemy := entity.Unit{
		ID:   "enemy2",
		Tags: []core.Tag{"mech"},
		Attributes: map[string]core.Attribute{
			"health": {Name: "health", Base: 50},
		},
	}

	ctx := EffectContext{
		SourceUnit: player,
		AllUnits: map[string]entity.Unit{
			"player1": player,
			"enemy1":  deadEnemy,
			"enemy2":  aliveEnemy,
		},
		PlayerUnitIDs:    map[string]bool{"player1": true},
		TargetConditions: nil, // No conditions
	}

	enemies := getEnemiesOf(player, ctx)

	// Dead enemies are filtered out even without explicit target conditions
	if len(enemies) != 1 {
		t.Errorf("expected 1 alive enemy, got %d", len(enemies))
	}
	if len(enemies) > 0 && enemies[0].ID != "enemy2" {
		t.Errorf("expected enemy2 (alive), got %s", enemies[0].ID)
	}
}

func TestEffectResult_Merge(t *testing.T) {
	r1 := EffectResult{
		ModifiedUnits: map[string]entity.Unit{
			"unit1": {ID: "unit1"},
		},
		FollowUpEvents: []FollowUpEvent{
			{Event: core.EventOnDamaged, TargetID: "unit1"},
		},
		LogEntries: []string{"log1"},
	}

	r2 := EffectResult{
		ModifiedUnits: map[string]entity.Unit{
			"unit2": {ID: "unit2"},
		},
		FollowUpEvents: []FollowUpEvent{
			{Event: core.EventOnDestroyed, TargetID: "unit2"},
		},
		LogEntries: []string{"log2"},
	}

	r1.Merge(r2)

	if len(r1.ModifiedUnits) != 2 {
		t.Errorf("expected 2 modified units, got %d", len(r1.ModifiedUnits))
	}
	if _, ok := r1.ModifiedUnits["unit1"]; !ok {
		t.Error("expected unit1 in modified units")
	}
	if _, ok := r1.ModifiedUnits["unit2"]; !ok {
		t.Error("expected unit2 in modified units")
	}

	if len(r1.FollowUpEvents) != 2 {
		t.Errorf("expected 2 follow-up events, got %d", len(r1.FollowUpEvents))
	}

	if len(r1.LogEntries) != 2 {
		t.Errorf("expected 2 log entries, got %d", len(r1.LogEntries))
	}
}
