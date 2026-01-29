package template

import "wulfaz/internal/entity"

type Registry struct {
	units map[string]entity.Unit
	items map[string]entity.Item
}

func NewRegistry() *Registry {
	return &Registry{
		units: make(map[string]entity.Unit),
		items: make(map[string]entity.Item),
	}
}

func (r *Registry) RegisterUnit(id string, unit entity.Unit) {
	r.units[id] = unit
}

func (r *Registry) GetUnit(id string) (entity.Unit, bool) {
	u, ok := r.units[id]
	return u, ok
}

func (r *Registry) RegisterItem(id string, item entity.Item) {
	r.items[id] = item
}

func (r *Registry) GetItem(id string) (entity.Item, bool) {
	i, ok := r.items[id]
	return i, ok
}
