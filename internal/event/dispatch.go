package event

import (
	"sort"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

// Dispatch collects all triggers that match the event type and pass condition evaluation.
// Returns triggers sorted by Priority (lower first), then by owner path for determinism.
func Dispatch(ctx TriggerContext) []CollectedTrigger {
	var collected []CollectedTrigger

	// Traverse unit-level triggers
	for _, trigger := range ctx.SourceUnit.Triggers {
		if trigger.Event == ctx.Event && evaluateConditions(trigger.Conditions, ctx.SourceUnit) {
			collected = append(collected, CollectedTrigger{
				Trigger: trigger,
				Owner: TriggerOwner{
					UnitID: ctx.SourceUnit.ID,
				},
			})
		}
	}

	// Get sorted part IDs for deterministic traversal
	partIDs := sortedPartIDs(ctx.SourceUnit.Parts)

	// Traverse parts in deterministic order
	for _, partID := range partIDs {
		part := ctx.SourceUnit.Parts[partID]
		// Part-level triggers
		for _, trigger := range part.Triggers {
			if trigger.Event == ctx.Event && evaluateConditions(trigger.Conditions, ctx.SourceUnit) {
				collected = append(collected, CollectedTrigger{
					Trigger: trigger,
					Owner: TriggerOwner{
						UnitID: ctx.SourceUnit.ID,
						PartID: partID,
					},
				})
			}
		}

		// Traverse mounts
		for _, mount := range part.Mounts {
			// Traverse items in mount
			for _, item := range mount.Contents {
				for _, trigger := range item.Triggers {
					if trigger.Event == ctx.Event && evaluateConditions(trigger.Conditions, ctx.SourceUnit) {
						collected = append(collected, CollectedTrigger{
							Trigger: trigger,
							Owner: TriggerOwner{
								UnitID:  ctx.SourceUnit.ID,
								PartID:  partID,
								MountID: mount.ID,
								ItemID:  item.ID,
							},
						})
					}
				}
			}
		}
	}

	// Sort by priority (lower first), then by owner path for determinism
	sort.SliceStable(collected, func(i, j int) bool {
		if collected[i].Trigger.Priority != collected[j].Trigger.Priority {
			return collected[i].Trigger.Priority < collected[j].Trigger.Priority
		}
		// Deterministic ordering by owner path
		return ownerPath(collected[i].Owner) < ownerPath(collected[j].Owner)
	})

	return collected
}

// ownerPath creates a deterministic string for sorting
func ownerPath(o TriggerOwner) string {
	return o.UnitID + "/" + o.PartID + "/" + o.MountID + "/" + o.ItemID
}

// evaluateConditions returns true if all conditions pass
func evaluateConditions(conditions []core.Condition, unit entity.Unit) bool {
	for _, cond := range conditions {
		if !evaluateCondition(cond, unit) {
			return false
		}
	}
	return true
}

// evaluateCondition evaluates a single condition against a unit
func evaluateCondition(cond core.Condition, unit entity.Unit) bool {
	switch cond.Type {
	case core.ConditionHasTag:
		tag, ok := cond.Params["tag"].(string)
		if !ok {
			return false
		}
		return hasTag(unit.Tags, core.Tag(tag))

	case core.ConditionAttrGTE:
		return evalAttrComparison(cond.Params, unit, func(val, threshold int) bool {
			return val >= threshold
		})

	case core.ConditionAttrLTE:
		return evalAttrComparison(cond.Params, unit, func(val, threshold int) bool {
			return val <= threshold
		})

	case core.ConditionAttrEQ:
		return evalAttrComparison(cond.Params, unit, func(val, threshold int) bool {
			return val == threshold
		})

	default:
		return false
	}
}

// evalAttrComparison evaluates an attribute comparison condition
func evalAttrComparison(params map[string]any, unit entity.Unit, compare func(val, threshold int) bool) bool {
	attrName, ok := params["attr"].(string)
	if !ok {
		return false
	}

	value, ok := getParamInt(params, "value")
	if !ok {
		return false
	}

	attr, exists := unit.Attributes[attrName]
	if !exists {
		return false
	}

	return compare(attr.Base, value)
}

// getParamInt extracts an int from params, handling both int and float64 (JSON)
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

// hasTag checks if a tag slice contains a specific tag
func hasTag(tags []core.Tag, target core.Tag) bool {
	for _, t := range tags {
		if t == target {
			return true
		}
	}
	return false
}

// sortedPartIDs returns part IDs in sorted order for deterministic iteration
func sortedPartIDs(parts map[string]entity.Part) []string {
	ids := make([]string, 0, len(parts))
	for id := range parts {
		ids = append(ids, id)
	}
	sort.Strings(ids)
	return ids
}
