package entity

import "wulfaz/internal/core"

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
		return Die{}
	}
	faces := make([]DieFace, len(d.Faces))
	copy(faces, d.Faces)
	return Die{Faces: faces}
}

// CopyRolledDie creates a deep copy of a RolledDie.
func CopyRolledDie(rd RolledDie) RolledDie {
	var faces []DieFace
	if rd.Faces != nil {
		faces = make([]DieFace, len(rd.Faces))
		copy(faces, rd.Faces)
	}
	return RolledDie{
		Faces:     faces,
		FaceIndex: rd.FaceIndex,
		Locked:    rd.Locked,
		Fired:     rd.Fired,
	}
}

// CopyRolledDiceSlice copies a slice of RolledDie.
func CopyRolledDiceSlice(dice []RolledDie) []RolledDie {
	if dice == nil {
		return nil
	}
	result := make([]RolledDie, len(dice))
	for i, d := range dice {
		result[i] = CopyRolledDie(d)
	}
	return result
}

// CopyDiceSlice copies a slice of Die.
func CopyDiceSlice(dice []Die) []Die {
	if dice == nil {
		return nil
	}
	result := make([]Die, len(dice))
	for i, d := range dice {
		result[i] = CopyDie(d)
	}
	return result
}

// CopyRolledDiceMap copies a map of unit ID to rolled dice slice.
func CopyRolledDiceMap(m map[string][]RolledDie) map[string][]RolledDie {
	if m == nil {
		return nil
	}
	result := make(map[string][]RolledDie, len(m))
	for k, v := range m {
		result[k] = CopyRolledDiceSlice(v)
	}
	return result
}

// CopyUnit creates a deep copy with a new ID.
func CopyUnit(u Unit, newID string) Unit {
	return Unit{
		ID:         newID,
		TemplateID: u.TemplateID,
		Tags:       core.CopyTags(u.Tags),
		Attributes: core.CopyAttributes(u.Attributes),
		Dice:       CopyDiceSlice(u.Dice),
		Pilot:      CopyPilot(u.Pilot),
		HasPilot:   u.HasPilot,
		Position:   u.Position,
	}
}
