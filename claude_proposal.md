# CLAUDE.md Addendum: Workflow System

> Proposed addition to CLAUDE.md. Do not merge until reviewed and approved.

---

## .workflow/ System

The `.workflow/` directory contains the structured workflow system for tracking
feature implementation across agent sessions. It consists of three files:

### .workflow/features.md -- Immutable Feature Contract

The complete feature specification for the Wulfaz engine. Every capability,
rule, pattern, prohibition, and growth trigger described in CLAUDE.md is
captured as a discrete feature with a unique ID, acceptance criteria, and
dependency graph.

**69 features** across 18 domains:
- Core Architecture (CORE-001 through CORE-005)
- Entity Lifecycle (LIFE-001 through LIFE-004)
- Systems Framework (SYS-001 through SYS-004)
- Main Loop Phases (PHASE-001 through PHASE-007)
- Adding New Components (ADD-001 through ADD-003)
- Events (EVT-001 through EVT-003)
- TileMap (TILE-001 through TILE-002)
- Data Pipeline (DATA-001 through DATA-002)
- Deterministic RNG (RNG-001 through RNG-002)
- Components (COMP-001)
- Existing Systems (ESYS-001 through ESYS-006)
- Testing & Validation (TEST-001 through TEST-003, VALID-001)
- Code Invariants (RULE-001 through RULE-003)
- Prohibitions (PROHIB-001 through PROHIB-011)
- Growth & Scaling (GROW-001 through GROW-003)
- Rendering (REND-001 through REND-006)
- Project Structure (STRUCT-001 through STRUCT-002)
- Main Loop Integration (LOOP-001)

**Immutability rules for agents:**
- MAY set a feature's status to `[x]` after all acceptance criteria are verified
- MAY NOT delete features from the list
- MAY NOT modify acceptance criteria or descriptions
- MAY NOT mark features as "not applicable"
- If a feature seems impossible, escalate to the user -- do NOT skip it

### .workflow/phases.md -- Work Methodology

Defines the five-phase workflow for implementing any feature:

1. **Research** -- understand context, read code, identify constraints
2. **Planning** -- design the approach, plan file changes, write test plan
3. **Implementation** -- write code, following all architectural rules
4. **Testing** -- cargo build, cargo test, validate_world(), criterion check
5. **Validation & Recording** -- update progress.jsonl, verify no regressions

Phase rules:
- No skipping phases
- No combining phases (prevents context contamination)
- Clean handoffs: pass distilled artifacts, not full context
- Backward transitions allowed (Testing -> Implementation for fixes)
- One feature at a time
- Dependency order: complete dependencies before starting a feature

### .workflow/progress.jsonl -- State Tracker

JSON Lines file with one line per feature. Persists progress across sessions.
Each line tracks:

```json
{
  "feature_id": "CORE-001",
  "status": "pending|in_progress|complete",
  "phase": "research|planning|implementation|testing|validation|done|null",
  "started_at": "ISO 8601 timestamp or null",
  "completed_at": "ISO 8601 timestamp or null",
  "notes": "string or null"
}
```

**Reading progress:** At session start, read `progress.jsonl` to understand
what is done, what is in progress, and what is next.

**Updating progress:** When starting a feature, update its line with
`status: "in_progress"`, the current phase, and `started_at`. When complete,
set `status: "complete"`, `phase: "done"`, and `completed_at`.

**Resumption:** If a session ends mid-feature, the progress file shows exactly
which feature and phase to resume from.

## How the Contract Was Built

The feature contract was built using the **Concentric Rings Method**:

1. **Pass 1 (Enumerate):** Listed every feature/capability as a one-liner
   extracted from CLAUDE.md and source code. Goal was completeness, not depth.
   Ended with a gap check.

2. **Pass 2 (Shape):** For each feature, defined inputs, outputs, and
   dependencies. Still no implementation detail. Ended with a gap check.

3. **Pass 3 (Specify):** For each feature, defined edge cases, validation
   rules, error states, and integration points. Ended with a gap check.

4. **Pass 4 (Refine):** Cross-checked all features against each other for
   gaps, conflicts, and missing glue. Verified dependency graph consistency.
   Ended with a gap check.

Each pass covered ALL features before proceeding to the next pass. No feature
was explored deeper than the current pass allowed. This prevents the common
failure mode of going deep on one feature while leaving others as stubs.

The audit trail for all four passes and their gap checks is preserved at the
bottom of `features.md`.

## Using the Workflow System

**Starting a session:**
1. Read `.workflow/progress.jsonl` to see current state
2. Find the next pending feature (respect dependency order)
3. Follow the five phases in `.workflow/phases.md`
4. Update progress.jsonl at each phase transition

**Feature selection order:**
- Start with features that have no dependencies (CORE-002, CORE-003, etc.)
- Then features whose dependencies are all complete
- Within a dependency tier, prefer lower-numbered features

**Measuring progress:**
- Count features by status in progress.jsonl
- `pending / in_progress / complete` gives a clear picture
- All 69 features at `complete` = project fully implemented
