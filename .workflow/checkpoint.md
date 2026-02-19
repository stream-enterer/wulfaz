# Checkpoint

## Active Task
None

## Completed
A07a — Occupant normalization improvements — COMPLETE (7afca67)

- Fixed year extraction: read `publication_date` as f64 instead of String
- Added abbreviation expansions: boul./bd. → boulevard, av. → avenue
- Added prefix strips: avenue, allee, faubourg, cite (with article variants)
- Updated existing tests, added 4 new normalization tests
- Filtered occupant loading to 1845–1855 (best Vasserot overlap)
- Final match rates: 1845 40.1%, 1850 37.1%, 1855 38.0% (52,909 matched)
- 50% target not reachable — gap is data coverage, not normalization

## Next Action
Pick next task from backlog.
