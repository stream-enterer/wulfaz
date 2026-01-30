package template

import (
	"fmt"

	"wulfaz/internal/entity"
)

// InstantiateUnit creates a unit instance from a registered template.
// Returns error if template not found.
func InstantiateUnit(reg *Registry, templateID, instanceID string) (entity.Unit, error) {
	tmpl, ok := reg.GetUnit(templateID)
	if !ok {
		return entity.Unit{}, fmt.Errorf("unit template %q not found", templateID)
	}
	unit := entity.CopyUnit(tmpl, instanceID)
	unit.TemplateID = templateID
	return unit, nil
}

// InstantiateItem creates an item instance from a registered template.
// Returns error if template not found.
func InstantiateItem(reg *Registry, templateID, instanceID string) (entity.Item, error) {
	tmpl, ok := reg.GetItem(templateID)
	if !ok {
		return entity.Item{}, fmt.Errorf("item template %q not found", templateID)
	}
	item := entity.CopyItem(tmpl, instanceID)
	item.TemplateID = templateID
	return item, nil
}
