# T02: 07-named-queries 02

**Slice:** S07 — **Milestone:** M001

## Description

Implement project and travel query functions with astrological criteria for finding favorable dates.

Purpose: Complete the set of named queries (wedding already exists) so users can find auspicious dates for starting projects and planning travel.
Output: src/queries/project.rs, src/queries/travel.rs, updated src/queries/types.rs and mod.rs

## Must-Haves

- [ ] find_project_dates() returns favorable dates for starting projects
- [ ] find_travel_dates() returns favorable dates for travel
- [ ] Both queries use Mercury direct + favorable Moon sign criteria
- [ ] ProjectCriteria and TravelCriteria structs with validation
- [ ] ProjectCandidate and TravelCandidate result structs

## Files

- `src/queries/project.rs`
- `src/queries/travel.rs`
- `src/queries/types.rs`
- `src/queries/mod.rs`
- `src/jobs/registry.rs`
