package tea

import (
	"fmt"
	"maps"
	"slices"

	"wulfaz/internal/core"
	"wulfaz/internal/entity"
	"wulfaz/internal/model"
)

type Model struct {
	Version int
	Phase   model.GamePhase
	Combat  model.CombatModel
	Seed    int64
	// Choice phase state
	ChoiceType        model.ChoiceType
	RewardChoicesLeft int
	Choices           []string
	// Run progression
	FightNumber int
	// F-155/F-156: Persistent player roster (survives between fights)
	PlayerRoster []entity.Unit
	// Drag-and-drop state for inter-combat repositioning
	DragState model.DragState
}

func (m Model) Update(msg model.Msg) (Model, model.Cmd) {
	switch msg := msg.(type) {
	case model.PlayerQuit:
		m.Phase = model.PhaseGameOver
		return m, nil

	case model.CombatEnded:
		return m.handleCombatEnded(msg)

	case model.ChoiceSelected:
		return m.handleChoiceSelected(msg)

	case model.CombatStarted:
		m.Phase = model.PhaseCombat
		m.FightNumber++
		m.Combat = msg.Combat
		// Trigger first round (Wave 3)
		return m, StartNextRound(m.Seed, 1, getAllUnits(m.Combat))

	case model.PlayerPaused:
		if m.Phase != model.PhaseCombat {
			return m, nil
		}
		combat := m.Combat
		combat.Phase = model.CombatPaused
		m.Combat = combat
		return m, nil

	case model.PlayerResumed:
		if m.Phase != model.PhaseCombat {
			return m, nil
		}
		combat := m.Combat
		combat.Phase = model.CombatActive
		m.Combat = combat
		return m, nil

	case model.RoundStarted:
		return m.handleRoundStarted(msg)
	case model.PreviewDone:
		return m.handlePreviewDone(msg)
	case model.DieLockToggled:
		return m.handleDieLockToggled(msg)
	case model.RerollRequested:
		return m.handleRerollRequested(msg)
	case model.DieSelected:
		return m.handleDieSelected(msg)
	case model.DieDeselected:
		return m.handleDieDeselected(msg)
	case model.DiceActivated:
		return m.handleDiceActivated(msg)
	case model.UnitDiceEffectsApplied:
		return m.handleUnitDiceEffectsApplied(msg)
	case model.PlayerCommandDone:
		return m.handlePlayerCommandDone(msg)
	case model.DicePhaseAdvanced:
		return m.handleDicePhaseAdvanced(msg)

	// AI targeting and execution
	case model.AITargetsComputed:
		return m.handleAITargetsComputed(msg)
	case model.ExecutionComplete:
		return m.handleExecutionComplete(msg)
	case model.RoundEnded:
		return m.handleRoundEnded(msg)
	case model.UndoRequested:
		return m.handleUndoRequested(msg)
	case model.DieUnlocked:
		return m.handleDieUnlocked(msg)
	case model.UnlockAllDiceRequested:
		return m.handleUnlockAllDiceRequested(msg)
	case model.AllDiceLocked:
		return m.handleAllDiceLocked(msg)
	case model.EndTurnRequested:
		return m.handleEndTurnRequested(msg)
	case model.EndTurnConfirmed:
		return m.handleEndTurnConfirmed(msg)
	case model.EndTurnCanceled:
		return m.handleEndTurnCanceled(msg)

	// Wave 7: Click-through execution
	case model.ExecutionAdvanceClicked:
		return m.handleExecutionAdvanceClicked(msg)
	case model.RoundEndClicked:
		return m.handleRoundEndClicked(msg)

	// Drag-and-drop messages
	case model.UnitDragStarted:
		return m.handleUnitDragStarted(msg)
	case model.UnitDragMoved:
		return m.handleUnitDragMoved(msg)
	case model.UnitDragEnded:
		return m.handleUnitDragEnded(msg)
	case model.UnitDragCanceled:
		return m.handleUnitDragCanceled(msg)
	}
	panic(fmt.Sprintf("tea.Update: unhandled Msg type %T", msg))
}

// checkCombatEnd returns the victor if combat has ended, or VictorNone if ongoing.
// Combat ends when a command unit dies.
func (m Model) checkCombatEnd() model.Victor {
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
		return model.VictorPlayer // Player wins ties per DESIGN.md
	case !enemyCmdAlive:
		return model.VictorPlayer
	case !playerCmdAlive:
		return model.VictorEnemy
	default:
		return model.VictorNone
	}
}

// applyCombatEnd checks for combat end and updates model if combat is over.
// Returns the updated model and a Cmd that emits CombatEnded if combat ended.
func (m Model) applyCombatEnd() (Model, model.Cmd) {
	if m.Combat.Phase != model.CombatActive {
		return m, nil
	}
	victor := m.checkCombatEnd()
	if victor == model.VictorNone {
		return m, nil
	}
	combat := m.Combat
	combat.Phase = model.CombatResolved
	combat.Victor = victor.String()
	var logMsg string
	if victor == model.VictorDraw {
		logMsg = "combat ended: draw"
	} else {
		logMsg = "combat ended: " + victor.String() + " wins"
	}
	combat.Log = appendLogEntry(combat.Log, logMsg)
	m.Combat = combat
	return m, func() model.Msg { return model.CombatEnded{Victor: victor} }
}

// Helper functions

func getAllUnits(combat model.CombatModel) []entity.Unit {
	all := make([]entity.Unit, 0, len(combat.PlayerUnits)+len(combat.EnemyUnits))
	all = append(all, combat.PlayerUnits...)
	all = append(all, combat.EnemyUnits...)
	return all
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

// pruneDeadTargets removes entries from a target map where the source or destination unit is dead.
func pruneDeadTargets(targets map[string]string, sources, dests []entity.Unit) map[string]string {
	if len(targets) == 0 {
		return targets
	}
	alive := make(map[string]bool, len(sources)+len(dests))
	for _, u := range sources {
		if u.IsAlive() {
			alive[u.ID] = true
		}
	}
	for _, u := range dests {
		if u.IsAlive() {
			alive[u.ID] = true
		}
	}
	pruned := make(map[string]string, len(targets))
	for src, dst := range targets {
		if alive[src] && alive[dst] {
			pruned[src] = dst
		}
	}
	return pruned
}

// syncRosterFromCombat returns deep copies of surviving combat units as new roster.
// Filters dead units here (combat may end mid-execution before handleRoundEnded).
func syncRosterFromCombat(combatUnits []entity.Unit) []entity.Unit {
	alive := removeDeadUnits(combatUnits)
	return DeepCopyUnits(alive)
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
	if m.Phase != model.PhaseCombat {
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

func (m Model) handleRoundStarted(msg model.RoundStarted) (Model, model.Cmd) {
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

func (m Model) handlePreviewDone(_ model.PreviewDone) (Model, model.Cmd) {
	// Validate game and combat state
	if m.Phase != model.PhaseCombat {
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
		return m, func() model.Msg { return model.PlayerCommandDone{} }
	}
	return m, nil
}

func (m Model) handleDieLockToggled(msg model.DieLockToggled) (Model, model.Cmd) {
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

func (m Model) handleRerollRequested(msg model.RerollRequested) (Model, model.Cmd) {
	// Validate game state
	if m.Phase != model.PhaseCombat || m.Combat.Phase != model.CombatActive {
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
		return m, func() model.Msg { return model.PlayerCommandDone{} }
	}
	return m, nil
}

func (m Model) handleDieSelected(msg model.DieSelected) (Model, model.Cmd) {
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

func (m Model) handleDieDeselected(_ model.DieDeselected) (Model, model.Cmd) {
	// Validate game and combat state
	if m.Phase != model.PhaseCombat {
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

func (m Model) handleDiceActivated(msg model.DiceActivated) (Model, model.Cmd) {
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

func (m Model) handleUnitDiceEffectsApplied(msg model.UnitDiceEffectsApplied) (Model, model.Cmd) {
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
						ColorRGBA: model.ColorTextShield,
						StartedAt: msg.Timestamp,
						YOffset:   offset,
					})
					offset = min(offset+1, model.MaxTextStack)
				}
				healthDamage := result.Value - result.ShieldAbsorbed
				if healthDamage > 0 {
					combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
						UnitID:    result.TargetUnitID,
						Text:      fmt.Sprintf("-%d", healthDamage),
						ColorRGBA: model.ColorTextDamage,
						StartedAt: msg.Timestamp,
						YOffset:   offset,
					})
				}
			case entity.DieHeal:
				combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
					UnitID:    result.TargetUnitID,
					Text:      fmt.Sprintf("+%d", result.Value),
					ColorRGBA: model.ColorTextHeal,
					StartedAt: msg.Timestamp,
					YOffset:   offset,
				})
			case entity.DieShield:
				combat.FloatingTexts = append(combat.FloatingTexts, model.FloatingText{
					UnitID:    result.TargetUnitID,
					Text:      fmt.Sprintf("+%d", result.Value),
					ColorRGBA: model.ColorTextShield,
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

	// Prune target entries for dead units so subsequent execution steps skip them.
	combat.EnemyTargets = pruneDeadTargets(combat.EnemyTargets, combat.EnemyUnits, combat.PlayerUnits)
	combat.EnemyDefenseTargets = pruneDeadTargets(combat.EnemyDefenseTargets, combat.EnemyUnits, combat.EnemyUnits)

	m.Combat = combat

	// Check if combat ended
	if victor := m.checkCombatEnd(); victor != model.VictorNone {
		return m.applyCombatEnd()
	}

	// Chain next enemy unit during execution phase
	if combat.DicePhase == model.DicePhaseExecution {
		return m.advanceEnemyExecution(msg.SourceUnitID, msg.Timestamp)
	}

	// Auto-advance when all player dice are activated
	if combat.DicePhase == model.DicePhasePlayerCommand && allPlayerDiceActivated(m.Combat) {
		return m, func() model.Msg { return model.PlayerCommandDone{} }
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
	return min(count, model.MaxTextStack)
}

func (m Model) handlePlayerCommandDone(_ model.PlayerCommandDone) (Model, model.Cmd) {
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

func (m Model) handleDicePhaseAdvanced(msg model.DicePhaseAdvanced) (Model, model.Cmd) {
	combat := m.Combat
	combat.DicePhase = msg.NewPhase
	m.Combat = combat
	return m, nil
}

func (m Model) handleAllDiceLocked(_ model.AllDiceLocked) (Model, model.Cmd) {
	if m.Phase != model.PhaseCombat {
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

func (m Model) handleEndTurnRequested(msg model.EndTurnRequested) (Model, model.Cmd) {
	if m.Phase != model.PhaseCombat {
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
		return m, func() model.Msg { return model.PlayerCommandDone{} }
	}

	// Enter confirmation state
	combat := m.Combat
	combat.EndTurnConfirmPending = true
	combat.UsableDiceRemaining = msg.UsableDiceCount
	m.Combat = combat
	return m, nil
}

func (m Model) handleEndTurnConfirmed(_ model.EndTurnConfirmed) (Model, model.Cmd) {
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

	return m, func() model.Msg { return model.PlayerCommandDone{} }
}

func (m Model) handleEndTurnCanceled(_ model.EndTurnCanceled) (Model, model.Cmd) {
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

func (m Model) handleAITargetsComputed(msg model.AITargetsComputed) (Model, model.Cmd) {
	combat := m.Combat
	combat.EnemyTargets = maps.Clone(msg.Targets)
	combat.EnemyDefenseTargets = maps.Clone(msg.DefenseTargets)

	// Build preview arrows showing AI intent
	combat.ActiveArrows = computeAllPreviewArrows(combat)

	m.Combat = combat
	return m, nil
}

func (m Model) handleExecutionComplete(_ model.ExecutionComplete) (Model, model.Cmd) {
	// Allow from Execution (normal) or RoundEnd (timer fired after phase already set by click)
	if m.Combat.DicePhase != model.DicePhaseExecution && m.Combat.DicePhase != model.DicePhaseRoundEnd {
		return m, nil
	}
	combat := m.Combat
	combat.DicePhase = model.DicePhaseRoundEnd
	m.Combat = combat
	return m, func() model.Msg { return model.RoundEnded{} }
}

func (m Model) handleRoundEnded(_ model.RoundEnded) (Model, model.Cmd) {
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
	if victor := m.checkCombatEnd(); victor != model.VictorNone {
		return m.applyCombatEnd()
	}

	// Increment round and start next round immediately
	combat.Round++
	m.Combat = combat
	return m, StartNextRound(m.Seed, combat.Round, getAllUnits(m.Combat))
}

func (m Model) handleUndoRequested(_ model.UndoRequested) (Model, model.Cmd) {
	if m.Phase != model.PhaseCombat {
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

func (m Model) handleDieUnlocked(msg model.DieUnlocked) (Model, model.Cmd) {
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

func (m Model) handleUnlockAllDiceRequested(_ model.UnlockAllDiceRequested) (Model, model.Cmd) {
	if m.Phase != model.PhaseCombat {
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

// handleExecutionAdvanceClicked starts per-unit enemy execution on click.
func (m Model) handleExecutionAdvanceClicked(msg model.ExecutionAdvanceClicked) (Model, model.Cmd) {
	if m.Combat.DicePhase != model.DicePhaseExecution {
		return m, nil
	}
	combat := m.Combat

	// Prune expired floating texts
	combat.FloatingTexts = pruneExpiredTexts(combat.FloatingTexts, msg.Timestamp)
	m.Combat = combat

	return m.advanceEnemyExecution("", msg.Timestamp)
}

// advanceEnemyExecution finds the next enemy unit after afterUnitID with targets and fires its Cmd.
// When all units are processed, transitions to DicePhaseRoundEnd.
func (m Model) advanceEnemyExecution(afterUnitID string, timestamp int64) (Model, model.Cmd) {
	combat := m.Combat
	found := afterUnitID == ""
	for _, unit := range combat.EnemyUnits {
		if !found {
			if unit.ID == afterUnitID {
				found = true
			}
			continue
		}
		_, hasDmg := combat.EnemyTargets[unit.ID]
		_, hasDef := combat.EnemyDefenseTargets[unit.ID]
		if hasDmg || hasDef {
			return m, ApplyEnemyUnitEffects(unit.ID, m.Combat, timestamp)
		}
	}
	// All enemy units processed
	combat.DicePhase = model.DicePhaseRoundEnd
	m.Combat = combat
	return m, nil
}

// pruneExpiredTexts removes floating texts older than CombatTextDuration.
func pruneExpiredTexts(texts []model.FloatingText, nowNano int64) []model.FloatingText {
	cutoff := nowNano - int64(model.CombatTextDuration)
	result := texts[:0] // Reuse backing array
	for _, t := range texts {
		if t.StartedAt > cutoff {
			result = append(result, t)
		}
	}
	return result
}

// handleRoundEndClicked advances past round end when player clicks.
func (m Model) handleRoundEndClicked(_ model.RoundEndClicked) (Model, model.Cmd) {
	if m.Combat.DicePhase != model.DicePhaseRoundEnd {
		return m, nil
	}
	// Check victory before starting next round
	if victor := m.checkCombatEnd(); victor != model.VictorNone {
		return m.applyCombatEnd()
	}
	// Clear floating texts and trigger round end flow
	combat := m.Combat
	combat.FloatingTexts = nil
	m.Combat = combat
	return m, func() model.Msg { return model.ExecutionComplete{} }
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

func (m Model) handleCombatEnded(msg model.CombatEnded) (Model, model.Cmd) {
	if msg.Victor == model.VictorPlayer {
		// F-155/F-156: Persist surviving units (syncRosterFromCombat filters dead)
		m.PlayerRoster = syncRosterFromCombat(m.Combat.PlayerUnits)

		m.Phase = model.PhaseInterCombat
		m.ChoiceType = model.ChoiceReward
		m.RewardChoicesLeft = 2
		m.Choices = []string{"Reward A", "Reward B", "Reward C"}
		m.DragState = model.DragState{} // Clear any existing drag state
	} else {
		m.Phase = model.PhaseGameOver
	}
	return m, nil
}

func (m Model) handleChoiceSelected(msg model.ChoiceSelected) (Model, model.Cmd) {
	// Phase guard - only process during inter-combat phase
	if m.Phase != model.PhaseInterCombat {
		return m, nil
	}
	// Bounds validation - reject invalid indices
	if msg.Index < 0 || msg.Index >= len(m.Choices) {
		return m, nil
	}
	if m.ChoiceType == model.ChoiceReward {
		m.RewardChoicesLeft--
		if m.RewardChoicesLeft > 0 {
			m.Choices = []string{"Reward D", "Reward E", "Reward F"}
		} else {
			m.ChoiceType = model.ChoiceFight
			m.Choices = []string{"Fight: Easy", "Fight: Medium", "Fight: Hard"}
		}
	}
	return m, nil
}

// ===== Drag-and-Drop Handlers =====

func (m Model) handleUnitDragStarted(msg model.UnitDragStarted) (Model, model.Cmd) {
	if m.Phase != model.PhaseInterCombat {
		return m, nil
	}
	m.DragState = model.DragState{
		IsDragging:    true,
		DraggedUnitID: msg.UnitID,
		OriginalIndex: msg.OriginalIndex,
		CurrentX:      msg.StartX,
		CurrentY:      msg.StartY,
	}
	return m, nil
}

func (m Model) handleUnitDragMoved(msg model.UnitDragMoved) (Model, model.Cmd) {
	if m.Phase != model.PhaseInterCombat || !m.DragState.IsDragging {
		return m, nil
	}
	m.DragState.CurrentX = msg.CurrentX
	m.DragState.CurrentY = msg.CurrentY
	return m, nil
}

func (m Model) handleUnitDragEnded(msg model.UnitDragEnded) (Model, model.Cmd) {
	if m.Phase != model.PhaseInterCombat || !m.DragState.IsDragging {
		return m, nil
	}
	// Reorder if valid and different position
	if msg.InsertionIndex >= 0 && msg.InsertionIndex != m.DragState.OriginalIndex {
		m.PlayerRoster = reorderRoster(m.PlayerRoster, m.DragState.OriginalIndex, msg.InsertionIndex)
	}
	m.DragState = model.DragState{} // Clear
	return m, nil
}

func (m Model) handleUnitDragCanceled(_ model.UnitDragCanceled) (Model, model.Cmd) {
	m.DragState = model.DragState{} // Clear
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

	// NOTE: No toIdx adjustment needed — computeInsertionIndex already
	// skips the dragged unit, so toIdx is already in post-removal space.

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
