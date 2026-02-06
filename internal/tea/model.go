package tea

import (
	"fmt"
	"maps"
	"slices"

	"wulfaz/internal/core"
	"wulfaz/internal/effect"
	"wulfaz/internal/entity"
	"wulfaz/internal/event"
	"wulfaz/internal/model"
)

// Wave 7: Constants for floating text
const (
	CombatTextDuration = 1500000000 // 1.5 seconds in nanoseconds
	MaxTextStack       = 3          // Cap stacking to prevent overflow
)

// Color constants as uint32 (0xRRGGBBAA)
const (
	ColorTextDamage = 0xFF5050FF // Red
	ColorTextHeal   = 0x50FF50FF // Green
	ColorTextShield = 0xAAAAAAFF // Grey
)

type GamePhase int

const (
	PhaseMenu GamePhase = iota
	PhaseCombat
	PhaseInterCombat // Board visible, rewards/fight as overlays, repositioning enabled
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
	case VictorNone:
		return ""
	case VictorPlayer:
		return "player"
	case VictorEnemy:
		return "enemy"
	case VictorDraw:
		return "draw"
	}
	return ""
}

// DragState tracks unit drag-and-drop state during inter-combat phase.
type DragState struct {
	IsDragging    bool
	DraggedUnitID string
	OriginalIndex int // Roster index (board units only, excludes command)
	CurrentX      int // Mouse position
	CurrentY      int
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
	// F-155/F-156: Persistent player roster (survives between fights)
	PlayerRoster []entity.Unit
	// Drag-and-drop state for inter-combat repositioning
	DragState DragState
}

func (m Model) Update(msg Msg) (Model, Cmd) {
	switch msg := msg.(type) {
	case PlayerQuit:
		m.Phase = PhaseGameOver
		return m, nil

	case CombatEnded:
		return m.handleCombatEnded(msg)

	case ChoiceSelected:
		return m.handleChoiceSelected(msg)

	case CombatStarted:
		m.Phase = PhaseCombat
		m.FightNumber++
		m.Combat = msg.Combat
		// Trigger first round (Wave 3)
		return m, StartNextRound(m.Seed, 1, getAllUnits(m.Combat))

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
	case UnitDiceEffectsApplied:
		return m.handleUnitDiceEffectsApplied(msg)
	case PlayerCommandDone:
		return m.handlePlayerCommandDone(msg)
	case DicePhaseAdvanced:
		return m.handleDicePhaseAdvanced(msg)

	// AI targeting and execution
	case AITargetsComputed:
		return m.handleAITargetsComputed(msg)
	case AllAttacksResolved:
		return m.handleAllAttacksResolved(msg)
	case ExecutionComplete:
		return m.handleExecutionComplete(msg)
	case RoundEnded:
		return m.handleRoundEnded(msg)
	case UndoRequested:
		return m.handleUndoRequested(msg)
	case DieUnlocked:
		return m.handleDieUnlocked(msg)
	case UnlockAllDiceRequested:
		return m.handleUnlockAllDiceRequested(msg)
	case AllDiceLocked:
		return m.handleAllDiceLocked(msg)
	case EndTurnRequested:
		return m.handleEndTurnRequested(msg)
	case EndTurnConfirmed:
		return m.handleEndTurnConfirmed(msg)
	case EndTurnCanceled:
		return m.handleEndTurnCanceled(msg)

	// Wave 7: Click-through execution
	case ExecutionAdvanceClicked:
		return m.handleExecutionAdvanceClicked(msg)
	case RoundEndClicked:
		return m.handleRoundEndClicked(msg)

	// Drag-and-drop messages
	case UnitDragStarted:
		return m.handleUnitDragStarted(msg)
	case UnitDragMoved:
		return m.handleUnitDragMoved(msg)
	case UnitDragEnded:
		return m.handleUnitDragEnded(msg)
	case UnitDragCanceled:
		return m.handleUnitDragCanceled(msg)

	default:
		return m, nil
	}
}

// handleTriggersCollected executes effects for collected triggers
func (m Model) handleTriggersCollected(msg TriggersCollected) (Model, Cmd) {
	// Check cascade depth limit
	if msg.Depth >= core.MaxCascadeDepth {
		combat := m.Combat
		combat.Log = appendLogEntry(combat.Log, "cascade depth limit reached")
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

	// Add log entries (bounded for safety)
	combat.Log = appendLogEntries(combat.Log, msg.LogEntries)

	// Copy unit slices before modification (TEA immutability)
	if len(msg.ModifiedUnits) > 0 {
		combat.PlayerUnits = slices.Clone(combat.PlayerUnits)
		combat.EnemyUnits = slices.Clone(combat.EnemyUnits)

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
// Combat ends when a command unit dies.
func (m Model) checkCombatEnd() Victor {
	var playerCmdAlive, enemyCmdAlive bool

	for _, u := range m.Combat.PlayerUnits {
		if u.IsCommand() && u.IsAlive() {
			playerCmdAlive = true
			break
		}
	}
	for _, u := range m.Combat.EnemyUnits {
		if u.IsCommand() && u.IsAlive() {
			enemyCmdAlive = true
			break
		}
	}

	switch {
	case !playerCmdAlive && !enemyCmdAlive:
		return VictorPlayer // Player wins ties per DESIGN.md
	case !enemyCmdAlive:
		return VictorPlayer
	case !playerCmdAlive:
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
	var logMsg string
	if victor == VictorDraw {
		logMsg = "combat ended: draw"
	} else {
		logMsg = "combat ended: " + victor.String() + " wins"
	}
	combat.Log = appendLogEntry(combat.Log, logMsg)
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

// appendLogEntry adds an entry, pruning oldest if over MaxLogEntries.
// Returns new slice (TEA immutability).
func appendLogEntry(log []string, entry string) []string {
	newLog := make([]string, len(log), len(log)+1)
	copy(newLog, log)
	newLog = append(newLog, entry)
	if len(newLog) > model.MaxLogEntries {
		pruned := make([]string, model.MaxLogEntries)
		copy(pruned, newLog[len(newLog)-model.MaxLogEntries:])
		return pruned
	}
	return newLog
}

// appendLogEntries adds multiple entries with bounded size.
// Returns new slice (TEA immutability).
func appendLogEntries(log []string, entries []string) []string {
	if len(entries) == 0 {
		return log
	}
	newLog := make([]string, len(log), len(log)+len(entries))
	copy(newLog, log)
	newLog = append(newLog, entries...)
	if len(newLog) > model.MaxLogEntries {
		pruned := make([]string, model.MaxLogEntries)
		copy(pruned, newLog[len(newLog)-model.MaxLogEntries:])
		return pruned
	}
	return newLog
}

func createUndoSnapshot(combat model.CombatModel) model.UndoSnapshot {
	return model.UndoSnapshot{
		RolledDice:       entity.CopyRolledDiceMap(combat.RolledDice),
		RerollsRemaining: combat.RerollsRemaining,
		ActivatedDice:    maps.Clone(combat.ActivatedDice),
		PlayerTargets:    maps.Clone(combat.PlayerTargets),
		SelectedUnitID:   combat.SelectedUnitID,
		PlayerUnits:      DeepCopyUnits(combat.PlayerUnits),
		Log:              slices.Clone(combat.Log),
		FloatingTexts:    slices.Clone(combat.FloatingTexts),
		ActiveArrows:     model.CopyArrows(combat.ActiveArrows),
	}
}

// DeepCopyUnits creates deep copies of units preserving their IDs.
// Exported for use by app package when building combat from roster.
func DeepCopyUnits(units []entity.Unit) []entity.Unit {
	if units == nil {
		return nil
	}
	copied := make([]entity.Unit, len(units))
	for i, u := range units {
		copied[i] = entity.CopyUnit(u, u.ID) // Same ID, deep copy
	}
	return copied
}

// removeDeadUnits filters out units with health <= 0.
func removeDeadUnits(units []entity.Unit) []entity.Unit {
	alive := make([]entity.Unit, 0, len(units))
	for _, u := range units {
		if u.IsAlive() {
			alive = append(alive, u)
		}
	}
	return alive
}

// syncRosterFromCombat returns deep copies of surviving combat units as new roster.
// Filters dead units here (combat may end mid-execution before handleRoundEnded).
func syncRosterFromCombat(combatUnits []entity.Unit) []entity.Unit {
	alive := removeDeadUnits(combatUnits)
	return DeepCopyUnits(alive)
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
		Position:   unit.Position,
	}
}

// ===== Dice Phase Helpers (Wave 2) =====

// FindPlayerCommandUnit returns the player's command unit (or nil).
// Exported for use by app package.
func FindPlayerCommandUnit(combat model.CombatModel) *entity.Unit {
	for i := range combat.PlayerUnits {
		if combat.PlayerUnits[i].IsCommand() {
			return &combat.PlayerUnits[i]
		}
	}
	return nil
}

// isValidDiceInteraction validates dice interaction prerequisites.
// Rejects interactions when: not in combat phase, combat is paused, or wrong dice phase.
func (m Model) isValidDiceInteraction(unitID string, requiredPhase model.DicePhase) bool {
	// Check game-level phase
	if m.Phase != PhaseCombat {
		return false
	}
	// Reject when paused
	if m.Combat.Phase != model.CombatActive {
		return false
	}
	// Check dice phase
	if m.Combat.DicePhase != requiredPhase {
		return false
	}
	// Validate unit is a player unit
	return m.Combat.IsPlayerUnit(unitID)
}

// findUnitByID returns a pointer to the unit with given ID, or nil.
func findUnitByID(combat model.CombatModel, unitID string) *entity.Unit {
	for i := range combat.PlayerUnits {
		if combat.PlayerUnits[i].ID == unitID {
			return &combat.PlayerUnits[i]
		}
	}
	for i := range combat.EnemyUnits {
		if combat.EnemyUnits[i].ID == unitID {
			return &combat.EnemyUnits[i]
		}
	}
	return nil
}

// buildHPSnapshot creates map of unitID -> {health, shields} for simultaneous resolution.
func buildHPSnapshot(combat model.CombatModel) map[string][2]int {
	snapshot := make(map[string][2]int)
	for _, u := range combat.PlayerUnits {
		hp := 0
		if h, ok := u.Attributes["health"]; ok {
			hp = h.Base
		}
		shields := 0
		if s, ok := u.Attributes["shields"]; ok {
			shields = s.Base
		}
		snapshot[u.ID] = [2]int{hp, shields}
	}
	for _, u := range combat.EnemyUnits {
		hp := 0
		if h, ok := u.Attributes["health"]; ok {
			hp = h.Base
		}
		shields := 0
		if s, ok := u.Attributes["shields"]; ok {
			shields = s.Base
		}
		snapshot[u.ID] = [2]int{hp, shields}
	}
	return snapshot
}

// updateUnitHP updates a unit's health/shields in a slice by ID.
func updateUnitHP(units []entity.Unit, unitID string, newHP, newShields int) {
	for i, u := range units {
		if u.ID != unitID {
			continue
		}
		attrs := core.CopyAttributes(u.Attributes)
		if h, ok := attrs["health"]; ok {
			h.Base = newHP
			attrs["health"] = h
		}
		if s, ok := attrs["shields"]; ok {
			s.Base = newShields
			attrs["shields"] = s
		} else if newShields > 0 {
			attrs["shields"] = core.Attribute{Name: "shields", Base: newShields, Min: 0}
		}
		units[i].Attributes = attrs
		break
	}
}

// allPlayerDiceActivated checks if all player unit dice are activated.
// Blank faces are skipped - they don't need activation.
func allPlayerDiceActivated(combat model.CombatModel) bool {
	for _, u := range combat.PlayerUnits {
		if !u.IsAlive() || len(u.Dice) == 0 {
			continue
		}
		rolledDice, exists := combat.RolledDice[u.ID]
		if !exists {
			continue
		}
		// Skip units where ALL rolled dice are blank
		if !entity.HasNonBlankDie(rolledDice) {
			continue
		}
		if !combat.ActivatedDice[u.ID] {
			return false
		}
	}
	return true
}

// AllPlayerDiceLocked checks if all player unit dice are locked.
// Exported for use by renderer and app packages.
func AllPlayerDiceLocked(combat model.CombatModel) bool {
	for _, u := range combat.PlayerUnits {
		if !u.IsAlive() || len(u.Dice) == 0 {
			continue
		}
		rolledDice, exists := combat.RolledDice[u.ID]
		if !exists {
			continue
		}
		if !entity.IsUnitLocked(rolledDice) {
			return false
		}
	}
	return true
}

// CountUsablePlayerDice returns the count of unactivated non-blank dice.
// Exported for use by app package when dispatching EndTurnRequested.
func CountUsablePlayerDice(combat model.CombatModel) int {
	count := 0
	for _, u := range combat.PlayerUnits {
		if !u.IsAlive() || len(u.Dice) == 0 {
			continue
		}
		rolledDice, exists := combat.RolledDice[u.ID]
		if !exists {
			continue
		}
		if !entity.HasNonBlankDie(rolledDice) {
			continue
		}
		if combat.ActivatedDice[u.ID] {
			continue
		}
		count++
	}
	return count
}

// ===== Dice Phase Handlers (Wave 2) =====

func (m Model) handleRoundStarted(msg RoundStarted) (Model, Cmd) {
	combat := m.Combat
	combat.Round = msg.Round
	combat.DicePhase = model.DicePhasePreview
	combat.RerollsRemaining = model.DefaultRerollsPerRound
	combat.RolledDice = make(map[string][]entity.RolledDie)
	combat.ActivatedDice = make(map[string]bool)
	combat.PlayerTargets = make(map[string]string)
	combat.EnemyTargets = make(map[string]string)
	combat.EnemyDefenseTargets = make(map[string]string)
	combat.SelectedUnitID = ""
	combat.FloatingTexts = nil
	combat.EndTurnConfirmPending = false
	combat.UsableDiceRemaining = 0

	// Convert roll indices to []RolledDie for all units
	allUnits := getAllUnits(combat)
	for _, unit := range allUnits {
		faceIndices, ok := msg.UnitRolls[unit.ID]
		if !ok || len(unit.Dice) == 0 {
			continue
		}
		rolledDice := make([]entity.RolledDie, len(unit.Dice))
		for i, die := range unit.Dice {
			faceIdx := 0
			if i < len(faceIndices) {
				faceIdx = faceIndices[i]
			}
			rolledDice[i] = entity.RolledDie{
				Faces:     die.Faces,
				FaceIndex: faceIdx,
				Locked:    false,
				Fired:     false,
			}
		}
		combat.RolledDice[unit.ID] = rolledDice
		combat.ActivatedDice[unit.ID] = false
	}

	m.Combat = combat

	// Compute AI targets for enemy units
	return m, ComputeAITargets(m.Combat, m.Seed+int64(msg.Round)*7919)
}

func (m Model) handlePreviewDone(_ PreviewDone) (Model, Cmd) {
	// Validate game and combat state
	if m.Phase != PhaseCombat {
		return m, nil
	}
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	if m.Combat.DicePhase != model.DicePhasePreview {
		return m, nil
	}
	combat := m.Combat
	combat.DicePhase = model.DicePhasePlayerCommand
	combat.ActiveArrows = nil // Clear preview arrows (Wave 7)

	// Announce new round in combat log
	combat.Log = appendLogEntry(combat.Log, fmt.Sprintf("--- Round %d ---", combat.Round))

	// Initialize undo system for this round
	combat.InitialRerolls = combat.RerollsRemaining
	combat.UndoStack = []model.UndoSnapshot{createUndoSnapshot(combat)}

	m.Combat = combat

	// Auto-advance if player has no actionable dice (all blank)
	if allPlayerDiceActivated(m.Combat) {
		return m, func() Msg { return PlayerCommandDone{} }
	}
	return m, nil
}

func (m Model) handleDieLockToggled(msg DieLockToggled) (Model, Cmd) {
	if !m.isValidDiceInteraction(msg.UnitID, model.DicePhasePlayerCommand) {
		return m, nil
	}

	combat := m.Combat
	combat.RolledDice = entity.CopyRolledDiceMap(combat.RolledDice)

	if dice, ok := combat.RolledDice[msg.UnitID]; ok {
		newLocked := !entity.IsUnitLocked(dice)
		for i := range dice {
			dice[i].Locked = newLocked
		}
		combat.RolledDice[msg.UnitID] = dice
	}

	m.Combat = combat
	return m, nil
}

func (m Model) handleRerollRequested(msg RerollRequested) (Model, Cmd) {
	// Validate game state
	if m.Phase != PhaseCombat || m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	if m.Combat.RerollsRemaining <= 0 {
		return m, nil
	}

	combat := m.Combat
	combat.RerollsRemaining--
	combat.RolledDice = entity.CopyRolledDiceMap(combat.RolledDice)

	// Apply new face indices per-die for all rerolled dice
	for unitID, newFaceIndices := range msg.Results {
		if dice, ok := combat.RolledDice[unitID]; ok {
			for i := range dice {
				if i < len(newFaceIndices) {
					dice[i].FaceIndex = newFaceIndices[i]
				}
			}
			combat.RolledDice[unitID] = dice
		}
	}

	// Auto-lock all player dice if no rerolls remaining
	if combat.RerollsRemaining == 0 {
		for _, u := range combat.PlayerUnits {
			if dice, ok := combat.RolledDice[u.ID]; ok {
				for i := range dice {
					dice[i].Locked = true
				}
				combat.RolledDice[u.ID] = dice
			}
		}
	}

	// Reset undo stack - rerolls are not undoable, only activations are
	combat.UndoStack = []model.UndoSnapshot{createUndoSnapshot(combat)}

	m.Combat = combat

	// Auto-advance if player has no actionable dice after rerolls exhausted
	if m.Combat.RerollsRemaining == 0 && allPlayerDiceActivated(m.Combat) {
		return m, func() Msg { return PlayerCommandDone{} }
	}
	return m, nil
}

func (m Model) handleDieSelected(msg DieSelected) (Model, Cmd) {
	if !m.isValidDiceInteraction(msg.UnitID, model.DicePhasePlayerCommand) {
		return m, nil
	}
	// Check unit has a die
	_, exists := m.Combat.RolledDice[msg.UnitID]
	if !exists {
		return m, nil
	}
	// Check die not already activated
	if m.Combat.ActivatedDice[msg.UnitID] {
		return m, nil
	}

	combat := m.Combat
	combat.SelectedUnitID = msg.UnitID
	m.Combat = combat
	return m, nil
}

func (m Model) handleDieDeselected(_ DieDeselected) (Model, Cmd) {
	// Validate game and combat state
	if m.Phase != PhaseCombat {
		return m, nil
	}
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}

	combat := m.Combat
	combat.SelectedUnitID = ""
	m.Combat = combat
	return m, nil
}

func (m Model) handleDiceActivated(msg DiceActivated) (Model, Cmd) {
	if !m.isValidDiceInteraction(msg.SourceUnitID, model.DicePhasePlayerCommand) {
		return m, nil
	}
	// Validate source is selected unit
	if msg.SourceUnitID != m.Combat.SelectedUnitID {
		return m, nil
	}

	rolledDice, exists := m.Combat.RolledDice[msg.SourceUnitID]
	if !exists {
		return m, nil
	}

	// Determine target type
	targetIsEnemy := m.Combat.IsEnemyUnit(msg.TargetUnitID)
	targetIsPlayer := m.Combat.IsPlayerUnit(msg.TargetUnitID)
	if !targetIsEnemy && !targetIsPlayer {
		return m, nil
	}

	// Validate compatible unfired dice exist
	if targetIsEnemy {
		if !entity.HasUnfiredDieOfType(rolledDice, entity.DieDamage) {
			return m, nil
		}
		// F-167: Validate target can be attacked
		targetUnit := findUnitByID(m.Combat, msg.TargetUnitID)
		if targetUnit != nil && !CanTargetUnit(*targetUnit, m.Combat.EnemyUnits) {
			return m, nil
		}
	}
	if targetIsPlayer {
		if !entity.HasUnfiredDieOfType(rolledDice, entity.DieShield) && !entity.HasUnfiredDieOfType(rolledDice, entity.DieHeal) {
			return m, nil
		}
	}

	combat := m.Combat
	// Push undo snapshot before changes
	combat.UndoStack = append(slices.Clone(combat.UndoStack), createUndoSnapshot(combat))

	combat.RolledDice = entity.CopyRolledDiceMap(combat.RolledDice)
	combat.ActivatedDice = maps.Clone(combat.ActivatedDice)
	combat.PlayerTargets = maps.Clone(combat.PlayerTargets)

	// Mark compatible unfired dice as Fired
	newDice := combat.RolledDice[msg.SourceUnitID]
	for i := range newDice {
		if newDice[i].Fired {
			continue
		}
		face := newDice[i].CurrentFace()
		if targetIsEnemy && face.Type == entity.DieDamage {
			newDice[i].Fired = true
		}
		if targetIsPlayer && (face.Type == entity.DieShield || face.Type == entity.DieHeal) {
			newDice[i].Fired = true
		}
	}
	combat.RolledDice[msg.SourceUnitID] = newDice

	combat.PlayerTargets[msg.SourceUnitID] = msg.TargetUnitID

	// Add targeting arrows for fired dice
	for _, rd := range newDice {
		if !rd.Fired {
			continue
		}
		face := rd.CurrentFace()
		if face.Type == entity.DieBlank {
			continue
		}
		// Only add arrows for dice we just fired (compatible with this target)
		if targetIsEnemy && face.Type == entity.DieDamage {
			combat.ActiveArrows = append(combat.ActiveArrows, model.TargetingArrow{
				SourceUnitID: msg.SourceUnitID,
				TargetUnitID: msg.TargetUnitID,
				EffectType:   face.Type,
				IsDashed:     false,
			})
			break // One arrow per activation, not per die
		}
		if targetIsPlayer && (face.Type == entity.DieShield || face.Type == entity.DieHeal) {
			combat.ActiveArrows = append(combat.ActiveArrows, model.TargetingArrow{
				SourceUnitID: msg.SourceUnitID,
				TargetUnitID: msg.TargetUnitID,
				EffectType:   face.Type,
				IsDashed:     false,
			})
			break
		}
	}

	// Check if all non-blank dice are fired
	if entity.AllNonBlankFired(newDice) {
		combat.ActivatedDice[msg.SourceUnitID] = true
		combat.SelectedUnitID = ""
	}
	// Otherwise keep unit selected for second target

	m.Combat = combat

	// Apply compatible dice effects
	return m, ApplyCompatibleDiceEffects(msg.SourceUnitID, msg.TargetUnitID, rolledDice, targetIsEnemy, m.Combat, msg.Timestamp)
}

func (m Model) handleUnitDiceEffectsApplied(msg UnitDiceEffectsApplied) (Model, Cmd) {
	combat := m.Combat

	// Copy unit slices before modification
	combat.PlayerUnits = slices.Clone(combat.PlayerUnits)
	combat.EnemyUnits = slices.Clone(combat.EnemyUnits)

	// Apply each result
	for _, result := range msg.Results {
		// Update target's health/shields
		switch result.Effect {
		case entity.DieDamage:
			updateUnitHP(combat.PlayerUnits, result.TargetUnitID, result.NewHealth, result.NewShields)
			updateUnitHP(combat.EnemyUnits, result.TargetUnitID, result.NewHealth, result.NewShields)
		case entity.DieShield:
			updateUnitShields(combat.PlayerUnits, result.TargetUnitID, result.NewShields)
			updateUnitShields(combat.EnemyUnits, result.TargetUnitID, result.NewShields)
		case entity.DieHeal:
			updateUnitHealth(combat.PlayerUnits, result.TargetUnitID, result.NewHealth)
			updateUnitHealth(combat.EnemyUnits, result.TargetUnitID, result.NewHealth)
		case entity.DieBlank:
			// No effect
		}

		// Create floating text per result
		if msg.Timestamp > 0 {
			offset := countTextsForUnit(combat.FloatingTexts, result.TargetUnitID)

			switch result.Effect {
			case entity.DieDamage:
				if result.ShieldAbsorbed > 0 {
					combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
						UnitID:    result.TargetUnitID,
						Text:      fmt.Sprintf("-%d", result.ShieldAbsorbed),
						ColorRGBA: ColorTextShield,
						StartedAt: msg.Timestamp,
						YOffset:   offset,
					})
					offset = min(offset+1, MaxTextStack)
				}
				healthDamage := result.Value - result.ShieldAbsorbed
				if healthDamage > 0 {
					combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
						UnitID:    result.TargetUnitID,
						Text:      fmt.Sprintf("-%d", healthDamage),
						ColorRGBA: ColorTextDamage,
						StartedAt: msg.Timestamp,
						YOffset:   offset,
					})
				}
			case entity.DieHeal:
				combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
					UnitID:    result.TargetUnitID,
					Text:      fmt.Sprintf("+%d", result.Value),
					ColorRGBA: ColorTextHeal,
					StartedAt: msg.Timestamp,
					YOffset:   offset,
				})
			case entity.DieShield:
				combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
					UnitID:    result.TargetUnitID,
					Text:      fmt.Sprintf("+%d", result.Value),
					ColorRGBA: ColorTextShield,
					StartedAt: msg.Timestamp,
					YOffset:   offset,
				})
			case entity.DieBlank:
				// No floating text
			}
		}

		// Log each result
		combat.Log = appendLogEntry(combat.Log, fmt.Sprintf("%s -> %s: %s %d",
			msg.SourceUnitID, result.TargetUnitID, result.Effect, result.Value))
	}

	m.Combat = combat

	// Check if combat ended
	if victor := m.checkCombatEnd(); victor != VictorNone {
		return m.applyCombatEnd()
	}

	// Auto-advance when all player dice are activated
	if combat.DicePhase == model.DicePhasePlayerCommand && allPlayerDiceActivated(m.Combat) {
		return m, func() Msg { return PlayerCommandDone{} }
	}

	return m, nil
}

// updateUnitShields sets shields for a unit in the slice.
func updateUnitShields(units []entity.Unit, unitID string, newShields int) {
	for i, u := range units {
		if u.ID != unitID {
			continue
		}
		attrs := core.CopyAttributes(u.Attributes)
		if s, ok := attrs["shields"]; ok {
			s.Base = newShields
			attrs["shields"] = s
		} else if newShields > 0 {
			attrs["shields"] = core.Attribute{Name: "shields", Base: newShields, Min: 0}
		}
		units[i].Attributes = attrs
		break
	}
}

// updateUnitHealth sets health for a unit in the slice.
func updateUnitHealth(units []entity.Unit, unitID string, newHealth int) {
	for i, u := range units {
		if u.ID != unitID {
			continue
		}
		attrs := core.CopyAttributes(u.Attributes)
		if h, ok := attrs["health"]; ok {
			h.Base = newHealth
			attrs["health"] = h
		}
		units[i].Attributes = attrs
		break
	}
}

// countTextsForUnit returns the number of floating texts for a unit (capped).
func countTextsForUnit(texts []model.FloatingText, unitID string) int {
	count := 0
	for _, t := range texts {
		if t.UnitID == unitID {
			count++
		}
	}
	return min(count, MaxTextStack)
}

func (m Model) handlePlayerCommandDone(_ PlayerCommandDone) (Model, Cmd) {
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}

	combat := m.Combat
	combat.DicePhase = model.DicePhaseExecution
	combat.SelectedUnitID = ""
	combat.EndTurnConfirmPending = false
	combat.UsableDiceRemaining = 0
	// Clear undo stack when exiting phase
	combat.UndoStack = nil
	combat.InitialRerolls = 0
	m.Combat = combat

	// Wait for player click to execute attacks
	return m, nil
}

func (m Model) handleDicePhaseAdvanced(msg DicePhaseAdvanced) (Model, Cmd) {
	combat := m.Combat
	combat.DicePhase = msg.NewPhase
	m.Combat = combat
	return m, nil
}

func (m Model) handleAllDiceLocked(_ AllDiceLocked) (Model, Cmd) {
	if m.Phase != PhaseCombat {
		return m, nil
	}
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	if AllPlayerDiceLocked(m.Combat) {
		return m, nil // Already all locked
	}

	combat := m.Combat
	combat.RolledDice = entity.CopyRolledDiceMap(combat.RolledDice)

	for _, u := range combat.PlayerUnits {
		if !u.IsAlive() || len(u.Dice) == 0 {
			continue
		}
		if dice, ok := combat.RolledDice[u.ID]; ok {
			for i := range dice {
				dice[i].Locked = true
			}
			combat.RolledDice[u.ID] = dice
		}
	}

	m.Combat = combat
	return m, nil
}

func (m Model) handleEndTurnRequested(msg EndTurnRequested) (Model, Cmd) {
	if m.Phase != PhaseCombat {
		return m, nil
	}
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	if !AllPlayerDiceLocked(m.Combat) {
		return m, nil // Must lock all dice first
	}

	// Edge case: 0 usable dice - skip confirmation
	if msg.UsableDiceCount == 0 {
		combat := m.Combat
		combat.Log = appendLogEntry(combat.Log, "ended turn early (no usable dice)")
		m.Combat = combat
		return m, func() Msg { return PlayerCommandDone{} }
	}

	// Enter confirmation state
	combat := m.Combat
	combat.EndTurnConfirmPending = true
	combat.UsableDiceRemaining = msg.UsableDiceCount
	m.Combat = combat
	return m, nil
}

func (m Model) handleEndTurnConfirmed(_ EndTurnConfirmed) (Model, Cmd) {
	if !m.Combat.EndTurnConfirmPending {
		return m, nil
	}

	combat := m.Combat
	combat.EndTurnConfirmPending = false
	diceWord := "dice"
	if combat.UsableDiceRemaining == 1 {
		diceWord = "die"
	}
	combat.Log = appendLogEntry(combat.Log,
		fmt.Sprintf("ended turn early (%d %s skipped)", combat.UsableDiceRemaining, diceWord))
	combat.UsableDiceRemaining = 0
	m.Combat = combat

	return m, func() Msg { return PlayerCommandDone{} }
}

func (m Model) handleEndTurnCanceled(_ EndTurnCanceled) (Model, Cmd) {
	if !m.Combat.EndTurnConfirmPending {
		return m, nil
	}

	combat := m.Combat
	combat.EndTurnConfirmPending = false
	combat.Log = appendLogEntry(combat.Log, "canceled end turn")
	combat.UsableDiceRemaining = 0
	m.Combat = combat
	return m, nil
}

// ===== AI Targeting and Execution Handlers =====

func (m Model) handleAITargetsComputed(msg AITargetsComputed) (Model, Cmd) {
	combat := m.Combat
	combat.EnemyTargets = maps.Clone(msg.Targets)
	combat.EnemyDefenseTargets = maps.Clone(msg.DefenseTargets)

	// Build preview arrows showing AI intent
	combat.ActiveArrows = computeAllPreviewArrows(combat)

	m.Combat = combat
	return m, nil
}

func (m Model) handleAllAttacksResolved(msg AllAttacksResolved) (Model, Cmd) {
	combat := m.Combat
	combat.PlayerUnits = slices.Clone(combat.PlayerUnits)
	combat.EnemyUnits = slices.Clone(combat.EnemyUnits)

	// Apply all attacks and create floating texts
	for _, atk := range msg.Attacks {
		updateUnitHP(combat.PlayerUnits, atk.TargetID, atk.NewHealth, atk.NewShields)
		updateUnitHP(combat.EnemyUnits, atk.TargetID, atk.NewHealth, atk.NewShields)

		// Create floating text(s)
		texts := formatAttackTexts(atk, msg.Timestamp, combat.FloatingTexts)
		combat.FloatingTexts = append(combat.FloatingTexts, texts...)
	}

	// Process defense results (enemy shield/heal against own allies)
	for _, result := range msg.DefenseResults {
		switch result.Effect {
		case entity.DieShield:
			updateUnitShields(combat.EnemyUnits, result.TargetUnitID, result.NewShields)
		case entity.DieHeal:
			updateUnitHealth(combat.EnemyUnits, result.TargetUnitID, result.NewHealth)
		case entity.DieDamage, entity.DieBlank:
			// Defense results only use shield/heal effects
		}

		// Floating text
		offset := countTextsForUnit(combat.FloatingTexts, result.TargetUnitID)
		switch result.Effect {
		case entity.DieHeal:
			combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
				UnitID:    result.TargetUnitID,
				Text:      fmt.Sprintf("+%d", result.Value),
				ColorRGBA: ColorTextHeal,
				StartedAt: msg.Timestamp,
				YOffset:   offset,
			})
		case entity.DieShield:
			combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
				UnitID:    result.TargetUnitID,
				Text:      fmt.Sprintf("+%d", result.Value),
				ColorRGBA: ColorTextShield,
				StartedAt: msg.Timestamp,
				YOffset:   offset,
			})
		case entity.DieDamage, entity.DieBlank:
			// Defense results only use shield/heal effects
		}
	}

	// Log attacks with shield absorption details
	var logEntries []string
	for _, atk := range msg.Attacks {
		status := ""
		if atk.TargetDead {
			status = " [DESTROYED]"
		}
		if atk.ShieldAbsorbed > 0 {
			healthDmg := atk.Damage - atk.ShieldAbsorbed
			logEntries = append(logEntries, fmt.Sprintf("%s -> %s: %d dmg (%d absorbed, %d to health)%s",
				atk.AttackerID, atk.TargetID, atk.Damage, atk.ShieldAbsorbed, healthDmg, status))
		} else if atk.Damage > 0 {
			logEntries = append(logEntries, fmt.Sprintf("%s -> %s: %d dmg%s",
				atk.AttackerID, atk.TargetID, atk.Damage, status))
		}
	}
	// Log defense results
	for _, result := range msg.DefenseResults {
		logEntries = append(logEntries, fmt.Sprintf("enemy %s -> %s: %s %d",
			result.TargetUnitID, result.TargetUnitID, result.Effect, result.Value))
	}
	combat.Log = appendLogEntries(combat.Log, logEntries)

	// Move to round end
	combat.DicePhase = model.DicePhaseRoundEnd
	m.Combat = combat

	// Wait for click to continue
	return m, nil
}

// formatAttackTexts creates FloatingText entries for an attack.
// Shield absorption + overflow creates two stacked entries.
func formatAttackTexts(atk AttackResult, timestamp int64, existing []model.FloatingText) []model.FloatingText {
	// Count existing texts for this unit to determine stack offset
	offset := 0
	for _, t := range existing {
		if t.UnitID == atk.TargetID {
			offset++
		}
	}
	if offset > MaxTextStack {
		offset = MaxTextStack
	}

	var texts []model.FloatingText

	// Determine if this was shield absorption + overflow
	shieldDamage := atk.ShieldAbsorbed
	healthDamage := atk.Damage - shieldDamage

	if shieldDamage > 0 {
		texts = append(texts, model.FloatingText{
			UnitID:    atk.TargetID,
			Text:      fmt.Sprintf("-%d", shieldDamage),
			ColorRGBA: ColorTextShield,
			StartedAt: timestamp,
			YOffset:   min(offset, MaxTextStack),
		})
		offset++
	}

	if healthDamage > 0 {
		texts = append(texts, model.FloatingText{
			UnitID:    atk.TargetID,
			Text:      fmt.Sprintf("-%d", healthDamage),
			ColorRGBA: ColorTextDamage,
			StartedAt: timestamp,
			YOffset:   min(offset, MaxTextStack),
		})
	}

	return texts
}

func (m Model) handleExecutionComplete(_ ExecutionComplete) (Model, Cmd) {
	// Allow from Execution (normal) or RoundEnd (timer fired after phase already set by click)
	if m.Combat.DicePhase != model.DicePhaseExecution && m.Combat.DicePhase != model.DicePhaseRoundEnd {
		return m, nil
	}
	combat := m.Combat
	combat.DicePhase = model.DicePhaseRoundEnd
	m.Combat = combat
	return m, func() Msg { return RoundEnded{} }
}

func (m Model) handleRoundEnded(_ RoundEnded) (Model, Cmd) {
	// Guard: only process once per round (prevents double increment from multiple timers)
	if m.Combat.DicePhase != model.DicePhaseRoundEnd {
		return m, nil
	}
	combat := m.Combat
	combat.PlayerUnits = slices.Clone(combat.PlayerUnits)
	combat.EnemyUnits = slices.Clone(combat.EnemyUnits)

	// F-152: Remove dead units (HP <= 0) from combat
	combat.PlayerUnits = removeDeadUnits(combat.PlayerUnits)
	combat.EnemyUnits = removeDeadUnits(combat.EnemyUnits)

	// F-154: Expire ALL shields (including command units)
	expireShields := func(units []entity.Unit) {
		for i := range units {
			if s, ok := units[i].Attributes["shields"]; ok && s.Base > 0 {
				attrs := core.CopyAttributes(units[i].Attributes)
				s.Base = 0
				attrs["shields"] = s
				units[i].Attributes = attrs
			}
		}
	}
	expireShields(combat.PlayerUnits)
	expireShields(combat.EnemyUnits)

	// Reset phase state
	combat.DicePhase = model.DicePhaseNone
	combat.PlayerTargets = nil
	combat.EnemyTargets = nil
	combat.EnemyDefenseTargets = nil

	m.Combat = combat

	// Check if combat ended before starting next round
	if victor := m.checkCombatEnd(); victor != VictorNone {
		return m.applyCombatEnd()
	}

	// Increment round and start next round immediately
	combat.Round++
	m.Combat = combat
	return m, StartNextRound(m.Seed, combat.Round, getAllUnits(m.Combat))
}

func (m Model) handleUndoRequested(_ UndoRequested) (Model, Cmd) {
	if m.Phase != PhaseCombat {
		return m, nil
	}
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	// Need at least one action to undo
	if len(m.Combat.UndoStack) < 2 {
		return m, nil
	}

	combat := m.Combat
	// Pop last snapshot, restore previous
	newStack := make([]model.UndoSnapshot, len(combat.UndoStack)-1)
	copy(newStack, combat.UndoStack[:len(combat.UndoStack)-1])
	snapshot := newStack[len(newStack)-1]

	combat.RolledDice = entity.CopyRolledDiceMap(snapshot.RolledDice)
	combat.RerollsRemaining = snapshot.RerollsRemaining
	combat.ActivatedDice = maps.Clone(snapshot.ActivatedDice)
	combat.PlayerTargets = maps.Clone(snapshot.PlayerTargets)
	combat.SelectedUnitID = snapshot.SelectedUnitID
	combat.PlayerUnits = DeepCopyUnits(snapshot.PlayerUnits)
	combat.Log = slices.Clone(snapshot.Log)
	combat.FloatingTexts = slices.Clone(snapshot.FloatingTexts)
	combat.ActiveArrows = model.CopyArrows(snapshot.ActiveArrows)
	combat.UndoStack = newStack
	combat.EndTurnConfirmPending = false

	m.Combat = combat
	return m, nil
}

func (m Model) handleDieUnlocked(msg DieUnlocked) (Model, Cmd) {
	if !m.isValidDiceInteraction(msg.UnitID, model.DicePhasePlayerCommand) {
		return m, nil
	}
	if m.Combat.RerollsRemaining <= 0 {
		return m, nil
	}

	combat := m.Combat
	dice, exists := combat.RolledDice[msg.UnitID]
	if !exists || !entity.IsUnitLocked(dice) || combat.ActivatedDice[msg.UnitID] {
		return m, nil
	}

	combat.RolledDice = entity.CopyRolledDiceMap(combat.RolledDice)
	newDice := combat.RolledDice[msg.UnitID]
	for i := range newDice {
		newDice[i].Locked = false
	}
	combat.RolledDice[msg.UnitID] = newDice
	m.Combat = combat
	return m, nil
}

func (m Model) handleUnlockAllDiceRequested(_ UnlockAllDiceRequested) (Model, Cmd) {
	if m.Phase != PhaseCombat {
		return m, nil
	}
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	if m.Combat.DicePhase != model.DicePhasePlayerCommand {
		return m, nil
	}
	if m.Combat.RerollsRemaining <= 0 {
		return m, nil
	}

	combat := m.Combat
	combat.RolledDice = entity.CopyRolledDiceMap(combat.RolledDice)

	// Unlock all non-activated player dice
	for _, u := range combat.PlayerUnits {
		if !u.IsAlive() || len(u.Dice) == 0 {
			continue
		}
		if dice, ok := combat.RolledDice[u.ID]; ok && !combat.ActivatedDice[u.ID] {
			for i := range dice {
				dice[i].Locked = false
			}
			combat.RolledDice[u.ID] = dice
		}
	}

	// Clear selection and confirmation state (returning to lock phase)
	combat.SelectedUnitID = ""
	combat.EndTurnConfirmPending = false
	m.Combat = combat
	return m, nil
}

// handleExecutionAdvanceClicked executes all attacks simultaneously on click.
func (m Model) handleExecutionAdvanceClicked(msg ExecutionAdvanceClicked) (Model, Cmd) {
	if m.Combat.DicePhase != model.DicePhaseExecution {
		return m, nil
	}
	combat := m.Combat

	// Prune expired floating texts
	combat.FloatingTexts = pruneExpiredTexts(combat.FloatingTexts, msg.Timestamp)
	m.Combat = combat

	// Execute all attacks simultaneously
	return m, ExecuteAllAttacks(m.Combat, msg.Timestamp)
}

// pruneExpiredTexts removes floating texts older than CombatTextDuration.
func pruneExpiredTexts(texts []model.FloatingText, nowNano int64) []model.FloatingText {
	cutoff := nowNano - int64(CombatTextDuration)
	result := texts[:0] // Reuse backing array
	for _, t := range texts {
		if t.StartedAt > cutoff {
			result = append(result, t)
		}
	}
	return result
}

// handleRoundEndClicked advances past round end when player clicks.
func (m Model) handleRoundEndClicked(_ RoundEndClicked) (Model, Cmd) {
	if m.Combat.DicePhase != model.DicePhaseRoundEnd {
		return m, nil
	}
	// Check victory before starting next round
	if victor := m.checkCombatEnd(); victor != VictorNone {
		return m.applyCombatEnd()
	}
	// Clear floating texts and trigger round end flow
	combat := m.Combat
	combat.FloatingTexts = nil
	m.Combat = combat
	return m, func() Msg { return ExecutionComplete{} }
}

// computeAllPreviewArrows shows both player (solid) and enemy (dashed) arrows.
func computeAllPreviewArrows(combat model.CombatModel) []model.TargetingArrow {
	arrows := computeEnemyPreviewArrows(combat) // Dashed
	arrows = append(arrows, computePlayerPreviewArrows(combat)...)
	return arrows
}

// computePlayerPreviewArrows shows arrows from PlayerTargets map (if set).
func computePlayerPreviewArrows(combat model.CombatModel) []model.TargetingArrow {
	var arrows []model.TargetingArrow

	for sourceID, targetID := range combat.PlayerTargets {
		rolledDice, exists := combat.RolledDice[sourceID]
		if !exists {
			continue
		}
		arrows = append(arrows, model.TargetingArrow{
			SourceUnitID: sourceID,
			TargetUnitID: targetID,
			EffectType:   entity.PrimaryEffectType(rolledDice),
			IsDashed:     false,
		})
	}
	return arrows
}

// computeEnemyPreviewArrows builds dashed arrows from EnemyTargets and EnemyDefenseTargets.
func computeEnemyPreviewArrows(combat model.CombatModel) []model.TargetingArrow {
	var arrows []model.TargetingArrow

	// Damage arrows from EnemyTargets
	for sourceID, targetID := range combat.EnemyTargets {
		rolledDice, exists := combat.RolledDice[sourceID]
		if !exists {
			continue
		}
		// Find the damage effect type for this arrow
		effectType := entity.DieBlank
		for _, rd := range rolledDice {
			if rd.CurrentFace().Type == entity.DieDamage {
				effectType = entity.DieDamage
				break
			}
		}
		if effectType == entity.DieBlank {
			continue
		}
		arrows = append(arrows, model.TargetingArrow{
			SourceUnitID: sourceID,
			TargetUnitID: targetID,
			EffectType:   effectType,
			IsDashed:     true,
		})
	}

	// Defense arrows from EnemyDefenseTargets
	for sourceID, allyID := range combat.EnemyDefenseTargets {
		rolledDice, exists := combat.RolledDice[sourceID]
		if !exists {
			continue
		}
		// Find the shield/heal effect type
		effectType := entity.DieBlank
		for _, rd := range rolledDice {
			ft := rd.CurrentFace().Type
			if ft == entity.DieShield || ft == entity.DieHeal {
				effectType = ft
				break
			}
		}
		if effectType == entity.DieBlank {
			continue
		}
		arrows = append(arrows, model.TargetingArrow{
			SourceUnitID: sourceID,
			TargetUnitID: allyID,
			EffectType:   effectType,
			IsDashed:     true,
		})
	}

	return arrows
}

func (m Model) handleCombatEnded(msg CombatEnded) (Model, Cmd) {
	if msg.Victor == VictorPlayer {
		// F-155/F-156: Persist surviving units (syncRosterFromCombat filters dead)
		m.PlayerRoster = syncRosterFromCombat(m.Combat.PlayerUnits)

		m.Phase = PhaseInterCombat
		m.ChoiceType = ChoiceReward
		m.RewardChoicesLeft = 2
		m.Choices = []string{"Reward A", "Reward B", "Reward C"}
		m.DragState = DragState{} // Clear any existing drag state
	} else {
		m.Phase = PhaseGameOver
	}
	return m, nil
}

func (m Model) handleChoiceSelected(msg ChoiceSelected) (Model, Cmd) {
	// Phase guard - only process during inter-combat phase
	if m.Phase != PhaseInterCombat {
		return m, nil
	}
	// Bounds validation - reject invalid indices
	if msg.Index < 0 || msg.Index >= len(m.Choices) {
		return m, nil
	}
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
}

// ===== Drag-and-Drop Handlers =====

func (m Model) handleUnitDragStarted(msg UnitDragStarted) (Model, Cmd) {
	if m.Phase != PhaseInterCombat {
		return m, nil
	}
	m.DragState = DragState{
		IsDragging:    true,
		DraggedUnitID: msg.UnitID,
		OriginalIndex: msg.OriginalIndex,
		CurrentX:      msg.StartX,
		CurrentY:      msg.StartY,
	}
	return m, nil
}

func (m Model) handleUnitDragMoved(msg UnitDragMoved) (Model, Cmd) {
	if m.Phase != PhaseInterCombat || !m.DragState.IsDragging {
		return m, nil
	}
	m.DragState.CurrentX = msg.CurrentX
	m.DragState.CurrentY = msg.CurrentY
	return m, nil
}

func (m Model) handleUnitDragEnded(msg UnitDragEnded) (Model, Cmd) {
	if m.Phase != PhaseInterCombat || !m.DragState.IsDragging {
		return m, nil
	}
	// Reorder if valid and different position
	if msg.InsertionIndex >= 0 && msg.InsertionIndex != m.DragState.OriginalIndex {
		m.PlayerRoster = reorderRoster(m.PlayerRoster, m.DragState.OriginalIndex, msg.InsertionIndex)
	}
	m.DragState = DragState{} // Clear
	return m, nil
}

func (m Model) handleUnitDragCanceled(_ UnitDragCanceled) (Model, Cmd) {
	m.DragState = DragState{} // Clear
	return m, nil
}

// reorderRoster moves unit from fromIdx to toIdx, returns new slice.
// Indices are relative to board units only (command unit excluded).
func reorderRoster(roster []entity.Unit, fromIdx, toIdx int) []entity.Unit {
	// Separate command unit
	var cmd *entity.Unit
	var board []entity.Unit
	for i := range roster {
		if roster[i].IsCommand() {
			c := roster[i]
			cmd = &c
		} else {
			board = append(board, roster[i])
		}
	}

	// Bounds check
	if fromIdx < 0 || fromIdx >= len(board) || toIdx < 0 || toIdx > len(board) {
		return roster
	}
	if fromIdx == toIdx {
		return roster
	}

	// Remove from original position
	unit := board[fromIdx]
	board = append(board[:fromIdx], board[fromIdx+1:]...)

	// Adjust toIdx if needed (removal shifted indices)
	if toIdx > fromIdx {
		toIdx--
	}

	// Insert at new position
	board = append(board[:toIdx], append([]entity.Unit{unit}, board[toIdx:]...)...)

	// Rebuild roster: command first (if exists), then board units
	result := make([]entity.Unit, 0, len(roster))
	if cmd != nil {
		result = append(result, *cmd)
	}
	result = append(result, board...)

	return result
}
