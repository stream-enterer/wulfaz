package template

import "wulfaz/internal/entity"

type Registry struct {
	units map[string]entity.Unit
}

func NewRegistry() *Registry {
	return &Registry{
		units: make(map[string]entity.Unit),
	}
}

func (r *Registry) RegisterUnit(id string, unit entity.Unit) {
	r.units[id] = unit
}

func (r *Registry) GetUnit(id string) (entity.Unit, bool) {
	u, ok := r.units[id]
	return u, ok
}
