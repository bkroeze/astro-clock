# Astro Clock

A CLI application for generating astrological charts with Swiss Ephemeris integration. Calculate natal charts, analyze aspects, serve charts over HTTP, and query astrological databases.

## Quick Start

```bash
# Generate a chart with current time
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --output mychart.png

# Generate chart as markdown table
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format md --output chart.md

# Analyze aspects
cargo run --bin astro-clock -- aspects --lat 37.7749 --lon -122.4194 --orb 3

# Start HTTP server
cargo run --bin astro-clock -- serve --port 3000
```

## Installation

```bash
# Build the application
cargo build --release

# Run tests
cargo test
```

## CLI Usage

### Commands

```bash
# Chart generation
cargo run --bin astro-clock -- chart [OPTIONS]

# Aspect analysis
cargo run --bin astro-clock -- aspects [OPTIONS]

# HTTP server
cargo run --bin astro-clock -- serve [OPTIONS]
```

### Chart Command

Generate an astrological chart:

```bash
cargo run --bin astro-clock -- chart \\
  --lat 37.7749 \\
  --lon -122.4194 \\
  --time "2026-03-01T12:00:00-08:00" \\
  --output chart.png \\
  --format png \\
  --house Placidus \\
  --orb 3.0
```

**Options:**
- `--lat <LAT>` - Latitude (-90 to 90)
- `--lon <LON>` - Longitude (-180 to 180)
- `--time <TIME>` - ISO 8601 timestamp (defaults to now)
- `--output <FILE>` - Output file path (auto-generated if not specified)
- `--format <FORMAT>` - Output format: `png`, `webp`, `md`, `svg` (default: png)
- `--house <SYSTEM>` - House system (default: Placidus)
- `--orb <DEGREES>` - Maximum orb for aspect detection in markdown (default: 3.0)

### Aspects Command

Analyze planetary aspects:

```bash
cargo run --bin astro-clock -- aspects \\
  --lat 37.7749 \\
  --lon -122.4194 \\
  --time "2026-03-01T12:00:00-08:00" \\
  --orb 3.0 \\
  --house Placidus
```

**Output includes:**
- Moon void-of-course status
- Aspect table with orb values
- Grand trines

### Serve Command

Start HTTP server for chart generation:

```bash
cargo run --bin astro-clock -- serve --host 127.0.0.1 --port 3000
```

**Endpoints:**
- `GET /health` - Health check
- `GET /chart?lat=<lat>&lon=<lon>&time=<iso8601>&format=<png|svg>` - Generate chart image output (`png` default, or `svg`)
- `GET /api/v1/chart/data?lat=<lat>&lon=<lon>&time=<iso8601>&house=<SYSTEM>&traditional=<bool>` - Generate structured chart data as JSON (`traditional=true` defaults to Whole Sign unless `house` is provided and limits output to traditional planets)

## Configuration

Create a RON configuration file (optional):

```ron
(
    chart: (
        width: 800,
        height: 800,
        background_color: "#000000",
        text_color: "#ffffff",
        house_color: "#333333",
        planet_color: "#ffcc00",
        zodiac_color: "#444444",
        location: (
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
        ),
        orb: 3.0,
        house_system: "Placidus",
    ),
    server: (
        host: "127.0.0.1",
        port: 3000,
        cors_origins: ["http://localhost:3000"],
    ),
    database: (
        enabled: false,
        url: "postgresql://user:password@localhost/astro",
        table_name: "charts",
    ),
)
```

Use with: `cargo run --bin astro-clock -- --config config.ron chart`

## House Systems

Supported house systems:
- Placidus (default)
- Koch
- Equal
- Whole
- Porphyry
- Regiomontanus
- Campanus
- Morinus
- Alcabitus
- Topocentric
- Vehlow

## Output Formats

### PNG (default)
Generates an 800x800 pixel chart wheel:
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format png --output chart.png
```

### WebP
Compressed chart image:
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format webp --output chart.webp
```

### Markdown
Text table with planetary positions and aspects:
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format md --output chart.md
```

### SVG (Standalone)
Vector chart that works without font installation:
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format svg --output chart.svg
```

The SVG file contains embedded glyph paths and can be viewed in any browser or image viewer without the Astronomicon font installed.

## Astronomicon Glyph SVGs

Generate one SVG file per mapped Astronomicon glyph with:

```bash
just make-svg
```

The recipe reads `fonts/astronomicon.csv` and `fonts/AstronomiconFonts_1.1/Astronomicon.ttf`, writes individual SVG files to `assets/astronomicon/`, and refreshes the embedded glyph registry at `src/svg_glyph_paths.rs`. Each SVG uses `currentColor`, so callers can color the glyphs with CSS or SVG attributes.

`fonts/astronomicon.csv` is strict, unquoted CSV with one glyph per line:
`single-character glyph key,output basename`. Output basenames must be unique,
non-empty, and must not contain `/` or `\`; extra commas are rejected.

## Example Usage

### Current Chart (San Francisco)
```bash
cargo run --bin astro-clock -- chart --lat 37.7749 --lon -122.4194 --format png
# Outputs: mmddyy-HHMMSS.png
```

### Specific Date/Time
```bash
cargo run --bin astro-clock -- chart \\
  --lat 40.7128 \\
  --lon -74.0060 \\
  --time "2026-03-15T14:30:00-04:00" \\
  --output equinox.png
```

### Whole Sign Houses
```bash
cargo run --bin astro-clock -- chart \\
  --lat 37.7749 \\
  --lon -122.4194 \\
  --house Whole \\
  --output whole_sign.png
```

### Aspect Analysis with Custom Orb
```bash
cargo run --bin astro-clock -- aspects --lat 37.7749 --lon -122.4194 --orb 5.0
```

## Database Queries (Optional)

When compiled with the `db` feature and configured with PostgreSQL:

```bash
# Build with database support
cargo build --features db
```

**Available queries:**
- Wedding dates (favorable Moon signs + Venus aspects)
- Void-of-course Moon periods
- Retrograde periods
- Exact aspects

**Note:** Electoral query CLI functionality is currently only available via database queries, not direct CLI commands. Use the database API or HTTP endpoints for these features.

## File Structure

```
astro-clock/
├── assets/
│   └── astronomicon/       # Generated Astronomicon SVG glyphs
├── fonts/
│   ├── astronomicon.csv    # Astronomicon glyph export map
│   └── AstronomiconFonts_1.1/
├── src/
│   ├── bin/
│   │   ├── main.rs          # Application entry point
│   │   └── export_astronomicon_svg.rs
│   ├── cli/
│   │   ├── app.rs           # CLI argument parsing and main logic
│   │   └── mod.rs
│   ├── chart.rs             # Chart data structures and traits
│   ├── swiss_eph_impl.rs    # Swiss Ephemeris implementation
│   ├── renderer.rs          # Chart rendering (PNG/WebP)
│   ├── svg_glyph_paths.rs   # Generated embedded SVG glyph paths
│   ├── svg_glyphs.rs        # SVG glyph registry
│   ├── svg_renderer.rs      # Chart rendering (SVG)
│   ├── output_handler.rs    # Output format handling
│   ├── aspects.rs           # Aspect calculations
│   ├── config.rs            # Configuration management (RON format)
│   ├── server/mod.rs        # HTTP server
│   ├── ephemeris/mod.rs     # Ephemeris calculations
│   ├── database/            # Database layer (optional)
│   └── queries/             # Astrological queries (optional)
├── docs/                   # Generated charts and documentation
└── Cargo.toml
```

## Dependencies

- `swiss-eph` - Swiss Ephemeris FFI bindings
- `tiny-skia` - 2D graphics rendering
- `fontdue` - Font rasterization
- `axum` - HTTP web framework
- `clap` - CLI argument parsing
- `ron` - Rusty Object Notation for config
- `chrono` - Date/time handling
- `serde` - Serialization
- `sqlx` - Async SQL (optional, requires `db` feature)

## Troubleshooting

### Missing Ephemeris Data
The application uses the embedded Swiss Ephemeris. No external data files required.

### House System Errors
```bash
# Invalid house system names will show available options:
cargo run --bin astro-clock -- chart --house invalid_system
# Error: Invalid house system: 'invalid_system'. Valid options: Placidus, Koch, ...
```

### Time Format
Use ISO 8601 format (RFC 3339):
- `"2026-03-01T12:00:00-08:00"` (with timezone)
- `"2026-03-01T20:00:00Z"` (UTC)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Submit a pull request

## License

This project is licensed under the AGPL-3.0 license (same as Swiss Ephemeris).
