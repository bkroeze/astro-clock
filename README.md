---
date: "2026-02-21"
tags: [astrology,project]
description: "Notes and documentation for my astrology clock app"
---

# Base Idea

A cli app that generates a graphic of the current "Chart of Now", which is the
natal chart for a child born at X instant, defaulting time-of-execution

This will also require the lat/long for the chart, which can be read from a
config file or env value, or else overridden with a switch

## Tech Stack

- Rust
- [swiss-eph](https://crates.io/crates/swiss-eph) library for ephemeris lookup
- Posgresql db, with ??? add-on for time-seq
- simple http server option, with an endpoint to return webp files (need library choice)
- Unsure what graphics library to use


