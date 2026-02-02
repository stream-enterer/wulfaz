package core

type OnUnmet int

const (
	OnUnmetDisabled OnUnmet = iota
	OnUnmetCannotMount
	OnUnmetWarning
)

type Requirement struct {
	Scope     Scope
	Condition Condition
	OnUnmet   OnUnmet
}
