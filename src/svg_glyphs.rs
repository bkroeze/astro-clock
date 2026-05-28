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
            "quintile_alternate",
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

    #[test]
    fn test_registry_contains_generated_csv_entries() {
        let registry = GlyphRegistry::new();
        assert_eq!(GLYPHS.len(), 67);

        for (name, glyph) in GLYPHS {
            assert!(registry.contains(name), "Missing generated glyph: {name}");
            let registered = registry.get(name).unwrap();
            assert_eq!(registered.path, glyph.path, "Wrong path for {name}");
            assert_eq!(
                registered.view_box, glyph.view_box,
                "Wrong view box for {name}"
            );
            assert_eq!(
                registered.baseline_offset, glyph.baseline_offset,
                "Wrong baseline offset for {name}"
            );
            assert!(!registered.path.is_empty(), "Empty path for {name}");
            assert!(registered.width() > 0.0, "Zero width for {name}");
            assert!(registered.height() > 0.0, "Zero height for {name}");
        }
    }

    #[test]
    fn test_earth_antimony_alias_does_not_replace_earth() {
        let registry = GlyphRegistry::new();

        assert_eq!(registry.get("earth").unwrap().path, EARTH.path);
        assert_eq!(
            registry.get("earth_antimony").unwrap().path,
            EARTH_ANTINOMY.path
        );
        assert_eq!(
            registry.get("earth_antinomy").unwrap().path,
            EARTH_ANTINOMY.path
        );
    }
}
