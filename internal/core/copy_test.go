package core

import "testing"

func TestCopyTags_Nil(t *testing.T) {
	result := CopyTags(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyTags_Independence(t *testing.T) {
	orig := []Tag{"a", "b", "c"}
	copied := CopyTags(orig)

	copied[0] = "modified"
	if orig[0] != "a" {
		t.Error("original was mutated")
	}
}

func TestCopyAttributes_Nil(t *testing.T) {
	result := CopyAttributes(nil)
	if result != nil {
		t.Error("expected nil for nil input")
	}
}

func TestCopyAttributes_Independence(t *testing.T) {
	orig := map[string]Attribute{
		"health": {Name: "health", Base: 100, Min: 0, Max: 200},
	}
	copied := CopyAttributes(orig)

	copied["health"] = Attribute{Name: "health", Base: 999}

	if orig["health"].Base != 100 {
		t.Error("original was mutated")
	}
}
