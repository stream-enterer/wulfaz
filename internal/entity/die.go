package entity

// DieType represents the type of effect a die produces.
type DieType string

const (
	DieDamage DieType = "damage"
	DieShield DieType = "shield"
	DieHeal   DieType = "heal"
)

// Die represents a single die that a unit can roll.
// Faces contains the value for each face (len = number of faces).
// Example: [2, 2, 3, 4, 0, 0] is a 6-sided die with values 2,2,3,4,0,0.
type Die struct {
	Type  DieType
	Faces []int
}

// RolledDie represents a die that has been rolled with its current result.
type RolledDie struct {
	Type      DieType
	Faces     []int
	FaceIndex int  // Index into Faces array (for reroll tracking)
	Result    int  // The actual face value (Faces[FaceIndex])
	Locked    bool // Whether locked from rerolling
}
