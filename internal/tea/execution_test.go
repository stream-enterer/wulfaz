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

	// Step 1: PositionResolved starts flash timer (Wave 7)
	newM, cmd := m.Update(msg)

	// Flash targets should be set
	if newM.Combat.FlashTargets == nil || len(newM.Combat.FlashTargets) == 0 {
		t.Errorf("FlashTargets should be set after PositionResolved")
	}

	// Should return timer request for flash display
	if cmd == nil {
		t.Fatal("expected timer cmd, got nil")
	}
	timerReq, ok := cmd().(StartTimerRequested)
	if !ok {
		t.Fatalf("expected StartTimerRequested, got %T", cmd())
	}
	if timerReq.ID != TimerExecAdvance {
		t.Errorf("timer ID = %s, want %s", timerReq.ID, TimerExecAdvance)
	}

	// Step 2: TimerFired triggers victory check
	newM, cmd = newM.Update(TimerFired{ID: TimerExecAdvance})

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

	cmd := ResolvePosition(model.FiringPosition{Position: 0, PlayerUnits: []string{"attacker"}}, combat)
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

	cmd := ResolvePosition(model.FiringPosition{Position: 9, PlayerUnits: []string{"attacker"}}, combat)
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

	cmd := ResolvePosition(model.FiringPosition{Position: 9, PlayerUnits: []string{"attacker"}}, combat)
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

	cmd := ResolvePosition(model.FiringPosition{Position: 0, PlayerUnits: []string{"attacker"}}, combat)
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

	cmd := ResolvePosition(model.FiringPosition{Position: 0, PlayerUnits: []string{"attacker"}}, combat)
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
