package entity

// DieType represents the type of effect a die produces.
type DieType string

const (
	DieDamage DieType = "damage"
	DieShield DieType = "shield"
	DieHeal   DieType = "heal"
	DieBlank  DieType = "blank"
)

// DieFace represents a single face on a die.
type DieFace struct {
	Type  DieType
	Value int
}

// Die represents a die template with multiple faces.
type Die struct {
	Faces []DieFace
}

// RolledDie represents a rolled die with current state.
type RolledDie struct {
	Faces     []DieFace // All faces on this die
	FaceIndex int       // Current face index
	Locked    bool      // Whether locked from rerolling
	Fired     bool      // Whether this die has been activated/spent
}

// CurrentFace returns the die face at the current index.
func (rd RolledDie) CurrentFace() DieFace {
	if rd.FaceIndex < 0 || rd.FaceIndex >= len(rd.Faces) {
		return DieFace{Type: DieBlank, Value: 0}
	}
	return rd.Faces[rd.FaceIndex]
}

// Type returns the type of the current face.
func (rd RolledDie) Type() DieType { return rd.CurrentFace().Type }

// Value returns the value of the current face.
func (rd RolledDie) Value() int { return rd.CurrentFace().Value }

// IsUnitLocked returns true if the unit's dice are locked.
// All dice on a unit lock together, so we check the first die.
func IsUnitLocked(dice []RolledDie) bool {
	if len(dice) == 0 {
		return false
	}
	return dice[0].Locked
}

// HasNonBlankDie returns true if any die has a non-blank current face.
func HasNonBlankDie(dice []RolledDie) bool {
	for _, d := range dice {
		if d.CurrentFace().Type != DieBlank {
			return true
		}
	}
	return false
}

// HasDieOfType returns true if any die has a current face of the given type.
func HasDieOfType(dice []RolledDie, t DieType) bool {
	for _, d := range dice {
		if d.CurrentFace().Type == t {
			return true
		}
	}
	return false
}

// HasUnfiredDieOfType returns true if any unfired die has a current face of the given type.
func HasUnfiredDieOfType(dice []RolledDie, t DieType) bool {
	for _, d := range dice {
		if !d.Fired && d.CurrentFace().Type == t {
			return true
		}
	}
	return false
}

// AllNonBlankFired returns true if every non-blank die has Fired == true.
// Returns true for blank-only units (vacuous truth).
func AllNonBlankFired(dice []RolledDie) bool {
	for _, d := range dice {
		if d.CurrentFace().Type != DieBlank && !d.Fired {
			return false
		}
	}
	return true
}

// PrimaryEffectType returns the type of the first non-blank die face, or DieBlank if all blank.
func PrimaryEffectType(dice []RolledDie) DieType {
	for _, d := range dice {
		if d.CurrentFace().Type != DieBlank {
			return d.CurrentFace().Type
		}
	}
	return DieBlank
}
