package template

import (
	"fmt"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

// EquipItem adds an item to a unit's mount, returning a new Unit (immutable).
// partID: key in Unit.Parts map
// mountIdx: index in Part.Mounts slice
// Returns error if: part not found, mount index invalid, mount locked,
// criteria not met, capacity exceeded, max items exceeded.
func EquipItem(unit entity.Unit, partID string, mountIdx int, item entity.Item) (entity.Unit, error) {
	part, ok := unit.Parts[partID]
	if !ok {
		return entity.Unit{}, fmt.Errorf("part %q not found", partID)
	}

	if mountIdx < 0 || mountIdx >= len(part.Mounts) {
		return entity.Unit{}, fmt.Errorf("mount index %d out of range (part %q has %d mounts)", mountIdx, partID, len(part.Mounts))
	}

	mount := part.Mounts[mountIdx]

	if mount.Locked {
		return entity.Unit{}, fmt.Errorf("mount %q is locked", mount.ID)
	}

	if !CanMount(mount, item) {
		return entity.Unit{}, fmt.Errorf("item %q does not meet mount %q criteria", item.ID, mount.ID)
	}

	capacityAttr := mount.CapacityAttribute
	if capacityAttr == "" {
		capacityAttr = "size"
	}
	itemSize := getItemCapacity(item, capacityAttr)
	used := usedCapacity(mount, capacityAttr)
	if mount.Capacity > 0 && used+itemSize > mount.Capacity {
		return entity.Unit{}, fmt.Errorf("mount %q capacity exceeded (%d used + %d item > %d capacity)", mount.ID, used, itemSize, mount.Capacity)
	}

	if mount.MaxItems >= 0 && len(mount.Contents) >= mount.MaxItems {
		return entity.Unit{}, fmt.Errorf("mount %q max items exceeded (%d items, max %d)", mount.ID, len(mount.Contents), mount.MaxItems)
	}

	// Copy unit
	newUnit := entity.CopyUnit(unit, unit.ID)

	// Get part (already copied)
	newPart := newUnit.Parts[partID]

	// Copy mounts slice before modification
	newMounts := make([]entity.Mount, len(newPart.Mounts))
	copy(newMounts, newPart.Mounts)

	// Copy target mount's contents before appending
	newMount := newMounts[mountIdx]
	newContents := make([]entity.Item, len(newMount.Contents), len(newMount.Contents)+1)
	copy(newContents, newMount.Contents)
	newContents = append(newContents, entity.CopyItem(item, item.ID))
	newMount.Contents = newContents

	// Write back
	newMounts[mountIdx] = newMount
	newPart.Mounts = newMounts
	newUnit.Parts[partID] = newPart

	return newUnit, nil
}

// CanMount checks if an item can be placed in a mount.
func CanMount(mount entity.Mount, item entity.Item) bool {
	return matchesCriteria(item.Tags, mount.Accepts)
}

// matchesCriteria checks item tags against mount acceptance criteria.
// Logic:
//   - RequiresAll: item must have ALL these tags (AND)
//   - RequiresAny: item must have AT LEAST ONE of these tags (OR), unless empty
//   - Forbids: item must have NONE of these tags
func matchesCriteria(itemTags []core.Tag, criteria entity.MountCriteria) bool {
	tagSet := make(map[core.Tag]bool, len(itemTags))
	for _, tag := range itemTags {
		tagSet[tag] = true
	}

	// RequiresAll: item must have ALL these tags
	for _, required := range criteria.RequiresAll {
		if !tagSet[required] {
			return false
		}
	}

	// RequiresAny: item must have AT LEAST ONE of these tags (unless empty)
	if len(criteria.RequiresAny) > 0 {
		hasAny := false
		for _, tag := range criteria.RequiresAny {
			if tagSet[tag] {
				hasAny = true
				break
			}
		}
		if !hasAny {
			return false
		}
	}

	// Forbids: item must have NONE of these tags
	for _, forbidden := range criteria.Forbids {
		if tagSet[forbidden] {
			return false
		}
	}

	return true
}

// getItemCapacity returns the item's capacity attribute value, defaulting to 1 if not present.
func getItemCapacity(item entity.Item, attrName string) int {
	if attr, ok := item.Attributes[attrName]; ok {
		return attr.Base
	}
	return 1
}

// usedCapacity returns sum of capacity attribute values of items in mount.
func usedCapacity(mount entity.Mount, attrName string) int {
	total := 0
	for _, item := range mount.Contents {
		total += getItemCapacity(item, attrName)
	}
	return total
}
