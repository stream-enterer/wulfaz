package event

import (
	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

// TriggerOwner identifies which entity owns a trigger
type TriggerOwner struct {
	UnitID  string
	PartID  string // empty if unit-level
	MountID string // empty if part-level
	ItemID  string // empty if not item-level
}

// TriggerContext provides state for trigger evaluation
type TriggerContext struct {
	Event      core.EventType
	SourceUnit entity.Unit
	AllUnits   []entity.Unit
	Tick       int
	Rolls      []int
}

// CollectedTrigger pairs trigger with its owner
type CollectedTrigger struct {
	Trigger core.Trigger
	Owner   TriggerOwner
}
