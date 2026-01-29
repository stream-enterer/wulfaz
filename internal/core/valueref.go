package core

// ValueRef is static int for MVP
// POST-MVP: adds Ref string, Expr string
type ValueRef struct {
	Value int
}

func (v ValueRef) Resolve() int { return v.Value }
