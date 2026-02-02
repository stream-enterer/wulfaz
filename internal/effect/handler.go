package effect

import (
	"fmt"
	"maps"
	"sort"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/event"
)

// EffectContext provides state for effect execution
type EffectContext struct {
	Owner            event.TriggerOwner
	SourceUnit       entity.Unit
	AllUnits         map[string]entity.Unit // keyed by unit ID
	PlayerUnitIDs    map[string]bool        // set of player-side unit IDs
	Rolls            []int
	TargetConditions []core.Condition // conditions to filter target units
}

// Handle executes an effect and returns the result
func Handle(effectName string, params map[string]any, ctx EffectContext) EffectResult {
	switch effectName {
	case "deal_damage":
		return handleDealDamage(params, ctx)
	case "consume_ammo":
		return handleConsumeAmmo(params, ctx)
	case "deal_splash_damage":
		// MVP: same as deal_damage (radius used post-MVP)
		return handleDealDamage(params, ctx)
	default:
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("unknown effect: %s", effectName)},
		}
	}
}

// handleDealDamage reduces target's health and emits follow-up events
func handleDealDamage(params map[string]any, ctx EffectContext) EffectResult {
	damage, ok := core.GetParamInt(params, "damage")
	if !ok {
		return EffectResult{
			LogEntries: []string{"deal_damage: missing damage param"},
		}
	}

	target, ok := resolveTarget(params, ctx)
	if !ok {
		return EffectResult{} // No valid target, silent no-op
	}

	// Check target has health attribute
	healthAttr, hasHealth := target.Attributes["health"]
	if !hasHealth {
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("deal_damage: target %s has no health attribute", target.ID)},
		}
	}

	// Calculate new health
	newHealth := healthAttr.Base - damage

	// Apply floor if Min is set (0 means no floor per core/attribute.go)
	if healthAttr.Min != 0 && newHealth < healthAttr.Min {
		newHealth = healthAttr.Min
	}
	// For health, also clamp to 0 minimum regardless of Min setting
	if newHealth < 0 {
		newHealth = 0
	}

	// Create modified unit (immutable copy)
	modifiedTarget := copyUnit(target)
	modifiedAttr := healthAttr
	modifiedAttr.Base = newHealth
	modifiedTarget.Attributes["health"] = modifiedAttr

	result := EffectResult{
		ModifiedUnits: map[string]entity.Unit{
			target.ID: modifiedTarget,
		},
		LogEntries: []string{fmt.Sprintf("%s dealt %d damage to %s (health: %d -> %d)",
			ctx.SourceUnit.ID, damage, target.ID, healthAttr.Base, newHealth)},
	}

	// Emit follow-up events
	if newHealth < healthAttr.Base {
		result.FollowUpEvents = append(result.FollowUpEvents, FollowUpEvent{
			Event:    core.EventOnDamaged,
			SourceID: ctx.SourceUnit.ID,
			TargetID: target.ID,
		})
	}

	if newHealth <= 0 && healthAttr.Base > 0 {
		result.FollowUpEvents = append(result.FollowUpEvents, FollowUpEvent{
			Event:    core.EventOnDestroyed,
			SourceID: ctx.SourceUnit.ID,
			TargetID: target.ID,
		})
	}

	return result
}

// handleConsumeAmmo reduces owning item's ammo attribute
func handleConsumeAmmo(params map[string]any, ctx EffectContext) EffectResult {
	amount, ok := core.GetParamInt(params, "amount")
	if !ok {
		amount = 1 // default to consuming 1 ammo
	}

	// Find the item that owns this trigger
	if ctx.Owner.ItemID == "" {
		return EffectResult{
			LogEntries: []string{"consume_ammo: not triggered by an item"},
		}
	}

	// Navigate to the item
	part, ok := ctx.SourceUnit.Parts[ctx.Owner.PartID]
	if !ok {
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("consume_ammo: part %s not found", ctx.Owner.PartID)},
		}
	}

	var mountIndex int
	var mountFound bool
	for i := range part.Mounts {
		if part.Mounts[i].ID == ctx.Owner.MountID {
			mountIndex = i
			mountFound = true
			break
		}
	}
	if !mountFound {
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("consume_ammo: mount %s not found", ctx.Owner.MountID)},
		}
	}
	mount := part.Mounts[mountIndex]

	var itemIndex int
	var itemFound bool
	for i := range mount.Contents {
		if mount.Contents[i].ID == ctx.Owner.ItemID {
			itemIndex = i
			itemFound = true
			break
		}
	}
	if !itemFound {
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("consume_ammo: item %s not found", ctx.Owner.ItemID)},
		}
	}
	item := mount.Contents[itemIndex]

	// Check ammo attribute
	ammoAttr, hasAmmo := item.Attributes["ammo"]
	if !hasAmmo {
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("consume_ammo: item %s has no ammo attribute", item.ID)},
		}
	}

	// Calculate new ammo
	newAmmo := max(0, ammoAttr.Base-amount)

	// Create modified unit (deep copy with updated item)
	modifiedUnit := copyUnit(ctx.SourceUnit)
	modifiedPart := modifiedUnit.Parts[ctx.Owner.PartID]

	// Copy mounts slice
	modifiedMounts := make([]entity.Mount, len(modifiedPart.Mounts))
	copy(modifiedMounts, modifiedPart.Mounts)
	modifiedPart.Mounts = modifiedMounts

	// Copy contents slice
	modifiedContents := make([]entity.Item, len(modifiedMounts[mountIndex].Contents))
	copy(modifiedContents, modifiedMounts[mountIndex].Contents)
	modifiedMounts[mountIndex].Contents = modifiedContents

	// Update the item's ammo
	modifiedItem := modifiedContents[itemIndex]
	modifiedItemAttrs := make(map[string]core.Attribute, len(modifiedItem.Attributes))
	maps.Copy(modifiedItemAttrs, modifiedItem.Attributes)
	modifiedAmmoAttr := ammoAttr
	modifiedAmmoAttr.Base = newAmmo
	modifiedItemAttrs["ammo"] = modifiedAmmoAttr
	modifiedItem.Attributes = modifiedItemAttrs
	modifiedContents[itemIndex] = modifiedItem

	modifiedUnit.Parts[ctx.Owner.PartID] = modifiedPart

	return EffectResult{
		ModifiedUnits: map[string]entity.Unit{
			ctx.SourceUnit.ID: modifiedUnit,
		},
		LogEntries: []string{fmt.Sprintf("%s consumed %d ammo (ammo: %d -> %d)",
			item.ID, amount, ammoAttr.Base, newAmmo)},
	}
}

// resolveTarget determines which unit is the effect target
func resolveTarget(params map[string]any, ctx EffectContext) (entity.Unit, bool) {
	targetStr, ok := params["target"].(string)
	if !ok {
		targetStr = "enemy" // default target
	}

	switch targetStr {
	case "self":
		return ctx.SourceUnit, true
	case "enemy":
		enemies := getEnemiesOf(ctx.SourceUnit, ctx)
		if len(enemies) == 0 {
			return entity.Unit{}, false
		}
		// MVP: target first enemy
		return enemies[0], true
	case "ally":
		// POST-MVP: for now, target self
		return ctx.SourceUnit, true
	default:
		// Try to resolve as unit ID
		if unit, ok := ctx.AllUnits[targetStr]; ok {
			return unit, true
		}
		return entity.Unit{}, false
	}
}

// getEnemiesOf returns living units not on the same side as source, sorted by ID for determinism.
// Uses PlayerUnitIDs from context to determine sides.
// Filters out dead units (IsAlive check) and by TargetConditions if present.
func getEnemiesOf(source entity.Unit, ctx EffectContext) []entity.Unit {
	var enemies []entity.Unit
	sourceIsPlayer := ctx.PlayerUnitIDs[source.ID]

	// Collect enemies
	for _, unit := range ctx.AllUnits {
		if unit.ID == source.ID {
			continue
		}
		if !unit.IsAlive() {
			continue
		}
		unitIsPlayer := ctx.PlayerUnitIDs[unit.ID]
		if sourceIsPlayer != unitIsPlayer {
			// Filter by target conditions
			if len(ctx.TargetConditions) > 0 {
				if !core.EvaluateConditions(ctx.TargetConditions, unit.Tags, unit.Attributes) {
					continue
				}
			}
			enemies = append(enemies, unit)
		}
	}

	// Sort by ID for deterministic targeting
	sort.Slice(enemies, func(i, j int) bool {
		return enemies[i].ID < enemies[j].ID
	})

	return enemies
}

// copyUnit creates a copy of a unit with new maps and slices for modification safety
func copyUnit(u entity.Unit) entity.Unit {
	newAttrs := make(map[string]core.Attribute, len(u.Attributes))
	maps.Copy(newAttrs, u.Attributes)

	newParts := make(map[string]entity.Part, len(u.Parts))
	maps.Copy(newParts, u.Parts)

	newTags := make([]core.Tag, len(u.Tags))
	copy(newTags, u.Tags)

	newTriggers := make([]core.Trigger, len(u.Triggers))
	copy(newTriggers, u.Triggers)

	newAbilities := make([]core.Ability, len(u.Abilities))
	copy(newAbilities, u.Abilities)

	var newDice []entity.Die
	if u.Dice != nil {
		newDice = make([]entity.Die, len(u.Dice))
		for i, d := range u.Dice {
			if d.Faces != nil {
				faces := make([]entity.DieFace, len(d.Faces))
				copy(faces, d.Faces)
				newDice[i] = entity.Die{Faces: faces}
			}
		}
	}

	return entity.Unit{
		ID:         u.ID,
		TemplateID: u.TemplateID,
		Tags:       newTags,
		Attributes: newAttrs,
		Parts:      newParts,
		Triggers:   newTriggers,
		Abilities:  newAbilities,
		Dice:       newDice,
		Pilot:      u.Pilot,
		HasPilot:   u.HasPilot,
		Position:   u.Position,
	}
}

