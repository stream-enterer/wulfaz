package entity

import "wulfaz/internal/core"

type Part struct {
	ID          string
	TemplateID  string
	Tags        []core.Tag
	Attributes  map[string]core.Attribute
	Mounts      []Mount // VALUE type
	Connections map[string][]string
	Triggers    []core.Trigger
	Abilities   []core.Ability
}
