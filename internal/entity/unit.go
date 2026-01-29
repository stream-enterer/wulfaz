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
