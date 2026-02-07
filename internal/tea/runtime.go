package tea

import "wulfaz/internal/model"

type Runtime struct {
	model Model
}

func NewRuntime(seed int64) *Runtime {
	return &Runtime{
		model: Model{
			Version: 1,
			Phase:   model.PhaseMenu,
			Seed:    seed,
		},
	}
}

// Model returns the current model state (for testing)
func (r *Runtime) Model() Model {
	return r.model
}

func (r *Runtime) Dispatch(msg model.Msg) {
	// Unpack batched messages first
	if batch, ok := msg.(model.BatchedMsgs); ok {
		for _, m := range batch.Msgs {
			r.Dispatch(m)
		}
		return
	}

	var cmd model.Cmd
	r.model, cmd = r.model.Update(msg)
	if cmd != nil {
		result := cmd()
		if result != nil {
			r.Dispatch(result)
		}
	}
}
