package tea

import "wulfaz/internal/model"

type GamePhase int

const (
	PhaseMenu GamePhase = iota
	PhaseCombat
	PhaseShop
	PhaseEvent
	PhaseGameOver
)

type Model struct {
	Version int
	Phase   GamePhase
	Combat  model.CombatModel
	Seed    int64
}

func (m Model) Update(msg Msg) (Model, Cmd) {
	switch msg.(type) {
	case PlayerQuit:
		m.Phase = PhaseGameOver
		return m, nil
	default:
		return m, nil
	}
}

func (m Model) View() string {
	return "Wulfaz MVP - scaffold"
}
