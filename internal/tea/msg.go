package tea

import (
	"wulfaz/internal/core"
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

type CombatTicked struct{ Rolls []int }

func (CombatTicked) isMsg() {}

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
