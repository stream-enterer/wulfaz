package tea

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

// ===== Shield Expiration Tests =====

func TestRoundEnded_ShieldExpiration(t *testing.T) {
	m := Model{
		Version: 1,
		Seed:    42,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseRoundEnd,
			Round:     1,
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
					Attributes: map[string]core.Attribute{
						"health":  {Base: 100},
						"shields": {Base: 15}, // Should expire
					},
				},
				{
					ID:       "player1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health":  {Base: 50},
						"shields": {Base: 8}, // Should expire
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}}},
					Attributes: map[string]core.Attribute{
						"health":  {Base: 100},
						"shields": {Base: 20}, // Should expire
					},
				},
			},
		},
	}

	newM, _ := m.Update(model.RoundEnded{})

	// All shields should be 0
	for _, u := range newM.Combat.PlayerUnits {
		if s, ok := u.Attributes["shields"]; ok && s.Base != 0 {
			t.Errorf("%s shields = %d, want 0", u.ID, s.Base)
		}
	}
	for _, u := range newM.Combat.EnemyUnits {
		if s, ok := u.Attributes["shields"]; ok && s.Base != 0 {
			t.Errorf("%s shields = %d, want 0", u.ID, s.Base)
		}
	}

	// Round should increment immediately
	if newM.Combat.Round != 2 {
		t.Errorf("Round = %d, want 2", newM.Combat.Round)
	}
}

// ===== Combat Loop Tests =====

func TestCombatStarted_TriggersFirstRound(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseMenu,
		Seed:    42,
	}

	combat := model.CombatModel{
		Phase: model.CombatActive,
		PlayerUnits: []entity.Unit{
			{
				ID:       "player_cmd",
				Position: -1,
				Tags:     []core.Tag{"command"},
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 12}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}}},
				Attributes: map[string]core.Attribute{
					"health": {Base: 100},
				},
			},
		},
		EnemyUnits: []entity.Unit{
			{
				ID:       "enemy_cmd",
				Position: -1,
				Tags:     []core.Tag{"command"},
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 5}, {Type: entity.DieDamage, Value: 8}, {Type: entity.DieDamage, Value: 12}, {Type: entity.DieBlank, Value: 0}, {Type: entity.DieBlank, Value: 0}}}},
				Attributes: map[string]core.Attribute{
					"health": {Base: 100},
				},
			},
		},
	}

	msg := model.CombatStarted{Combat: combat}
	newM, cmd := m.Update(msg)

	if newM.Phase != model.PhaseCombat {
		t.Errorf("Phase = %v, want PhaseCombat", newM.Phase)
	}
	if newM.FightNumber != 1 {
		t.Errorf("FightNumber = %d, want 1", newM.FightNumber)
	}

	// Should return StartNextRound cmd
	if cmd == nil {
		t.Fatal("expected cmd for first round")
	}
	result := cmd()
	if _, ok := result.(model.RoundStarted); !ok {
		t.Errorf("expected RoundStarted, got %T", result)
	}
}

func TestCheckCombatEnd_CommandUnitBased(t *testing.T) {
	tests := []struct {
		name           string
		playerCmdAlive bool
		enemyCmdAlive  bool
		expected       model.Victor
	}{
		{"both alive", true, true, model.VictorNone},
		{"enemy cmd dead", true, false, model.VictorPlayer},
		{"player cmd dead", false, true, model.VictorEnemy},
		{"both dead - player wins tie", false, false, model.VictorPlayer},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			playerHP := 100
			if !tt.playerCmdAlive {
				playerHP = 0
			}
			enemyHP := 100
			if !tt.enemyCmdAlive {
				enemyHP = 0
			}

			m := Model{
				Combat: model.CombatModel{
					Phase: model.CombatActive,
					PlayerUnits: []entity.Unit{
						{
							ID:   "player_cmd",
							Tags: []core.Tag{"command"},
							Attributes: map[string]core.Attribute{
								"health": {Base: playerHP},
							},
						},
						{
							ID: "player1", // Non-command, should be ignored
							Attributes: map[string]core.Attribute{
								"health": {Base: 50},
							},
						},
					},
					EnemyUnits: []entity.Unit{
						{
							ID:   "enemy_cmd",
							Tags: []core.Tag{"command"},
							Attributes: map[string]core.Attribute{
								"health": {Base: enemyHP},
							},
						},
					},
				},
			}

			result := m.checkCombatEnd()
			if result != tt.expected {
				t.Errorf("checkCombatEnd() = %v, want %v", result, tt.expected)
			}
		})
	}
}

// ===== Enemy Execution Tests (Per-Unit via UnitDiceEffectsApplied) =====

func TestEnemyExecution_AppliesDamage(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
			EnemyTargets:        map[string]string{"enemy1": "player_cmd"},
			EnemyDefenseTargets: map[string]string{},
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				{
					ID:       "enemy1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health": {Base: 50},
					},
				},
			},
		},
	}

	msg := model.UnitDiceEffectsApplied{
		SourceUnitID: "enemy1",
		Results: []model.DiceEffectResult{
			{
				TargetUnitID: "player_cmd",
				Effect:       entity.DieDamage,
				Value:        20,
				NewHealth:    80,
				NewShields:   0,
			},
		},
		Timestamp: 1000,
	}

	newM, _ := m.Update(msg)

	// Check damage applied to player_cmd
	for _, u := range newM.Combat.PlayerUnits {
		if u.ID == "player_cmd" {
			if u.Attributes["health"].Base != 80 {
				t.Errorf("player_cmd health = %d, want 80", u.Attributes["health"].Base)
			}
		}
	}
}

func TestEnemyExecution_VictoryCheck(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
			EnemyTargets:        map[string]string{"enemy1": "enemy_cmd"},
			EnemyDefenseTargets: map[string]string{},
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 10}, // Low HP, will die
					},
				},
				{
					ID:       "enemy1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health": {Base: 50},
					},
				},
			},
		},
	}

	msg := model.UnitDiceEffectsApplied{
		SourceUnitID: "enemy1",
		Results: []model.DiceEffectResult{
			{
				TargetUnitID: "enemy_cmd",
				Effect:       entity.DieDamage,
				Value:        10,
				NewHealth:    0,
				NewShields:   0,
			},
		},
		Timestamp: 1000,
	}

	newM, cmd := m.Update(msg)

	// Victory detected immediately during execution (not deferred)
	if newM.Combat.Phase != model.CombatResolved {
		t.Errorf("Combat.Phase = %v, want CombatResolved", newM.Combat.Phase)
	}
	if newM.Combat.Victor != "player" {
		t.Errorf("Victor = %s, want player", newM.Combat.Victor)
	}

	// Should return CombatEnded cmd
	if cmd == nil {
		t.Fatal("expected CombatEnded cmd, got nil")
	}
	result := cmd()
	if ended, ok := result.(model.CombatEnded); !ok || ended.Victor != model.VictorPlayer {
		t.Errorf("expected CombatEnded{VictorPlayer}, got %T", result)
	}
}

// ===== Execution Complete Tests =====

func TestExecutionComplete_TransitionsToRoundEnd(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
		Seed:    42,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
			Round:     1,
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
		},
	}

	// ExecutionComplete should transition to RoundEnd
	m1, cmd1 := m.Update(model.ExecutionComplete{})
	if m1.Combat.DicePhase != model.DicePhaseRoundEnd {
		t.Errorf("after ExecutionComplete: DicePhase = %v, want DicePhaseRoundEnd", m1.Combat.DicePhase)
	}
	if cmd1 == nil {
		t.Fatal("expected RoundEnded cmd, got nil")
	}

	// Process the RoundEnded
	msg1 := cmd1()
	if _, ok := msg1.(model.RoundEnded); !ok {
		t.Fatalf("expected RoundEnded, got %T", msg1)
	}

	m2, _ := m1.Update(msg1)
	if m2.Combat.Round != 2 {
		t.Errorf("Round = %d, want 2", m2.Combat.Round)
	}
}

// ===== Enemy Defense Results Tests =====

func TestEnemyExecution_DefenseResults(t *testing.T) {
	// Enemy shield/heal dice should apply to enemy allies via UnitDiceEffectsApplied.
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
			EnemyTargets:        map[string]string{},
			EnemyDefenseTargets: map[string]string{"enemy_cmd": "enemy1"},
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 80},
					},
				},
				{
					ID:       "enemy1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health":  {Base: 40},
						"shields": {Base: 0},
					},
				},
			},
		},
	}

	msg := model.UnitDiceEffectsApplied{
		SourceUnitID: "enemy_cmd",
		Results: []model.DiceEffectResult{
			{
				TargetUnitID: "enemy1",
				Effect:       entity.DieShield,
				Value:        5,
				NewHealth:    40,
				NewShields:   5,
			},
			{
				TargetUnitID: "enemy_cmd",
				Effect:       entity.DieHeal,
				Value:        10,
				NewHealth:    90,
				NewShields:   0,
			},
		},
		Timestamp: 1000,
	}

	newM, _ := m.Update(msg)

	// Check shields applied to enemy1
	for _, u := range newM.Combat.EnemyUnits {
		if u.ID == "enemy1" {
			if u.Attributes["shields"].Base != 5 {
				t.Errorf("enemy1 shields = %d, want 5", u.Attributes["shields"].Base)
			}
		}
		if u.ID == "enemy_cmd" {
			if u.Attributes["health"].Base != 90 {
				t.Errorf("enemy_cmd health = %d, want 90", u.Attributes["health"].Base)
			}
		}
	}
}

// ===== ApplyEnemyUnitEffects Cmd Tests =====

func TestApplyEnemyUnitEffects_DamageWithShieldAbsorption(t *testing.T) {
	combat := model.CombatModel{
		PlayerUnits: []entity.Unit{
			{
				ID: "player1",
				Attributes: map[string]core.Attribute{
					"health":  {Base: 50},
					"shields": {Base: 3},
				},
			},
		},
		EnemyUnits: []entity.Unit{
			{
				ID: "enemy1",
				Attributes: map[string]core.Attribute{
					"health": {Base: 40},
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"enemy1": {
				{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 5}}, FaceIndex: 0},
			},
		},
		EnemyTargets:        map[string]string{"enemy1": "player1"},
		EnemyDefenseTargets: map[string]string{},
	}

	cmd := ApplyEnemyUnitEffects("enemy1", combat, 1000)
	result := cmd().(model.UnitDiceEffectsApplied)

	if result.SourceUnitID != "enemy1" {
		t.Errorf("SourceUnitID = %s, want enemy1", result.SourceUnitID)
	}
	if len(result.Results) != 1 {
		t.Fatalf("len(Results) = %d, want 1", len(result.Results))
	}
	r := result.Results[0]
	if r.TargetUnitID != "player1" {
		t.Errorf("TargetUnitID = %s, want player1", r.TargetUnitID)
	}
	if r.ShieldAbsorbed != 3 {
		t.Errorf("ShieldAbsorbed = %d, want 3", r.ShieldAbsorbed)
	}
	if r.NewShields != 0 {
		t.Errorf("NewShields = %d, want 0", r.NewShields)
	}
	if r.NewHealth != 48 {
		t.Errorf("NewHealth = %d, want 48", r.NewHealth)
	}
}

func TestApplyEnemyUnitEffects_HealCappedAtMaxHealth(t *testing.T) {
	combat := model.CombatModel{
		EnemyUnits: []entity.Unit{
			{
				ID: "enemy1",
				Attributes: map[string]core.Attribute{
					"health":     {Base: 45},
					"max_health": {Base: 50},
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"enemy1": {
				{Faces: []entity.DieFace{{Type: entity.DieHeal, Value: 10}}, FaceIndex: 0},
			},
		},
		EnemyTargets:        map[string]string{},
		EnemyDefenseTargets: map[string]string{"enemy1": "enemy1"},
	}

	cmd := ApplyEnemyUnitEffects("enemy1", combat, 1000)
	result := cmd().(model.UnitDiceEffectsApplied)

	if len(result.Results) != 1 {
		t.Fatalf("len(Results) = %d, want 1", len(result.Results))
	}
	if result.Results[0].NewHealth != 50 {
		t.Errorf("NewHealth = %d, want 50 (capped at max_health)", result.Results[0].NewHealth)
	}
}

func TestApplyEnemyUnitEffects_OnlyDamageDice(t *testing.T) {
	combat := model.CombatModel{
		PlayerUnits: []entity.Unit{
			{
				ID: "player1",
				Attributes: map[string]core.Attribute{
					"health": {Base: 100},
				},
			},
		},
		EnemyUnits: []entity.Unit{
			{
				ID: "enemy1",
				Attributes: map[string]core.Attribute{
					"health": {Base: 40},
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"enemy1": {
				{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 3}}, FaceIndex: 0},
				{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 4}}, FaceIndex: 0},
			},
		},
		EnemyTargets:        map[string]string{"enemy1": "player1"},
		EnemyDefenseTargets: map[string]string{},
	}

	cmd := ApplyEnemyUnitEffects("enemy1", combat, 1000)
	result := cmd().(model.UnitDiceEffectsApplied)

	if len(result.Results) != 2 {
		t.Fatalf("len(Results) = %d, want 2", len(result.Results))
	}
	// First die: 3 dmg -> 97 HP
	if result.Results[0].NewHealth != 97 {
		t.Errorf("Results[0].NewHealth = %d, want 97", result.Results[0].NewHealth)
	}
	// Second die: 4 dmg -> 93 HP
	if result.Results[1].NewHealth != 93 {
		t.Errorf("Results[1].NewHealth = %d, want 93", result.Results[1].NewHealth)
	}
}

func TestApplyEnemyUnitEffects_OnlyHealDice(t *testing.T) {
	combat := model.CombatModel{
		EnemyUnits: []entity.Unit{
			{
				ID: "enemy_cmd",
				Attributes: map[string]core.Attribute{
					"health":     {Base: 80},
					"max_health": {Base: 100},
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"enemy_cmd": {
				{Faces: []entity.DieFace{{Type: entity.DieHeal, Value: 5}}, FaceIndex: 0},
			},
		},
		EnemyTargets:        map[string]string{},
		EnemyDefenseTargets: map[string]string{"enemy_cmd": "enemy_cmd"},
	}

	cmd := ApplyEnemyUnitEffects("enemy_cmd", combat, 1000)
	result := cmd().(model.UnitDiceEffectsApplied)

	if len(result.Results) != 1 {
		t.Fatalf("len(Results) = %d, want 1", len(result.Results))
	}
	if result.Results[0].Effect != entity.DieHeal {
		t.Errorf("Effect = %v, want DieHeal", result.Results[0].Effect)
	}
	if result.Results[0].NewHealth != 85 {
		t.Errorf("NewHealth = %d, want 85", result.Results[0].NewHealth)
	}
}

// ===== Dead Unit Target Pruning Tests =====

func TestPruneDeadTargets(t *testing.T) {
	alive := func(id string) entity.Unit {
		return entity.Unit{ID: id, Attributes: map[string]core.Attribute{"health": {Base: 10}}}
	}
	dead := func(id string) entity.Unit {
		return entity.Unit{ID: id, Attributes: map[string]core.Attribute{"health": {Base: 0}}}
	}

	tests := []struct {
		name    string
		targets map[string]string
		sources []entity.Unit
		dests   []entity.Unit
		want    map[string]string
	}{
		{
			name:    "empty map unchanged",
			targets: map[string]string{},
			sources: []entity.Unit{alive("s1")},
			dests:   []entity.Unit{alive("d1")},
			want:    map[string]string{},
		},
		{
			name:    "all alive unchanged",
			targets: map[string]string{"s1": "d1", "s2": "d2"},
			sources: []entity.Unit{alive("s1"), alive("s2")},
			dests:   []entity.Unit{alive("d1"), alive("d2")},
			want:    map[string]string{"s1": "d1", "s2": "d2"},
		},
		{
			name:    "dead source removed",
			targets: map[string]string{"alive1": "dest1", "dead1": "dest2"},
			sources: []entity.Unit{alive("alive1"), dead("dead1")},
			dests:   []entity.Unit{alive("dest1"), alive("dest2")},
			want:    map[string]string{"alive1": "dest1"},
		},
		{
			name:    "dead dest removed",
			targets: map[string]string{"src1": "alive1", "src2": "dead1"},
			sources: []entity.Unit{alive("src1"), alive("src2")},
			dests:   []entity.Unit{alive("alive1"), dead("dead1")},
			want:    map[string]string{"src1": "alive1"},
		},
		{
			name:    "both dead clears all",
			targets: map[string]string{"dead1": "dead2"},
			sources: []entity.Unit{dead("dead1")},
			dests:   []entity.Unit{dead("dead2")},
			want:    map[string]string{},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := pruneDeadTargets(tt.targets, tt.sources, tt.dests)
			if len(got) != len(tt.want) {
				t.Fatalf("len = %d, want %d; got %v", len(got), len(tt.want), got)
			}
			for k, v := range tt.want {
				if got[k] != v {
					t.Errorf("got[%q] = %q, want %q", k, got[k], v)
				}
			}
		})
	}
}

func TestEnemyExecution_SkipsDeadSource(t *testing.T) {
	// Bug 1: Dead enemy (killed during player command) should not execute.
	// enemy1 is dead (HP=0) but has a target entry. After processing any
	// UnitDiceEffectsApplied, pruning should remove enemy1's entry so
	// advanceEnemyExecution skips it.
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
			EnemyTargets: map[string]string{
				"enemy1": "player1", // enemy1 is dead, should be pruned
				"enemy2": "player1", // enemy2 is alive, should execute
			},
			EnemyDefenseTargets: map[string]string{},
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				{
					ID:       "player1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health": {Base: 50},
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				{
					ID:       "enemy1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health": {Base: 0}, // Dead!
					},
				},
				{
					ID:       "enemy2",
					Position: 1,
					Attributes: map[string]core.Attribute{
						"health": {Base: 30},
					},
				},
			},
		},
	}

	// Simulate enemy2 executing (which triggers pruning). enemy2 deals 5 damage to player1.
	msg := model.UnitDiceEffectsApplied{
		SourceUnitID: "enemy2",
		Results: []model.DiceEffectResult{
			{
				TargetUnitID: "player1",
				Effect:       entity.DieDamage,
				Value:        5,
				NewHealth:    45,
				NewShields:   0,
			},
		},
		Timestamp: 1000,
	}

	newM, _ := m.Update(msg)

	// enemy1's target entry should have been pruned (dead source).
	// enemy2→player1 survives (both alive).
	if _, ok := newM.Combat.EnemyTargets["enemy1"]; ok {
		t.Error("dead enemy1 should be pruned from EnemyTargets")
	}
	if len(newM.Combat.EnemyTargets) != 1 {
		t.Errorf("EnemyTargets len = %d, want 1", len(newM.Combat.EnemyTargets))
	}
}

func TestEnemyExecution_SkipsDeadTarget_ChainKill(t *testing.T) {
	// Bug 2: enemy1 kills player1, then enemy2 should not target the dead player1.
	// After enemy1's results are applied, pruning removes enemy2→player1 entry.
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
			EnemyTargets: map[string]string{
				"enemy1": "player1", // enemy1 just killed player1
				"enemy2": "player1", // enemy2 also targets player1 — should be pruned
			},
			EnemyDefenseTargets: map[string]string{},
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				{
					ID:       "player1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health": {Base: 10}, // Will die from enemy1's attack
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				{
					ID:       "enemy1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health": {Base: 40},
					},
				},
				{
					ID:       "enemy2",
					Position: 1,
					Attributes: map[string]core.Attribute{
						"health": {Base: 30},
					},
				},
			},
		},
	}

	// enemy1's effects kill player1 (HP 10 → 0)
	msg := model.UnitDiceEffectsApplied{
		SourceUnitID: "enemy1",
		Results: []model.DiceEffectResult{
			{
				TargetUnitID: "player1",
				Effect:       entity.DieDamage,
				Value:        10,
				NewHealth:    0,
				NewShields:   0,
			},
		},
		Timestamp: 1000,
	}

	newM, cmd := m.Update(msg)

	// Both entries pruned: enemy1→player1 (dead dest) and enemy2→player1 (dead dest).
	if len(newM.Combat.EnemyTargets) != 0 {
		t.Errorf("EnemyTargets len = %d, want 0; got %v", len(newM.Combat.EnemyTargets), newM.Combat.EnemyTargets)
	}

	// With no targets left, execution should transition to DicePhaseRoundEnd.
	if cmd != nil {
		t.Errorf("expected nil cmd (all remaining targets pruned), got non-nil")
	}
	if newM.Combat.DicePhase != model.DicePhaseRoundEnd {
		t.Errorf("DicePhase = %v, want DicePhaseRoundEnd", newM.Combat.DicePhase)
	}
}

func TestEnemyExecution_SkipsDeadDefenseTarget(t *testing.T) {
	// Bug 2 (defense variant): enemy_cmd has heal targeting dead enemy1.
	// Pruning should remove the entry.
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
			EnemyTargets:        map[string]string{},
			EnemyDefenseTargets: map[string]string{"enemy_cmd": "enemy1"},
			PlayerUnits: []entity.Unit{
				{
					ID:       "player_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				{
					ID:       "enemy1",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health": {Base: 0}, // Dead!
					},
				},
			},
		},
	}

	// Process a no-op UnitDiceEffectsApplied to trigger pruning
	msg := model.UnitDiceEffectsApplied{
		SourceUnitID: "enemy_cmd",
		Results:      nil,
		Timestamp:    1000,
	}

	newM, _ := m.Update(msg)

	if len(newM.Combat.EnemyDefenseTargets) != 0 {
		t.Errorf("EnemyDefenseTargets len = %d, want 0; got %v",
			len(newM.Combat.EnemyDefenseTargets), newM.Combat.EnemyDefenseTargets)
	}
}
