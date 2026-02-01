package tea

import (
	"maps"
	"strings"

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

func (v Victor) String() string {
	switch v {
	case VictorPlayer:
		return "player"
	case VictorEnemy:
		return "enemy"
	case VictorDraw:
		return "draw"
	default:
		return ""
	}
}

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

	// Copy combat model (TEA immutability)
	combat := m.Combat
	combat.Tick++

	// Decrement all cooldowns (copy map for TEA immutability)
	newCDs := make(map[string]int, len(combat.ItemCooldowns))
	for path, remaining := range combat.ItemCooldowns {
		if remaining > 0 {
			newCDs[path] = remaining - 1
		} else {
			newCDs[path] = 0
		}
	}
	combat.ItemCooldowns = newCDs
	m.Combat = combat

	// Collect all units
	allUnits := getAllUnits(m.Combat)

	// Dispatch on_combat_tick to all units, track which items fire
	var allTriggers []CollectedTrigger
	var firedPaths []string

	for _, unit := range allUnits {
		if !unit.IsAlive() {
			continue
		}
		ctx := event.TriggerContext{
			Event:         core.EventOnCombatTick,
			SourceUnit:    unit,
			AllUnits:      allUnits,
			Tick:          m.Combat.Tick,
			Rolls:         msg.Rolls,
			ItemCooldowns: m.Combat.ItemCooldowns,
		}
		collected := event.Dispatch(ctx)
		for _, ct := range collected {
			allTriggers = append(allTriggers, toMsgCollectedTrigger(ct))
			// Track item paths that fired (for cooldown reset)
			if ct.Owner.ItemID != "" {
				firedPaths = append(firedPaths, event.OwnerPath(ct.Owner))
			}
		}
	}

	// Reset cooldowns for items that fired
	if len(firedPaths) > 0 {
		combat = m.Combat
		resetCDs := make(map[string]int, len(combat.ItemCooldowns))
		for k, v := range combat.ItemCooldowns {
			resetCDs[k] = v
		}
		for _, path := range firedPaths {
			if cd := lookupItemCooldown(allUnits, path); cd > 0 {
				resetCDs[path] = cd
			}
		}
		combat.ItemCooldowns = resetCDs
		m.Combat = combat
	}

	if len(allTriggers) == 0 {
		return m, nil
	}

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
		maps.Copy(unitMap, effectResult.ModifiedUnits)
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

// checkCombatEnd returns the victor if combat has ended, or VictorNone if ongoing.
func (m Model) checkCombatEnd() Victor {
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
		return VictorDraw
	case !enemyAlive:
		return VictorPlayer
	case !playerAlive:
		return VictorEnemy
	default:
		return VictorNone
	}
}

// applyCombatEnd checks for combat end and updates model if combat is over.
// Returns the updated model and a Cmd that emits CombatEnded if combat ended.
func (m Model) applyCombatEnd() (Model, Cmd) {
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	victor := m.checkCombatEnd()
	if victor == VictorNone {
		return m, nil
	}
	combat := m.Combat
	combat.Phase = model.CombatResolved
	combat.Victor = victor.String()
	newLog := make([]string, len(combat.Log), len(combat.Log)+1)
	copy(newLog, combat.Log)
	if victor == VictorDraw {
		combat.Log = append(newLog, "combat ended: draw")
	} else {
		combat.Log = append(newLog, "combat ended: "+victor.String()+" wins")
	}
	m.Combat = combat
	return m, func() Msg { return CombatEnded{Victor: victor} }
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
	newAttrs := maps.Clone(unit.Attributes)

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
		Dice:       unit.Dice,
		Pilot:      unit.Pilot,
		HasPilot:   unit.HasPilot,
	}
}

// lookupItemCooldown finds an item's base cooldown attribute by path
func lookupItemCooldown(units []entity.Unit, path string) int {
	parts := strings.Split(path, "/")
	if len(parts) != 4 {
		return 0
	}
	unitID, partID, mountID, itemID := parts[0], parts[1], parts[2], parts[3]

	for _, unit := range units {
		if unit.ID != unitID {
			continue
		}
		part, ok := unit.Parts[partID]
		if !ok {
			continue
		}
		for _, mount := range part.Mounts {
			if mount.ID != mountID {
				continue
			}
			for _, item := range mount.Contents {
				if item.ID == itemID {
					if cd, ok := item.Attributes["cooldown"]; ok {
						return cd.Base
					}
					return 0
				}
			}
		}
	}
	return 0
}
