package core

type EventType string

const (
	EventOnDamaged   EventType = "on_damaged"
	EventOnDestroyed EventType = "on_destroyed"
	EventOnTurnStart EventType = "on_turn_start"
	EventOnTurnEnd   EventType = "on_turn_end"
	EventOnActivate  EventType = "on_activate"
	EventOnCascade   EventType = "cascade" // follow-up events from effects
)

type Trigger struct {
	Event            EventType
	Conditions       []Condition // evaluated against SOURCE unit
	TargetConditions []Condition // evaluated against TARGET units
	EffectName       string      // effect name to invoke
	Params           map[string]any
	Priority         int // lower = earlier
}
