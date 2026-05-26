# SVG Glyph Rendering Design

**Date:** 2026-03-26  
**Topic:** SVG Glyph Rendering for Astrological Charts  
**Status:** Approved

## Overview

Implement SVG output for astrological charts using embedded glyph paths instead of font references. This ensures charts display correctly without requiring the Astronomicon font to be installed on the viewing system.

## Background

The project uses the Astronomicon font to render astrological symbols (planets, zodiac signs, aspects) in chart visualizations. Currently, PNG/WebP output works well but requires font rasterization at render time. SVG output should be standalone and viewable in any browser or image viewer without external font dependencies.

## Requirements

- **Standalone SVGs:** No external font dependencies
- **Target Glyphs:** 67 mapped glyphs (planets, zodiac signs, aspects, points, alchemical symbols, elements)
- **Consistent Rendering:** Identical appearance across all viewers
- **Integration:** New output format alongside existing PNG/WebP

## Glyph Mapping

Based on `fonts/astronomicon.csv`, we need these semantic glyphs:

### Planets and Points (21 glyphs)
- Sun (Q), Moon (R), Mercury (S), Venus (T), Mars (U), Jupiter (V), Saturn (W), Uranus (X), Neptune (Y), Pluto (Z), alternates, nodes, Earth, Lilith, Vulcan

### Zodiac Signs (13 glyphs)
- Aries (A), Taurus (B), Gemini (C), Cancer (D), Leo (E), Virgo (F), Libra (G), Scorpio (H), Sagittarius (I), Capricorn (USA and Europe variants), Aquarius (K), Pisces (L)

### Aspects and Lots (14 glyphs)
- Conjunction (!), Sextile (%), Square (#), Trine ($), Opposition (") and other mapped aspects, Part of Fortune, Part of Spirit

### Other (19 glyphs)
- Asteroids, Chiron, Pholus, chart markers, alchemical symbols, elements, pentagram, hexagram

## Architecture

### Components

```
src/
├── svg_glyphs.rs      # Glyph registry with embedded SVG paths
├── svg_renderer.rs    # SVG generation logic
└── output_handler.rs  # Extended to support SVG format
```

### Glyph Registry

Stores extracted Bezier paths as compile-time constants:

```rust
pub struct GlyphRegistry {
    glyphs: HashMap<&'static str, GlyphData>,
}

pub struct GlyphData {
    /// SVG path "d" attribute
    path: &'static str,
    /// View box for proper scaling (min_x, min_y, width, height)
    view_box: (f32, f32, f32, f32),
    /// Baseline offset for vertical alignment
    baseline_offset: f32,
}
```

Glyphs are referenced by semantic names (e.g., "sun", "aries", "conjunction") rather than font letters.

### SVG Renderer

Generates standalone SVG documents using the glyph registry:

1. Calculate chart geometry (same logic as PNG renderer)
2. Look up glyph paths by semantic key
3. Render as `<path>` elements with transform attributes
4. Include wheel, house cusps, aspects, planet positions

**Example Output:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 800">
  <!-- Background -->
  <rect width="800" height="800" fill="white"/>
  
  <!-- Outer wheel -->
  <circle cx="400" cy="400" r="280" fill="none" stroke="black" stroke-width="2"/>
  
  <!-- House cusp lines -->
  <line x1="400" y1="120" x2="400" y2="280" stroke="black" stroke-width="1.5"/>
  
  <!-- Planet symbol (rendered as path, not text) -->
  <path d="M10,20 C6.48,2..." 
        transform="translate(680, 400) scale(0.5)" 
        fill="black"/>
</svg>
```

## Extraction Process

One-time step to extract glyph paths from Astronomicon.ttf:

1. Add `ttf-parser` crate as a dev-dependency
2. Create extraction script that:
   - Opens the TTF file
   - Iterates through needed codepoints
   - Extracts outline data (move_to, line_to, quad_to, curve_to)
   - Converts to SVG path format
   - Generates Rust code with static constants
3. Run once, commit generated code
4. Remove extraction script and ttf-parser dependency

## Integration Points

### Output Handler

Extend existing output handling to support SVG:

```rust
pub enum OutputFormat {
    Png,
    Webp,
    Markdown,
    Svg,  // New
}
```

### CLI

SVG format available via `--format svg` flag, same as other formats.

### Server

HTTP endpoint serves SVG with `Content-Type: image/svg+xml`.

## Benefits

1. **Truly Standalone:** No font installation required
2. **Consistent:** Renders identically in all SVG viewers
3. **Efficient:** Only embeds mapped glyphs (67 vs. full font)
4. **Simple:** No runtime font dependencies

## Trade-offs

- One-time extraction step required when glyphs change
- Cannot easily customize glyph appearance per-chart (acceptable for current requirements)

## Next Steps

1. Create implementation plan with writing-plans skill
2. Extract glyph paths from TTF
3. Implement glyph registry module
4. Implement SVG renderer
5. Integrate with output handler
6. Add tests
