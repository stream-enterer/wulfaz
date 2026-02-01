package tea

import (
	"fmt"
	"maps"

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

	case TriggersCollected:
		return m.handleTriggersCollected(msg)

	case EffectsResolved:
		return m.handleEffectsResolved(msg)

	case RoundStarted:
		return m.handleRoundStarted(msg)
	case PreviewDone:
		return m.handlePreviewDone(msg)
	case DieLockToggled:
		return m.handleDieLockToggled(msg)
	case RerollRequested:
		return m.handleRerollRequested(msg)
	case DieSelected:
		return m.handleDieSelected(msg)
	case DieDeselected:
		return m.handleDieDeselected(msg)
	case DiceActivated:
		return m.handleDiceActivated(msg)
	case DiceEffectApplied:
		return m.handleDiceEffectApplied(msg)
	case PlayerCommandDone:
		return m.handlePlayerCommandDone(msg)
	case DicePhaseAdvanced:
		return m.handleDicePhaseAdvanced(msg)

	default:
		return m, nil
	}
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

// ===== Dice Phase Helpers (Wave 2) =====

// isPlayerUnit checks if unit ID belongs to player side.
func isPlayerUnit(combat model.CombatModel, unitID string) bool {
	for _, u := range combat.PlayerUnits {
		if u.ID == unitID {
			return true
		}
	}
	return false
}

// isEnemyUnit checks if unit ID belongs to enemy side.
func isEnemyUnit(combat model.CombatModel, unitID string) bool {
	for _, u := range combat.EnemyUnits {
		if u.ID == unitID {
			return true
		}
	}
	return false
}

// findPlayerCommandUnit returns the player's command unit (or nil).
func findPlayerCommandUnit(combat model.CombatModel) *entity.Unit {
	for i := range combat.PlayerUnits {
		if combat.PlayerUnits[i].IsCommand() {
			return &combat.PlayerUnits[i]
		}
	}
	return nil
}

// allCommandDiceActivated checks if all player command dice are activated.
func allCommandDiceActivated(combat model.CombatModel) bool {
	cmd := findPlayerCommandUnit(combat)
	if cmd == nil {
		return true
	}
	activated := combat.ActivatedDice[cmd.ID]
	if activated == nil {
		return false
	}
	for _, a := range activated {
		if !a {
			return false
		}
	}
	return true
}

// ===== Dice Phase Handlers (Wave 2) =====

func (m Model) handleRoundStarted(msg RoundStarted) (Model, Cmd) {
	combat := m.Combat
	combat.Round = msg.Round
	combat.DicePhase = model.DicePhasePreview
	combat.RerollsRemaining = model.DefaultRerollsPerRound
	combat.RolledDice = make(map[string][]entity.RolledDie)
	combat.ActivatedDice = make(map[string][]bool)
	combat.SelectedUnitID = ""
	combat.SelectedDieIndex = -1

	// Convert roll indices to RolledDie for all units
	allUnits := getAllUnits(combat)
	for _, unit := range allUnits {
		rolls, ok := msg.UnitRolls[unit.ID]
		if !ok || len(unit.Dice) == 0 {
			continue
		}
		rolled := make([]entity.RolledDie, len(unit.Dice))
		for i, die := range unit.Dice {
			faceIdx := 0
			if i < len(rolls) {
				faceIdx = rolls[i]
			}
			result := 0
			if faceIdx < len(die.Faces) {
				result = die.Faces[faceIdx]
			}
			rolled[i] = entity.RolledDie{
				Type:      die.Type,
				Faces:     die.Faces, // Share reference (immutable template data)
				FaceIndex: faceIdx,
				Result:    result,
				Locked:    false,
			}
		}
		combat.RolledDice[unit.ID] = rolled
		combat.ActivatedDice[unit.ID] = make([]bool, len(unit.Dice))
	}

	m.Combat = combat
	return m, nil
}

func (m Model) handlePreviewDone(_ PreviewDone) (Model, Cmd) {
	if m.Combat.DicePhase != model.DicePhasePreview {
		return m, nil
	}
	combat := m.Combat
	combat.DicePhase = model.DicePhasePlayerCommand
	m.Combat = combat
	return m, nil
}

func (m Model) handleDieLockToggled(msg DieLockToggled) (Model, Cmd) {
	// Only in PlayerCommand phase
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	// Only for player's command unit
	cmd := findPlayerCommandUnit(m.Combat)
	if cmd == nil || msg.UnitID != cmd.ID {
		return m, nil
	}

	combat := m.Combat
	combat.RolledDice = entity.CopyRolledDiceMap(combat.RolledDice)

	if dice, ok := combat.RolledDice[msg.UnitID]; ok && msg.DieIndex < len(dice) {
		dice[msg.DieIndex].Locked = !dice[msg.DieIndex].Locked
	}

	m.Combat = combat
	return m, nil
}

func (m Model) handleRerollRequested(msg RerollRequested) (Model, Cmd) {
	// Only in PlayerCommand phase with rerolls remaining
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	if m.Combat.RerollsRemaining <= 0 {
		return m, nil
	}
	// Only for player's command unit
	cmd := findPlayerCommandUnit(m.Combat)
	if cmd == nil || msg.UnitID != cmd.ID {
		return m, nil
	}

	combat := m.Combat
	combat.RerollsRemaining--
	combat.RolledDice = entity.CopyRolledDiceMap(combat.RolledDice)

	dice := combat.RolledDice[msg.UnitID]
	for i := range dice {
		if !dice[i].Locked && i < len(msg.Results) {
			faceIdx := msg.Results[i]
			dice[i].FaceIndex = faceIdx
			if faceIdx < len(dice[i].Faces) {
				dice[i].Result = dice[i].Faces[faceIdx]
			}
		}
	}

	// Auto-lock all dice if no rerolls remaining
	if combat.RerollsRemaining == 0 {
		for i := range dice {
			dice[i].Locked = true
		}
	}

	m.Combat = combat
	return m, nil
}

func (m Model) handleDieSelected(msg DieSelected) (Model, Cmd) {
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	// Only player's command unit dice can be selected
	cmd := findPlayerCommandUnit(m.Combat)
	if cmd == nil || msg.UnitID != cmd.ID {
		return m, nil
	}
	// Validate die index is within bounds
	rolled := m.Combat.RolledDice[msg.UnitID]
	if msg.DieIndex < 0 || msg.DieIndex >= len(rolled) {
		return m, nil
	}
	// Check die not already activated
	activated := m.Combat.ActivatedDice[msg.UnitID]
	if activated != nil && msg.DieIndex < len(activated) && activated[msg.DieIndex] {
		return m, nil
	}

	combat := m.Combat
	combat.SelectedUnitID = msg.UnitID
	combat.SelectedDieIndex = msg.DieIndex
	m.Combat = combat
	return m, nil
}

func (m Model) handleDieDeselected(_ DieDeselected) (Model, Cmd) {
	combat := m.Combat
	combat.SelectedUnitID = ""
	combat.SelectedDieIndex = -1
	m.Combat = combat
	return m, nil
}

func (m Model) handleDiceActivated(msg DiceActivated) (Model, Cmd) {
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	// Validate source is selected die
	if msg.SourceUnitID != m.Combat.SelectedUnitID || msg.DieIndex != m.Combat.SelectedDieIndex {
		return m, nil
	}

	rolled := m.Combat.RolledDice[msg.SourceUnitID]
	if msg.DieIndex >= len(rolled) {
		return m, nil
	}
	die := rolled[msg.DieIndex]

	// Targeting validation per DESIGN.md:
	// - damage: enemy only
	// - shield/heal: friendly only
	targetIsEnemy := isEnemyUnit(m.Combat, msg.TargetUnitID)
	targetIsPlayer := isPlayerUnit(m.Combat, msg.TargetUnitID)

	if die.Type == entity.DieDamage && !targetIsEnemy {
		return m, nil // Invalid: damage must target enemy
	}
	if (die.Type == entity.DieShield || die.Type == entity.DieHeal) && !targetIsPlayer {
		return m, nil // Invalid: shield/heal must target friendly
	}

	// Mark die as activated, clear selection
	combat := m.Combat
	combat.ActivatedDice = entity.CopyActivatedMap(combat.ActivatedDice)
	if combat.ActivatedDice[msg.SourceUnitID] == nil {
		combat.ActivatedDice[msg.SourceUnitID] = make([]bool, len(rolled))
	}
	combat.ActivatedDice[msg.SourceUnitID][msg.DieIndex] = true
	combat.SelectedUnitID = ""
	combat.SelectedDieIndex = -1
	m.Combat = combat

	// Return Cmd to apply effect
	return m, ApplyDiceEffect(msg.SourceUnitID, msg.TargetUnitID, die.Type, die.Result, m.Combat)
}

func (m Model) handleDiceEffectApplied(msg DiceEffectApplied) (Model, Cmd) {
	combat := m.Combat

	// Copy unit slices before modification
	combat.PlayerUnits = copyUnitSlice(combat.PlayerUnits)
	combat.EnemyUnits = copyUnitSlice(combat.EnemyUnits)

	// Update target unit's health/shields
	updateUnit := func(units []entity.Unit) {
		for i, u := range units {
			if u.ID != msg.TargetUnitID {
				continue
			}
			attrs := core.CopyAttributes(u.Attributes)

			if msg.Effect == entity.DieDamage || msg.Effect == entity.DieHeal {
				if h, ok := attrs["health"]; ok {
					h.Base = msg.NewHealth
					attrs["health"] = h
				}
			}
			if msg.Effect == entity.DieShield || msg.Effect == entity.DieDamage {
				if s, ok := attrs["shields"]; ok {
					s.Base = msg.NewShields
					attrs["shields"] = s
				} else if msg.NewShields > 0 {
					attrs["shields"] = core.Attribute{Name: "shields", Base: msg.NewShields, Min: 0}
				}
			}

			units[i].Attributes = attrs
			break
		}
	}

	updateUnit(combat.PlayerUnits)
	updateUnit(combat.EnemyUnits)

	// Add to combat log (copy slice before appending per TEA immutability)
	newLog := make([]string, len(combat.Log), len(combat.Log)+1)
	copy(newLog, combat.Log)
	combat.Log = append(newLog, fmt.Sprintf("%s -> %s: %s %d",
		msg.SourceUnitID, msg.TargetUnitID, msg.Effect, msg.Value))

	m.Combat = combat
	return m, nil
}

func (m Model) handlePlayerCommandDone(_ PlayerCommandDone) (Model, Cmd) {
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}

	combat := m.Combat
	combat.DicePhase = model.DicePhaseEnemyCommand
	combat.SelectedUnitID = ""
	combat.SelectedDieIndex = -1
	m.Combat = combat

	// TODO: In future, trigger enemy command AI here
	// For now, skip to execution
	return m, AdvanceDicePhase(model.DicePhaseExecution)
}

func (m Model) handleDicePhaseAdvanced(msg DicePhaseAdvanced) (Model, Cmd) {
	combat := m.Combat
	combat.DicePhase = msg.NewPhase
	m.Combat = combat
	return m, nil
}
