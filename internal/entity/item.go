package entity

import "wulfaz/internal/core"

type Item struct {
	ID                string
	TemplateID        string
	Tags              []core.Tag
	Attributes        map[string]core.Attribute
	Triggers          []core.Trigger
	Abilities         []core.Ability
	ProvidedModifiers []core.ProvidedModifier
	Requirements      []core.Requirement
}
