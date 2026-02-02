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
