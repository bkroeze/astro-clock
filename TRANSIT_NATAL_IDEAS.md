# Transit-to-Natal API: Assumption-Breaking Ideas

## Current System Context
- **Body enum**: Sun(0) through Pluto(9)
- **AspectType enum**: Conjunction, Sextile, Square, Trine, Opposition
- **Query pattern**: Criteria structs → SQL → QueryResult<T>
- **Job system**: Sync/async execution with job registry
- **Database**: planet_positions, aspect_summaries, lunar_conditions, retrograde_periods

---

## Idea 1: The Natal Chart is Not Fixed — "Progressed Natal Anchoring"

**Summary**: Instead of calculating transits to a static birth chart, allow the natal chart to evolve through secondary progressions. The endpoint accepts a `progression_method` parameter that moves the natal chart forward at different rates (day-for-year, true solar arc, etc.), calculating transits to the *progressed* positions.

**Why It Matters**: Traditional APIs treat the natal chart as frozen in time, but psychological astrology recognizes that the birth chart unfolds and matures. A 40-year-old's transits to their 40-year-old progressed Sun carry more weight than transits to their newborn Sun. This reframes transits as a dialogue between evolving inner nature and external circumstances.

**Evidence/Grounding Hooks**:
- Build on existing `planet_positions` table (can store progressed positions as computed columns)
- Leverage `Body` enum's `from_id()`/`to_id()` pattern for body mapping
- Extends existing `QueryResult<T>` pattern with optional `progression_context` field
- Follows established pattern of `WeddingCriteria` with builder methods like `.with_progression_method()`

---

## Idea 2: Aspects Are Continuous, Not Binary — "Orb-Weighted Intensity Curves"

**Summary**: Return transit data as intensity curves rather than discrete aspect events. Instead of "Mars square natal Venus on 2024-06-15," return a time series showing the waxing/waning intensity of the aspect from first entry into orb through exactness to final exit, with per-hour intensity values.

**Why It Matters**: Traditional APIs force consumers to reconstruct aspect narratives from isolated data points. Astrologers think in terms of "building toward" and "separating from" — an aspect is an arc, not a point. This treats transits as weather systems with fronts and pressure gradients, not binary switches.

**Evidence/Grounding Hooks**:
- Database already stores positions at any time granularity (not just daily)
- `AspectConfig.orb` pattern in `aspects.rs` provides the math foundation
- Can reuse `angular_distance()` and `check_aspect()` functions from `aspects.rs`
- Fits `QueryResult` pattern with `data` field containing time-series arrays
- Aligns with existing `ExactAspect` struct (applying/separating field already exists)

---

## Idea 3: Bodies Have Relationship Histories — "Aspect Memory & Cycle Tracking"

**Summary**: Each transit result includes the complete history of that specific body-to-natal-body relationship: when was the last conjunction? How many times has Saturn squared this natal Moon in the person's life? What's the phase of their synodic cycle? Return the transit as a chapter in an ongoing story.

**Why It Matters**: Astrology is deeply narrative — Saturn's return isn't just a transit, it's a 29-year life chapter completion. Traditional APIs treat each transit as an isolated event, but the meaning comes from context. A Saturn square hits differently if it's the first time vs. the third time in a lifetime.

**Evidence/Grounding Hooks**:
- `RetrogradePeriod` table already tracks cyclic data with shadow periods
- Can extend `ExactAspect` struct with `cycle_number`, `phase_in_cycle` fields
- Database schema supports historical queries (planet_positions has full time range)
- Follows pattern of `WeddingCandidate` carrying derived metadata
- Job system can pre-compute and cache cycle data (similar to `ChunkGenerator`)

---

## Idea 4: Orbs Are Contextual, Not Universal — "Dynamic Orb Resolution"

**Summary**: Instead of a fixed orb (e.g., 3° for all aspects), calculate orbs dynamically based on multiple factors: the body's speed (faster = tighter orb), the aspect type (conjunctions get wider orbs), the house placement, and whether it's applying or separating. Return both the calculated orb threshold and the confidence score.

**Why It Matters**: Traditional astrology software uses rigid orb tables (3° for major aspects, 1° for minor), but real-world observation shows orbs should vary. A fast-moving Moon should have a tighter orb than slow-moving Pluto. This moves from rule-based to observational orb calculation.

**Evidence/Grounding Hooks**:
- `AspectConfig.orb` in `aspects.rs` already parameterized — can become dynamic
- Body enum has speed data available in ephemeris
- Can add `orb_calculation_context` to result metadata
- Extends existing `orb` field in `ExactAspect` with `orb_methodology` enum
- Pattern similar to `RetrogradeStatus` (Direct/Retrograde/PreShadow/PostShadow) for orb sources

---

## Idea 5: Transits Are Multi-Body Configurations — "Transit Aspect Patterns"

**Summary**: Instead of returning individual transits (transiting Mars square natal Venus), return *configurations* — moments when multiple transiting bodies form recognizable patterns with the natal chart (Grand Cross to natal angles, T-square involving natal Sun). The unit of analysis is the pattern, not the pair.

**Why It Matters**: Astrologers don't read transits one at a time — they look for clusters and patterns. Traditional APIs atomize what should be holistic. A Mars-Pluto conjunction both aspecting natal Sun is exponentially more significant than either transit alone. This reframes the API as a pattern detector, not a pair calculator.

**Evidence/Grounding Hooks**:
- `find_grand_trines()` in `aspects.rs` already implements pattern detection
- `GrandTrine` struct provides pattern representation template
- Database `aspect_summaries` table already aggregates aspect counts per time
- Can extend `AspectAnalysis` struct to include `transit_configurations` field
- Job system can batch-compute pattern detections (similar to `QueryTemplateRegistry`)

---

## Idea 6: Time is Relative — "Transit Timeline Projection"

**Summary**: Accept multiple timeline perspectives in a single request: chronological (linear time), cyclical (where are we in the Saturn cycle?), and event-based (transits triggered by other transits — "when Mars activates the Uranus square"). Return results organized by timeline type.

**Why It Matters**: Traditional APIs assume users want "what's happening on June 15th," but astrologers often think cyclically ("where am I in my Saturn return?") or in terms of trigger chains ("when does the Mars transit activate the longer Uranus-Pluto square?"). This treats time as multi-dimensional rather than linear.

**Evidence/Grounding Hooks**:
- `RetrogradeStatus` shows the system already tracks phase-based states
- Can add `TimelinePerspective` enum similar to `TravelPurpose`
- `QueryJobResult` can be extended with `timeline_metadata` field
- Chunking system already supports time-series data generation
- Pattern follows existing `WeddingCriteria` → `QueryResult` flow with timeline parameter

---

## Idea 7: Interpretation is Data — "Semantic Transit Layers"

**Summary**: Return transits with multiple interpretation layers that can be toggled: traditional (dignities, sect, hayz), modern (psychological, evolutionary), mundane (world-events correlation), or technical (pure geometry). Each layer adds fields without changing the core calculation.

**Why It Matters**: Traditional APIs separate calculation from interpretation — they give positions and expect clients to interpret. But interpretation rules are algorithmic (Saturn in Capricorn = Saturn strengthened). Including optional semantic layers acknowledges that "what is happening" and "what it means" are both computable and should be decoupled but available.

**Evidence/Grounding Hooks**:
- `ZodiacSign` enum already has sign characteristics (favorable_for_wedding)
- `AspectType` has `is_favorable()` and `is_challenging()` methods
- Can add `interpretation_layer` parameter to criteria (like `TravelPurpose`)
- `QueryResult` metadata pattern supports optional enrichment fields
- Extends existing approach of computed fields in query results

---

## Idea 8: The User Defines Importance — "Weighted Transit Scoring"

**Summary**: Allow users to provide a personal weight map — "my natal Venus is super important, Mercury less so" — and return transits scored by personal relevance rather than astrological textbook importance. Include an API to learn weights from user feedback over time.

**Why It Matters**: Traditional APIs apply uniform importance rules (outer planet transits matter more), but importance is contextual. A natal chart with Venus ruling the Ascendant makes Venus transits crucial. This moves from one-size-fits-all astrology to personalized scoring, treating the API as an adaptive system.

**Evidence/Grounding Hooks**:
- `WeddingCriteria` with `min_venus_aspects` shows weighted scoring pattern exists
- Can add `body_weights` hashmap parameter to criteria
- `aspect_summaries` table structure supports weighted aggregations
- `QueryResult` can include `personalized_score` alongside `favorable_aspects`
- Job system can cache personalized calculations (similar to `ensure_data_loaded` pattern)

---

## Summary Table

| Idea | Assumption Broken | Reframe |
|------|------------------|---------|
| 1. Progressed Anchoring | Natal chart is static | Natal chart evolves, transits dialogue with evolved self |
| 2. Intensity Curves | Aspects are binary events | Aspects are continuous weather systems |
| 3. Aspect Memory | Transits are isolated | Transits are chapters in ongoing cycles |
| 4. Dynamic Orbs | Orbs are universal constants | Orbs are contextual and calculated |
| 5. Transit Patterns | Unit of analysis is body pair | Unit of analysis is multi-body configuration |
| 6. Timeline Projection | Time is linear | Time is multi-dimensional (chronological, cyclical, event-based) |
| 7. Semantic Layers | Calculation ≠ Interpretation | Both are algorithmic and composable |
| 8. Weighted Scoring | Importance is universal | Importance is personal and learnable |

---

## Implementation Priority (Based on Codebase Fit)

1. **Idea 2 (Intensity Curves)** — Fits cleanly into existing `ExactAspect` + `AspectConfig` patterns
2. **Idea 4 (Dynamic Orbs)** — Extends existing `orb` parameterization naturally
3. **Idea 5 (Transit Patterns)** — Builds on `GrandTrine` + `find_grand_trines()` foundation
4. **Idea 7 (Semantic Layers)** — Leverages existing `is_favorable()` / `favorable_for_wedding()` patterns
5. **Idea 8 (Weighted Scoring)** — Extends `min_venus_aspects` pattern to general weights
6. **Idea 3 (Aspect Memory)** — Requires new table but follows `RetrogradePeriod` pattern
7. **Idea 1 (Progressed Anchoring)** — Requires computed position storage
8. **Idea 6 (Timeline Projection)** — Most architecturally different, fundamental reframing
