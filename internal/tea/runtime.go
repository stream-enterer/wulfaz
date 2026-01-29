package tea

type Runtime struct {
	model Model
}

func NewRuntime(seed int64) *Runtime {
	return &Runtime{
		model: Model{
			Version: 1,
			Phase:   PhaseMenu,
			Seed:    seed,
		},
	}
}

func (r *Runtime) Run() {
	panic("not implemented")
}

func (r *Runtime) Dispatch(msg Msg) {
	// Unpack batched messages first
	if batch, ok := msg.(BatchedMsgs); ok {
		for _, m := range batch.Msgs {
			r.Dispatch(m)
		}
		return
	}

	var cmd Cmd
	r.model, cmd = r.model.Update(msg)
	if cmd != nil {
		result := cmd()
		if result != nil {
			r.Dispatch(result)
		}
	}
}
