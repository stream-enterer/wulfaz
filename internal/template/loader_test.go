package template

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/sblinch/kdl-go"
)

// Unit tests for parsing helpers (inline KDL strings)

func TestParseTags(t *testing.T) {
	kdlStr := `tags "weapon" "energy" "laser"`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	tags := parseTags(doc.Nodes[0])
	if len(tags) != 3 {
		t.Fatalf("expected 3 tags, got %d", len(tags))
	}
	if tags[0] != "weapon" || tags[1] != "energy" || tags[2] != "laser" {
		t.Errorf("unexpected tags: %v", tags)
	}
}

func TestParseAttributes(t *testing.T) {
	kdlStr := `attributes {
		attribute name="hp" base=100 min=0 max=200
		attribute name="speed" base=5
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	attrs, err := parseAttributes(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseAttributes: %v", err)
	}

	if len(attrs) != 2 {
		t.Fatalf("expected 2 attributes, got %d", len(attrs))
	}

	hp, ok := attrs["hp"]
	if !ok {
		t.Fatal("missing hp attribute")
	}
	if hp.Name != "hp" || hp.Base != 100 || hp.Min != 0 || hp.Max != 200 {
		t.Errorf("unexpected hp attribute: %+v", hp)
	}

	speed, ok := attrs["speed"]
	if !ok {
		t.Fatal("missing speed attribute")
	}
	if speed.Name != "speed" || speed.Base != 5 || speed.Min != 0 || speed.Max != 0 {
		t.Errorf("unexpected speed attribute: %+v", speed)
	}
}

func TestParseUnit(t *testing.T) {
	kdlStr := `unit id="test_mech" {
		tags "mech" "medium"
		attributes {
			attribute name="combat_width" base=2
		}
	}`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	unit, err := parseUnit(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseUnit: %v", err)
	}

	if unit.ID != "test_mech" {
		t.Errorf("expected id=test_mech, got %v", unit.ID)
	}
	if len(unit.Tags) != 2 {
		t.Errorf("expected 2 tags, got %d", len(unit.Tags))
	}
}

// Integration tests

func TestLoadUnitsFromDir(t *testing.T) {
	reg := NewRegistry()
	dir := filepath.Join("..", "..", "data", "templates", "units")
	err := LoadUnitsFromDir(dir, reg)
	if err != nil {
		t.Fatalf("LoadUnitsFromDir: %v", err)
	}

	// Check small_mech loaded
	small, smallOK := reg.GetUnit("small_mech")
	if !smallOK {
		t.Error("small_mech not registered")
	} else {
		if small.ID != "small_mech" {
			t.Errorf("small_mech ID: got %q, want %q", small.ID, "small_mech")
		}
		if cw, cwOK := small.Attributes["combat_width"]; !cwOK || cw.Base != 1 {
			t.Errorf("small_mech combat_width: got %+v", small.Attributes["combat_width"])
		}
	}

	// Check medium_mech loaded
	medium, mediumOK := reg.GetUnit("medium_mech")
	if !mediumOK {
		t.Error("medium_mech not registered")
	} else {
		if medium.ID != "medium_mech" {
			t.Errorf("medium_mech ID: got %q, want %q", medium.ID, "medium_mech")
		}
		if cw, cwOK := medium.Attributes["combat_width"]; !cwOK || cw.Base != 2 {
			t.Errorf("medium_mech combat_width: got %+v", medium.Attributes["combat_width"])
		}
	}

	// Check large_mech loaded
	large, ok := reg.GetUnit("large_mech")
	if !ok {
		t.Error("large_mech not registered")
	} else {
		if large.ID != "large_mech" {
			t.Errorf("large_mech ID: got %q, want %q", large.ID, "large_mech")
		}
		if cw, ok := large.Attributes["combat_width"]; !ok || cw.Base != 3 {
			t.Errorf("large_mech combat_width: got %+v", large.Attributes["combat_width"])
		}
	}
}

func TestLoadUnitsFromDir_EmptyDir(t *testing.T) {
	// Create empty temp directory
	dir, err := os.MkdirTemp("", "empty_units")
	if err != nil {
		t.Fatalf("create temp dir: %v", err)
	}
	defer os.RemoveAll(dir)

	reg := NewRegistry()
	err = LoadUnitsFromDir(dir, reg)
	if err != nil {
		t.Errorf("LoadUnitsFromDir on empty dir: %v", err)
	}
}

func TestLoadUnitsFromDir_InvalidKDL(t *testing.T) {
	// Create temp directory with invalid KDL
	dir, err := os.MkdirTemp("", "invalid_units")
	if err != nil {
		t.Fatalf("create temp dir: %v", err)
	}
	defer os.RemoveAll(dir)

	// Write invalid KDL file
	invalidKDL := `unit id="broken" { this is not valid kdl!!!`
	if writeErr := os.WriteFile(filepath.Join(dir, "broken.kdl"), []byte(invalidKDL), 0644); writeErr != nil {
		t.Fatalf("write invalid KDL: %v", writeErr)
	}

	reg := NewRegistry()
	err = LoadUnitsFromDir(dir, reg)
	if err == nil {
		t.Error("expected error for invalid KDL, got nil")
	}
}

func TestLoadUnitsFromDir_MissingID(t *testing.T) {
	// Create temp directory with KDL missing ID
	dir, err := os.MkdirTemp("", "missing_id_units")
	if err != nil {
		t.Fatalf("create temp dir: %v", err)
	}
	defer os.RemoveAll(dir)

	// Write KDL file with missing ID
	missingIDKDL := `unit {
		tags "mech"
	}`
	if writeErr := os.WriteFile(filepath.Join(dir, "missing.kdl"), []byte(missingIDKDL), 0644); writeErr != nil {
		t.Fatalf("write missing ID KDL: %v", writeErr)
	}

	reg := NewRegistry()
	err = LoadUnitsFromDir(dir, reg)
	if err == nil {
		t.Error("expected error for missing ID, got nil")
	}
}

func TestParseError_Format(t *testing.T) {
	tests := []struct {
		err      ParseError
		expected string
	}{
		{
			ParseError{File: "test.kdl", Node: "unit", Field: "id", Message: "missing"},
			"test.kdl: unit.id: missing",
		},
		{
			ParseError{File: "test.kdl", Node: "unit", Message: "invalid structure"},
			"test.kdl: unit: invalid structure",
		},
		{
			ParseError{File: "test.kdl", Message: "parse error"},
			"test.kdl: parse error",
		},
	}

	for _, tt := range tests {
		result := tt.err.Error()
		if result != tt.expected {
			t.Errorf("ParseError.Error(): got %q, want %q", result, tt.expected)
		}
	}
}
