package model

import (
	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

type Msg interface {
	isMsg() // sealed
}

type Cmd func() Msg

func None() Cmd { return nil }

func Batch(cmds ...Cmd) Cmd {
	if len(cmds) == 0 {
		return nil
	}
	return func() Msg {
		var msgs []Msg
		for _, cmd := range cmds {
			if cmd != nil {
				if msg := cmd(); msg != nil {
					msgs = append(msgs, msg)
				}
			}
		}
		switch len(msgs) {
		case 0:
			return nil
		case 1:
			return msgs[0]
		default:
			return BatchedMsgs{Msgs: msgs}
		}
	}
}

// Constants for floating text
const (
	CombatTextDuration = 1500000000 // 1.5 seconds in nanoseconds
	MaxTextStack       = 3          // Cap stacking to prevent overflow
)

// Color constants as uint32 (0xRRGGBBAA)
const (
	ColorTextDamage = 0xFF5050FF // Red
	ColorTextHeal   = 0x50FF50FF // Green
	ColorTextShield = 0xAAAAAAFF // Grey
)

type GamePhase int

const (
	PhaseMenu GamePhase = iota
	PhaseCombat
	PhaseInterCombat // Board visible, rewards/fight as overlays, repositioning enabled
	PhaseGameOver
)

type ChoiceType int

const (
	ChoiceReward ChoiceType = iota
	ChoiceFight
)

type Victor int

const (
	VictorNone Victor = iota
	VictorPlayer
	VictorEnemy
	VictorDraw
)

func (v Victor) String() string {
	switch v {
	case VictorNone:
		return ""
	case VictorPlayer:
		return "player"
	case VictorEnemy:
		return "enemy"
	case VictorDraw:
		return "draw"
	}
	return ""
}

// DragState tracks unit drag-and-drop state during inter-combat phase.
type DragState struct {
	IsDragging    bool
	DraggedUnitID string
	OriginalIndex int // Roster index (board units only, excludes command)
	CurrentX      int // Mouse position
	CurrentY      int
}

// Combat messages

// CombatEnded signals combat has concluded with a victor
type CombatEnded struct{ Victor Victor }

func (CombatEnded) isMsg() {}

// CombatStarted signals a new combat should begin (carries the combat state)
type CombatStarted struct{ Combat CombatModel }

func (CombatStarted) isMsg() {}

// ChoiceSelected signals the player selected a choice option
type ChoiceSelected struct{ Index int }

func (ChoiceSelected) isMsg() {}

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
// UnitRolls maps UnitID -> face indices (one per die on the unit).
type RoundStarted struct {
	Round     int
	UnitRolls map[string][]int // UnitID -> []faceIndex
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
	Results map[string][]int // UnitID -> new face indices (one per die, only unlocked units)
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

// DiceActivated signals a die activation on a target (split-activation: compatible dice fire).
type DiceActivated struct {
	SourceUnitID string
	TargetUnitID string
	Timestamp    int64 // Unix nanoseconds from App layer
}

func (DiceActivated) isMsg() {}

// DiceEffectResult records one die's effect resolution.
type DiceEffectResult struct {
	TargetUnitID   string
	Effect         entity.DieType
	Value          int
	NewHealth      int
	NewShields     int
	ShieldAbsorbed int // How much damage was absorbed by shields
}

// UnitDiceEffectsApplied signals all compatible dice effects resolved for a unit activation.
type UnitDiceEffectsApplied struct {
	SourceUnitID string
	Results      []DiceEffectResult
	Timestamp    int64 // Unix nanoseconds for floating text
}

func (UnitDiceEffectsApplied) isMsg() {}

// PlayerCommandDone signals player finished their command phase.
type PlayerCommandDone struct{}

func (PlayerCommandDone) isMsg() {}

// DicePhaseAdvanced signals transition to next dice phase.
type DicePhaseAdvanced struct {
	NewPhase DicePhase
}

func (DicePhaseAdvanced) isMsg() {}

// ===== AI Targeting Messages =====

// AITargetsComputed signals AI has computed targets for all enemy units.
type AITargetsComputed struct {
	Targets        map[string]string // EnemyUnitID -> PlayerTargetID (damage)
	DefenseTargets map[string]string // EnemyUnitID -> AllyTargetID (shield/heal)
}

func (AITargetsComputed) isMsg() {}

// AllAttacksResolved signals all attacks have been resolved simultaneously.
type AllAttacksResolved struct {
	Attacks        []AttackResult
	DefenseResults []DiceEffectResult // Enemy shield/heal results against own allies
	Timestamp      int64
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

// UndoRequested signals player wants to undo the last action.
type UndoRequested struct{}

func (UndoRequested) isMsg() {}

// DieUnlocked signals a specific die was unlocked via right-click.
type DieUnlocked struct {
	UnitID string
}

func (DieUnlocked) isMsg() {}

// UnlockAllDiceRequested signals player wants to unlock all dice to return to lock phase.
type UnlockAllDiceRequested struct{}

func (UnlockAllDiceRequested) isMsg() {}

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

// ===== Wave 7: Click-through Messages =====

// ExecutionAdvanceClicked signals player clicked to advance execution.
// Timestamp is set by runtime (App layer) to maintain Update purity.
type ExecutionAdvanceClicked struct {
	Timestamp int64 // Unix nanoseconds
}

func (ExecutionAdvanceClicked) isMsg() {}

// RoundEndClicked signals player clicked to advance past round end.
type RoundEndClicked struct{}

func (RoundEndClicked) isMsg() {}

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
