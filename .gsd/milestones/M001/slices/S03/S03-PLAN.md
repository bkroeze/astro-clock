# S03: Query System

**Goal:** Create the core query infrastructure including shared types, error handling, and criteria structs for all electoral astrology queries.
**Demo:** Create the core query infrastructure including shared types, error handling, and criteria structs for all electoral astrology queries.

## Must-Haves


## Tasks

- [x] **T01: 03-query-system 01** `est:5min`
  - Create the core query infrastructure including shared types, error handling, and criteria structs for all electoral astrology queries.

Purpose: Establish the foundation for specialized query functions with consistent interfaces, proper error handling, and metadata tracking.
Output: src/queries/ module with types.rs, error.rs, and mod.rs exports.
- [x] **T02: 03-query-system 02** `est:5min`
  - Implement wedding date query (QUERY-01) and void-of-course Moon period query (QUERY-02) with optimized SQL using aspect_summaries table.

Purpose: Enable electoral astrology queries for finding optimal wedding dates and detecting VoC Moon periods.
Output: wedding.rs and voc.rs query modules with async query functions.
- [x] **T03: 03-query-system 03** `est:5min`
  - Implement retrograde period query (QUERY-03), exact aspect query (QUERY-04), and performance benchmarks (QUERY-05) to verify sub-100ms query times.

Purpose: Complete the query system with retrograde detection, aspect search, and performance verification.
Output: retrograde.rs, aspects.rs, benchmark.rs modules and RetrogradePeriod schema type.

## Files Likely Touched

- `src/queries/mod.rs`
- `src/queries/types.rs`
- `src/queries/error.rs`
- `src/lib.rs`
- `src/queries/wedding.rs`
- `src/queries/voc.rs`
- `src/queries/mod.rs`
- `migrations/005_create_retrograde_periods.sql`
- `src/queries/retrograde.rs`
- `src/queries/aspects.rs`
- `src/queries/mod.rs`
- `src/queries/benchmark.rs`
- `src/database/schema.rs`
