package core

type Attribute struct {
	Name string
	Base int
	Min  int // 0 = no floor
	Max  int // 0 = no ceiling (use MaxInt)
}
