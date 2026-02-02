package core

type TargetType int

const (
	TargetNone TargetType = iota
	TargetSelf
	TargetAlly
	TargetEnemy
	TargetAnyUnit
	TargetPart
	TargetItem
	TargetPosition
)

type Targeting struct {
	Type   TargetType
	Range  int
	Count  int // default 1
	Filter []Tag
}

type Cost struct {
	Attribute string // "heat", "energy", "ammo"
	Scope     Scope
	Amount    ValueRef
}

type EffectRef struct {
	EffectName string
	Params     map[string]any
	Delay      int
	Conditions []Condition
}

type Ability struct {
	ID                 string
	Tags               []Tag
	Conditions         []Condition
	Costs              []Cost
	Targeting          Targeting
	Effects            []EffectRef
	Cooldown           int
	Charges            int       // -1 = unlimited
	ChargeRestoreEvent EventType // event that restores charges
}
