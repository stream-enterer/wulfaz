package tea

import (
	"fmt"
	"testing"

	"wulfaz/internal/model"
)

func TestUpdate_PlayerQuit(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
	}

	newModel, cmd := m.Update(model.PlayerQuit{})

	if newModel.Phase != model.PhaseGameOver {
		t.Errorf("expected PhaseGameOver, got %d", newModel.Phase)
	}

	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_CombatEnded_PlayerWins(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
	}

	newModel, cmd := m.Update(model.CombatEnded{Victor: model.VictorPlayer})

	if newModel.Phase != model.PhaseInterCombat {
		t.Errorf("expected PhaseInterCombat, got %d", newModel.Phase)
	}
	if newModel.ChoiceType != model.ChoiceReward {
		t.Errorf("expected ChoiceReward, got %d", newModel.ChoiceType)
	}
	if newModel.RewardChoicesLeft != 2 {
		t.Errorf("expected 2 reward choices left, got %d", newModel.RewardChoicesLeft)
	}
	if len(newModel.Choices) != 3 {
		t.Errorf("expected 3 choices, got %d", len(newModel.Choices))
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_CombatEnded_PlayerLoses(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
	}

	newModel, cmd := m.Update(model.CombatEnded{Victor: model.VictorEnemy})

	if newModel.Phase != model.PhaseGameOver {
		t.Errorf("expected PhaseGameOver, got %d", newModel.Phase)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_CombatEnded_Draw(t *testing.T) {
	m := Model{
		Version: 1,
		Phase:   model.PhaseCombat,
	}

	newModel, cmd := m.Update(model.CombatEnded{Victor: model.VictorDraw})

	if newModel.Phase != model.PhaseGameOver {
		t.Errorf("expected PhaseGameOver, got %d", newModel.Phase)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_ChoiceSelected_Reward(t *testing.T) {
	m := Model{
		Version:           1,
		Phase:             model.PhaseInterCombat,
		ChoiceType:        model.ChoiceReward,
		RewardChoicesLeft: 2,
		Choices:           []string{"A", "B", "C"},
	}

	// First reward selection
	newModel, cmd := m.Update(model.ChoiceSelected{Index: 0})

	if newModel.RewardChoicesLeft != 1 {
		t.Errorf("expected 1 reward choice left, got %d", newModel.RewardChoicesLeft)
	}
	if newModel.ChoiceType != model.ChoiceReward {
		t.Errorf("expected ChoiceReward, got %d", newModel.ChoiceType)
	}
	if cmd != nil {
		t.Error("expected nil command")
	}

	// Second reward selection should switch to fight selection
	newModel, cmd = newModel.Update(model.ChoiceSelected{Index: 1})

	if newModel.RewardChoicesLeft != 0 {
		t.Errorf("expected 0 reward choices left, got %d", newModel.RewardChoicesLeft)
	}
	if newModel.ChoiceType != model.ChoiceFight {
		t.Errorf("expected ChoiceFight, got %d", newModel.ChoiceType)
	}
	if len(newModel.Choices) != 3 {
		t.Errorf("expected 3 fight choices, got %d", len(newModel.Choices))
	}
	if cmd != nil {
		t.Error("expected nil command")
	}
}

func TestUpdate_CombatStarted(t *testing.T) {
	m := Model{
		Version:     1,
		Phase:       model.PhaseInterCombat,
		FightNumber: 1,
	}

	combat := model.CombatModel{
		Phase: model.CombatActive,
		Log:   []string{"Fight 2 started"},
	}

	newModel, cmd := m.Update(model.CombatStarted{Combat: combat})

	if newModel.Phase != model.PhaseCombat {
		t.Errorf("expected PhaseCombat, got %d", newModel.Phase)
	}
	if newModel.FightNumber != 2 {
		t.Errorf("expected FightNumber 2, got %d", newModel.FightNumber)
	}
	if newModel.Combat.Phase != model.CombatActive {
		t.Errorf("expected combat phase CombatActive")
	}
	// Wave 3: CombatStarted now returns StartNextRound cmd
	if cmd == nil {
		t.Error("expected StartNextRound command (Wave 3)")
	}
}

func TestChoiceSelected_ValidatesIndex(t *testing.T) {
	m := Model{
		Version:    1,
		Phase:      model.PhaseInterCombat,
		ChoiceType: model.ChoiceReward,
		Choices:    []string{"A", "B", "C"},
	}

	// Out of bounds - should be no-op
	newM, _ := m.Update(model.ChoiceSelected{Index: 99})
	if newM.Choices[0] != "A" {
		t.Error("out-of-bounds index should not change state")
	}

	// Negative - should be no-op
	newM2, _ := m.Update(model.ChoiceSelected{Index: -1})
	if newM2.Choices[0] != "A" {
		t.Error("negative index should not change state")
	}
}

func TestChoiceSelected_RequiresChoicePhase(t *testing.T) {
	m := Model{
		Version:    1,
		Phase:      model.PhaseCombat, // Wrong phase
		ChoiceType: model.ChoiceReward,
		Choices:    []string{"A", "B", "C"},
	}

	newM, _ := m.Update(model.ChoiceSelected{Index: 0})

	// Should be no-op - not in choice phase
	if newM.Phase != model.PhaseCombat {
		t.Error("should not change phase when not in PhaseInterCombat")
	}
}

func TestAppendLogEntry_Bounded(t *testing.T) {
	// Create log at max capacity
	log := make([]string, model.MaxLogEntries)
	for i := range log {
		log[i] = fmt.Sprintf("entry %d", i)
	}

	newLog := appendLogEntry(log, "new entry")

	if len(newLog) != model.MaxLogEntries {
		t.Errorf("expected %d entries, got %d", model.MaxLogEntries, len(newLog))
	}
	// First entry should be pruned, last entry should be new
	if newLog[0] == "entry 0" {
		t.Error("oldest entry should have been pruned")
	}
	if newLog[len(newLog)-1] != "new entry" {
		t.Error("new entry should be at end")
	}
}

func TestAppendLogEntries_Bounded(t *testing.T) {
	log := make([]string, model.MaxLogEntries-2)
	for i := range log {
		log[i] = fmt.Sprintf("entry %d", i)
	}

	// Add 5 entries when only 2 slots available
	newLog := appendLogEntries(log, []string{"a", "b", "c", "d", "e"})

	if len(newLog) != model.MaxLogEntries {
		t.Errorf("expected %d entries, got %d", model.MaxLogEntries, len(newLog))
	}
	// Should keep most recent entries
	if newLog[len(newLog)-1] != "e" {
		t.Error("most recent entry should be 'e'")
	}
}

func TestAppendLogEntry_Immutable(t *testing.T) {
	original := []string{"a", "b", "c"}
	originalLen := len(original)

	newLog := appendLogEntry(original, "d")

	if len(original) != originalLen {
		t.Error("original slice should not be modified")
	}
	if len(newLog) != 4 {
		t.Errorf("new log should have 4 entries, got %d", len(newLog))
	}
}

func TestAppendLogEntries_Empty(t *testing.T) {
	original := []string{"a", "b"}
	newLog := appendLogEntries(original, []string{})

	// Should return same slice (optimization)
	if &original[0] != &newLog[0] {
		t.Error("empty entries should return original slice")
	}
}

