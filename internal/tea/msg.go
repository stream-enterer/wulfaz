package tea

type Msg interface {
	isMsg() // sealed
}

// Combat messages
type CombatStarted struct{ Seed int64 }

func (CombatStarted) isMsg() {}

type CombatTicked struct{ Rolls []int }

func (CombatTicked) isMsg() {}

type AbilityActivated struct {
	SourceID  string
	AbilityID string
	TargetID  string
	Rolls     []int
}

func (AbilityActivated) isMsg() {}

// Player control messages
type PlayerPaused struct{}

func (PlayerPaused) isMsg() {}

type PlayerResumed struct{}

func (PlayerResumed) isMsg() {}

type PlayerQuit struct{}

func (PlayerQuit) isMsg() {}
