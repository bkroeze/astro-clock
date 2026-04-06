# T03: 03-query-system 03

**Slice:** S03 — **Milestone:** M001

## Description

Implement retrograde period query (QUERY-03), exact aspect query (QUERY-04), and performance benchmarks (QUERY-05) to verify sub-100ms query times.

Purpose: Complete the query system with retrograde detection, aspect search, and performance verification.
Output: retrograde.rs, aspects.rs, benchmark.rs modules and RetrogradePeriod schema type.

## Must-Haves

- [ ] Retrograde query returns periods with status calculation
- [ ] Retrograde query supports planet filtering
- [ ] Exact aspect query filters by orb threshold and aspect types
- [ ] Exact aspect query supports body pair filtering
- [ ] Wedding query completes in under 100ms for 60-day ranges (benchmark verified)

## Files

- `src/queries/retrograde.rs`
- `src/queries/aspects.rs`
- `src/queries/mod.rs`
- `src/queries/benchmark.rs`
- `src/database/schema.rs`
