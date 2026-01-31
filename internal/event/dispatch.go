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
		if trigger.Event == ctx.Event && core.EvaluateConditions(trigger.Conditions, ctx.SourceUnit.Tags, ctx.SourceUnit.Attributes) {
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
			if trigger.Event == ctx.Event && core.EvaluateConditions(trigger.Conditions, ctx.SourceUnit.Tags, ctx.SourceUnit.Attributes) {
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
				// Skip items on cooldown (only for on_combat_tick)
				if ctx.Event == core.EventOnCombatTick && ctx.ItemCooldowns != nil {
					path := OwnerPath(TriggerOwner{
						UnitID:  ctx.SourceUnit.ID,
						PartID:  partID,
						MountID: mount.ID,
						ItemID:  item.ID,
					})
					if remaining, ok := ctx.ItemCooldowns[path]; ok && remaining > 0 {
						continue
					}
				}

				for _, trigger := range item.Triggers {
					if trigger.Event == ctx.Event && core.EvaluateConditions(trigger.Conditions, ctx.SourceUnit.Tags, ctx.SourceUnit.Attributes) {
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
		return OwnerPath(collected[i].Owner) < OwnerPath(collected[j].Owner)
	})

	return collected
}

// OwnerPath creates a deterministic path string from owner components
func OwnerPath(o TriggerOwner) string {
	return o.UnitID + "/" + o.PartID + "/" + o.MountID + "/" + o.ItemID
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
