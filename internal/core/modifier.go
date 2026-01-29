package core

type ModifierOp int

const (
	ModifierOpAdd ModifierOp = iota
	ModifierOpMult
	ModifierOpSet
	ModifierOpMin // floor
	ModifierOpMax // ceiling
)

type Scope string

const (
	ScopeSelf     Scope = "self"
	ScopeUnit     Scope = "unit"
	ScopePart     Scope = "part"
	ScopeAdjacent Scope = "adjacent"
	ScopeMount    Scope = "mount"
)

type Modifier struct {
	SourceID   string
	Operation  ModifierOp
	Value      int
	StackGroup string // "" = always stacks
}

type ProvidedModifier struct {
	Scope       Scope
	ScopeFilter []Tag
	Attribute   string
	Operation   ModifierOp
	Value       int
	StackGroup  string
	Conditions  []Condition
}
