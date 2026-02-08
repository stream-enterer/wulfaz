package core

import "maps"

// CopyTags copies a tag slice.
func CopyTags(tags []Tag) []Tag {
	if tags == nil {
		return nil
	}
	result := make([]Tag, len(tags))
	copy(result, tags)
	return result
}

// CopyAttributes copies an attribute map (Attribute is pure value).
func CopyAttributes(attrs map[string]Attribute) map[string]Attribute {
	return maps.Clone(attrs)
}
