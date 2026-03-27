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
#[derive(Debug, Clone)]
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

        // Planets
        glyphs.insert("sun", &SUN);
        glyphs.insert("moon", &MOON);
        glyphs.insert("mercury", &MERCURY);
        glyphs.insert("venus", &VENUS);
        glyphs.insert("mars", &MARS);
        glyphs.insert("jupiter", &JUPITER);
        glyphs.insert("saturn", &SATURN);
        glyphs.insert("uranus", &URANUS);
        glyphs.insert("neptune", &NEPTUNE);
        glyphs.insert("pluto", &PLUTO);

        // Zodiac signs
        glyphs.insert("aries", &ARIES);
        glyphs.insert("taurus", &TAURUS);
        glyphs.insert("gemini", &GEMINI);
        glyphs.insert("cancer", &CANCER);
        glyphs.insert("leo", &LEO);
        glyphs.insert("virgo", &VIRGO);
        glyphs.insert("libra", &LIBRA);
        glyphs.insert("scorpio", &SCORPIO);
        glyphs.insert("sagittarius", &SAGITTARIUS);
        glyphs.insert("capricorn", &CAPRICORN);
        glyphs.insert("aquarius", &AQUARIUS);
        glyphs.insert("pisces", &PISCES);

        // Aspects
        glyphs.insert("conjunction", &CONJUNCTION);
        glyphs.insert("sextile", &SEXTILE);
        glyphs.insert("square", &SQUARE);
        glyphs.insert("trine", &TRINE);
        glyphs.insert("opposition", &OPPOSITION);
        glyphs.insert("quincunx", &QUINCUNX);
        glyphs.insert("semi_sextile", &SEMI_SEXTILE);
        glyphs.insert("semi_square", &SEMI_SQUARE);
        glyphs.insert("sesquisquare", &SESQUISQUARE);
        glyphs.insert("biquintile", &BIQUINTILE);
        glyphs.insert("quintile", &QUINTILE);
        glyphs.insert("semi_quintile", &SEMI_QUINTILE);
        glyphs.insert("quindecile", &QUINDECILE);

        // Other
        glyphs.insert("retrograde", &RETROGRADE);
        glyphs.insert("north_node", &NORTH_NODE);
        glyphs.insert("south_node", &SOUTH_NODE);
        glyphs.insert("chiron", &CHIRON);

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_contains_all_planets() {
        let registry = GlyphRegistry::new();
        let planets = [
            "sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune",
            "pluto",
        ];

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
        let signs = [
            "aries",
            "taurus",
            "gemini",
            "cancer",
            "leo",
            "virgo",
            "libra",
            "scorpio",
            "sagittarius",
            "capricorn",
            "aquarius",
            "pisces",
        ];

        for sign in &signs {
            assert!(registry.contains(sign), "Missing sign: {}", sign);
        }
    }

    #[test]
    fn test_registry_contains_all_aspects() {
        let registry = GlyphRegistry::new();
        let aspects = [
            "conjunction",
            "sextile",
            "square",
            "trine",
            "opposition",
            "quincunx",
            "semi_sextile",
            "semi_square",
            "sesquisquare",
            "biquintile",
            "quintile",
            "semi_quintile",
            "quindecile",
        ];

        for aspect in &aspects {
            assert!(registry.contains(aspect), "Missing aspect: {}", aspect);
        }
    }

    #[test]
    fn test_glyph_dimensions() {
        let registry = GlyphRegistry::new();
        let glyph = registry.get("sun").unwrap();

        assert!(glyph.width() > 0.0, "Glyph width should be positive");
        assert!(glyph.height() > 0.0, "Glyph height should be positive");
        assert!(!glyph.path.is_empty(), "Glyph path should not be empty");
    }
}
