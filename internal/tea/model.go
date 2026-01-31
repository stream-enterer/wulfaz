package tea

import (
	"wulfaz/internal/core"
	"wulfaz/internal/effect"
	"wulfaz/internal/entity"
	"wulfaz/internal/event"
	"wulfaz/internal/model"
)

type GamePhase int

const (
	PhaseMenu GamePhase = iota
	PhaseCombat
	PhaseChoice // reward or fight selection
	PhaseGameOver
)

type ChoiceType int

const (
	ChoiceReward ChoiceType = iota
	ChoiceFight
)

type Victor int

const (
	VictorNone Victor = iota
	VictorPlayer
	VictorEnemy
	VictorDraw
)

type Model struct {
	Version int
	Phase   GamePhase
	Combat  model.CombatModel
	Seed    int64
	// Choice phase state
	ChoiceType        ChoiceType
	RewardChoicesLeft int
	Choices           []string
	// Run progression
	FightNumber int
}

func (m Model) Update(msg Msg) (Model, Cmd) {
	switch msg := msg.(type) {
	case PlayerQuit:
		m.Phase = PhaseGameOver
		return m, nil

	case CombatEnded:
		if msg.Victor == VictorPlayer {
			m.Phase = PhaseChoice
			m.ChoiceType = ChoiceReward
			m.RewardChoicesLeft = 2
			m.Choices = []string{"Reward A", "Reward B", "Reward C"}
		} else {
			m.Phase = PhaseGameOver
		}
		return m, nil

	case ChoiceSelected:
		if m.ChoiceType == ChoiceReward {
			m.RewardChoicesLeft--
			if m.RewardChoicesLeft > 0 {
				m.Choices = []string{"Reward D", "Reward E", "Reward F"}
			} else {
				m.ChoiceType = ChoiceFight
				m.Choices = []string{"Fight: Easy", "Fight: Medium", "Fight: Hard"}
			}
		}
		return m, nil

	case CombatStarted:
		m.Phase = PhaseCombat
		m.FightNumber++
		m.Combat = msg.Combat
		return m, nil

	case PlayerPaused:
		if m.Phase != PhaseCombat {
			return m, nil
		}
		combat := m.Combat
		combat.Phase = model.CombatPaused
		m.Combat = combat
		return m, nil

	case PlayerResumed:
		if m.Phase != PhaseCombat {
			return m, nil
		}
		combat := m.Combat
		combat.Phase = model.CombatActive
		m.Combat = combat
		return m, nil

	case CombatTicked:
		return m.handleCombatTicked(msg)

	case TriggersCollected:
		return m.handleTriggersCollected(msg)

	case EffectsResolved:
		return m.handleEffectsResolved(msg)

	default:
		return m, nil
	}
}

// handleCombatTicked dispatches on_combat_tick to all units and collects triggers
func (m Model) handleCombatTicked(msg CombatTicked) (Model, Cmd) {
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}

	// Increment tick
	combat := m.Combat
	combat.Tick++
	m.Combat = combat

	// Collect all units
	allUnits := getAllUnits(m.Combat)

	// Dispatch on_combat_tick to all units
	var allTriggers []CollectedTrigger

	for _, unit := range allUnits {
		if !unit.IsAlive() {
			continue
		}
		ctx := event.TriggerContext{
			Event:      core.EventOnCombatTick,
			SourceUnit: unit,
			AllUnits:   allUnits,
			Tick:       m.Combat.Tick,
			Rolls:      msg.Rolls,
		}
		collected := event.Dispatch(ctx)
		for _, ct := range collected {
			allTriggers = append(allTriggers, toMsgCollectedTrigger(ct))
		}
	}

	if len(allTriggers) == 0 {
		return m, nil
	}

	// Return command that yields TriggersCollected
	return m, buildTriggersCollectedCmd(string(core.EventOnCombatTick), allTriggers, msg.Rolls, 0)
}

// handleTriggersCollected executes effects for collected triggers
func (m Model) handleTriggersCollected(msg TriggersCollected) (Model, Cmd) {
	// Check cascade depth limit
	if msg.Depth >= core.MaxCascadeDepth {
		combat := m.Combat
		combat.Log = append(combat.Log, "cascade depth limit reached")
		m.Combat = combat
		return m, nil
	}

	if len(msg.Triggers) == 0 {
		return m, nil
	}

	// Build unit map and player IDs set for effect context
	unitMap := buildUnitMap(m.Combat)
	playerIDs := buildPlayerUnitIDs(m.Combat)

	// Execute each trigger's effect
	var result effect.EffectResult
	result.ModifiedUnits = make(map[string]entity.Unit)

	for _, trigger := range msg.Triggers {
		// Find source unit
		sourceUnit, ok := unitMap[trigger.Owner.UnitID]
		if !ok {
			continue
		}

		// Re-evaluate source conditions with current unit state
		// (unit may have died since trigger was collected)
		if len(trigger.Conditions) > 0 {
			if !core.EvaluateConditions(trigger.Conditions, sourceUnit.Tags, sourceUnit.Attributes) {
				continue
			}
		}

		// Build effect context
		ctx := effect.EffectContext{
			Owner:            toEventTriggerOwner(trigger.Owner),
			SourceUnit:       sourceUnit,
			AllUnits:         unitMap,
			PlayerUnitIDs:    playerIDs,
			Rolls:            msg.Rolls,
			TargetConditions: trigger.TargetConditions,
		}

		// Execute effect
		effectResult := effect.Handle(trigger.EffectName, trigger.Params, ctx)

		// Merge results
		result.Merge(effectResult)

		// Update unit map with modified units for subsequent effects
		for id, unit := range effectResult.ModifiedUnits {
			unitMap[id] = unit
		}
	}

	// Convert modified units to serializable format
	modifiedUnits := make(ModifiedUnitsMap)
	for id, unit := range result.ModifiedUnits {
		attrs := make(map[string]AttributeValue)
		for name, attr := range unit.Attributes {
			attrs[name] = AttributeValue{
				Base: attr.Base,
				Min:  attr.Min,
				Max:  attr.Max,
			}
		}
		modifiedUnits[id] = ModifiedUnit{Attributes: attrs}
	}

	var followUps []FollowUpEvent
	for _, fe := range result.FollowUpEvents {
		followUps = append(followUps, FollowUpEvent{
			Event:    string(fe.Event),
			SourceID: fe.SourceID,
			TargetID: fe.TargetID,
		})
	}

	return m, buildEffectsResolvedCmd(modifiedUnits, followUps, result.LogEntries, msg.Depth)
}

// handleEffectsResolved applies modifications and dispatches follow-up events
func (m Model) handleEffectsResolved(msg EffectsResolved) (Model, Cmd) {
	combat := m.Combat

	// Copy log slice before appending (TEA immutability)
	newLog := make([]string, len(combat.Log), len(combat.Log)+len(msg.LogEntries))
	copy(newLog, combat.Log)
	combat.Log = append(newLog, msg.LogEntries...)

	// Copy unit slices before modification (TEA immutability)
	if len(msg.ModifiedUnits) > 0 {
		combat.PlayerUnits = copyUnitSlice(combat.PlayerUnits)
		combat.EnemyUnits = copyUnitSlice(combat.EnemyUnits)

		// Apply modified units to combat model
		for unitID, mods := range msg.ModifiedUnits {
			// Check player units
			for i, unit := range combat.PlayerUnits {
				if unit.ID == unitID {
					combat.PlayerUnits[i] = applyModifications(unit, mods)
					break
				}
			}
			// Check enemy units
			for i, unit := range combat.EnemyUnits {
				if unit.ID == unitID {
					combat.EnemyUnits[i] = applyModifications(unit, mods)
					break
				}
			}
		}
	}

	m.Combat = combat

	// Check for follow-up events
	if len(msg.FollowUpEvents) == 0 {
		return m.applyCombatEnd()
	}

	// Dispatch follow-up events
	allUnits := getAllUnits(m.Combat)
	var allTriggers []CollectedTrigger

	for _, fe := range msg.FollowUpEvents {
		eventType := core.EventType(fe.Event)

		// Find target unit as source for the triggered event
		var sourceUnit entity.Unit
		for _, u := range allUnits {
			if u.ID == fe.TargetID {
				sourceUnit = u
				break
			}
		}
		if sourceUnit.ID == "" {
			continue
		}

		ctx := event.TriggerContext{
			Event:      eventType,
			SourceUnit: sourceUnit,
			AllUnits:   allUnits,
			Tick:       m.Combat.Tick,
		}
		collected := event.Dispatch(ctx)
		for _, ct := range collected {
			allTriggers = append(allTriggers, toMsgCollectedTrigger(ct))
		}
	}

	if len(allTriggers) == 0 {
		return m.applyCombatEnd()
	}

	// Return command for cascade
	return m, buildTriggersCollectedCmd(string(core.EventOnCascade), allTriggers, nil, msg.Depth+1)
}

// checkCombatEnd returns the victor if combat has ended, or "" if ongoing.
func (m Model) checkCombatEnd() string {
	playerAlive := false
	for _, u := range m.Combat.PlayerUnits {
		if u.IsAlive() {
			playerAlive = true
			break
		}
	}
	enemyAlive := false
	for _, u := range m.Combat.EnemyUnits {
		if u.IsAlive() {
			enemyAlive = true
			break
		}
	}

	switch {
	case !playerAlive && !enemyAlive:
		return "draw"
	case !enemyAlive:
		return "player"
	case !playerAlive:
		return "enemy"
	default:
		return ""
	}
}

// applyCombatEnd checks for combat end and updates model if combat is over.
// Returns the updated model and a Cmd that emits CombatEnded if combat ended.
func (m Model) applyCombatEnd() (Model, Cmd) {
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	victor := m.checkCombatEnd()
	if victor == "" {
		return m, nil
	}
	combat := m.Combat
	combat.Phase = model.CombatResolved
	combat.Victor = victor
	newLog := make([]string, len(combat.Log), len(combat.Log)+1)
	copy(newLog, combat.Log)
	if victor == "draw" {
		combat.Log = append(newLog, "combat ended: draw")
	} else {
		combat.Log = append(newLog, "combat ended: "+victor+" wins")
	}
	m.Combat = combat
	v := victorFromString(victor)
	return m, func() Msg { return CombatEnded{Victor: v} }
}

// victorFromString converts the legacy string victor to the Victor enum.
func victorFromString(s string) Victor {
	switch s {
	case "player":
		return VictorPlayer
	case "enemy":
		return VictorEnemy
	case "draw":
		return VictorDraw
	default:
		return VictorNone
	}
}

// Helper functions

func getAllUnits(combat model.CombatModel) []entity.Unit {
	all := make([]entity.Unit, 0, len(combat.PlayerUnits)+len(combat.EnemyUnits))
	all = append(all, combat.PlayerUnits...)
	all = append(all, combat.EnemyUnits...)
	return all
}

func buildUnitMap(combat model.CombatModel) map[string]entity.Unit {
	m := make(map[string]entity.Unit)
	for _, u := range combat.PlayerUnits {
		m[u.ID] = u
	}
	for _, u := range combat.EnemyUnits {
		m[u.ID] = u
	}
	return m
}

func buildPlayerUnitIDs(combat model.CombatModel) map[string]bool {
	ids := make(map[string]bool)
	for _, u := range combat.PlayerUnits {
		ids[u.ID] = true
	}
	return ids
}

func toMsgCollectedTrigger(ct event.CollectedTrigger) CollectedTrigger {
	return CollectedTrigger{
		EffectName: ct.Trigger.EffectName,
		Params:     ct.Trigger.Params,
		Priority:   ct.Trigger.Priority,
		Owner: TriggerOwner{
			UnitID:  ct.Owner.UnitID,
			PartID:  ct.Owner.PartID,
			MountID: ct.Owner.MountID,
			ItemID:  ct.Owner.ItemID,
		},
		Conditions:       ct.Trigger.Conditions,
		TargetConditions: ct.Trigger.TargetConditions,
	}
}

func toEventTriggerOwner(to TriggerOwner) event.TriggerOwner {
	return event.TriggerOwner{
		UnitID:  to.UnitID,
		PartID:  to.PartID,
		MountID: to.MountID,
		ItemID:  to.ItemID,
	}
}

func buildTriggersCollectedCmd(eventType string, triggers []CollectedTrigger, rolls []int, depth int) Cmd {
	return func() Msg {
		return TriggersCollected{
			Event:    eventType,
			Triggers: triggers,
			Rolls:    rolls,
			Depth:    depth,
		}
	}
}

func buildEffectsResolvedCmd(modifiedUnits ModifiedUnitsMap, followUps []FollowUpEvent, logEntries []string, depth int) Cmd {
	return func() Msg {
		return EffectsResolved{
			ModifiedUnits:  modifiedUnits,
			FollowUpEvents: followUps,
			LogEntries:     logEntries,
			Depth:          depth,
		}
	}
}

func copyUnitSlice(units []entity.Unit) []entity.Unit {
	if units == nil {
		return nil
	}
	copied := make([]entity.Unit, len(units))
	copy(copied, units)
	return copied
}

// applyModifications applies serialized modifications to a unit
func applyModifications(unit entity.Unit, mods ModifiedUnit) entity.Unit {
	// Copy attributes map
	newAttrs := make(map[string]core.Attribute)
	for k, v := range unit.Attributes {
		newAttrs[k] = v
	}

	// Apply attribute modifications
	for name, attrVal := range mods.Attributes {
		newAttrs[name] = core.Attribute{
			Name: name,
			Base: attrVal.Base,
			Min:  attrVal.Min,
			Max:  attrVal.Max,
		}
	}

	// Return new unit with updated attributes
	return entity.Unit{
		ID:         unit.ID,
		TemplateID: unit.TemplateID,
		Tags:       unit.Tags,
		Attributes: newAttrs,
		Parts:      unit.Parts,
		Triggers:   unit.Triggers,
		Abilities:  unit.Abilities,
		Pilot:      unit.Pilot,
		HasPilot:   unit.HasPilot,
	}
}
