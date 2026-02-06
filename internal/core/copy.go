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

// CopyCondition copies a Condition (has Params map).
func CopyCondition(c Condition) Condition {
	return Condition{
		Type:   c.Type,
		Params: maps.Clone(c.Params),
	}
}

// CopyConditions copies a slice of Conditions.
func CopyConditions(cs []Condition) []Condition {
	if cs == nil {
		return nil
	}
	result := make([]Condition, len(cs))
	for i, c := range cs {
		result[i] = CopyCondition(c)
	}
	return result
}

// CopyTrigger copies a Trigger (has Conditions slice + TargetConditions slice + Params map).
func CopyTrigger(t Trigger) Trigger {
	return Trigger{
		Event:            t.Event,
		Conditions:       CopyConditions(t.Conditions),
		TargetConditions: CopyConditions(t.TargetConditions),
		EffectName:       t.EffectName,
		Params:           maps.Clone(t.Params),
		Priority:         t.Priority,
	}
}

// CopyTriggers copies a slice of Triggers.
func CopyTriggers(ts []Trigger) []Trigger {
	if ts == nil {
		return nil
	}
	result := make([]Trigger, len(ts))
	for i, t := range ts {
		result[i] = CopyTrigger(t)
	}
	return result
}

// CopyEffectRef copies an EffectRef (has Params map + Conditions slice).
func CopyEffectRef(e EffectRef) EffectRef {
	return EffectRef{
		EffectName: e.EffectName,
		Params:     maps.Clone(e.Params),
		Delay:      e.Delay,
		Conditions: CopyConditions(e.Conditions),
	}
}

// CopyEffectRefs copies a slice of EffectRefs.
func CopyEffectRefs(es []EffectRef) []EffectRef {
	if es == nil {
		return nil
	}
	result := make([]EffectRef, len(es))
	for i, e := range es {
		result[i] = CopyEffectRef(e)
	}
	return result
}

// CopyCost copies a Cost (ValueRef is pure value, no deep copy needed).
func CopyCost(c Cost) Cost {
	return c
}

// CopyCosts copies a slice of Costs.
func CopyCosts(cs []Cost) []Cost {
	if cs == nil {
		return nil
	}
	result := make([]Cost, len(cs))
	copy(result, cs)
	return result
}

// CopyTargeting copies Targeting (has Filter []Tag).
func CopyTargeting(t Targeting) Targeting {
	return Targeting{
		Type:   t.Type,
		Range:  t.Range,
		Count:  t.Count,
		Filter: CopyTags(t.Filter),
	}
}

// CopyAbility copies an Ability (has Tags, Conditions, Costs, Effects, Targeting).
func CopyAbility(a Ability) Ability {
	return Ability{
		ID:                 a.ID,
		Tags:               CopyTags(a.Tags),
		Conditions:         CopyConditions(a.Conditions),
		Costs:              CopyCosts(a.Costs),
		Targeting:          CopyTargeting(a.Targeting),
		Effects:            CopyEffectRefs(a.Effects),
		Cooldown:           a.Cooldown,
		Charges:            a.Charges,
		ChargeRestoreEvent: a.ChargeRestoreEvent,
	}
}

// CopyAbilities copies a slice of Abilities.
func CopyAbilities(as []Ability) []Ability {
	if as == nil {
		return nil
	}
	result := make([]Ability, len(as))
	for i, a := range as {
		result[i] = CopyAbility(a)
	}
	return result
}

// CopyProvidedModifier copies a ProvidedModifier (has ScopeFilter + Conditions).
func CopyProvidedModifier(pm ProvidedModifier) ProvidedModifier {
	return ProvidedModifier{
		Scope:       pm.Scope,
		ScopeFilter: CopyTags(pm.ScopeFilter),
		Attribute:   pm.Attribute,
		Operation:   pm.Operation,
		Value:       pm.Value,
		StackGroup:  pm.StackGroup,
		Conditions:  CopyConditions(pm.Conditions),
	}
}

// CopyProvidedModifiers copies a slice of ProvidedModifiers.
func CopyProvidedModifiers(pms []ProvidedModifier) []ProvidedModifier {
	if pms == nil {
		return nil
	}
	result := make([]ProvidedModifier, len(pms))
	for i, pm := range pms {
		result[i] = CopyProvidedModifier(pm)
	}
	return result
}

// CopyRequirement copies a Requirement (has Condition).
func CopyRequirement(r Requirement) Requirement {
	return Requirement{
		Scope:     r.Scope,
		Condition: CopyCondition(r.Condition),
		OnUnmet:   r.OnUnmet,
	}
}

// CopyRequirements copies a slice of Requirements.
func CopyRequirements(rs []Requirement) []Requirement {
	if rs == nil {
		return nil
	}
	result := make([]Requirement, len(rs))
	for i, r := range rs {
		result[i] = CopyRequirement(r)
	}
	return result
}

// CopyAttributes copies an attribute map (Attribute is pure value).
func CopyAttributes(attrs map[string]Attribute) map[string]Attribute {
	return maps.Clone(attrs)
}
