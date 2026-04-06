# T06: 04-performance 06

**Slice:** S04 — **Milestone:** M001

## Description

Integrate the existing `is_major_aspect()` function into `calculate_aspects()` to filter minor aspects before storage, achieving the planned ~80% storage reduction.

Purpose: The aspect filtering infrastructure exists but is not connected. This gap closure connects the filtering logic to achieve the storage optimization goal.
Output: Modified chunk_generator.rs with integrated aspect filtering and no dead code warnings.

## Must-Haves

- [ ] "is_major_aspect() is called in calculate_aspects() to filter aspects"
- [ ] "Only aspects within 8° orb of major angles (0°, 60°, 90°, 120°, 180°) are stored"
- [ ] "Storage reduction of ~80% is achieved compared to storing all aspects"
- [ ] "No dead code warnings for unused MAJOR_ASPECT_ANGLES constant"
- [ ] "No dead code warnings for unused is_major_aspect() function"

## Files

- `src/database/chunk_generator.rs`
