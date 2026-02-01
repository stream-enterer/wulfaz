package entity

import "wulfaz/internal/core"

// CopyMountCriteria copies MountCriteria (has 3 tag slices).
func CopyMountCriteria(mc MountCriteria) MountCriteria {
	return MountCriteria{
		RequiresAll: core.CopyTags(mc.RequiresAll),
		RequiresAny: core.CopyTags(mc.RequiresAny),
		Forbids:     core.CopyTags(mc.Forbids),
	}
}

// CopyItem creates a deep copy with a new ID.
// Copies: Tags, Attributes, Triggers, Abilities, ProvidedModifiers, Requirements.
func CopyItem(item Item, newID string) Item {
	return Item{
		ID:                newID,
		TemplateID:        item.TemplateID,
		Tags:              core.CopyTags(item.Tags),
		Attributes:        core.CopyAttributes(item.Attributes),
		Triggers:          core.CopyTriggers(item.Triggers),
		Abilities:         core.CopyAbilities(item.Abilities),
		ProvidedModifiers: core.CopyProvidedModifiers(item.ProvidedModifiers),
		Requirements:      core.CopyRequirements(item.Requirements),
	}
}

// CopyMount creates a deep copy.
// Copies: Tags, Accepts (MountCriteria), Contents (each Item deep copied).
func CopyMount(m Mount) Mount {
	var contents []Item
	if m.Contents != nil {
		contents = make([]Item, len(m.Contents))
		for i, item := range m.Contents {
			contents[i] = CopyItem(item, item.ID)
		}
	}

	return Mount{
		ID:                m.ID,
		Tags:              core.CopyTags(m.Tags),
		Accepts:           CopyMountCriteria(m.Accepts),
		Capacity:          m.Capacity,
		CapacityAttribute: m.CapacityAttribute,
		MaxItems:          m.MaxItems,
		Locked:            m.Locked,
		Contents:          contents,
	}
}

// CopyMounts copies a slice of Mounts.
func CopyMounts(ms []Mount) []Mount {
	if ms == nil {
		return nil
	}
	result := make([]Mount, len(ms))
	for i, m := range ms {
		result[i] = CopyMount(m)
	}
	return result
}

// CopyConnections copies the connections map.
func CopyConnections(conns map[string][]string) map[string][]string {
	if conns == nil {
		return nil
	}
	result := make(map[string][]string, len(conns))
	for k, v := range conns {
		if v == nil {
			result[k] = nil
		} else {
			copied := make([]string, len(v))
			copy(copied, v)
			result[k] = copied
		}
	}
	return result
}

// CopyPart creates a deep copy.
// Copies: Tags, Attributes, Mounts, Connections, Triggers, Abilities.
func CopyPart(p Part) Part {
	return Part{
		ID:          p.ID,
		TemplateID:  p.TemplateID,
		Tags:        core.CopyTags(p.Tags),
		Attributes:  core.CopyAttributes(p.Attributes),
		Mounts:      CopyMounts(p.Mounts),
		Connections: CopyConnections(p.Connections),
		Triggers:    core.CopyTriggers(p.Triggers),
		Abilities:   core.CopyAbilities(p.Abilities),
	}
}

// CopyParts copies the parts map.
func CopyParts(parts map[string]Part) map[string]Part {
	if parts == nil {
		return nil
	}
	result := make(map[string]Part, len(parts))
	for k, v := range parts {
		result[k] = CopyPart(v)
	}
	return result
}

// CopyPilot creates a copy of a Pilot.
func CopyPilot(p Pilot) Pilot {
	return Pilot{
		ID:   p.ID,
		Name: p.Name,
	}
}

// CopyDie creates a deep copy of a Die.
func CopyDie(d Die) Die {
	if d.Faces == nil {
		return Die{Type: d.Type}
	}
	faces := make([]int, len(d.Faces))
	copy(faces, d.Faces)
	return Die{Type: d.Type, Faces: faces}
}

// CopyDice copies a slice of Dice.
func CopyDice(dice []Die) []Die {
	if dice == nil {
		return nil
	}
	result := make([]Die, len(dice))
	for i, d := range dice {
		result[i] = CopyDie(d)
	}
	return result
}

// CopyUnit creates a deep copy with a new ID.
// Copies: Tags, Attributes, Parts, Triggers, Abilities, Dice, Pilot.
func CopyUnit(u Unit, newID string) Unit {
	return Unit{
		ID:         newID,
		TemplateID: u.TemplateID,
		Tags:       core.CopyTags(u.Tags),
		Attributes: core.CopyAttributes(u.Attributes),
		Parts:      CopyParts(u.Parts),
		Triggers:   core.CopyTriggers(u.Triggers),
		Abilities:  core.CopyAbilities(u.Abilities),
		Dice:       CopyDice(u.Dice),
		Pilot:      CopyPilot(u.Pilot),
		HasPilot:   u.HasPilot,
	}
}
