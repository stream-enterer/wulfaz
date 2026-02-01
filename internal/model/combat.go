package model

import "wulfaz/internal/entity"

type CombatPhase int

const (
	CombatSetup CombatPhase = iota
	CombatActive
	CombatPaused
	CombatResolved
)

type DicePhase int

const (
	DicePhaseNone DicePhase = iota
	DicePhasePreview       // All dice rolled, player sees enemy plan
	DicePhasePlayerCommand // Player manipulates their command dice
	DicePhaseEnemyCommand  // Enemy activates their command dice
	DicePhaseExecution     // Units fire in position order
	DicePhaseRoundEnd      // Shields expire, round cleanup
)

type CombatModel struct {
	// Existing fields
	PlayerUnits   []entity.Unit
	EnemyUnits    []entity.Unit
	Tick          int            // Legacy tick system (kept for now)
	Phase         CombatPhase
	Log           []string
	Victor        string         // "player", "enemy", "draw", or ""
	ItemCooldowns map[string]int // Legacy cooldowns

	// Dice phase fields (Wave 2)
	Round            int                           // Current round number (1-indexed)
	DicePhase        DicePhase                     // Current dice phase
	RolledDice       map[string][]entity.RolledDie // UnitID -> rolled dice with results
	RerollsRemaining int                           // Player's rerolls left this round
	SelectedUnitID   string                        // Unit whose die is selected (empty if none)
	SelectedDieIndex int                           // Index of selected die (-1 if none)
	ActivatedDice    map[string][]bool             // UnitID -> which dice have been activated
}
