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

func TestParseUnitDice(t *testing.T) {
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

	dice, err := parseUnitDice(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseUnitDice: %v", err)
	}

	if len(dice) != 1 {
		t.Fatalf("expected 1 die, got %d", len(dice))
	}
	if len(dice[0].Faces) != 3 {
		t.Fatalf("expected 3 faces, got %d", len(dice[0].Faces))
	}
	if dice[0].Faces[0].Type != entity.DieDamage || dice[0].Faces[0].Value != 5 {
		t.Errorf("dice[0].Faces[0] = %+v, want damage/5", dice[0].Faces[0])
	}
	if dice[0].Faces[2].Type != entity.DieBlank {
		t.Errorf("dice[0].Faces[2].Type = %v, want blank", dice[0].Faces[2].Type)
	}
}

func TestParseUnitDice_NoDie(t *testing.T) {
	kdlStr := `unit {
        health 100
    }`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	dice, err := parseUnitDice(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseUnitDice: %v", err)
	}

	if dice != nil {
		t.Error("expected nil dice for unit without die")
	}
}

func TestParseUnitDice_MultipleDice(t *testing.T) {
	kdlStr := `unit {
        die {
            face "damage" 3
            face "damage" 5
            face "blank"
        }
        die {
            face "shield" 2
            face "heal" 1
            face "blank"
        }
    }`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	dice, err := parseUnitDice(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseUnitDice: %v", err)
	}

	if len(dice) != 2 {
		t.Fatalf("expected 2 dice, got %d", len(dice))
	}
	if dice[0].Faces[0].Type != entity.DieDamage {
		t.Errorf("dice[0].Faces[0].Type = %v, want damage", dice[0].Faces[0].Type)
	}
	if dice[1].Faces[0].Type != entity.DieShield {
		t.Errorf("dice[1].Faces[0].Type = %v, want shield", dice[1].Faces[0].Type)
	}
}
