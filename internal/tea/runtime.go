package tea

type Runtime struct {
	model Model
}

func NewRuntime(seed int64) *Runtime {
	return &Runtime{
		model: Model{
			Phase: PhaseMenu,
			Seed:  seed,
		},
	}
}

func (r *Runtime) Run() {
	panic("not implemented")
}

func (r *Runtime) Dispatch(msg Msg) {
	var cmd Cmd
	r.model, cmd = r.model.Update(msg)
	if cmd != nil {
		result := cmd()
		if result != nil {
			r.Dispatch(result)
		}
	}
}
