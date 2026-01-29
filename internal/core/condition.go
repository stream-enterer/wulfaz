package core

type ConditionType string

const (
	ConditionHasTag  ConditionType = "has_tag"
	ConditionAttrGTE ConditionType = "attr_gte"
	ConditionAttrLTE ConditionType = "attr_lte"
	ConditionAttrEQ  ConditionType = "attr_eq"
)

// Condition is leaf-only for MVP
// POST-MVP: adds AND/OR/NOT wrappers
type Condition struct {
	Type   ConditionType
	Params map[string]any
}
