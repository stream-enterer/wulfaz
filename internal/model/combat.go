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
	DicePhaseNone          DicePhase = iota
	DicePhasePreview                 // All dice rolled, player sees enemy plan
	DicePhasePlayerCommand           // Player manipulates their command dice
	DicePhaseEnemyCommand            // Enemy activates their command dice
	DicePhaseExecution               // Units fire in position order
	DicePhaseRoundEnd                // Shields expire, round cleanup
)

// DefaultRerollsPerRound is the number of rerolls the player gets per round.
const DefaultRerollsPerRound = 2

// MaxLogEntries bounds combat log size. Display shows last 20 lines.
const MaxLogEntries = 500

type CombatModel struct {
	// Existing fields
	PlayerUnits []entity.Unit
	EnemyUnits  []entity.Unit
	Phase       CombatPhase
	Log         []string
	Victor      string // "player", "enemy", "draw", or ""

	// Dice phase fields (Wave 2)
	Round            int                           // Current round number (1-indexed)
	DicePhase        DicePhase                     // Current dice phase
	RolledDice       map[string][]entity.RolledDie // UnitID -> rolled dice with results
	RerollsRemaining int                           // Player's rerolls left this round
	SelectedUnitID   string                        // Unit whose die is selected (empty if none)
	SelectedDieIndex int                           // Index of selected die (-1 if none)
	ActivatedDice    map[string][]bool             // UnitID -> which dice have been activated

	// Execution phase fields (Wave 3)
	FiringOrder        []FiringPosition // Positions to resolve in order
	CurrentFiringIndex int              // Index into FiringOrder

	// Visualization state (Wave 7)
	ActiveArrows  []TargetingArrow          // Arrows to render
	FlashTargets  map[string]entity.DieType // UnitID -> effect type (for flash color)
	ExecutionAnim ExecutionAnimState        // Current animation state
}

// FiringPosition groups units at same board position for simultaneous resolution
type FiringPosition struct {
	Position    int      // Board position (0-9)
	PlayerUnits []string // Unit IDs of player units at this position
	EnemyUnits  []string // Unit IDs of enemy units at this position
}

// ExecutionAnimState tracks execution animation progress
type ExecutionAnimState int

const (
	ExecAnimNone      ExecutionAnimState = iota
	ExecAnimShowArrows                   // Arrows visible, waiting to resolve
	ExecAnimResolving                    // Damage applied, targets flashing
)

// TargetingArrow represents a line from attacker to target
type TargetingArrow struct {
	SourceUnitID string
	TargetUnitID string
	EffectType   entity.DieType // damage/shield/heal for coloring
	IsDashed     bool           // true for enemy preview arrows
}

// IsPlayerUnit returns true if unitID belongs to player side.
func (c CombatModel) IsPlayerUnit(unitID string) bool {
	for _, u := range c.PlayerUnits {
		if u.ID == unitID {
			return true
		}
	}
	return false
}

// IsEnemyUnit returns true if unitID belongs to enemy side.
func (c CombatModel) IsEnemyUnit(unitID string) bool {
	for _, u := range c.EnemyUnits {
		if u.ID == unitID {
			return true
		}
	}
	return false
}
