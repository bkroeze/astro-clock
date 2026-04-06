# M001: Migration

**Vision:** A CLI application that generates astrological natal charts ("Chart of Now") and provides electoral astrology query capabilities for finding optimal timing.

## Success Criteria


## Slices

- [x] **S01: Database Schema** `risk:medium` `depends:[]`
  > After this: Create TimescaleDB hypertables and supporting tables for astrological time-series data.
- [x] **S02: Data Loading** `risk:medium` `depends:[S01]`
  > After this: Create compact chunk data structures and add LRU cache dependency.
- [x] **S03: Query System** `risk:medium` `depends:[S02]`
  > After this: Create the core query infrastructure including shared types, error handling, and criteria structs for all electoral astrology queries.
- [x] **S04: Performance** `risk:medium` `depends:[S03]`
  > After this: Implement multi-resolution storage using TimescaleDB continuous aggregates to reduce database storage by storing outer planets at lower resolution (60-minute) while maintaining 1-minute resolution for Moon and 5-minute for inner planets.
- [x] **S05: Job Infrastructure** `risk:medium` `depends:[S04]`
  > After this: Create database schema for the job system including the jobs table and loaded_days tracking table.
- [x] **S06: Data Loading** `risk:medium` `depends:[S05]`
  > After this: Create LoadJobHandler — a JobHandler trait implementation that orchestrates day-level incremental planetary data loading.
- [x] **S07: Named Queries** `risk:medium` `depends:[S06]`
  > After this: Create QueryJobHandler and QueryTemplateRegistry to enable named query execution as jobs with automatic data loading and structured JSON results.
