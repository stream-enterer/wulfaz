package tea

import (
	"testing"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

// ===== Targeting Tests =====

func TestGetOccupiedSpaces(t *testing.T) {
	tests := []struct {
		name     string
		unit     entity.Unit
		expected []int
	}{
		{
			name: "small unit width 1",
			unit: entity.Unit{
				ID:       "small",
				Position: 3,
				Attributes: map[string]core.Attribute{
					"combat_width": {Base: 1},
				},
			},
			expected: []int{3},
		},
		{
			name: "medium unit width 2",
			unit: entity.Unit{
				ID:       "medium",
				Position: 0,
				Attributes: map[string]core.Attribute{
					"combat_width": {Base: 2},
				},
			},
			expected: []int{0, 1},
		},
		{
			name: "large unit width 3",
			unit: entity.Unit{
				ID:       "large",
				Position: 5,
				Attributes: map[string]core.Attribute{
					"combat_width": {Base: 3},
				},
			},
			expected: []int{5, 6, 7},
		},
		{
			name: "off-board command unit",
			unit: entity.Unit{
				ID:       "cmd",
				Position: -1,
				Attributes: map[string]core.Attribute{
					"combat_width": {Base: 3},
				},
			},
			expected: nil,
		},
		{
			name: "unit without combat_width defaults to 1",
			unit: entity.Unit{
				ID:         "nowidth",
				Position:   2,
				Attributes: map[string]core.Attribute{},
			},
			expected: []int{2},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := GetOccupiedSpaces(tt.unit)
			if len(result) != len(tt.expected) {
				t.Errorf("GetOccupiedSpaces() = %v, want %v", result, tt.expected)
				return
			}
			for i := range result {
				if result[i] != tt.expected[i] {
					t.Errorf("GetOccupiedSpaces()[%d] = %d, want %d", i, result[i], tt.expected[i])
				}
			}
		})
	}
}

func TestFindOverlappingEnemies(t *testing.T) {
	// Attacker at position 0-1 (width 2)
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
			ID:       "enemy1",
			Position: 0, // Overlaps at 0
			Attributes: map[string]core.Attribute{
				"combat_width": {Base: 1},
				"health":       {Base: 30},
			},
		},
		{
			ID:       "enemy2",
			Position: 5, // No overlap
			Attributes: map[string]core.Attribute{
				"combat_width": {Base: 1},
				"health":       {Base: 30},
			},
		},
		{
			ID:       "enemy3",
			Position: 1, // Overlaps at 1
			Attributes: map[string]core.Attribute{
				"combat_width": {Base: 2},
				"health":       {Base: 30},
			},
		},
	}

	result := FindOverlappingEnemies(attacker, enemies)

	if len(result) != 2 {
		t.Fatalf("FindOverlappingEnemies() returned %d enemies, want 2", len(result))
	}
	// Should find enemy1 and enemy3, not enemy2
	ids := map[string]bool{}
	for _, e := range result {
		ids[e.ID] = true
	}
	if !ids["enemy1"] || !ids["enemy3"] || ids["enemy2"] {
		t.Errorf("Expected enemy1 and enemy3, got %v", ids)
	}
}

func TestFindOverlappingEnemies_ExcludesDeadAndOffboard(t *testing.T) {
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
			ID:       "dead_enemy",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 0}, // Dead
			},
		},
		{
			ID:       "offboard_enemy",
			Position: -1, // Off-board
			Attributes: map[string]core.Attribute{
				"health": {Base: 30},
			},
		},
	}

	result := FindOverlappingEnemies(attacker, enemies)

	if len(result) != 0 {
		t.Errorf("FindOverlappingEnemies() should exclude dead/offboard, got %d", len(result))
	}
}

func TestSelectTarget_Gap(t *testing.T) {
	// Attacker with no overlapping enemies and ALL enemies dead -> should target command unit
	// Per F-167: Units only target units. Command only targetable when all enemy units dead.
	attacker := entity.Unit{
		ID:       "attacker",
		Position: 9, // Far right, no enemies
		Attributes: map[string]core.Attribute{
			"combat_width": {Base: 1},
			"health":       {Base: 50},
		},
	}

	enemies := []entity.Unit{
		{
			ID:       "enemy1",
			Position: 0, // No overlap with position 9
			Attributes: map[string]core.Attribute{
				"health": {Base: 0}, // Dead - so command is targetable
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
		t.Errorf("SelectTarget() = %s, want enemy_cmd (gap + all units dead -> command)", result)
	}
}

func TestSelectTarget_OverlappingPreferred(t *testing.T) {
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
			ID:       "overlapping",
			Position: 0,
			Attributes: map[string]core.Attribute{
				"health": {Base: 30},
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

	if result != "overlapping" {
		t.Errorf("SelectTarget() = %s, want overlapping (prefer overlap over gap)", result)
	}
}

// ===== Execution Flow Tests =====

func TestExecutionStarted_EmptyBoard(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
		},
	}

	msg := ExecutionStarted{FiringOrder: nil}
	newM, cmd := m.Update(msg)

	if len(newM.Combat.FiringOrder) != 0 {
		t.Error("FiringOrder should be empty")
	}

	// Should return ExecutionComplete
	if cmd == nil {
		t.Fatal("expected cmd")
	}
	result := cmd()
	if _, ok := result.(ExecutionComplete); !ok {
		t.Errorf("expected ExecutionComplete, got %T", result)
	}
}

func TestPositionResolved_AppliesDamage(t *testing.T) {
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
			FiringOrder:        []model.FiringPosition{{Position: 0}},
			CurrentFiringIndex: 0,
		},
	}

	msg := PositionResolved{
		Position: 0,
		Attacks: []AttackResult{
			{
				AttackerID: "player1",
				TargetID:   "enemy1",
				DieIndex:   0,
				Damage:     20,
				NewHealth:  30,
				NewShields: 0,
				TargetDead: false,
			},
		},
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

func TestPositionResolved_VictoryCheck(t *testing.T) {
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
			FiringOrder:        []model.FiringPosition{{Position: 0}},
			CurrentFiringIndex: 0,
		},
	}

	msg := PositionResolved{
		Position: 0,
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
	}

	// Step 1: PositionResolved applies damage and creates floating texts
	newM, cmd := m.Update(msg)

	// Should return timer request for round end pause on victory
	if cmd == nil {
		t.Fatal("expected timer cmd, got nil")
	}
	timerReq, ok := cmd().(StartTimerRequested)
	if !ok {
		t.Fatalf("expected StartTimerRequested, got %T", cmd())
	}
	if timerReq.ID != TimerRoundEnd {
		t.Errorf("timer ID = %s, want %s", timerReq.ID, TimerRoundEnd)
	}

	// Step 2: TimerFired triggers victory check
	newM, cmd = newM.Update(TimerFired{ID: TimerRoundEnd})

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

// ===== Shield Tests =====

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

func TestEnemyCommandResolved_AppliesEffects(t *testing.T) {
	m := Model{
		Version: 1,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseEnemyCommand,
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
			},
		},
	}

	msg := EnemyCommandResolved{
		Actions: []EnemyDiceAction{
			{
				SourceUnitID: "enemy_cmd",
				TargetUnitID: "player1",
				DieIndex:     0,
				Effect:       entity.DieDamage,
				Value:        12,
			},
		},
	}

	newM, cmd := m.Update(msg)

	// Check damage applied
	for _, u := range newM.Combat.PlayerUnits {
		if u.ID == "player1" {
			if u.Attributes["health"].Base != 38 { // 50 - 12
				t.Errorf("player1 health = %d, want 38", u.Attributes["health"].Base)
			}
		}
	}

	// Should advance to execution
	if newM.Combat.DicePhase != model.DicePhaseExecution {
		t.Errorf("DicePhase = %v, want Execution", newM.Combat.DicePhase)
	}

	// Should return ExecuteExecution cmd
	if cmd == nil {
		t.Fatal("expected cmd")
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

// ===== Helper Function Tests =====

func TestBuildFiringOrder(t *testing.T) {
	combat := model.CombatModel{
		PlayerUnits: []entity.Unit{
			{
				ID:       "player_cmd",
				Position: -1, // Off-board, excluded
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
			{
				ID:       "player2",
				Position: 5,
				Attributes: map[string]core.Attribute{
					"health": {Base: 30},
				},
			},
		},
		EnemyUnits: []entity.Unit{
			{
				ID:       "enemy_cmd",
				Position: -1, // Off-board, excluded
				Tags:     []core.Tag{"command"},
				Attributes: map[string]core.Attribute{
					"health": {Base: 100},
				},
			},
			{
				ID:       "enemy1",
				Position: 0, // Same position as player1
				Attributes: map[string]core.Attribute{
					"health": {Base: 40},
				},
			},
			{
				ID:       "enemy2",
				Position: 3,
				Attributes: map[string]core.Attribute{
					"health": {Base: 35},
				},
			},
		},
	}

	order := buildFiringOrder(combat)

	// Should have positions 0, 3, 5 (left-to-right)
	if len(order) != 3 {
		t.Fatalf("buildFiringOrder() returned %d positions, want 3", len(order))
	}

	expectedPositions := []int{0, 3, 5}
	for i, pos := range expectedPositions {
		if order[i].Position != pos {
			t.Errorf("order[%d].Position = %d, want %d", i, order[i].Position, pos)
		}
	}

	// Position 0 should have both player1 and enemy1
	if len(order[0].PlayerUnits) != 1 || order[0].PlayerUnits[0] != "player1" {
		t.Errorf("order[0].PlayerUnits = %v, want [player1]", order[0].PlayerUnits)
	}
	if len(order[0].EnemyUnits) != 1 || order[0].EnemyUnits[0] != "enemy1" {
		t.Errorf("order[0].EnemyUnits = %v, want [enemy1]", order[0].EnemyUnits)
	}
}

func TestBuildFiringOrder_ExcludesDeadUnits(t *testing.T) {
	combat := model.CombatModel{
		PlayerUnits: []entity.Unit{
			{
				ID:       "dead_player",
				Position: 0,
				Attributes: map[string]core.Attribute{
					"health": {Base: 0}, // Dead
				},
			},
		},
		EnemyUnits: []entity.Unit{
			{
				ID:       "alive_enemy",
				Position: 0,
				Attributes: map[string]core.Attribute{
					"health": {Base: 50},
				},
			},
		},
	}

	order := buildFiringOrder(combat)

	if len(order) != 1 {
		t.Fatalf("expected 1 position, got %d", len(order))
	}
	if len(order[0].PlayerUnits) != 0 {
		t.Errorf("dead player should not be in firing order")
	}
	if len(order[0].EnemyUnits) != 1 {
		t.Errorf("alive enemy should be in firing order")
	}
}

func TestFindLowestHPAliveUnit(t *testing.T) {
	units := []entity.Unit{
		{
			ID: "high_hp",
			Attributes: map[string]core.Attribute{
				"health": {Base: 100},
			},
		},
		{
			ID: "low_hp",
			Attributes: map[string]core.Attribute{
				"health": {Base: 20},
			},
		},
		{
			ID: "dead",
			Attributes: map[string]core.Attribute{
				"health": {Base: 0},
			},
		},
		{
			ID: "medium_hp",
			Attributes: map[string]core.Attribute{
				"health": {Base: 50},
			},
		},
	}

	result := findLowestHPAliveUnit(units)

	if result != "low_hp" {
		t.Errorf("findLowestHPAliveUnit() = %s, want low_hp", result)
	}
}

func TestFindLowestHPAliveUnit_Empty(t *testing.T) {
	var units []entity.Unit
	result := findLowestHPAliveUnit(units)
	if result != "" {
		t.Errorf("findLowestHPAliveUnit(empty) = %s, want empty", result)
	}
}

// ===== Wave 4: Overflow Integration Tests =====

func TestResolvePosition_OverflowDamage(t *testing.T) {
	// Setup: Attacker with 50 damage die, two overlapping enemies (30 and 40 HP)
	// Expected: 50 damage kills first (30 HP), 20 overflow to second
	combat := model.CombatModel{
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
			{
				ID:       "attacker",
				Position: 0,
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 50}}}},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 50},
					"combat_width": {Base: 2},
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
		},
		RolledDice: map[string][]entity.RolledDie{
			"attacker": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 50}}, FaceIndex: 0}},
		},
		FiringOrder:        []model.FiringPosition{{Position: 0, PlayerUnits: []string{"attacker"}}},
		CurrentFiringIndex: 0,
	}

	cmd := ResolvePosition(model.FiringPosition{Position: 0, PlayerUnits: []string{"attacker"}}, combat, 0)
	msg := cmd()

	resolved, ok := msg.(PositionResolved)
	if !ok {
		t.Fatalf("expected PositionResolved, got %T", msg)
	}

	// Should have 2 attack results (overflow)
	if len(resolved.Attacks) != 2 {
		t.Fatalf("expected 2 attacks (overflow), got %d", len(resolved.Attacks))
	}

	// First attack should kill enemy1
	if resolved.Attacks[0].TargetID != "enemy1" {
		t.Errorf("first attack target = %s, want enemy1", resolved.Attacks[0].TargetID)
	}
	if resolved.Attacks[0].Damage != 30 {
		t.Errorf("first attack damage = %d, want 30", resolved.Attacks[0].Damage)
	}
	if !resolved.Attacks[0].TargetDead {
		t.Error("first attack should kill enemy1")
	}

	// Second attack should overflow to enemy2
	if resolved.Attacks[1].TargetID != "enemy2" {
		t.Errorf("second attack target = %s, want enemy2", resolved.Attacks[1].TargetID)
	}
	if resolved.Attacks[1].Damage != 20 {
		t.Errorf("second attack damage = %d, want 20 (overflow)", resolved.Attacks[1].Damage)
	}
	if resolved.Attacks[1].NewHealth != 20 {
		t.Errorf("enemy2 NewHP = %d, want 20", resolved.Attacks[1].NewHealth)
	}
}

func TestResolvePosition_GapToCommand(t *testing.T) {
	// Setup: All enemies dead, attacker has gap -> damage hits command
	combat := model.CombatModel{
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
			{
				ID:       "attacker",
				Position: 9, // Far right, no enemies overlap
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 25}}}},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 50},
					"combat_width": {Base: 1},
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
				ID:       "dead_enemy",
				Position: 0,
				Attributes: map[string]core.Attribute{
					"health": {Base: 0}, // Dead
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"attacker": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 25}}, FaceIndex: 0}},
		},
		FiringOrder:        []model.FiringPosition{{Position: 9, PlayerUnits: []string{"attacker"}}},
		CurrentFiringIndex: 0,
	}

	cmd := ResolvePosition(model.FiringPosition{Position: 9, PlayerUnits: []string{"attacker"}}, combat, 0)
	msg := cmd()

	resolved, ok := msg.(PositionResolved)
	if !ok {
		t.Fatalf("expected PositionResolved, got %T", msg)
	}

	// Should hit command unit
	if len(resolved.Attacks) != 1 {
		t.Fatalf("expected 1 attack, got %d", len(resolved.Attacks))
	}
	if resolved.Attacks[0].TargetID != "enemy_cmd" {
		t.Errorf("attack target = %s, want enemy_cmd", resolved.Attacks[0].TargetID)
	}
	if resolved.Attacks[0].Damage != 25 {
		t.Errorf("attack damage = %d, want 25", resolved.Attacks[0].Damage)
	}
}

func TestResolvePosition_GapWithLiveUnits_DamageWasted(t *testing.T) {
	// Setup: Gap exists but live enemies elsewhere -> damage wasted (F-167)
	combat := model.CombatModel{
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
			{
				ID:       "attacker",
				Position: 9, // Far right, no enemies overlap
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 25}}}},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 50},
					"combat_width": {Base: 1},
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
				ID:       "alive_enemy",
				Position: 0, // Alive but not overlapping
				Attributes: map[string]core.Attribute{
					"health": {Base: 30},
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"attacker": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 25}}, FaceIndex: 0}},
		},
		FiringOrder:        []model.FiringPosition{{Position: 9, PlayerUnits: []string{"attacker"}}},
		CurrentFiringIndex: 0,
	}

	cmd := ResolvePosition(model.FiringPosition{Position: 9, PlayerUnits: []string{"attacker"}}, combat, 0)
	msg := cmd()

	resolved, ok := msg.(PositionResolved)
	if !ok {
		t.Fatalf("expected PositionResolved, got %T", msg)
	}

	// Should have NO attacks (damage wasted)
	if len(resolved.Attacks) != 0 {
		t.Errorf("expected 0 attacks (damage wasted), got %d", len(resolved.Attacks))
		for _, a := range resolved.Attacks {
			t.Logf("  attack: %+v", a)
		}
	}
}

func TestResolvePosition_MultiDieSeparateTargets(t *testing.T) {
	// Setup: Two damage dice, first kills target, second should find new target
	combat := model.CombatModel{
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
			{
				ID:       "attacker",
				Position: 0,
				Dice: []entity.Die{
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 30}}},
					{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 20}}},
				},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 50},
					"combat_width": {Base: 2},
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
					"health": {Base: 20}, // Lowest HP, will be killed by die 1
				},
			},
			{
				ID:       "enemy2",
				Position: 1,
				Attributes: map[string]core.Attribute{
					"health": {Base: 40},
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"attacker": {
				{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 30}}, FaceIndex: 0},
				{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 20}}, FaceIndex: 0},
			},
		},
		FiringOrder:        []model.FiringPosition{{Position: 0, PlayerUnits: []string{"attacker"}}},
		CurrentFiringIndex: 0,
	}

	cmd := ResolvePosition(model.FiringPosition{Position: 0, PlayerUnits: []string{"attacker"}}, combat, 0)
	msg := cmd()

	resolved, ok := msg.(PositionResolved)
	if !ok {
		t.Fatalf("expected PositionResolved, got %T", msg)
	}

	// Die 1 (30 damage) hits enemy1 (20 HP), kills it, 10 overflow to enemy2
	// Die 2 (20 damage) hits enemy2 (now 30 HP after overflow)
	// Total: enemy1 killed, enemy2 takes 10+20 = 30 damage -> 10 HP left

	// Count attacks per target
	enemy1Attacks := 0
	enemy2Attacks := 0
	var enemy2TotalDamage int
	for _, a := range resolved.Attacks {
		if a.TargetID == "enemy1" {
			enemy1Attacks++
		} else if a.TargetID == "enemy2" {
			enemy2Attacks++
			enemy2TotalDamage += a.Damage
		}
	}

	if enemy1Attacks != 1 {
		t.Errorf("enemy1 attacks = %d, want 1", enemy1Attacks)
	}
	// enemy2 should be hit by overflow from die 1 + full die 2
	if enemy2Attacks < 1 {
		t.Errorf("enemy2 attacks = %d, want at least 1", enemy2Attacks)
	}
	// Total damage to enemy2 should be 10 (overflow) + 20 (die 2) = 30
	if enemy2TotalDamage != 30 {
		t.Errorf("enemy2 total damage = %d, want 30", enemy2TotalDamage)
	}
}

func TestResolvePosition_OverflowStopsAtCommand(t *testing.T) {
	// Setup: 100 damage, only 50 HP overlapping -> excess wasted, NOT to command
	combat := model.CombatModel{
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
			{
				ID:       "attacker",
				Position: 0,
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 100}}}},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 50},
					"combat_width": {Base: 1},
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
		RolledDice: map[string][]entity.RolledDie{
			"attacker": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 100}}, FaceIndex: 0}},
		},
		FiringOrder:        []model.FiringPosition{{Position: 0, PlayerUnits: []string{"attacker"}}},
		CurrentFiringIndex: 0,
	}

	cmd := ResolvePosition(model.FiringPosition{Position: 0, PlayerUnits: []string{"attacker"}}, combat, 0)
	msg := cmd()

	resolved, ok := msg.(PositionResolved)
	if !ok {
		t.Fatalf("expected PositionResolved, got %T", msg)
	}

	// Should only hit enemy1, NOT overflow to command
	if len(resolved.Attacks) != 1 {
		t.Errorf("expected 1 attack (no overflow to command), got %d", len(resolved.Attacks))
	}
	if len(resolved.Attacks) > 0 && resolved.Attacks[0].TargetID != "enemy1" {
		t.Errorf("attack target = %s, want enemy1", resolved.Attacks[0].TargetID)
	}
	// Verify command not hit
	for _, a := range resolved.Attacks {
		if a.TargetID == "enemy_cmd" {
			t.Error("overflow should NOT hit command unit")
		}
	}
}

// ===== Round Skip Prevention Tests =====

func TestMultipleExecutionComplete_NoRoundSkip(t *testing.T) {
	// Regression test: multiple ExecutionComplete messages should not cause round skipping
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

	// First ExecutionComplete should transition to RoundEnd and trigger RoundEnded
	m1, cmd1 := m.Update(ExecutionComplete{})
	if m1.Combat.DicePhase != model.DicePhaseRoundEnd {
		t.Errorf("after first ExecutionComplete: DicePhase = %v, want DicePhaseRoundEnd", m1.Combat.DicePhase)
	}
	if cmd1 == nil {
		t.Fatal("expected RoundEnded cmd, got nil")
	}

	// Process the RoundEnded from first ExecutionComplete
	msg1 := cmd1()
	if _, ok := msg1.(RoundEnded); !ok {
		t.Fatalf("expected RoundEnded, got %T", msg1)
	}

	m2, _ := m1.Update(msg1)
	if m2.Combat.Round != 2 {
		t.Errorf("Round = %d, want 2", m2.Combat.Round)
	}

	// Additional RoundEnded messages should be ignored (phase changed by handleRoundEnded)
	// Simulate what happens when multiple timers fire: phase is no longer RoundEnd
	m3, cmd3 := m2.Update(RoundEnded{})
	if cmd3 != nil {
		t.Error("duplicate RoundEnded should return nil cmd")
	}
	if m3.Combat.Round != 2 {
		t.Errorf("Round after duplicate = %d, want 2 (no change)", m3.Combat.Round)
	}
}

func TestExecutionAdvanceClicked_SetsRoundEndPhase(t *testing.T) {
	// Verify that when all positions are resolved, clicking sets DicePhaseRoundEnd
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Seed:    42,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseExecution,
			Round:     1,
			FiringOrder: []model.FiringPosition{
				{Position: 0, PlayerUnits: []string{"p1"}},
			},
			CurrentFiringIndex: 1, // All positions already resolved
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

	// Click when all positions resolved should set RoundEnd phase
	m1, _ := m.Update(ExecutionAdvanceClicked{Timestamp: 1000})
	if m1.Combat.DicePhase != model.DicePhaseRoundEnd {
		t.Errorf("DicePhase = %v, want DicePhaseRoundEnd", m1.Combat.DicePhase)
	}

	// Second click should be ignored (not in Execution phase anymore)
	m2, cmd2 := m1.Update(ExecutionAdvanceClicked{Timestamp: 2000})
	if cmd2 != nil {
		t.Error("second click should be ignored when not in Execution phase")
	}
	if m2.Combat.DicePhase != model.DicePhaseRoundEnd {
		t.Errorf("DicePhase after second click = %v, want DicePhaseRoundEnd", m2.Combat.DicePhase)
	}
}

// ===== F-192: Dead Target Skip Tests =====

func TestEnemyCommand_TargetDiesFromEarlierDie(t *testing.T) {
	// Setup:
	// - Enemy command has 3 damage dice, each dealing 10 damage
	// - Player unit has 10 HP (will die from first die)
	// Flow:
	// 1. All 3 dice target the same player unit (lowest HP)
	// 2. Die 1 kills the unit
	// 3. Dies 2 and 3 should skip (target dead)
	// Verify: Only 10 damage applied, 2 skips logged, no crash
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
		Combat: model.CombatModel{
			Phase:     model.CombatActive,
			DicePhase: model.DicePhaseEnemyCommand,
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
					ID:       "low_hp_unit",
					Position: 0,
					Attributes: map[string]core.Attribute{
						"health": {Base: 10}, // Will die from first die
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

	// Simulate 3 dice all targeting the same unit
	msg := EnemyCommandResolved{
		Actions: []EnemyDiceAction{
			{SourceUnitID: "enemy_cmd", TargetUnitID: "low_hp_unit", DieIndex: 0, Effect: entity.DieDamage, Value: 10},
			{SourceUnitID: "enemy_cmd", TargetUnitID: "low_hp_unit", DieIndex: 1, Effect: entity.DieDamage, Value: 10},
			{SourceUnitID: "enemy_cmd", TargetUnitID: "low_hp_unit", DieIndex: 2, Effect: entity.DieDamage, Value: 10},
		},
	}

	newM, _ := m.Update(msg)

	// The unit should be dead (first die killed it)
	var targetUnit *entity.Unit
	for i := range newM.Combat.PlayerUnits {
		if newM.Combat.PlayerUnits[i].ID == "low_hp_unit" {
			targetUnit = &newM.Combat.PlayerUnits[i]
			break
		}
	}

	if targetUnit == nil {
		t.Fatal("low_hp_unit not found")
	}

	// HP should be 0 (first die dealt 10, killed it)
	hp := 0
	if h, ok := targetUnit.Attributes["health"]; ok {
		hp = h.Base
	}
	if hp != 0 {
		t.Errorf("low_hp_unit HP = %d, want 0", hp)
	}

	// Log should contain skip messages
	skipCount := 0
	for _, entry := range newM.Combat.Log {
		if entry == "Enemy: enemy_cmd skipped (target dead)" {
			skipCount++
		}
	}
	if skipCount != 2 {
		t.Errorf("expected 2 skip messages in log, got %d", skipCount)
	}
}

// ===== F-193: Simultaneous Death Resolution Tests =====

func TestResolvePosition_MutualKill(t *testing.T) {
	// Setup: Two units at same position, each can kill the other
	// Both should deal damage since attacks are calculated before HP is updated
	combat := model.CombatModel{
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
			{
				ID:       "player1",
				Position: 0,
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 50}}}},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 30}, // Will die from enemy
					"combat_width": {Base: 1},
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
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 50}}}},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 30}, // Will die from player
					"combat_width": {Base: 1},
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"player1": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 50}}, FaceIndex: 0}},
			"enemy1":  {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 50}}, FaceIndex: 0}},
		},
		FiringOrder:        []model.FiringPosition{{Position: 0, PlayerUnits: []string{"player1"}, EnemyUnits: []string{"enemy1"}}},
		CurrentFiringIndex: 0,
	}

	cmd := ResolvePosition(model.FiringPosition{Position: 0, PlayerUnits: []string{"player1"}, EnemyUnits: []string{"enemy1"}}, combat, 0)
	msg := cmd()

	resolved, ok := msg.(PositionResolved)
	if !ok {
		t.Fatalf("expected PositionResolved, got %T", msg)
	}

	// Both units should have attacked (player-first, but both should attack)
	playerAttacked := false
	enemyAttacked := false
	for _, a := range resolved.Attacks {
		if a.AttackerID == "player1" {
			playerAttacked = true
		}
		if a.AttackerID == "enemy1" {
			enemyAttacked = true
		}
	}

	if !playerAttacked {
		t.Error("player1 should have attacked")
	}
	if !enemyAttacked {
		t.Error("enemy1 should have attacked (simultaneous resolution)")
	}
}

func TestResolvePosition_DeadAttackerStillAttacks(t *testing.T) {
	// Setup: Unit A kills Unit B first (player-first), Unit B also has damage dice
	// Verify: Unit B still deals damage (was alive at start of position resolution)
	combat := model.CombatModel{
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
			{
				ID:       "player1",
				Position: 0,
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 100}}}},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 50},
					"combat_width": {Base: 1},
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
				Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 25}}}},
				Attributes: map[string]core.Attribute{
					"health":       {Base: 20}, // Will die from player's 100 damage
					"combat_width": {Base: 1},
				},
			},
		},
		RolledDice: map[string][]entity.RolledDie{
			"player1": {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 100}}, FaceIndex: 0}},
			"enemy1":  {{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 25}}, FaceIndex: 0}},
		},
		FiringOrder:        []model.FiringPosition{{Position: 0, PlayerUnits: []string{"player1"}, EnemyUnits: []string{"enemy1"}}},
		CurrentFiringIndex: 0,
	}

	cmd := ResolvePosition(model.FiringPosition{Position: 0, PlayerUnits: []string{"player1"}, EnemyUnits: []string{"enemy1"}}, combat, 0)
	msg := cmd()

	resolved, ok := msg.(PositionResolved)
	if !ok {
		t.Fatalf("expected PositionResolved, got %T", msg)
	}

	// Enemy1 should still attack player1, even though player kills enemy first (HP snapshot)
	enemyDamageDealt := 0
	for _, a := range resolved.Attacks {
		if a.AttackerID == "enemy1" && a.TargetID == "player1" {
			enemyDamageDealt += a.Damage
		}
	}

	if enemyDamageDealt != 25 {
		t.Errorf("enemy1 damage dealt = %d, want 25 (should attack via HP snapshot)", enemyDamageDealt)
	}
}

// ===== F-190: Pure Command vs Command Tests =====

func TestCombat_OnlyCommandsRemain(t *testing.T) {
	// Setup: Both sides have only command units (all board units dead)
	// Flow: Round starts, dice roll, player activates, enemy activates
	// Verify: Combat functions correctly, no panic on empty firing order
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
					Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 10}}}},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				// No board units (or all dead)
			},
			EnemyUnits: []entity.Unit{
				{
					ID:       "enemy_cmd",
					Position: -1,
					Tags:     []core.Tag{"command"},
					Dice:     []entity.Die{{Faces: []entity.DieFace{{Type: entity.DieDamage, Value: 10}}}},
					Attributes: map[string]core.Attribute{
						"health": {Base: 100},
					},
				},
				// No board units (or all dead)
			},
			FiringOrder:        []model.FiringPosition{}, // Empty - no board units
			CurrentFiringIndex: 0,
		},
	}

	// Execution with empty firing order should complete immediately
	msg := ExecutionStarted{FiringOrder: nil}
	newM, cmd := m.Update(msg)

	if cmd == nil {
		t.Fatal("expected cmd for empty execution")
	}

	// Should return ExecutionComplete immediately
	result := cmd()
	if _, ok := result.(ExecutionComplete); !ok {
		t.Errorf("expected ExecutionComplete for empty firing order, got %T", result)
	}

	// Verify FiringOrder is empty
	if len(newM.Combat.FiringOrder) != 0 {
		t.Errorf("FiringOrder should be empty, got %d positions", len(newM.Combat.FiringOrder))
	}
}

func TestExecution_EmptyFiringOrder(t *testing.T) {
	// Setup: Create ExecutionStarted with empty FiringOrder
	// Verify: Handler returns ExecutionComplete{} immediately
	m := Model{
		Version: 1,
		Phase:   PhaseCombat,
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
			},
		},
	}

	// ExecutionStarted with nil FiringOrder
	msg := ExecutionStarted{FiringOrder: nil}
	_, cmd := m.Update(msg)

	if cmd == nil {
		t.Fatal("expected cmd")
	}

	result := cmd()
	if _, ok := result.(ExecutionComplete); !ok {
		t.Errorf("expected ExecutionComplete, got %T", result)
	}

	// Also test with empty slice
	msg2 := ExecutionStarted{FiringOrder: []model.FiringPosition{}}
	_, cmd2 := m.Update(msg2)

	if cmd2 == nil {
		t.Fatal("expected cmd for empty slice")
	}

	result2 := cmd2()
	if _, ok := result2.(ExecutionComplete); !ok {
		t.Errorf("expected ExecutionComplete for empty slice, got %T", result2)
	}
}
