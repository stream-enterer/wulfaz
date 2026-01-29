package core

type OnUnmet int

const (
	OnUnmetDisabled OnUnmet = iota
	OnUnmetCannotMount
	OnUnmetWarning
)

type Requirement struct {
	Scope     string // "unit", "part", "mount"
	Condition Condition
	OnUnmet   OnUnmet
}
