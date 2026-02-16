# CDDA Reference Documents — Evaluation

## Overall Verdict: Useful but uneven. ~65% implementable without C++ source.

### What's genuinely good

Documents 00, 03, 09, 10 are excellent. The dependency graph, submap/coordinate system, supporting systems (with concrete failure modes), and antipattern checklist are the kind of documentation that actually prevents weeks of wasted work. Document 10 alone justifies the project — 37 specific traps with tier classifications and "WHY IT'S HIDDEN" explanations.

Data model coverage is comprehensive. All types, fields, flags, cross-references. 154 STUB OK callouts, 18 SCOPE BOUNDARY markers, 24 PORTING TRAP warnings. Someone reading these will know what to load, what to stub, and what to skip.

The tiering system works. Tier 0/1/2/3 + EXCLUDED gives clear implementation priority.

### What's weak

**Algorithms are hand-waved.** This is the critical gap. The documents describe *what* data exists but not *how* key generation algorithms work:

- **City street layout** (04): "iterate... 1-in-4 chance to place building" — no pseudocode, no function names
- **Overmap special placement scoring** (04): mentioned but not detailed
- **Field propagation** (08/09): "Dijkstra-like priority queue" — no pseudocode for `percent_spread` mechanics
- **Highway generation** (04): "Bezier curves" mentioned, no algorithm shown
- **Forest biome composition** (05): sequence/chance interaction unclear

A developer will hit a wall at "I've loaded all the data, now how do I generate a city?" and need to read the C++ source anyway.

**Some spec requirements missed:**
- UNCERTAIN callouts: spec requires "at least 1" — only 1 found (barely passes)
- UI/UX DECISION callouts: spec requires "at least 5" — only 1 found (fails)
- The spec's "Index Entry Format" template (fields table with Required/References/Wulfaz scope columns) was not followed consistently
- No finalization dependency graph as a standalone section
- Serialization format completely absent
- Error handling strategy (missing references -> crash? log? silent?) undocumented

### Signal-to-noise issues

- **01_JSON_TYPE_REGISTRY** has too much filler — example JSON snippets that anyone can read from source files
- **02_TERRAIN_AND_FURNITURE** lists all 139 flags without prioritization ("implement these 15 on day one, stub these 84")
- **07_PALETTE_SYSTEM** includes a palette directory (content inventory, not system documentation)
- **04_OVERMAP_GENERATION** reads like design notes, not implementation reference

### Most likely failure mode

Data loads cleanly -> mapgen rows parse -> buildings render as blank rectangles (palette composition subtlety wrong) -> fix palettes -> cities are empty (building_bin/region_settings wiring missed) -> fix that -> roads don't connect to buildings (connection placement algorithm not documented) -> stuck, must read C++ source.

The gap is always at the algorithm layer, never at the data model layer.

### What I'd change if redoing this

1. Move 10_ANTIPATTERN_CHECKLIST to position 01 — read traps before reading systems
2. Add pseudocode for city generation, overmap placement, field propagation
3. Add a "Day One Implementation Order" document: what 15 things to build first
4. Cut 30% of 01/02/05/07 (remove padding, example JSON, content inventories)
5. Use 09's format (What/Algorithm/Failure Mode/Source) everywhere
6. Add 5+ UI/UX DECISION callouts per spec requirement
7. Document finalization order as a standalone dependency graph

### Per-document ratings

| Doc | Actionability | Completeness | Accuracy | Signal/Noise | Notes |
|-----|---------------|--------------|----------|--------------|-------|
| 00 | 5/5 | 4/5 | 5/5 | 5/5 | Best opening doc. Read first or waste weeks. |
| 01 | 3/5 | 4/5 | 5/5 | 2/5 | Useful as reference, too much filler. Cut 40%. |
| 02 | 4/5 | 4/5 | 5/5 | 3/5 | Solid but dense. Needs "day one" priority split for 139 flags. |
| 03 | 5/5 | 5/5 | 5/5 | 4/5 | Best technical doc. Clear, concrete, actionable. |
| 04 | 2/5 | 3/5 | 4/5 | 2/5 | Hand-wavy algorithms. Reads like design notes, not impl reference. |
| 05 | 3/5 | 4/5 | 4/5 | 3/5 | Comprehensive taxonomy but weak on runtime behavior. |
| 06 | 4/5 | 4/5 | 5/5 | 4/5 | Very good. Execution flow summary should be at the top. |
| 07 | 4/5 | 3/5 | 5/5 | 4/5 | Good but missing error cases. Palette directory is noise. |
| 08 | 3/5 | 3/5 | 4/5 | 4/5 | Adequate for schema, weak for implementation. Feels rushed. |
| 09 | 4/5 | 4/5 | 5/5 | 5/5 | Excellent format. Every doc should use this structure. |
| 10 | 5/5 | 4/5 | 5/5 | 5/5 | Perfect. Should be document 01 not 10. |

### Bottom line

**For the stated purpose (consumed by a different LLM reimplementing in Rust):** the data model documentation is strong enough that the consuming LLM will correctly structure types, load data, and know what to stub. But it will fail on generation algorithms and need supplemental research into the C++ source for city generation, overmap placement, and field propagation. The antipattern checklist will save it from the most expensive mistakes.

**Grade: B.** Excellent schema docs, weak algorithm docs, some spec requirements missed.
