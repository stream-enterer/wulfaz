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

	newM, _ := m.Update(RoundEnded{})

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
		Phase:   PhaseMenu,
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

	msg := CombatStarted{Combat: combat}
	newM, cmd := m.Update(msg)

	if newM.Phase != PhaseCombat {
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
	if _, ok := result.(RoundStarted); !ok {
		t.Errorf("expected RoundStarted, got %T", result)
	}
}

func TestCheckCombatEnd_CommandUnitBased(t *testing.T) {
	tests := []struct {
		name           string
		playerCmdAlive bool
		enemyCmdAlive  bool
		expected       Victor
	}{
		{"both alive", true, true, VictorNone},
		{"enemy cmd dead", true, false, VictorPlayer},
		{"player cmd dead", false, true, VictorEnemy},
		{"both dead - player wins tie", false, false, VictorPlayer},
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

// ===== All Attacks Resolved Tests =====

func TestAllAttacksResolved_AppliesDamage(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
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

	msg := AllAttacksResolved{
		Attacks: []AttackResult{
			{
				AttackerID: "player1",
				TargetID:   "enemy1",
				Damage:     20,
				NewHealth:  30,
				NewShields: 0,
				TargetDead: false,
			},
		},
		Timestamp: 1000,
	}

	newM, _ := m.Update(msg)

	// Check damage applied to enemy1
	for _, u := range newM.Combat.EnemyUnits {
		if u.ID == "enemy1" {
			if u.Attributes["health"].Base != 30 {
				t.Errorf("enemy1 health = %d, want 30", u.Attributes["health"].Base)
			}
		}
	}
}

func TestAllAttacksResolved_VictoryCheck(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
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
			},
		},
	}

	msg := AllAttacksResolved{
		Attacks: []AttackResult{
			{
				AttackerID: "player1",
				TargetID:   "enemy_cmd",
				Damage:     10,
				NewHealth:  0,
				NewShields: 0,
				TargetDead: true,
			},
		},
		Timestamp: 1000,
	}

	newM, cmd := m.Update(msg)

	// Should return nil (waiting for click, not timer)
	if cmd != nil {
		t.Fatalf("expected nil cmd, got %T", cmd())
	}

	// Step 2: RoundEndClicked triggers victory check
	newM, cmd = newM.Update(RoundEndClicked{})

	// Combat should end
	if newM.Combat.Phase != model.CombatResolved {
		t.Errorf("Combat.Phase = %v, want CombatResolved", newM.Combat.Phase)
	}
	if newM.Combat.Victor != "player" {
		t.Errorf("Victor = %s, want player", newM.Combat.Victor)
	}

	// Should return CombatEnded
	if cmd != nil {
		result := cmd()
		if ended, ok := result.(CombatEnded); !ok || ended.Victor != VictorPlayer {
			t.Errorf("expected CombatEnded{VictorPlayer}, got %T", result)
		}
	}
}

// ===== Execution Complete Tests =====

func TestExecutionComplete_TransitionsToRoundEnd(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
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
	m1, cmd1 := m.Update(ExecutionComplete{})
	if m1.Combat.DicePhase != model.DicePhaseRoundEnd {
		t.Errorf("after ExecutionComplete: DicePhase = %v, want DicePhaseRoundEnd", m1.Combat.DicePhase)
	}
	if cmd1 == nil {
		t.Fatal("expected RoundEnded cmd, got nil")
	}

	// Process the RoundEnded
	msg1 := cmd1()
	if _, ok := msg1.(RoundEnded); !ok {
		t.Fatalf("expected RoundEnded, got %T", msg1)
	}

	m2, _ := m1.Update(msg1)
	if m2.Combat.Round != 2 {
		t.Errorf("Round = %d, want 2", m2.Combat.Round)
	}
}

// ===== Defense Results Tests =====

func TestAllAttacksResolved_DefenseResults(t *testing.T) {
	// Enemy shield/heal dice should apply to enemy allies via DefenseResults.
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
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

	msg := AllAttacksResolved{
		Attacks: []AttackResult{}, // No damage attacks
		DefenseResults: []DiceEffectResult{
			{
				TargetUnitID: "enemy1",
				Effect:       entity.DieShield,
				Value:        5,
				NewShields:   5,
			},
			{
				TargetUnitID: "enemy_cmd",
				Effect:       entity.DieHeal,
				Value:        10,
				NewHealth:    90,
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
