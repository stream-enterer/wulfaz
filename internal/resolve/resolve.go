package resolve

import (
	"maps"

	"wulfaz/internal/core"
	"wulfaz/internal/effect"
	"wulfaz/internal/entity"
	"wulfaz/internal/event"
	"wulfaz/internal/model"
)

// ResolveEffects creates a Cmd that executes effects for collected triggers.
// It iterates triggers, re-evaluates conditions, calls effect.Handle, and
// converts the result into a model.EffectsResolved Msg.
func ResolveEffects(triggers []model.CollectedTrigger, unitMap map[string]entity.Unit, playerIDs map[string]bool, rolls []int, depth int) model.Cmd {
	return func() model.Msg {
		var result effect.EffectResult
		result.ModifiedUnits = make(map[string]entity.Unit)

		for _, trigger := range triggers {
			sourceUnit, ok := unitMap[trigger.Owner.UnitID]
			if !ok {
				continue
			}

			// Re-evaluate source conditions with current unit state
			if len(trigger.Conditions) > 0 {
				if !core.EvaluateConditions(trigger.Conditions, sourceUnit.Tags, sourceUnit.Attributes) {
					continue
				}
			}

			ctx := effect.EffectContext{
				Owner:            toEventTriggerOwner(trigger.Owner),
				SourceUnit:       sourceUnit,
				AllUnits:         unitMap,
				PlayerUnitIDs:    playerIDs,
				Rolls:            rolls,
				TargetConditions: trigger.TargetConditions,
			}

			effectResult := effect.Handle(trigger.EffectName, trigger.Params, ctx)
			result.Merge(effectResult)

			// Update unit map with modified units for subsequent effects
			maps.Copy(unitMap, effectResult.ModifiedUnits)
		}

		// Convert modified units to serializable format
		modifiedUnits := make(model.ModifiedUnitsMap)
		for id, unit := range result.ModifiedUnits {
			attrs := make(map[string]model.AttributeValue)
			for name, attr := range unit.Attributes {
				attrs[name] = model.AttributeValue{
					Base: attr.Base,
					Min:  attr.Min,
					Max:  attr.Max,
				}
			}
			modifiedUnits[id] = model.ModifiedUnit{Attributes: attrs}
		}

		var followUps []model.FollowUpEvent
		for _, fe := range result.FollowUpEvents {
			followUps = append(followUps, model.FollowUpEvent{
				Event:    string(fe.Event),
				SourceID: fe.SourceID,
				TargetID: fe.TargetID,
			})
		}

		return model.EffectsResolved{
			ModifiedUnits:  modifiedUnits,
			FollowUpEvents: followUps,
			LogEntries:     result.LogEntries,
			Depth:          depth,
		}
	}
}

// CollectFollowUpTriggers creates a Cmd that dispatches follow-up events
// and collects matching triggers. Returns model.TriggersCollected.
func CollectFollowUpTriggers(followUps []model.FollowUpEvent, allUnits []entity.Unit, depth int) model.Cmd {
	return func() model.Msg {
		var allTriggers []model.CollectedTrigger

		for _, fe := range followUps {
			eventType := core.EventType(fe.Event)

			// Find target unit as source for the triggered event
			var sourceUnit entity.Unit
			for _, u := range allUnits {
				if u.ID == fe.TargetID {
					sourceUnit = u
					break
				}
			}
			if sourceUnit.ID == "" {
				continue
			}

			ctx := event.TriggerContext{
				Event:      eventType,
				SourceUnit: sourceUnit,
				AllUnits:   allUnits,
			}
			collected := event.Dispatch(ctx)
			for _, ct := range collected {
				allTriggers = append(allTriggers, toModelCollectedTrigger(ct))
			}
		}

		return model.TriggersCollected{
			Event:    string(core.EventOnCascade),
			Triggers: allTriggers,
			Rolls:    nil,
			Depth:    depth + 1,
		}
	}
}

func toEventTriggerOwner(to model.TriggerOwner) event.TriggerOwner {
	return event.TriggerOwner{
		UnitID:  to.UnitID,
		PartID:  to.PartID,
		MountID: to.MountID,
		ItemID:  to.ItemID,
	}
}

func toModelCollectedTrigger(ct event.CollectedTrigger) model.CollectedTrigger {
	return model.CollectedTrigger{
		EffectName: ct.Trigger.EffectName,
		Params:     ct.Trigger.Params,
		Priority:   ct.Trigger.Priority,
		Owner: model.TriggerOwner{
			UnitID:  ct.Owner.UnitID,
			PartID:  ct.Owner.PartID,
			MountID: ct.Owner.MountID,
			ItemID:  ct.Owner.ItemID,
		},
		Conditions:       ct.Trigger.Conditions,
		TargetConditions: ct.Trigger.TargetConditions,
	}
}
