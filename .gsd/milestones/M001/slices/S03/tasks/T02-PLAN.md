# T02: 03-query-system 02

**Slice:** S03 — **Milestone:** M001

## Description

Implement wedding date query (QUERY-01) and void-of-course Moon period query (QUERY-02) with optimized SQL using aspect_summaries table.

Purpose: Enable electoral astrology queries for finding optimal wedding dates and detecting VoC Moon periods.
Output: wedding.rs and voc.rs query modules with async query functions.

## Must-Haves

- [ ] Wedding query returns dates with favorable Moon signs and Venus aspects
- [ ] Wedding query excludes void-of-course periods
- [ ] Wedding query completes in under 100ms for 60-day ranges
- [ ] VoC query returns periods with start/end times and durations
- [ ] VoC query supports minimum duration filtering

## Files

- `src/queries/wedding.rs`
- `src/queries/voc.rs`
- `src/queries/mod.rs`
- `migrations/005_create_retrograde_periods.sql`
