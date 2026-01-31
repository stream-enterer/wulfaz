package core

// EvaluateConditions returns true if all conditions pass against the given tags/attributes.
// Returns true if conditions slice is empty or nil.
func EvaluateConditions(conditions []Condition, tags []Tag, attributes map[string]Attribute) bool {
	for _, cond := range conditions {
		if !EvaluateCondition(cond, tags, attributes) {
			return false
		}
	}
	return true
}

// EvaluateCondition evaluates a single condition.
func EvaluateCondition(cond Condition, tags []Tag, attributes map[string]Attribute) bool {
	switch cond.Type {
	case ConditionHasTag:
		tag, ok := cond.Params["tag"].(string)
		if !ok {
			return false
		}
		return HasTag(tags, Tag(tag))
	case ConditionAttrGTE:
		return evalAttrComparison(cond.Params, attributes, func(v, t int) bool { return v >= t })
	case ConditionAttrLTE:
		return evalAttrComparison(cond.Params, attributes, func(v, t int) bool { return v <= t })
	case ConditionAttrEQ:
		return evalAttrComparison(cond.Params, attributes, func(v, t int) bool { return v == t })
	default:
		return false
	}
}

// HasTag checks if a tag slice contains a specific tag. Exported for reuse.
func HasTag(tags []Tag, target Tag) bool {
	for _, t := range tags {
		if t == target {
			return true
		}
	}
	return false
}

func evalAttrComparison(params map[string]any, attributes map[string]Attribute, compare func(val, threshold int) bool) bool {
	// Use "attribute" key to match KDL convention
	attrName, ok := params["attribute"].(string)
	if !ok {
		return false
	}

	value, ok := getParamInt(params, "value")
	if !ok {
		return false
	}

	attr, exists := attributes[attrName]
	if !exists {
		return false
	}

	return compare(attr.Base, value)
}

func getParamInt(params map[string]any, key string) (int, bool) {
	v, ok := params[key]
	if !ok {
		return 0, false
	}
	switch val := v.(type) {
	case int:
		return val, true
	case float64:
		return int(val), true
	default:
		return 0, false
	}
}
