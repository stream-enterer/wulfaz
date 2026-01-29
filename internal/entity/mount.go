package entity

import "wulfaz/internal/core"

type MountCriteria struct {
	RequiresAll []core.Tag
	RequiresAny []core.Tag
	Forbids     []core.Tag
}

type Mount struct {
	ID                string
	Tags              []core.Tag
	Accepts           MountCriteria
	Capacity          int
	CapacityAttribute string // default "size"
	MaxItems          int    // -1 = unlimited
	Locked            bool
	Contents          []Item // VALUE type
}
