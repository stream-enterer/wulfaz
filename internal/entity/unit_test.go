package entity

import (
	"testing"

	"wulfaz/internal/core"
)

func TestUnit_IsCommand(t *testing.T) {
	tests := []struct {
		name string
		tags []core.Tag
		want bool
	}{
		{"no tags", nil, false},
		{"empty tags", []core.Tag{}, false},
		{"other tags", []core.Tag{"mech", "heavy"}, false},
		{"command tag", []core.Tag{"command"}, true},
		{"command with others", []core.Tag{"mech", "command", "large"}, true},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			u := Unit{Tags: tt.tags}
			if got := u.IsCommand(); got != tt.want {
				t.Errorf("IsCommand() = %v, want %v", got, tt.want)
			}
		})
	}
}
