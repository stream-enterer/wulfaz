package tea

type Cmd func() Msg

func None() Cmd { return nil }

func Batch(cmds ...Cmd) Cmd {
	if len(cmds) == 0 {
		return nil
	}
	return func() Msg {
		for _, cmd := range cmds {
			if cmd != nil {
				if msg := cmd(); msg != nil {
					return msg // simplified: return first
				}
			}
		}
		return nil
	}
}
