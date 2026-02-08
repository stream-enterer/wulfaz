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

// parseDieType validates and converts a string to DieType.
func parseDieType(s string) (entity.DieType, error) {
	switch s {
	case "damage":
		return entity.DieDamage, nil
	case "shield":
		return entity.DieShield, nil
	case "heal":
		return entity.DieHeal, nil
	case "blank":
		return entity.DieBlank, nil
	default:
		return "", fmt.Errorf("unknown die type: %q", s)
	}
}

// toInt converts various numeric types to int.
func toInt(v any) (int, error) {
	switch val := v.(type) {
	case int64:
		return int(val), nil
	case int:
		return val, nil
	case float64:
		return int(val), nil
	}
	return 0, fmt.Errorf("not an integer")
}

// parseDieFace parses a single "face" node.
func parseDieFace(node *document.Node, filename string) (entity.DieFace, error) {
	args := node.Arguments
	if len(args) == 0 {
		return entity.DieFace{}, &ParseError{File: filename, Node: "face", Message: "missing type argument"}
	}
	typeStr, ok := args[0].Value.(string)
	if !ok {
		return entity.DieFace{}, &ParseError{File: filename, Node: "face", Message: "type must be string"}
	}
	dieType, err := parseDieType(typeStr)
	if err != nil {
		return entity.DieFace{}, &ParseError{File: filename, Node: "face", Field: "type", Message: err.Error()}
	}

	// Blank faces have no value
	if dieType == entity.DieBlank {
		return entity.DieFace{Type: entity.DieBlank, Value: 0}, nil
	}

	// Non-blank faces require a value
	if len(args) < 2 {
		return entity.DieFace{}, &ParseError{File: filename, Node: "face", Message: "missing value argument"}
	}
	value, err := toInt(args[1].Value)
	if err != nil {
		return entity.DieFace{}, &ParseError{File: filename, Node: "face", Field: "value", Message: "must be integer"}
	}
	return entity.DieFace{Type: dieType, Value: value}, nil
}

// parseDie parses a single "die" node with child "face" nodes.
func parseDie(node *document.Node, filename string) (entity.Die, error) {
	var faces []entity.DieFace
	for _, faceNode := range findChildren(node, "face") {
		face, err := parseDieFace(faceNode, filename)
		if err != nil {
			return entity.Die{}, err
		}
		faces = append(faces, face)
	}
	if len(faces) == 0 {
		return entity.Die{}, &ParseError{File: filename, Node: "die", Message: "die must have at least one face"}
	}
	return entity.Die{Faces: faces}, nil
}

// parseUnitDice parses a "dice" block containing one or more die nodes.
func parseUnitDice(node *document.Node, filename string) ([]entity.Die, error) {
	dieNodes := findChildren(node, "die")
	if len(dieNodes) == 0 {
		return nil, nil
	}
	dice := make([]entity.Die, 0, len(dieNodes))
	for _, dn := range dieNodes {
		die, err := parseDie(dn, filename)
		if err != nil {
			return nil, err
		}
		dice = append(dice, die)
	}
	return dice, nil
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

// Entity parsers

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

	// Parse dice (one or more die per unit)
	if diceNode := findChild(node, "dice"); diceNode != nil {
		dice, err := parseUnitDice(diceNode, filename)
		if err != nil {
			return entity.Unit{}, err
		}
		unit.Dice = dice
	}

	return unit, nil
}
