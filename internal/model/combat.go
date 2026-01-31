package model

import "wulfaz/internal/entity"

type CombatPhase int

const (
	CombatSetup CombatPhase = iota
	CombatActive
	CombatPaused
	CombatResolved
)

type CombatModel struct {
	PlayerUnits []entity.Unit
	EnemyUnits  []entity.Unit
	Tick        int
	Phase       CombatPhase
	Log         []string
	Victor      string // "player", "enemy", "draw", or "" (ongoing)
}
