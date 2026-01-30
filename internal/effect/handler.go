package effect

import (
	"fmt"
	"sort"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/event"
)

// EffectContext provides state for effect execution
type EffectContext struct {
	Owner      event.TriggerOwner
	SourceUnit entity.Unit
	AllUnits   map[string]entity.Unit // keyed by unit ID
	Rolls      []int
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
	damage, ok := getParamInt(params, "damage")
	if !ok {
		return EffectResult{
			LogEntries: []string{"deal_damage: missing damage param"},
		}
	}

	target, ok := resolveTarget(params, ctx)
	if !ok {
		return EffectResult{
			LogEntries: []string{"deal_damage: could not resolve target"},
		}
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
	amount, ok := getParamInt(params, "amount")
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

	var foundMount *entity.Mount
	var mountIndex int
	for i := range part.Mounts {
		if part.Mounts[i].ID == ctx.Owner.MountID {
			foundMount = &part.Mounts[i]
			mountIndex = i
			break
		}
	}
	if foundMount == nil {
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("consume_ammo: mount %s not found", ctx.Owner.MountID)},
		}
	}

	var foundItem *entity.Item
	var itemIndex int
	for i := range foundMount.Contents {
		if foundMount.Contents[i].ID == ctx.Owner.ItemID {
			foundItem = &foundMount.Contents[i]
			itemIndex = i
			break
		}
	}
	if foundItem == nil {
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("consume_ammo: item %s not found", ctx.Owner.ItemID)},
		}
	}

	// Check ammo attribute
	ammoAttr, hasAmmo := foundItem.Attributes["ammo"]
	if !hasAmmo {
		return EffectResult{
			LogEntries: []string{fmt.Sprintf("consume_ammo: item %s has no ammo attribute", foundItem.ID)},
		}
	}

	// Calculate new ammo
	newAmmo := ammoAttr.Base - amount
	if newAmmo < 0 {
		newAmmo = 0
	}

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
	modifiedItemAttrs := make(map[string]core.Attribute)
	for k, v := range modifiedItem.Attributes {
		modifiedItemAttrs[k] = v
	}
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
			foundItem.ID, amount, ammoAttr.Base, newAmmo)},
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
		enemies := getEnemiesOf(ctx.SourceUnit, ctx.AllUnits)
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

// getEnemiesOf returns units not on the same side as source, sorted by ID for determinism.
// MVP: assumes units have "player" or "enemy" tag to determine side.
func getEnemiesOf(source entity.Unit, allUnits map[string]entity.Unit) []entity.Unit {
	var enemies []entity.Unit
	sourceIsPlayer := hasTag(source.Tags, "player")

	// Collect enemies
	for _, unit := range allUnits {
		if unit.ID == source.ID {
			continue
		}
		unitIsPlayer := hasTag(unit.Tags, "player")
		if sourceIsPlayer != unitIsPlayer {
			enemies = append(enemies, unit)
		}
	}

	// Sort by ID for deterministic targeting
	sort.Slice(enemies, func(i, j int) bool {
		return enemies[i].ID < enemies[j].ID
	})

	return enemies
}

// hasTag checks if a tag slice contains a specific tag
func hasTag(tags []core.Tag, target string) bool {
	for _, t := range tags {
		if string(t) == target {
			return true
		}
	}
	return false
}

// copyUnit creates a shallow copy of a unit with new attribute and part maps
func copyUnit(u entity.Unit) entity.Unit {
	newAttrs := make(map[string]core.Attribute)
	for k, v := range u.Attributes {
		newAttrs[k] = v
	}

	newParts := make(map[string]entity.Part)
	for k, v := range u.Parts {
		newParts[k] = v
	}

	newTags := make([]core.Tag, len(u.Tags))
	copy(newTags, u.Tags)

	newTriggers := make([]core.Trigger, len(u.Triggers))
	copy(newTriggers, u.Triggers)

	newAbilities := make([]core.Ability, len(u.Abilities))
	copy(newAbilities, u.Abilities)

	return entity.Unit{
		ID:         u.ID,
		TemplateID: u.TemplateID,
		Tags:       newTags,
		Attributes: newAttrs,
		Parts:      newParts,
		Triggers:   newTriggers,
		Abilities:  newAbilities,
		Pilot:      u.Pilot,
		HasPilot:   u.HasPilot,
	}
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
