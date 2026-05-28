# Example Charts

This directory contains example charts and documentation for the Astro Clock application.

## Generated Charts

Charts are generated in the following formats:

### PNG Charts
High-quality 800x800 pixel chart wheels with houses and planetary positions.

### WebP Charts
Compressed chart images for web use.

### Markdown Tables
Text-based tables showing planetary positions and aspects.

### SVG Charts
Standalone vector charts with embedded glyph paths.

## How to Generate Examples

```bash
# Generate PNG chart
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --output docs/example_sf.png

# Generate WebP chart
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format webp --output docs/example_sf.webp

# Generate markdown table
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format md --output docs/example_sf.md

# Generate SVG chart
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format svg --output docs/example_sf.svg

# Generate with specific date
cargo run --bin astro-clock -- chart \\
  --lat 40.7128 \\
  --lon -74.0060 \\
  --time "2026-03-15T14:30:00-04:00" \\
  --output docs/nyc_spring.png
```

Generate the standalone Astronomicon glyph assets with:

```bash
just make-svg
```

## Chart Types

### Natal Charts
Current planetary positions for a location:
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194
```

### Different House Systems

**Placidus (default):**
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --house Placidus
```

**Whole Sign:**
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --house Whole
```

**Koch:**
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --house Koch
```

**Equal:**
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --house Equal
```

### Aspect Analysis

Generate markdown with aspect analysis:
```bash
cargo run --bin astro-clock -- chart \\
  --lat 37.7749 \\
  --lon -122.4194 \\
  --format md \\
  --orb 5.0 \\
  --output docs/aspects_analysis.md
```

## Configuration

Create a config file to set defaults:

```bash
# Create config.ron
cat > config.ron << 'EOF'
(
    chart: (
        width: 800,
        height: 800,
        location: (
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
        ),
        house_system: "Placidus",
        orb: 3.0,
    ),
)
EOF

# Use config
cargo run --bin astro-clock -- --config config.ron chart --output docs/configured.png
```

## HTTP Server Examples

Start the server:
```bash
cargo run --bin astro-clock -- serve --port 3000
```

Get a chart:
```bash
curl "http://localhost:3000/chart?lat=37.7749&lon=-122.4194" > docs/server_chart.png
```

Check health:
```bash
curl http://localhost:3000/health
```

## Sample Output

### Markdown Table Example

```markdown
## Current Planets

**01-Mar-2026, 12:00 UT/GMT**

*House System: Placidus*

| Planet | Deg | Sign | Position | House |
|--------|-----|------|----------|-------|
| ☉ Sun | 10 | ♓ | 10°30' | 10 |
| ☽ Moon | 25 | ♊ | 25°15' | 1 |
| ☿ Mercury | 15 | ♓ | 15°45' | 10 |
...

## Aspects

| Aspect | Planet 1 | Planet 2 | Orb |
|--------|----------|----------|-----|
| ☌ Conjunction | Sun | Mercury | 5.25° |
| △ Trine | Moon | Mars | 2.10° |
```

## License

Chart designs and generated content are licensed under AGPL-3.0.
