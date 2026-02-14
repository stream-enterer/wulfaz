# Wulfaz Work Phases

> Discrete phase separation for implementing features from `features.md`.
> Each phase produces a defined artifact. Phases run sequentially for each
> feature. Never combine phases -- context contamination degrades quality.

## Starting a Session

1. Read `progress.jsonl` to see current state across all features.
2. If any feature has `status: "in_progress"`, resume it at the `phase`
   indicated in its progress entry.
3. If no feature is in progress, select the next `pending` feature whose
   dependencies (listed in `features.md`) are all `complete`.
4. Prefer features with no dependencies first, then lower-numbered IDs.
5. Follow the five phases below for the selected feature.
6. Update `progress.jsonl` at each phase transition.

---

## Phase 1: Research

**Objective:** Understand the feature's context within the existing codebase
and architecture. Identify all files, patterns, and constraints that apply.

**Entry Condition:** Feature selected from `features.md` with status `[ ]`.

**Activities:**
1. Read the feature's acceptance criteria in `features.md`
2. Read `CLAUDE.md` for architectural rules that apply to this feature
3. Read all source files that the feature touches or depends on
4. Identify which existing patterns (collect-then-apply, pending-death filter,
   etc.) must be followed
5. List all dependency features and confirm they are complete or in-progress
6. Note any edge cases or invariants that interact with this feature

**Artifacts Produced:**
- List of files to read/modify
- List of architectural constraints that apply
- List of dependency features and their status
- Notes on edge cases discovered

**Exit Condition:** All relevant code and constraints are understood. No
implementation decisions have been made yet.

**Handoff to Phase 2:** Pass the file list, constraint list, and edge case
notes. Do NOT pass full file contents -- only distilled findings.

---

## Phase 2: Planning

**Objective:** Design the approach for implementing the feature. Decide
which files to create or modify, what the code structure will look like,
and in what order changes should be made.

**Entry Condition:** Research phase artifacts are available.

**Activities:**
1. Review research findings (file list, constraints, edge cases)
2. Determine which files need to be created vs. modified
3. Design the data structures (if adding components or tables)
4. Plan the function signature and control flow
5. Identify which checklist applies (ADD-001, ADD-002, ADD-003) and verify
   all checklist steps are in the plan
6. Plan the test: what World state to construct, what to assert
7. Check for conflicts with other in-progress features

**Artifacts Produced:**
- Ordered list of implementation steps
- File-by-file change descriptions (what to add/modify, not the actual code)
- Test plan: setup, action, assertion
- Checklist verification (all required steps accounted for)

**Exit Condition:** Implementation can proceed step-by-step without
further design decisions.

**Handoff to Phase 3:** Pass the implementation plan and test plan.
Do NOT pass research notes -- they are already distilled into the plan.

---

## Phase 3: Implementation

**Objective:** Write the code. Execute the plan step by step. Focus on
code quality, correctness, and adherence to architectural rules.

**Entry Condition:** Planning phase artifacts are available.

**Activities:**
1. Execute implementation steps in order from the plan
2. For each code change, verify it follows:
   - Collect-then-apply pattern (SYS-003)
   - Pending-death filtering (LIFE-004)
   - No unwrap on lookups (RULE-001)
   - Skip missing entries (RULE-003)
   - Seeded RNG only (RNG-001)
3. If adding a property table, complete ALL 5 steps (ADD-002)
4. If adding a system, complete ALL 6 steps (ADD-001)
5. If adding an event type, complete ALL 4 steps (ADD-003)
6. Write the unit test as specified in the test plan
7. Do NOT run tests yet -- that is Phase 4

**Artifacts Produced:**
- Modified/created source files
- Unit test(s) written
- List of all files changed

**Exit Condition:** All planned code changes are written. Unit test exists.
No compilation attempted yet.

**Handoff to Phase 4:** Pass the list of changed files. Do NOT pass the
implementation plan -- the code itself is now the source of truth.

---

## Phase 4: Testing

**Objective:** Verify the implementation compiles, tests pass, and the
feature meets its acceptance criteria.

**Entry Condition:** Implementation phase artifacts (changed files) are
available.

**Activities:**
1. Run `cargo build` -- fix any compilation errors
2. Run `cargo test` -- fix any test failures
3. Run in debug mode to confirm `validate_world()` passes (if simulation
   code was changed)
4. Walk through each acceptance criterion in `features.md`:
   - Can the criterion be verified by test output? If so, verify.
   - Can the criterion be verified by code inspection? If so, inspect.
   - Document which criteria pass and which need fixes.
5. If any criterion fails, return to Phase 3 with specific fix instructions
6. Run `cargo clippy` for lint checks (optional but recommended)

**Artifacts Produced:**
- Build success confirmation
- Test pass/fail results
- Acceptance criteria verification results
- Fix list (if returning to Phase 3)

**Exit Condition:** `cargo build` succeeds. `cargo test` passes. All
acceptance criteria verified. `validate_world()` passes in debug builds.

**Handoff to Phase 5:** Pass verification results for recording.

---

## Phase 5: Validation & Recording

**Objective:** Record the feature as complete in `progress.jsonl` and
verify no regressions were introduced.

**Entry Condition:** Testing phase confirms all criteria pass.

**Activities:**
1. Mark all acceptance criteria as `[x]` in `features.md`
2. Update `progress.jsonl` with:
   - `"status": "complete"`
   - `"phase": "done"`
   - `"completed_at": "<ISO 8601 timestamp>"`
   - `"notes"` summarizing what was implemented
3. Run full test suite (`cargo test`) to check for regressions
4. If regressions found, do NOT mark complete -- return to Phase 3

**Artifacts Produced:**
- Updated `progress.jsonl` entry
- Full test suite pass confirmation
- Optional: commit message for version control

**Exit Condition:** Feature is recorded as complete. No regressions.
Ready to select next feature.

---

## progress.jsonl Schema

`progress.jsonl` is a JSON Lines file (one valid JSON object per line) that
persists feature progress across sessions. Each feature in `features.md` has
exactly one corresponding line.

**Fields:**

```json
{
  "feature_id": "CORE-001",
  "status": "pending|in_progress|complete",
  "phase": "research|planning|implementation|testing|validation|done|null",
  "started_at": "2026-02-14T10:30:00Z or null",
  "completed_at": "2026-02-14T11:45:00Z or null",
  "notes": "Free-text summary of work done, or null"
}
```

**Update protocol:**

- **Starting a feature:** Set `status` to `"in_progress"`, `phase` to
  `"research"`, `started_at` to current ISO 8601 timestamp.
- **Phase transition:** Update `phase` to the new phase name.
- **Completing a feature:** Set `status` to `"complete"`, `phase` to `"done"`,
  `completed_at` to current ISO 8601 timestamp, `notes` to a summary.
- **Session resumption:** Read the file, find lines with
  `status: "in_progress"` to identify work to resume. The `phase` field
  tells you which phase to re-enter.

**Mechanics:** To update a feature's entry, find the line containing its
`feature_id` and replace the entire line with the updated JSON object.
Each line is self-contained -- no line depends on any other line's content.

**Invariant:** Every `feature_id` in `features.md` has exactly one line in
`progress.jsonl`, and vice versa.

---

## Phase Transition Rules

1. **No skipping phases.** Every feature goes through all five phases,
   even if a phase is trivial (e.g., a prohibition feature may have
   a trivial Implementation phase).

2. **No combining phases.** Do not research while implementing. Do not
   plan while testing. Each phase has a single focus.

3. **Clean handoffs only.** Pass distilled artifacts between phases, not
   full conversation history or raw file contents.

4. **Backward transitions allowed.** Phase 4 (Testing) may send work
   back to Phase 3 (Implementation) with specific fix instructions.
   Phase 3 may send work back to Phase 2 (Planning) if the plan is
   insufficient. Always re-enter at the correct phase, not earlier.

5. **One feature at a time.** Do not start a new feature while another
   is in-progress (exception: blocked features may be paused while
   dependencies are completed).

6. **Dependency order.** Before starting a feature, verify all features
   listed in its "Dependencies" are complete. If not, complete
   dependencies first.

---

## Phase-Feature Matrix

| Feature Domain       | Research Depth | Planning Depth | Implementation Depth | Testing Depth |
|----------------------|----------------|----------------|----------------------|---------------|
| Core Architecture    | Deep           | Moderate       | Heavy                | Moderate      |
| Entity Lifecycle     | Moderate       | Moderate       | Heavy                | Heavy         |
| Systems Framework    | Moderate       | Light          | Heavy                | Heavy         |
| Main Loop Phases     | Light          | Moderate       | Heavy                | Moderate      |
| Checklists           | Light          | Light          | N/A (process only)   | Light         |
| Events               | Moderate       | Moderate       | Moderate             | Moderate      |
| TileMap              | Moderate       | Moderate       | Heavy                | Moderate      |
| Data Pipeline        | Moderate       | Moderate       | Heavy                | Moderate      |
| RNG                  | Light          | Light          | Light                | Heavy         |
| Components           | Light          | Light          | Moderate             | Light         |
| Existing Systems     | Moderate       | Moderate       | Heavy                | Heavy         |
| Testing/Validation   | Deep           | Moderate       | Heavy                | Self-referential |
| Code Invariants      | Deep           | Light          | Light (enforcement)  | Heavy         |
| Prohibitions         | Light          | Light          | N/A (audit only)     | Heavy (grep)  |
| Growth Patterns      | Moderate       | Moderate       | Triggered            | Light         |
| Rendering            | Deep           | Moderate       | Heavy                | Moderate      |
| Project Structure    | Light          | Light          | Light                | Light         |
| Main Loop Integration| Moderate       | Moderate       | Heavy                | Moderate      |
