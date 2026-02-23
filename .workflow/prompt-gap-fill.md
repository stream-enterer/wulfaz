# Prompt: UI Review Gap Fill

Use this prompt after `ui-review-conversation-index.md` exists.

---

You have a completed UI architecture review comparing our UI framework against CK3's patterns. The review covers 10 areas defined in `.workflow/ui-review-ck3.md`. The conversation index at `.workflow/ui-review-conversation-index.md` maps each area to the conversation files containing its findings and implementation work.

Your job is to reconcile what the findings recommended against what was actually implemented.

## Process

For each of the 10 areas:

1. **Read the findings conversation(s)** listed in the conversation index. Extract every concrete recommendation — bugs, antipatterns, missing features, proposed fixes. Be exhaustive; don't summarize away specifics.

2. **Read the implementation conversation(s)** if any exist. Identify every code change that was made (file edits to `src/ui/` and related files).

3. **Read the current source files** touched by implementations to verify the changes actually landed (conversations may have had failed attempts or partial work).

4. **Classify each finding** into one of:
   - **DONE** — the recommended change is in the current codebase
   - **PARTIAL** — some aspect was implemented but the finding isn't fully addressed (explain what's missing)
   - **SKIPPED** — the finding was explicitly decided against during implementation (note why if the conversation says)
   - **NOT STARTED** — no implementation was attempted
   - **AMBIGUOUS** — you can't determine the status (see below)

## Handling ambiguity

If you encounter any of these, do NOT guess — flag them in a dedicated section:
- A finding that's vague enough that you can't tell if current code satisfies it
- A recommendation that contradicts another recommendation from a different area
- Code that partially matches a finding but may have been written for a different reason
- Findings where the "right" implementation approach is unclear

For each ambiguous item, state: what the finding says, what the code looks like now, and what specifically is unclear.

## Output

Write results to `.workflow/ui-review-gap-report.md`:

```
# UI Review Gap Report

## Area N: <Name>

### Findings & Status

| # | Finding | Status | Notes |
|---|---------|--------|-------|
| 1 | ...     | DONE   | implemented in <commit or conversation> |
| 2 | ...     | NOT STARTED | |

### Implementation summary
Brief description of what was changed and in which files.

(repeat for all 10 areas)

---

## Ambiguous Items

| Area | Finding | Current Code | What's Unclear |
|------|---------|--------------|----------------|
| ...  | ...     | ...          | ...            |

---

## Backlog Additions

For every NOT STARTED finding, draft a backlog entry following the format in `.workflow/backlog.md`. Use the `UI-D` (deferred) prefix and continue numbering from the highest existing UI-D entry. Each entry needs:
- Bold task ID and title
- One-line description of what CK3 does and what we're missing
- `Needs:` dependencies if any (reference existing task IDs)
- `Test:` one concrete assertion

For PARTIAL findings, draft the entry describing only the remaining work.

Do NOT add entries for SKIPPED or DONE findings.
Do NOT add entries that duplicate existing backlog items — check `.workflow/backlog.md` first and reference the existing ID if one covers the same work.

Append new entries under the `## Deferred` section of `.workflow/backlog.md`.
```

## Constraints

- Do not modify any source code. This is a reporting task only.
- Do not fabricate findings. Every finding must trace back to a specific conversation.
- When in doubt, flag as AMBIGUOUS rather than guessing DONE or NOT STARTED.
