package effect

import (
	"wulfaz/internal/core"
	"wulfaz/internal/entity"
)

// EffectResult captures the outcome of executing an effect
type EffectResult struct {
	ModifiedUnits  map[string]entity.Unit // units that changed, keyed by unit ID
	FollowUpEvents []FollowUpEvent        // cascading events to dispatch
	LogEntries     []string               // combat log entries
}

// FollowUpEvent represents a cascading event triggered by an effect
type FollowUpEvent struct {
	Event    core.EventType
	SourceID string
	TargetID string
}

// Merge combines another EffectResult into this one.
// Note: For MVP, last-write-wins for ModifiedUnits if same unit modified twice.
func (r *EffectResult) Merge(other EffectResult) {
	if r.ModifiedUnits == nil {
		r.ModifiedUnits = make(map[string]entity.Unit)
	}
	for k, v := range other.ModifiedUnits {
		r.ModifiedUnits[k] = v
	}
	r.FollowUpEvents = append(r.FollowUpEvents, other.FollowUpEvents...)
	r.LogEntries = append(r.LogEntries, other.LogEntries...)
}
