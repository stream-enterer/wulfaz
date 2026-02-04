package tea

import (
	"time"

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
// UnitRolls maps UnitID -> face index (single die per unit).
type RoundStarted struct {
	Round     int
	UnitRolls map[string]int // UnitID -> faceIndex
}

func (RoundStarted) isMsg() {}

// PreviewDone signals player acknowledged preview, ready for command phase.
type PreviewDone struct{}

func (PreviewDone) isMsg() {}

// DieLockToggled signals the unit's die lock state was toggled.
type DieLockToggled struct {
	UnitID string
}

func (DieLockToggled) isMsg() {}

// RerollRequested signals player wants to reroll all unlocked player dice.
// Carries pre-rolled results (RNG in Cmd, not Update).
type RerollRequested struct {
	Results map[string]int // UnitID -> new face index (only unlocked units)
}

func (RerollRequested) isMsg() {}

// DieSelected signals player selected a unit's die for activation.
type DieSelected struct {
	UnitID string
}

func (DieSelected) isMsg() {}

// DieDeselected signals player deselected the current die.
type DieDeselected struct{}

func (DieDeselected) isMsg() {}

// DiceActivated signals a die effect was activated on a target.
type DiceActivated struct {
	SourceUnitID string
	TargetUnitID string
	Value        int            // The die's result value
	Effect       entity.DieType // damage/shield/heal
	Timestamp    int64          // Unix nanoseconds from App layer
}

func (DiceActivated) isMsg() {}

// DiceEffectApplied signals effect resolution completed.
type DiceEffectApplied struct {
	SourceUnitID   string
	TargetUnitID   string
	Effect         entity.DieType
	Value          int
	NewHealth      int
	NewShields     int
	ShieldAbsorbed int   // How much damage was absorbed by shields
	Timestamp      int64 // Unix nanoseconds for floating text
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

// ===== AI Targeting Messages =====

// AITargetsComputed signals AI has computed targets for all enemy units.
type AITargetsComputed struct {
	Targets map[string]string // EnemyUnitID -> PlayerTargetID
}

func (AITargetsComputed) isMsg() {}

// AllAttacksResolved signals all attacks have been resolved simultaneously.
type AllAttacksResolved struct {
	Attacks   []AttackResult
	Timestamp int64
}

func (AllAttacksResolved) isMsg() {}

// AttackResult records one attack's outcome.
type AttackResult struct {
	AttackerID     string
	TargetID       string
	Damage         int
	ShieldAbsorbed int // How much was absorbed by shields
	NewHealth      int
	NewShields     int
	TargetDead     bool
}

// ExecutionComplete signals all positions resolved.
type ExecutionComplete struct{}

func (ExecutionComplete) isMsg() {}

// RoundEnded signals round cleanup complete.
type RoundEnded struct{}

func (RoundEnded) isMsg() {}

// UnlockAllDice signals player wants to unlock all dice to re-enter lock phase.
type UnlockAllDice struct{}

func (UnlockAllDice) isMsg() {}

// AllDiceLocked signals player pressed ENTER to lock all remaining dice.
type AllDiceLocked struct{}

func (AllDiceLocked) isMsg() {}

// EndTurnRequested signals player pressed ENTER to request ending their turn.
type EndTurnRequested struct {
	UsableDiceCount int // Pre-computed count of usable dice remaining
}

func (EndTurnRequested) isMsg() {}

// EndTurnConfirmed signals player confirmed ending turn (pressed 'y').
type EndTurnConfirmed struct{}

func (EndTurnConfirmed) isMsg() {}

// EndTurnCanceled signals player canceled ending turn (pressed 'n').
type EndTurnCanceled struct{}

func (EndTurnCanceled) isMsg() {}

// ===== Wave 7: Timer Messages =====

// StartTimerRequested asks the runtime to start a timer.
// App intercepts this in dispatch() - it never reaches Update().
type StartTimerRequested struct {
	ID       string
	Duration time.Duration
}

func (StartTimerRequested) isMsg() {}

// TimerFired signals a timer completed.
type TimerFired struct {
	ID string
}

func (TimerFired) isMsg() {}

// ExecutionAdvanceClicked signals player clicked to advance execution.
// Timestamp is set by runtime (App layer) to maintain Update purity.
type ExecutionAdvanceClicked struct {
	Timestamp int64 // Unix nanoseconds
}

func (ExecutionAdvanceClicked) isMsg() {}

// ===== Drag-and-Drop Messages =====

// UnitDragStarted signals player began dragging a unit.
type UnitDragStarted struct {
	UnitID        string
	OriginalIndex int // Roster index (excluding command unit)
	StartX        int
	StartY        int
}

func (UnitDragStarted) isMsg() {}

// UnitDragMoved signals drag position changed.
type UnitDragMoved struct {
	CurrentX int
	CurrentY int
}

func (UnitDragMoved) isMsg() {}

// UnitDragEnded signals player released drag.
type UnitDragEnded struct {
	InsertionIndex int // -1 if canceled/invalid
}

func (UnitDragEnded) isMsg() {}

// UnitDragCanceled signals drag canceled (ESC/right-click).
type UnitDragCanceled struct{}

func (UnitDragCanceled) isMsg() {}
