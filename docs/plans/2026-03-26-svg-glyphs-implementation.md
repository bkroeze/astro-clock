# SVG Glyph Rendering Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add SVG output format to astro-clock that embeds astrological glyphs as SVG paths, making charts viewable without the Astronomicon font installed.

**Architecture:** Create a glyph registry with extracted SVG paths from the TTF font, a new SVG renderer module, and integrate with the existing output handler. The repository also includes a glyph exporter that writes standalone Astronomicon SVG assets under `assets/astronomicon/` and generated embedded glyph data in `src/svg_glyph_paths.rs`.

**Tech Stack:** Rust, `ttf-parser` (for glyph extraction), `svg` crate (already in Cargo.toml), existing chart/render infrastructure.

---

## Prerequisites

**Review the design doc:** `docs/plans/2026-03-26-svg-glyphs-design.md`

**Font file location:** `fonts/AstronomiconFonts_1.1/Astronomicon.ttf`

**Glyph mapping:** `fonts/astronomicon.csv`

**Needed glyphs (67 total):** all mapped entries in `fonts/astronomicon.csv`, including planets and alternates, zodiac signs, aspects, lots, asteroids, chart markers, alchemical symbols, and elements.

---

## Task 1: Set up TTF parser dependency for glyph extraction

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add ttf-parser dependency**

```toml
[dependencies]
ttf-parser = "0.25"

[dev-dependencies]
tempfile = "3"
tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
http = "1.0"
assert_cmd = "2.0"
predicates = "3.0"
```

**Step 2: Verify dependency added**

Run: `cargo check`
Expected: Success (no errors about ttf-parser)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add ttf-parser for glyph extraction"
```

---

## Task 2: Create glyph export binary

**Files:**
- Create: `src/bin/export_astronomicon_svg.rs`
- Modify: `Justfile`

**Step 1: Write export binary**

```rust
#[command(about = "Export mapped font glyphs to individual SVG files")]
struct Args {
    /// Strict, unquoted CSV: single-character glyph key,output basename
    map_csv: PathBuf,

    /// TrueType font file to export glyphs from
    ttf_file: PathBuf,

    /// Directory where SVG files will be written
    output_dir: PathBuf,

    /// Optional Rust source file for embedded glyph data
    #[arg(long)]
    rust_output: Option<PathBuf>,
}
```

The binary should read `fonts/astronomicon.csv`, extract each mapped glyph outline
from `fonts/AstronomiconFonts_1.1/Astronomicon.ttf`, and write one
`currentColor` SVG file per glyph to `assets/astronomicon/`. When
`--rust-output` is provided, it also writes generated embedded glyph data.

**Step 2: Add a Justfile recipe**

```make
make-svg:
    cargo run --bin export_astronomicon_svg -- fonts/astronomicon.csv fonts/AstronomiconFonts_1.1/Astronomicon.ttf assets/astronomicon --rust-output src/svg_glyph_paths.rs
```

**Step 3: Run the exporter**

Run: `just make-svg`
Expected: 67 generated SVG files in `assets/astronomicon/` and refreshed embedded glyph data in `src/svg_glyph_paths.rs`

**Step 4: Verify output**

Run: `ls assets/astronomicon | head`
Expected: See generated Astronomicon SVG files

**Step 5: Commit**

```bash
git add Justfile src/bin/export_astronomicon_svg.rs fonts/astronomicon.csv assets/astronomicon src/svg_glyph_paths.rs
git commit -m "feat: export Astronomicon glyph SVG assets"
```

---

## Task 3: Create glyph registry module

**Files:**
- Create: `src/svg_glyphs.rs`

**Step 1: Write glyph registry with extracted paths**

Use the generated Astronomicon path data in this module structure:

```rust
//! SVG Glyph Registry
//! Contains embedded SVG paths for astrological symbols.
//! These are extracted from the Astronomicon font and embedded as static data
//! so SVG charts can be standalone without font dependencies.

use std::collections::HashMap;

// Include the generated glyph data
include!("svg_glyph_paths.rs");

impl GlyphData {
    /// Get the width from view_box
    pub fn width(&self) -> f32 {
        self.view_box.2
    }

    /// Get the height from view_box
    pub fn height(&self) -> f32 {
        self.view_box.3
    }
}

/// Registry mapping semantic names to glyph data
pub struct GlyphRegistry {
    glyphs: HashMap<&'static str, &'static GlyphData>,
}

impl Default for GlyphRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GlyphRegistry {
    pub fn new() -> Self {
        let mut glyphs = HashMap::new();
        
        for (name, glyph) in GLYPHS {
            glyphs.insert(*name, *glyph);
        }

        // Compatibility aliases used by chart rendering and public callers.
        if let Some(glyph) = glyphs.get("capricorn_europe").copied() {
            glyphs.insert("capricorn", glyph);
        }
        if let Some(glyph) = glyphs.get("lunar_north_node").copied() {
            glyphs.insert("north_node", glyph);
        }
        if let Some(glyph) = glyphs.get("lunar_south_node").copied() {
            glyphs.insert("south_node", glyph);
        }
        if let Some(glyph) = glyphs.get("earth_antinomy").copied() {
            glyphs.insert("earth_antimony", glyph);
        }
        if let Some(glyph) = glyphs.get("quincunx_inconjunct").copied() {
            glyphs.insert("quincunx", glyph);
        }
        
        Self { glyphs }
    }
    
    /// Get glyph data by name
    pub fn get(&self, name: &str) -> Option<&GlyphData> {
        self.glyphs.get(name).copied()
    }
    
    /// Check if glyph exists
    pub fn contains(&self, name: &str) -> bool {
        self.glyphs.contains_key(name)
    }

    /// Iterate over every registered glyph.
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static GlyphData)> + '_ {
        self.glyphs.iter().map(|(name, glyph)| (*name, *glyph))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_contains_all_planets() {
        let registry = GlyphRegistry::new();
        let planets = ["sun", "moon", "mercury", "venus", "mars", 
                      "jupiter", "saturn", "uranus", "neptune", "pluto"];
        
        for planet in &planets {
            assert!(registry.contains(planet), "Missing planet: {}", planet);
            let glyph = registry.get(planet).unwrap();
            assert!(!glyph.path.is_empty(), "Empty path for {}", planet);
            assert!(glyph.width() > 0.0, "Zero width for {}", planet);
            assert!(glyph.height() > 0.0, "Zero height for {}", planet);
        }
    }
    
    #[test]
    fn test_registry_contains_all_zodiac() {
        let registry = GlyphRegistry::new();
        let signs = ["aries", "taurus", "gemini", "cancer", "leo", "virgo",
                    "libra", "scorpio", "sagittarius", "capricorn", "aquarius", "pisces"];
        
        for sign in &signs {
            assert!(registry.contains(sign), "Missing sign: {}", sign);
        }
    }
}
```

**Step 2: Update lib.rs to include the new module**

Modify: `src/lib.rs`

Add to the module declarations:
```rust
pub mod svg_glyphs;
```

**Step 3: Run tests**

Run: `cargo test svg_glyphs`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/svg_glyphs.rs src/lib.rs
git commit -m "feat: add glyph registry with embedded SVG paths"
```

---

## Task 4: Create SVG renderer module

**Files:**
- Create: `src/svg_renderer.rs`
- Modify: `src/lib.rs`

**Step 1: Create SVG renderer**

```rust
//! SVG Chart Renderer
//! Generates standalone SVG charts using embedded glyph paths from the glyph registry.

use crate::chart::ChartData;
use crate::svg_glyphs::GlyphRegistry;

/// SVG Chart Renderer
pub struct SvgRenderer {
    width: u32,
    height: u32,
    glyph_registry: GlyphRegistry,
}

impl SvgRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            glyph_registry: GlyphRegistry::new(),
        }
    }
    
    /// Render a chart to SVG string
    pub fn render_chart(&self, chart_data: &ChartData) -> Result<String, Box<dyn std::error::Error>> {
        let mut svg_elements = Vec::new();
        svg_elements.push(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}" style="background-color: white;">"#,
            self.width, self.height, self.width, self.height
        ));
        svg_elements.push(self.generate_glyph_defs());

        // Draw wheel, signs, aspects, planets, and house labels.
        // Glyph placements use `<use href="#glyph_name" transform="..."/>`.

        svg_elements.push("</svg>".to_string());
        Ok(svg_elements.join("\n"))
    }

    /// Generate sorted SVG defs for every registered glyph, including retrograde.
    fn generate_glyph_defs(&self) -> String {
        let mut defs = vec!["<defs>".to_string()];
        let mut glyphs: Vec<_> = self.glyph_registry.iter().collect();
        glyphs.sort_by_key(|(name, _)| *name);

        for (name, glyph) in glyphs {
            defs.push(format!(r#"<path id="{name}" d="{}" />"#, glyph.path));
        }

        defs.push("</defs>".to_string());
        defs.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::*;
    
    #[test]
    fn test_svg_renderer_creation() {
        let renderer = SvgRenderer::new(800, 800);
        assert_eq!(renderer.width, 800);
    }
    
    #[test]
    fn test_svg_rendering() {
        let renderer = SvgRenderer::new(800, 800);
        
        let chart_data = ChartData {
            geo_pos: GeoPos::new(40.7128, -74.0060, 0.0),
            julian_day: 2451545.0,
            planets: vec![
                PlanetPosition::new("Sun", Position::new(280.0, 0.0, 0.0, 0.0, 0.0, 0.0), false),
            ],
            houses: HouseCusps {
                asc: 180.0,
                mc: 90.0,
                dc: 0.0,
                ic: 270.0,
                houses: [180.0, 210.0, 240.0, 270.0, 300.0, 330.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0],
                system: HouseSystem::Placidus,
            },
            sidereal_time: 0.0,
        };
        
        let svg = renderer.render_chart(&chart_data).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<defs>"));
        assert!(svg.contains(r##"<use href="#sun""##));
    }
}
```

**Step 2: Add module to lib.rs**

Modify: `src/lib.rs`

Add: `pub mod svg_renderer;`

**Step 3: Run tests**

Run: `cargo test svg_renderer`
Expected: Tests pass (may need to fix imports based on actual chart module structure)

**Step 4: Commit**

```bash
git add src/svg_renderer.rs src/lib.rs
git commit -m "feat: add SVG renderer with embedded glyph paths"
```

---

## Task 5: Integrate SVG output into output handler

**Files:**
- Modify: `src/output_handler.rs`
- Modify: `src/cli/app.rs` (if format enum defined there)

**Step 1: Add SVG to OutputFormat enum**

In `src/output_handler.rs`, add `Svg` variant:

```rust
pub enum OutputFormat {
    Png,
    Webp,
    Markdown,
    Svg,  // New
}
```

Update the `FromStr` implementation:

```rust
impl std::str::FromStr for OutputFormat {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "png" => Ok(OutputFormat::Png),
            "webp" => Ok(OutputFormat::Webp),
            "md" | "markdown" => Ok(OutputFormat::Markdown),
            "svg" => Ok(OutputFormat::Svg),  // New
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}
```

**Step 2: Add SVG rendering logic**

In the output handling code, add a case for SVG:

```rust
use crate::svg_renderer::SvgRenderer;

// In the output handling function:
OutputFormat::Svg => {
    let renderer = SvgRenderer::new(800, 800);
    let svg_content = renderer.render_chart(chart_data)?;
    std::fs::write(output_path, svg_content)?;
}
```

**Step 3: Test SVG generation**

Run: `cargo run -- chart --lat 37.7749 --lon -122.4194 --format svg --output test_chart.svg`
Expected: Creates test_chart.svg file

Run: `head -30 test_chart.svg`
Expected: Valid SVG XML with embedded paths

**Step 4: Visual verification**

Open `test_chart.svg` in a browser or image viewer.
Expected: Shows astrological chart with symbols visible.

**Step 5: Commit**

```bash
git add src/output_handler.rs
git commit -m "feat: integrate SVG output format"
```

---

## Task 6: Add SVG support to HTTP server

**Files:**
- Modify: `src/server/mod.rs`

**Step 1: Add SVG endpoint or modify existing chart endpoint**

In the server code where chart generation happens, add SVG as an option:

```rust
// Example endpoint modification
async fn get_chart(
    Query(params): Query<ChartParams>,
) -> Result<Response, AppError> {
    let chart_data = generate_chart(&params).await?;
    
    match params.format.as_deref() {
        Some("svg") => {
            use crate::svg_renderer::SvgRenderer;
            let renderer = SvgRenderer::new(800, 800);
            let svg = renderer.render_chart(&chart_data)?;
            
            Ok(Response::builder()
                .header("Content-Type", "image/svg+xml")
                .body(svg)
                .unwrap())
        }
        _ => {
            // Existing PNG/WebP handling
            // ...
        }
    }
}
```

**Step 2: Test server endpoint**

Run: `cargo run -- serve --port 3000`

In another terminal:
Run: `curl -o test_server.svg "http://localhost:3000/chart?lat=37.7749&lon=-122.4194&format=svg"`

Expected: Downloads valid SVG file

**Step 3: Commit**

```bash
git add src/server/mod.rs
git commit -m "feat: add SVG output to HTTP server"
```

---

## Task 7: Verify glyph export artifacts

**Files:**
- Keep: `src/bin/export_astronomicon_svg.rs`
- Keep: `fonts/astronomicon.csv`
- Keep: `fonts/AstronomiconFonts_1.1/Astronomicon.ttf`
- Keep: `assets/astronomicon/*.svg`

**Step 1: Regenerate SVG assets**

Run: `just make-svg`
Expected: 67 generated SVG files in `assets/astronomicon/` and refreshed embedded glyph data in `src/svg_glyph_paths.rs`

**Step 2: Verify the exporter remains available**

Run: `cargo run --bin export_astronomicon_svg -- --help`
Expected: Help text for the Astronomicon SVG exporter

**Step 3: Verify build still works**

Run: `cargo build`
Expected: Success

**Step 4: Commit**

```bash
git add Justfile src/bin/export_astronomicon_svg.rs fonts assets/astronomicon src/svg_glyph_paths.rs
git commit -m "feat: add Astronomicon glyph asset workflow"
```

---

## Task 8: Add integration tests

**Files:**
- Create: `tests/svg_output_test.rs`

**Step 1: Write integration tests**

```rust
use astro_clock::{ChartCalculator, ChartConfig, GeoPos, HouseSystem};
use astro_clock::svg_renderer::SvgRenderer;

#[test]
fn test_svg_contains_all_planets() {
    let config = ChartConfig::new(
        HouseSystem::Placidus,
        GeoPos::new(40.7128, -74.0060, 0.0),
        2451545.0,
    );
    
    // This test assumes you have a way to create calculator
    // Adjust based on actual API
    let renderer = SvgRenderer::new(800, 800);
    
    // Create mock chart data or use calculator
    // ...
    
    // Render and verify
    // let svg = renderer.render_chart(&chart_data).unwrap();
    // assert!(svg.contains("sun") || svg.contains("Sun"));
    // Check for other planets...
}

#[test]
fn test_svg_valid_xml() {
    // Similar test to verify output is valid XML
}

#[test]
fn test_svg_standalone() {
    // Verify no external references (fonts, images)
    // Should not contain "font-family", "@import", etc.
}
```

**Step 2: Run integration tests**

Run: `cargo test --test svg_output_test`
Expected: Tests pass

**Step 3: Commit**

```bash
git add tests/svg_output_test.rs
git commit -m "test: add SVG output integration tests"
```

---

## Task 9: Update documentation

**Files:**
- Modify: `README.md`

**Step 1: Add SVG to README**

Add SVG to the output formats section:

```markdown
### SVG (Standalone)
Vector chart that works without font installation:
```bash
cargo run -- chart --lat 37.7749 --lon -122.4194 --format svg --output chart.svg
```

The SVG file contains embedded glyph paths and can be viewed in any browser or image viewer without the Astronomicon font installed.
```

**Step 2: Update CLI help if needed**

Check: `cargo run -- chart --help`
Ensure `--format` help mentions `svg` option.

**Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add SVG format documentation"
```

---

## Task 10: Final verification

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 2: Verify no font dependencies in output**

Generate an SVG:
Run: `cargo run -- chart --lat 40.7128 --lon -74.006 --format svg --output final_test.svg`

Check content:
Run: `cat final_test.svg | grep -i font`
Expected: No font references found

**Step 3: Test cross-platform viewing**

- Open SVG in Firefox
- Open SVG in Chrome
- Open SVG in an image viewer (e.g., `eog`, `feh`)

Expected: All show the chart correctly with astrological symbols visible.

**Step 4: Clean up test files**

Run: `rm -f test_chart.svg test_server.svg final_test.svg`

**Step 5: Final commit**

```bash
git add -A
git commit -m "feat: complete SVG glyph rendering implementation

- Extract glyph paths from Astronomicon.ttf
- Create glyph registry with embedded SVG paths
- Implement SVG renderer using embedded glyphs
- Add SVG output format to CLI and HTTP server
- Charts are now fully standalone without font dependencies"
```

---

## Summary

This implementation plan adds SVG output support to astro-clock using embedded glyph paths and checked-in glyph assets instead of font references. The approach:

1. **Extracts** glyph paths from the TTF font using ttf-parser
2. **Exports** mapped Astronomicon glyphs as standalone SVG assets
3. **Renders** charts using sorted `<defs>` and `<use>` elements with transforms
4. **Produces** standalone SVGs that work anywhere

**Key Benefits:**
- No font installation required on viewing systems
- Consistent rendering across all platforms
- Smaller files than base64-embedded fonts
- Simple, maintainable code

**Files Created:**
- `src/bin/export_astronomicon_svg.rs` - Astronomicon glyph SVG exporter
- `fonts/astronomicon.csv` - Glyph export mapping
- `assets/astronomicon/*.svg` - Generated glyph SVG assets
- `src/svg_glyph_paths.rs` - Generated embedded glyph data
- `src/svg_glyphs.rs` - Glyph registry with embedded paths
- `src/svg_renderer.rs` - SVG generation logic
- `tests/svg_output_test.rs` - Integration tests

**Files Modified:**
- `Justfile` - Added `make-svg` asset generation recipe
- `Cargo.toml` - Added ttf-parser, module exports
- `src/lib.rs` - Added module declarations
- `src/output_handler.rs` - Added SVG format
- `src/server/mod.rs` - Added SVG endpoint
- `README.md` - Documentation
