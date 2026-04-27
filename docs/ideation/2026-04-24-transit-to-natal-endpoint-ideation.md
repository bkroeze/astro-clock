---
date: 2026-04-24
topic: transit-to-natal-json-endpoint
focus: Create a new endpoint for transit to natal calculations, returning JSON
mode: repo-grounded
---

# Ideation: Transit-to-Natal JSON Endpoint

## Grounding Context

**Codebase Context:**
- Rust-based astro-clock app with axum HTTP server
- Existing query endpoints: `/api/v1/query/{wedding,project,travel}` with sync/async modes
- Job-based execution system with PostgreSQL persistence
- Database schema: planet_positions, aspect_summaries, lunar_conditions, retrograde_periods
- **Key insight:** System does electional astrology (finding dates), NOT natal charts yet

**Past Learnings:**
- Dual execution modes (sync/async) are standard for all job-creating endpoints
- Validation via `validate()` methods on criteria structs
- Response wrappers: QuerySyncResponse, QueryAsyncResponse, ErrorResponse
- Date format: YYYY-MM-DD, max 366 days
- QueryTemplateRegistry pattern for registering new query types

---

## Ranked Ideas

### 1. Natal Chart Registry with Caching

**Description:**  
POST once to register a natal chart with birth data (date, time, location), receive a `natal_id` UUID. Server caches computed house cusps and planet longitudes. All subsequent transit queries reference this ID instead of resending birth data.

**Rationale:**  
- Natal charts are immutable—no need to recalculate
- Reduces payload size for repeat queries
- Enables server-side caching and optimization
- Fits existing job-based architecture naturally
- **Compounding effect:** Enables subscription patterns, push notifications, and user profiles

**Downsides:**  
- Requires new database table for natal chart storage
- Adds complexity (CRUD operations for charts)
- Privacy considerations for birth data

**Confidence:** 90%  
**Complexity:** Medium  
**Status:** Unexplored

---

### 2. Aspect-First Query: "When Does Transit Occur"

**Description:**  
Invert the query model: instead of "what transits on date X," user specifies `natal_planet`, `transit_planet`, `aspect_type` and receives all dates in range when that exact aspect perfects. Example: "When will Saturn conjunct my natal Sun in the next 2 years?"

**Rationale:**  
- This is the #1 user question in astrology
- More actionable than scanning all transits
- Can leverage existing `aspect_summaries` aggregation pattern
- Enables long-range planning (Saturn returns, etc.)

**Downsides:**  
- Requires different query pattern than existing date-range queries
- May need to scan large date ranges
- Less useful for "what's happening today" use case

**Confidence:** 85%  
**Complexity:** Medium  
**Status:** Unexplored

---

### 3. Pre-Computed Transit Windows for Outer Planets

**Description:**  
When a natal chart is registered, pre-calculate and store date ranges when outer planets (Jupiter+) are within orb of natal points. Store in `transit_windows` table with start/end dates, orb min/max. Enables instant answers for "Am I in a Saturn return window?"

**Rationale:**  
- Outer planet transits last months/years—pre-computation is efficient
- Saturn return, Jupiter return are most asked-about transits
- Enables "upcoming transits" queries without scanning ephemeris
- Follows existing `retrograde_periods` caching pattern

**Downsides:**  
- Storage overhead (but manageable—outer planets only)
- Requires background job on chart registration
- Inner planets still need on-demand calculation

**Confidence:** 85%  
**Complexity:** Medium  
**Status:** Unexplored

---

### 4. Date-Only Transit API with Temporal Orbs

**Description:**  
Accept birth date only (no time) for users who don't know exact birth time. Use 24-hour orb windows for all transits, returning "sometime on this date" with confidence intervals. Critical because 30-40% of users lack exact birth times.

**Rationale:**  
- Expands addressable user base significantly
- Many astrology apps fail users without birth times
- Temporal orbs are a pragmatic solution
- Can still provide value (just less precise)

**Downsides:**  
- Less precise results
- Requires clear communication about uncertainty
- House-based transits impossible without time

**Confidence:** 90%  
**Complexity:** Low  
**Status:** Unexplored

---

### 5. Streaming Transit Response with SSE

**Description:**  
Instead of sync blocking or async polling, open a Server-Sent Events (SSE) connection and stream transit results as they're calculated. Client receives partial results immediately, with more arriving over time. Best for long date ranges.

**Rationale:**  
- Better UX than polling—user sees progress
- Enables progressive rendering in UI
- More efficient for large result sets
- Natural fit for Rust's async ecosystem

**Downsides:**  
- More complex client implementation
- Connection management overhead
- Not all clients support SSE

**Confidence:** 75%  
**Complexity:** Medium-High  
**Status:** Unexplored

---

### 6. Batch Transit Processor

**Description:**  
Accept an array of 100+ natal charts and a date range. Returns transit analysis for all charts in a single job. Optimized for electional astrology services comparing multiple candidates (wedding planners, travel agencies, event coordinators).

**Rationale:**  
- High-value use case for B2B/enterprise
- Reduces API call overhead significantly
- Efficient for comparison scenarios
- Fits existing job system well

**Downsides:**  
- Rate limiting and quota considerations
- Large result payload
- Database contention with many charts

**Confidence:** 70%  
**Complexity:** Medium  
**Status:** Unexplored

---

### 7. Transit Response with Metadata Enrichment

**Description:**  
Transit JSON response includes not just aspect data, but derived metadata: transit "strength" score (orb tightness × planet weight × applying vs separating), house position in natal chart, and zodiac sign of transiting planet.

**Rationale:**  
- Richer responses enable smarter client-side filtering
- Reduces need for follow-up queries
- Enables ranking and prioritization
- Can iterate on scoring algorithm without API changes

**Downsides:**  
- Increases response size
- Scoring algorithm is opinionated
- More computation per transit

**Confidence:** 80%  
**Complexity:** Low-Medium  
**Status:** Unexplored

---

## Rejection Summary

| # | Idea | Reason Rejected |
|---|------|-----------------|
| 1 | Progressed Natal Anchoring | Too vague, unclear implementation |
| 2 | Timeline Projection | Abstract, not actionable |
| 3 | Semantic Transit Layers | Better as brainstorm variant |
| 4 | Weighted Transit Scoring | Requires ML, too complex for MVP |
| 5 | Auto-Orb Calculation | Duplicated by #20 (more specific) |
| 6 | Transit Query Template | Just follows existing pattern |
| 7 | Infinite Storage Mode | Storage cost prohibitive |
| 8 | Machine-Optimized Binary | Complexity not justified |
| 9 | Instant-Only Endpoint | 50ms constraint unrealistic |
| 10 | Firehose All Transits | Poor UX, overwhelming data |
| 11 | Weather Grid Pattern | Overly complex for domain |
| 12 | Physics Snapshot Chain | Overly complex for domain |
| 13 | Intensity Curves | Complex visualization, not core |
| 14 | Transit Aspect Patterns | Can add later, not core endpoint |

---

## Cross-Cutting Combinations

1. **Registry + Caching + Pre-computation:** Combine ideas #1, #3 for powerful caching strategy
2. **Auto-Scoring + Metadata:** Combine idea #7 for rich, ranked responses
3. **Date-Only + Temporal Orbs:** Combine idea #4 for accessible UX
4. **Streaming + SSE:** Combine idea #5 for progressive UX
5. **Aspect-First + Batch:** Combine ideas #2, #6 for enterprise queries

---

## Recommended MVP

Start with:
- **#1 (Natal Registry)** - Foundation for caching and user management
- **#4 (Date-Only Support)** - Maximizes user accessibility
- **#7 (Metadata Enrichment)** - Enables rich client experiences

These three combine into a cohesive MVP that can later expand with #2, #3, #5, #6.
