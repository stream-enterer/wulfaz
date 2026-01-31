package entity

import "wulfaz/internal/core"

type Unit struct {
	ID         string
	TemplateID string
	Tags       []core.Tag
	Attributes map[string]core.Attribute
	Parts      map[string]Part // VALUE type
	Triggers   []core.Trigger
	Abilities  []core.Ability
	Pilot      Pilot
	HasPilot   bool
}

// IsAlive returns true if unit has health > 0, or has no health attribute.
// Units without health (e.g., terrain, decorations) are considered alive
// for dispatch purposes.
func (u Unit) IsAlive() bool {
	health, ok := u.Attributes["health"]
	return !ok || health.Base > 0
}
