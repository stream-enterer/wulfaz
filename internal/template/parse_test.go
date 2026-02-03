package template

import (
	"strings"
	"testing"

	kdl "github.com/sblinch/kdl-go"

	"wulfaz/internal/entity"
)

func TestParseDieType(t *testing.T) {
	tests := []struct {
		input   string
		want    entity.DieType
		wantErr bool
	}{
		{"damage", entity.DieDamage, false},
		{"shield", entity.DieShield, false},
		{"heal", entity.DieHeal, false},
		{"blank", entity.DieBlank, false},
		{"unknown", "", true},
	}
	for _, tt := range tests {
		got, err := parseDieType(tt.input)
		if (err != nil) != tt.wantErr {
			t.Errorf("parseDieType(%q) error = %v, wantErr %v", tt.input, err, tt.wantErr)
		}
		if got != tt.want {
			t.Errorf("parseDieType(%q) = %v, want %v", tt.input, got, tt.want)
		}
	}
}

func TestParseUnitDie(t *testing.T) {
	kdlStr := `unit {
        die {
            face "damage" 5
            face "damage" 8
            face "blank"
        }
    }`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	die, hasDie, err := parseUnitDie(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseUnitDie: %v", err)
	}

	if !hasDie {
		t.Fatal("expected hasDie=true")
	}
	if len(die.Faces) != 3 {
		t.Fatalf("expected 3 faces, got %d", len(die.Faces))
	}
	if die.Faces[0].Type != entity.DieDamage || die.Faces[0].Value != 5 {
		t.Errorf("die.Faces[0] = %+v, want damage/5", die.Faces[0])
	}
	if die.Faces[2].Type != entity.DieBlank {
		t.Errorf("die.Faces[2].Type = %v, want blank", die.Faces[2].Type)
	}
}

func TestParseUnitDie_NoDie(t *testing.T) {
	kdlStr := `unit {
        health 100
    }`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	_, hasDie, err := parseUnitDie(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseUnitDie: %v", err)
	}

	if hasDie {
		t.Error("expected hasDie=false for unit without die")
	}
}
