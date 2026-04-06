# T01: 03-query-system 01

**Slice:** S03 — **Milestone:** M001

## Description

Create the core query infrastructure including shared types, error handling, and criteria structs for all electoral astrology queries.

Purpose: Establish the foundation for specialized query functions with consistent interfaces, proper error handling, and metadata tracking.
Output: src/queries/ module with types.rs, error.rs, and mod.rs exports.

## Must-Haves

- [ ] Query module exists with public API
- [ ] QueryError enum handles database, invalid criteria, and timeout errors
- [ ] Criteria structs exist for all query types with validation
- [ ] Result types include metadata (execution time, rows examined)

## Files

- `src/queries/mod.rs`
- `src/queries/types.rs`
- `src/queries/error.rs`
- `src/lib.rs`
