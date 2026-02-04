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
	DicePhasePreview                 // All dice rolled, player sees AI targets as preview
	DicePhasePlayerCommand           // Player manipulates all player unit dice
	DicePhaseExecution               // All attacks resolve simultaneously
	DicePhaseRoundEnd                // Shields expire, round cleanup
)

// DefaultRerollsPerRound is the number of rerolls the player gets per round.
const DefaultRerollsPerRound = 2

// MaxLogEntries bounds combat log size. Display shows last 20 lines.
const MaxLogEntries = 500

// FloatingText represents scrolling combat text above a unit.
// Color is stored as 0xRRGGBBAA to avoid image/color dependency.
type FloatingText struct {
	UnitID    string
	Text      string // "+5", "-3", etc.
	ColorRGBA uint32 // 0xRRGGBBAA format
	StartedAt int64  // Unix nanoseconds (from Msg, not time.Now)
	YOffset   int    // Stack position (0, 1, 2, max 3)
}

// UndoSnapshot stores state at an undo point during DicePhasePlayerCommand.
type UndoSnapshot struct {
	RolledDice       map[string]entity.RolledDie
	RerollsRemaining int
	ActivatedDice    map[string]bool
	PlayerTargets    map[string]string
	SelectedUnitID   string
	PlayerUnits      []entity.Unit
	Log              []string
	FloatingTexts    []FloatingText
}

type CombatModel struct {
	// Existing fields
	PlayerUnits []entity.Unit
	EnemyUnits  []entity.Unit
	Phase       CombatPhase
	Log         []string
	Victor      string // "player", "enemy", "draw", or ""

	// Dice phase fields
	Round            int                         // Current round number (1-indexed)
	DicePhase        DicePhase                   // Current dice phase
	RolledDice       map[string]entity.RolledDie // UnitID -> single rolled die
	RerollsRemaining int                         // Player's rerolls left this round
	SelectedUnitID   string                      // Unit whose die is selected (empty if none)
	ActivatedDice    map[string]bool             // UnitID -> whether die has been activated
	PlayerTargets    map[string]string           // SourceUnitID -> TargetUnitID (player's assignments)
	EnemyTargets     map[string]string           // SourceUnitID -> TargetUnitID (AI's assignments)

	// End turn confirmation state
	EndTurnConfirmPending bool // True when waiting for y/n response
	UsableDiceRemaining   int  // Cached count for display during confirmation

	// Visualization state (Wave 7)
	ActiveArrows  []TargetingArrow // Arrows to render
	FloatingTexts []FloatingText   // Combat text to render

	// Undo system
	UndoStack      []UndoSnapshot // Snapshots for undo, cleared on phase exit
	InitialRerolls int            // RerollsRemaining at phase entry (for display)
}

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
