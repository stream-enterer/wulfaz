package tea

import (
	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

type Msg interface {
	isMsg() // sealed
}

// Combat messages

// CombatEnded signals combat has concluded with a victor
type CombatEnded struct{ Victor Victor }

func (CombatEnded) isMsg() {}

// CombatStarted signals a new combat should begin (carries the combat state)
type CombatStarted struct{ Combat model.CombatModel }

func (CombatStarted) isMsg() {}

// ChoiceSelected signals the player selected a choice option
type ChoiceSelected struct{ Index int }

func (ChoiceSelected) isMsg() {}

type AbilityActivated struct {
	SourceID  string
	AbilityID string
	TargetID  string
	Rolls     []int
}

func (AbilityActivated) isMsg() {}

// Player control messages
type PlayerPaused struct{}

func (PlayerPaused) isMsg() {}

type PlayerResumed struct{}

func (PlayerResumed) isMsg() {}

type PlayerQuit struct{}

func (PlayerQuit) isMsg() {}

// BatchedMsgs wraps multiple messages for batch command execution
type BatchedMsgs struct{ Msgs []Msg }

func (BatchedMsgs) isMsg() {}

// TriggersCollected is sent after dispatching an event and collecting matching triggers
type TriggersCollected struct {
	Event    string             // event type as string for serialization
	Triggers []CollectedTrigger // collected triggers with owner info
	Rolls    []int              // available rolls for effects
	Depth    int                // cascade depth for loop prevention
}

func (TriggersCollected) isMsg() {}

// CollectedTrigger pairs a trigger with its owner (mirrors event.CollectedTrigger for serialization)
type CollectedTrigger struct {
	EffectName       string
	Params           map[string]any
	Priority         int
	Owner            TriggerOwner
	Conditions       []core.Condition // source conditions, re-evaluated at execution time
	TargetConditions []core.Condition
}

// TriggerOwner identifies which entity owns a trigger (mirrors event.TriggerOwner)
type TriggerOwner struct {
	UnitID  string
	PartID  string
	MountID string
	ItemID  string
}

// EffectsResolved is sent after executing all effects from collected triggers
type EffectsResolved struct {
	ModifiedUnits  ModifiedUnitsMap // serializable unit modifications
	FollowUpEvents []FollowUpEvent  // cascading events
	LogEntries     []string         // combat log entries
	Depth          int              // cascade depth
}

// ModifiedUnitsMap holds unit modifications in a serializable format
// Key is unit ID, value contains the modified unit data
type ModifiedUnitsMap map[string]ModifiedUnit

// ModifiedUnit holds modified unit data in serializable format
type ModifiedUnit struct {
	Attributes map[string]AttributeValue
}

// AttributeValue is a serializable attribute value
type AttributeValue struct {
	Base int
	Min  int
	Max  int
}

func (EffectsResolved) isMsg() {}

// FollowUpEvent represents a cascading event (mirrors effect.FollowUpEvent)
type FollowUpEvent struct {
	Event    string
	SourceID string
	TargetID string
}

// ===== Dice Phase Messages (Wave 2) =====

// RoundStarted signals a new round began with pre-rolled dice.
// UnitRolls maps UnitID -> face indices (not values) for deterministic replay.
type RoundStarted struct {
	Round     int
	UnitRolls map[string][]int // UnitID -> []faceIndex
}

func (RoundStarted) isMsg() {}

// PreviewDone signals player acknowledged preview, ready for command phase.
type PreviewDone struct{}

func (PreviewDone) isMsg() {}

// DieLockToggled signals a die's lock state was toggled.
type DieLockToggled struct {
	UnitID   string
	DieIndex int
}

func (DieLockToggled) isMsg() {}

// RerollRequested signals player wants to reroll unlocked dice.
// Carries pre-rolled results (RNG in Cmd, not Update).
type RerollRequested struct {
	UnitID  string
	Results []int // New face indices for ALL dice (locked dice get same index)
}

func (RerollRequested) isMsg() {}

// DieSelected signals player selected a die for activation.
type DieSelected struct {
	UnitID   string
	DieIndex int
}

func (DieSelected) isMsg() {}

// DieDeselected signals player deselected the current die.
type DieDeselected struct{}

func (DieDeselected) isMsg() {}

// DiceActivated signals a die effect was activated on a target.
type DiceActivated struct {
	SourceUnitID string
	DieIndex     int
	TargetUnitID string
	Value        int            // The die's result value
	Effect       entity.DieType // damage/shield/heal
}

func (DiceActivated) isMsg() {}

// DiceEffectApplied signals effect resolution completed.
type DiceEffectApplied struct {
	SourceUnitID string
	TargetUnitID string
	Effect       entity.DieType
	Value        int
	NewHealth    int
	NewShields   int
}

func (DiceEffectApplied) isMsg() {}

// PlayerCommandDone signals player finished their command phase.
type PlayerCommandDone struct{}

func (PlayerCommandDone) isMsg() {}

// DicePhaseAdvanced signals transition to next dice phase.
type DicePhaseAdvanced struct {
	NewPhase model.DicePhase
}

func (DicePhaseAdvanced) isMsg() {}

// ===== Wave 3: Combat Phase Messages =====

// EnemyCommandResolved signals enemy AI finished activating command dice.
type EnemyCommandResolved struct {
	Actions []EnemyDiceAction
}

func (EnemyCommandResolved) isMsg() {}

// EnemyDiceAction records one enemy command die activation.
type EnemyDiceAction struct {
	SourceUnitID string
	TargetUnitID string
	DieIndex     int
	Effect       entity.DieType
	Value        int
}

// ExecutionStarted signals execution phase began with firing order.
type ExecutionStarted struct {
	FiringOrder []model.FiringPosition
}

func (ExecutionStarted) isMsg() {}

// PositionResolved signals one position's attacks calculated.
type PositionResolved struct {
	Position int
	Attacks  []AttackResult
}

func (PositionResolved) isMsg() {}

// AttackResult records one attack's outcome.
type AttackResult struct {
	AttackerID string
	TargetID   string
	DieIndex   int
	Damage     int
	NewHealth  int
	NewShields int
	TargetDead bool
}

// ExecutionComplete signals all positions resolved.
type ExecutionComplete struct{}

func (ExecutionComplete) isMsg() {}

// RoundEnded signals round cleanup complete.
type RoundEnded struct{}

func (RoundEnded) isMsg() {}

// RoundToastDismissed signals player clicked to dismiss round toast.
type RoundToastDismissed struct{}

func (RoundToastDismissed) isMsg() {}

// UnlockAllDice signals player wants to unlock all dice to re-enter lock phase.
type UnlockAllDice struct{}

func (UnlockAllDice) isMsg() {}
