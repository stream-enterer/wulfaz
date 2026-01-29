package core

type EventType string

const (
	EventOnDamaged    EventType = "on_damaged"
	EventOnDestroyed  EventType = "on_destroyed"
	EventOnCombatTick EventType = "on_combat_tick"
	EventOnTurnStart  EventType = "on_turn_start"
	EventOnTurnEnd    EventType = "on_turn_end"
	EventOnActivate   EventType = "on_activate"
)

type Trigger struct {
	Event      EventType
	Conditions []Condition
	EffectName string // effect name to invoke
	Params     map[string]any
	Priority   int // lower = earlier
}
