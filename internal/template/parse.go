package template

import (
	"fmt"

	"github.com/sblinch/kdl-go/document"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

// ParseError provides context for KDL parsing failures.
type ParseError struct {
	File    string
	Node    string
	Field   string
	Message string
}

func (e *ParseError) Error() string {
	if e.Field != "" {
		return fmt.Sprintf("%s: %s.%s: %s", e.File, e.Node, e.Field, e.Message)
	}
	if e.Node != "" {
		return fmt.Sprintf("%s: %s: %s", e.File, e.Node, e.Message)
	}
	return fmt.Sprintf("%s: %s", e.File, e.Message)
}

// Node navigation helpers

// nodeName returns the string name of a node.
func nodeName(node *document.Node) string {
	if s, ok := node.Name.Value.(string); ok {
		return s
	}
	return ""
}

// findChild returns the first child node with the given name, or nil if not found.
func findChild(node *document.Node, name string) *document.Node {
	for _, child := range node.Children {
		if nodeName(child) == name {
			return child
		}
	}
	return nil
}

// findChildren returns all child nodes with the given name.
func findChildren(node *document.Node, name string) []*document.Node {
	var result []*document.Node
	for _, child := range node.Children {
		if nodeName(child) == name {
			result = append(result, child)
		}
	}
	return result
}

// getStringProp returns a string property value by name.
func getStringProp(node *document.Node, name string) (string, error) {
	if val, ok := node.Properties[name]; ok {
		if s, ok := val.Value.(string); ok {
			return s, nil
		}
		return "", fmt.Errorf("property %q is not a string", name)
	}
	return "", fmt.Errorf("property %q not found", name)
}

// getOptStringProp returns a string property value or the default if not found.
func getOptStringProp(node *document.Node, name, defaultVal string) string {
	s, err := getStringProp(node, name)
	if err != nil {
		return defaultVal
	}
	return s
}

// getIntProp returns an int property value by name.
func getIntProp(node *document.Node, name string) (int, error) {
	if val, ok := node.Properties[name]; ok {
		switch v := val.Value.(type) {
		case int64:
			return int(v), nil
		case int:
			return v, nil
		case float64:
			return int(v), nil
		}
		return 0, fmt.Errorf("property %q is not an integer", name)
	}
	return 0, fmt.Errorf("property %q not found", name)
}

// getOptIntProp returns an int property value or the default if not found.
func getOptIntProp(node *document.Node, name string, defaultVal int) int {
	i, err := getIntProp(node, name)
	if err != nil {
		return defaultVal
	}
	return i
}

// getBoolProp returns a bool property value by name.
func getBoolProp(node *document.Node, name string) (bool, error) {
	if val, ok := node.Properties[name]; ok {
		if b, ok := val.Value.(bool); ok {
			return b, nil
		}
		return false, fmt.Errorf("property %q is not a boolean", name)
	}
	return false, fmt.Errorf("property %q not found", name)
}

// getOptBoolProp returns a bool property value or the default if not found.
func getOptBoolProp(node *document.Node, name string, defaultVal bool) bool {
	b, err := getBoolProp(node, name)
	if err != nil {
		return defaultVal
	}
	return b
}

// getStringArgs returns all string arguments (positional values) from a node.
func getStringArgs(node *document.Node) []string {
	var result []string
	for _, arg := range node.Arguments {
		if s, ok := arg.Value.(string); ok {
			result = append(result, s)
		}
	}
	return result
}

// getPropsAsMap returns all properties as a map[string]any.
func getPropsAsMap(node *document.Node) map[string]any {
	result := make(map[string]any)
	for name, val := range node.Properties {
		switch v := val.Value.(type) {
		case string:
			result[name] = v
		case int64:
			result[name] = int(v)
		case int:
			result[name] = v
		case float64:
			result[name] = v
		case bool:
			result[name] = v
		}
	}
	return result
}

// Enum converters

// parseEventType validates and converts a string to EventType.
func parseEventType(s string) (core.EventType, error) {
	switch s {
	case "on_damaged":
		return core.EventOnDamaged, nil
	case "on_destroyed":
		return core.EventOnDestroyed, nil
	case "on_combat_tick":
		return core.EventOnCombatTick, nil
	case "on_turn_start":
		return core.EventOnTurnStart, nil
	case "on_turn_end":
		return core.EventOnTurnEnd, nil
	case "on_activate":
		return core.EventOnActivate, nil
	default:
		return "", fmt.Errorf("unknown event type: %q", s)
	}
}

// parseOnUnmet validates and converts a string to OnUnmet.
func parseOnUnmet(s string) (core.OnUnmet, error) {
	switch s {
	case "disabled":
		return core.OnUnmetDisabled, nil
	case "cannot_mount":
		return core.OnUnmetCannotMount, nil
	case "warning":
		return core.OnUnmetWarning, nil
	default:
		return 0, fmt.Errorf("unknown on_unmet value: %q", s)
	}
}

// parseModifierOp validates and converts a string to ModifierOp.
func parseModifierOp(s string) (core.ModifierOp, error) {
	switch s {
	case "add":
		return core.ModifierOpAdd, nil
	case "mult":
		return core.ModifierOpMult, nil
	case "set":
		return core.ModifierOpSet, nil
	case "min":
		return core.ModifierOpMin, nil
	case "max":
		return core.ModifierOpMax, nil
	default:
		return 0, fmt.Errorf("unknown modifier operation: %q", s)
	}
}

// parseScope validates and converts a string to Scope.
func parseScope(s string) (core.Scope, error) {
	switch s {
	case "self":
		return core.ScopeSelf, nil
	case "unit":
		return core.ScopeUnit, nil
	case "part":
		return core.ScopePart, nil
	case "adjacent":
		return core.ScopeAdjacent, nil
	case "mount":
		return core.ScopeMount, nil
	default:
		return "", fmt.Errorf("unknown scope: %q", s)
	}
}

// parseConditionType validates and converts a string to ConditionType.
func parseConditionType(s string) (core.ConditionType, error) {
	switch s {
	case "has_tag":
		return core.ConditionHasTag, nil
	case "attr_gte":
		return core.ConditionAttrGTE, nil
	case "attr_lte":
		return core.ConditionAttrLTE, nil
	case "attr_eq":
		return core.ConditionAttrEQ, nil
	default:
		return "", fmt.Errorf("unknown condition type: %q", s)
	}
}

// Component parsers

// parseTags extracts tags from a "tags" node's string arguments.
func parseTags(node *document.Node) []core.Tag {
	args := getStringArgs(node)
	tags := make([]core.Tag, len(args))
	for i, s := range args {
		tags[i] = core.Tag(s)
	}
	return tags
}

// parseAttributes parses an "attributes" node into a map of attributes.
func parseAttributes(node *document.Node, filename string) (map[string]core.Attribute, error) {
	result := make(map[string]core.Attribute)
	for _, child := range findChildren(node, "attribute") {
		name, err := getStringProp(child, "name")
		if err != nil {
			return nil, &ParseError{File: filename, Node: "attribute", Field: "name", Message: err.Error()}
		}
		base, err := getIntProp(child, "base")
		if err != nil {
			return nil, &ParseError{File: filename, Node: "attribute", Field: "base", Message: err.Error()}
		}
		attr := core.Attribute{
			Name: name,
			Base: base,
			Min:  getOptIntProp(child, "min", 0),
			Max:  getOptIntProp(child, "max", 0),
		}
		result[name] = attr
	}
	return result, nil
}

// parseCondition parses a "condition" node.
func parseCondition(node *document.Node, filename string) (core.Condition, error) {
	typeStr, err := getStringProp(node, "type")
	if err != nil {
		return core.Condition{}, &ParseError{File: filename, Node: "condition", Field: "type", Message: err.Error()}
	}
	condType, err := parseConditionType(typeStr)
	if err != nil {
		return core.Condition{}, &ParseError{File: filename, Node: "condition", Field: "type", Message: err.Error()}
	}

	// Get all other properties as params (exclude "type")
	params := make(map[string]any)
	for name, val := range node.Properties {
		if name == "type" {
			continue
		}
		switch v := val.Value.(type) {
		case string:
			params[name] = v
		case int64:
			params[name] = int(v)
		case int:
			params[name] = v
		case float64:
			params[name] = v
		case bool:
			params[name] = v
		}
	}

	return core.Condition{
		Type:   condType,
		Params: params,
	}, nil
}

// parseTrigger parses a "trigger" node.
func parseTrigger(node *document.Node, filename string) (core.Trigger, error) {
	eventStr, err := getStringProp(node, "event")
	if err != nil {
		return core.Trigger{}, &ParseError{File: filename, Node: "trigger", Field: "event", Message: err.Error()}
	}
	event, err := parseEventType(eventStr)
	if err != nil {
		return core.Trigger{}, &ParseError{File: filename, Node: "trigger", Field: "event", Message: err.Error()}
	}

	effectName, err := getStringProp(node, "effect_name")
	if err != nil {
		return core.Trigger{}, &ParseError{File: filename, Node: "trigger", Field: "effect_name", Message: err.Error()}
	}

	priority := getOptIntProp(node, "priority", 0)

	// Parse params from child node
	var params map[string]any
	if paramsNode := findChild(node, "params"); paramsNode != nil {
		params = getPropsAsMap(paramsNode)
	}

	// Parse conditions from child nodes
	var conditions []core.Condition
	if conditionsNode := findChild(node, "conditions"); conditionsNode != nil {
		for _, condNode := range findChildren(conditionsNode, "condition") {
			cond, err := parseCondition(condNode, filename)
			if err != nil {
				return core.Trigger{}, err
			}
			conditions = append(conditions, cond)
		}
	}

	return core.Trigger{
		Event:      event,
		EffectName: effectName,
		Params:     params,
		Priority:   priority,
		Conditions: conditions,
	}, nil
}

// parseTriggers parses a "triggers" node.
func parseTriggers(node *document.Node, filename string) ([]core.Trigger, error) {
	var triggers []core.Trigger
	for _, triggerNode := range findChildren(node, "trigger") {
		trigger, err := parseTrigger(triggerNode, filename)
		if err != nil {
			return nil, err
		}
		triggers = append(triggers, trigger)
	}
	return triggers, nil
}

// parseRequirement parses a "requirement" node.
func parseRequirement(node *document.Node, filename string) (core.Requirement, error) {
	scope := getOptStringProp(node, "scope", "unit")

	onUnmetStr := getOptStringProp(node, "on_unmet", "disabled")
	onUnmet, err := parseOnUnmet(onUnmetStr)
	if err != nil {
		return core.Requirement{}, &ParseError{File: filename, Node: "requirement", Field: "on_unmet", Message: err.Error()}
	}

	condNode := findChild(node, "condition")
	if condNode == nil {
		return core.Requirement{}, &ParseError{File: filename, Node: "requirement", Field: "condition", Message: "missing condition"}
	}
	cond, err := parseCondition(condNode, filename)
	if err != nil {
		return core.Requirement{}, err
	}

	return core.Requirement{
		Scope:     scope,
		OnUnmet:   onUnmet,
		Condition: cond,
	}, nil
}

// parseRequirements parses a "requirements" node.
func parseRequirements(node *document.Node, filename string) ([]core.Requirement, error) {
	var reqs []core.Requirement
	for _, reqNode := range findChildren(node, "requirement") {
		req, err := parseRequirement(reqNode, filename)
		if err != nil {
			return nil, err
		}
		reqs = append(reqs, req)
	}
	return reqs, nil
}

// parseModifier parses a "modifier" node into a ProvidedModifier.
func parseModifier(node *document.Node, filename string) (core.ProvidedModifier, error) {
	scopeStr := getOptStringProp(node, "scope", "self")
	scope, err := parseScope(scopeStr)
	if err != nil {
		return core.ProvidedModifier{}, &ParseError{File: filename, Node: "modifier", Field: "scope", Message: err.Error()}
	}

	attr, err := getStringProp(node, "attribute")
	if err != nil {
		return core.ProvidedModifier{}, &ParseError{File: filename, Node: "modifier", Field: "attribute", Message: err.Error()}
	}

	opStr, err := getStringProp(node, "operation")
	if err != nil {
		return core.ProvidedModifier{}, &ParseError{File: filename, Node: "modifier", Field: "operation", Message: err.Error()}
	}
	op, err := parseModifierOp(opStr)
	if err != nil {
		return core.ProvidedModifier{}, &ParseError{File: filename, Node: "modifier", Field: "operation", Message: err.Error()}
	}

	value, err := getIntProp(node, "value")
	if err != nil {
		return core.ProvidedModifier{}, &ParseError{File: filename, Node: "modifier", Field: "value", Message: err.Error()}
	}

	stackGroup := getOptStringProp(node, "stack_group", "")

	// Parse scope_filter tags if present
	var scopeFilter []core.Tag
	if filterNode := findChild(node, "scope_filter"); filterNode != nil {
		args := getStringArgs(filterNode)
		for _, s := range args {
			scopeFilter = append(scopeFilter, core.Tag(s))
		}
	}

	// Parse conditions if present
	var conditions []core.Condition
	if conditionsNode := findChild(node, "conditions"); conditionsNode != nil {
		for _, condNode := range findChildren(conditionsNode, "condition") {
			cond, err := parseCondition(condNode, filename)
			if err != nil {
				return core.ProvidedModifier{}, err
			}
			conditions = append(conditions, cond)
		}
	}

	return core.ProvidedModifier{
		Scope:       scope,
		ScopeFilter: scopeFilter,
		Attribute:   attr,
		Operation:   op,
		Value:       value,
		StackGroup:  stackGroup,
		Conditions:  conditions,
	}, nil
}

// parseProvidedModifiers parses a "modifiers" node.
func parseProvidedModifiers(node *document.Node, filename string) ([]core.ProvidedModifier, error) {
	var mods []core.ProvidedModifier
	for _, modNode := range findChildren(node, "modifier") {
		mod, err := parseModifier(modNode, filename)
		if err != nil {
			return nil, err
		}
		mods = append(mods, mod)
	}
	return mods, nil
}

// parseMountCriteria parses an "accepts" node into MountCriteria.
func parseMountCriteria(node *document.Node) entity.MountCriteria {
	var criteria entity.MountCriteria

	if reqAllNode := findChild(node, "requires_all"); reqAllNode != nil {
		for _, s := range getStringArgs(reqAllNode) {
			criteria.RequiresAll = append(criteria.RequiresAll, core.Tag(s))
		}
	}

	if reqAnyNode := findChild(node, "requires_any"); reqAnyNode != nil {
		for _, s := range getStringArgs(reqAnyNode) {
			criteria.RequiresAny = append(criteria.RequiresAny, core.Tag(s))
		}
	}

	if forbidsNode := findChild(node, "forbids"); forbidsNode != nil {
		for _, s := range getStringArgs(forbidsNode) {
			criteria.Forbids = append(criteria.Forbids, core.Tag(s))
		}
	}

	return criteria
}

// parseMount parses a "mount" node.
func parseMount(node *document.Node, filename string) (entity.Mount, error) {
	id, err := getStringProp(node, "id")
	if err != nil {
		return entity.Mount{}, &ParseError{File: filename, Node: "mount", Field: "id", Message: err.Error()}
	}

	mount := entity.Mount{
		ID:                id,
		Capacity:          getOptIntProp(node, "capacity", 0),
		CapacityAttribute: getOptStringProp(node, "capacity_attribute", "size"),
		MaxItems:          getOptIntProp(node, "max_items", -1),
		Locked:            getOptBoolProp(node, "locked", false),
	}

	// Parse tags
	if tagsNode := findChild(node, "tags"); tagsNode != nil {
		mount.Tags = parseTags(tagsNode)
	}

	// Parse accepts criteria
	if acceptsNode := findChild(node, "accepts"); acceptsNode != nil {
		mount.Accepts = parseMountCriteria(acceptsNode)
	}

	return mount, nil
}

// Entity parsers

// parsePart parses a "part" node.
func parsePart(node *document.Node, filename string) (entity.Part, error) {
	id, err := getStringProp(node, "id")
	if err != nil {
		return entity.Part{}, &ParseError{File: filename, Node: "part", Field: "id", Message: err.Error()}
	}

	templateID := getOptStringProp(node, "template_id", "")

	part := entity.Part{
		ID:         id,
		TemplateID: templateID,
	}

	// Parse tags
	if tagsNode := findChild(node, "tags"); tagsNode != nil {
		part.Tags = parseTags(tagsNode)
	}

	// Parse attributes
	if attrsNode := findChild(node, "attributes"); attrsNode != nil {
		attrs, err := parseAttributes(attrsNode, filename)
		if err != nil {
			return entity.Part{}, err
		}
		part.Attributes = attrs
	}

	// Parse mounts
	if mountsNode := findChild(node, "mounts"); mountsNode != nil {
		for _, mountNode := range findChildren(mountsNode, "mount") {
			mount, err := parseMount(mountNode, filename)
			if err != nil {
				return entity.Part{}, err
			}
			part.Mounts = append(part.Mounts, mount)
		}
	}

	// Parse connections
	if connsNode := findChild(node, "connections"); connsNode != nil {
		part.Connections = make(map[string][]string)
		for _, connNode := range connsNode.Children {
			name := nodeName(connNode)
			part.Connections[name] = getStringArgs(connNode)
		}
	}

	// Parse triggers
	if triggersNode := findChild(node, "triggers"); triggersNode != nil {
		triggers, err := parseTriggers(triggersNode, filename)
		if err != nil {
			return entity.Part{}, err
		}
		part.Triggers = triggers
	}

	return part, nil
}

// parseUnit parses a "unit" node into an entity.Unit.
func parseUnit(node *document.Node, filename string) (entity.Unit, error) {
	id, err := getStringProp(node, "id")
	if err != nil {
		return entity.Unit{}, &ParseError{File: filename, Node: "unit", Field: "id", Message: err.Error()}
	}

	unit := entity.Unit{
		ID:         id,
		TemplateID: id,
	}

	// Parse tags
	if tagsNode := findChild(node, "tags"); tagsNode != nil {
		unit.Tags = parseTags(tagsNode)
	}

	// Parse attributes
	if attrsNode := findChild(node, "attributes"); attrsNode != nil {
		attrs, err := parseAttributes(attrsNode, filename)
		if err != nil {
			return entity.Unit{}, err
		}
		unit.Attributes = attrs
	}

	// Parse parts
	if partsNode := findChild(node, "parts"); partsNode != nil {
		unit.Parts = make(map[string]entity.Part)
		for _, partNode := range findChildren(partsNode, "part") {
			part, err := parsePart(partNode, filename)
			if err != nil {
				return entity.Unit{}, err
			}
			unit.Parts[part.ID] = part
		}
	}

	// Parse triggers
	if triggersNode := findChild(node, "triggers"); triggersNode != nil {
		triggers, err := parseTriggers(triggersNode, filename)
		if err != nil {
			return entity.Unit{}, err
		}
		unit.Triggers = triggers
	}

	return unit, nil
}

// parseItem parses an "item" node into an entity.Item.
func parseItem(node *document.Node, filename string) (entity.Item, error) {
	id, err := getStringProp(node, "id")
	if err != nil {
		return entity.Item{}, &ParseError{File: filename, Node: "item", Field: "id", Message: err.Error()}
	}

	item := entity.Item{
		ID:         id,
		TemplateID: id,
	}

	// Parse tags
	if tagsNode := findChild(node, "tags"); tagsNode != nil {
		item.Tags = parseTags(tagsNode)
	}

	// Parse attributes
	if attrsNode := findChild(node, "attributes"); attrsNode != nil {
		attrs, err := parseAttributes(attrsNode, filename)
		if err != nil {
			return entity.Item{}, err
		}
		item.Attributes = attrs
	}

	// Parse triggers
	if triggersNode := findChild(node, "triggers"); triggersNode != nil {
		triggers, err := parseTriggers(triggersNode, filename)
		if err != nil {
			return entity.Item{}, err
		}
		item.Triggers = triggers
	}

	// Parse requirements
	if reqsNode := findChild(node, "requirements"); reqsNode != nil {
		reqs, err := parseRequirements(reqsNode, filename)
		if err != nil {
			return entity.Item{}, err
		}
		item.Requirements = reqs
	}

	// Parse modifiers
	if modsNode := findChild(node, "modifiers"); modsNode != nil {
		mods, err := parseProvidedModifiers(modsNode, filename)
		if err != nil {
			return entity.Item{}, err
		}
		item.ProvidedModifiers = mods
	}

	return item, nil
}
