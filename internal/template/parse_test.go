package template

import (
	"strings"
	"testing"

	kdl "github.com/sblinch/kdl-go"

	"wulfaz/internal/entity"
)

// slicesEqual compares two int slices for equality (test helper).
func slicesEqual(a, b []int) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

func TestParseDieType(t *testing.T) {
	tests := []struct {
		input   string
		want    entity.DieType
		wantErr bool
	}{
		{"damage", entity.DieDamage, false},
		{"shield", entity.DieShield, false},
		{"heal", entity.DieHeal, false},
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

func TestParseFaces(t *testing.T) {
	tests := []struct {
		input   string
		want    []int
		wantErr bool
	}{
		{"2,2,3,4,0,0", []int{2, 2, 3, 4, 0, 0}, false},
		{"5, 5, 8, 12, 0, 0", []int{5, 5, 8, 12, 0, 0}, false}, // spaces
		{"1", []int{1}, false},
		{"abc", nil, true},
	}
	for _, tt := range tests {
		got, err := parseFaces(tt.input)
		if (err != nil) != tt.wantErr {
			t.Errorf("parseFaces(%q) error = %v, wantErr %v", tt.input, err, tt.wantErr)
		}
		if !tt.wantErr && !slicesEqual(got, tt.want) {
			t.Errorf("parseFaces(%q) = %v, want %v", tt.input, got, tt.want)
		}
	}
}

func TestParseDice(t *testing.T) {
	kdlStr := `dice {
        die type="damage" faces="2,2,3,4,0,0"
        die type="shield" faces="5,5,8,12,0,0"
    }`
	doc, err := kdl.Parse(strings.NewReader(kdlStr))
	if err != nil {
		t.Fatalf("parse KDL: %v", err)
	}

	dice, err := parseDice(doc.Nodes[0], "test.kdl")
	if err != nil {
		t.Fatalf("parseDice: %v", err)
	}

	if len(dice) != 2 {
		t.Fatalf("expected 2 dice, got %d", len(dice))
	}
	if dice[0].Type != entity.DieDamage {
		t.Errorf("dice[0].Type = %v, want damage", dice[0].Type)
	}
	if dice[1].Type != entity.DieShield {
		t.Errorf("dice[1].Type = %v, want shield", dice[1].Type)
	}
}
