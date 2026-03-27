//! Extract SVG paths from Astronomicon.ttf
//! Run: cargo run --bin extract_glyphs

use std::collections::HashMap;
use std::fs;
use std::io::Write;

/// Data for a single glyph
#[derive(Debug, Clone)]
pub struct GlyphData {
    /// SVG path "d" attribute
    pub path: &'static str,
    /// View box (min_x, min_y, width, height)
    pub view_box: (f32, f32, f32, f32),
    /// Baseline offset for vertical alignment
    pub baseline_offset: f32,
}

fn main() -> std::io::Result<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let font_path = format!("{}/fonts/Astronomicon.ttf", manifest_dir);
    let output_path = format!("{}/src/svg_glyph_paths.rs", manifest_dir);

    let font_data = fs::read(&font_path).expect("Failed to read font file");
    let font = ttf_parser::Face::parse(&font_data, 0).expect("Failed to parse font");

    let glyphs: HashMap<&str, char> = [
        // Planets
        ("sun", 'Q'),
        ("moon", 'R'),
        ("mercury", 'S'),
        ("venus", 'T'),
        ("mars", 'U'),
        ("jupiter", 'V'),
        ("saturn", 'W'),
        ("uranus", 'X'),
        ("neptune", 'Y'),
        ("pluto", 'Z'),
        // Zodiac
        ("aries", 'A'),
        ("taurus", 'B'),
        ("gemini", 'C'),
        ("cancer", 'D'),
        ("leo", 'E'),
        ("virgo", 'F'),
        ("libra", 'G'),
        ("scorpio", 'H'),
        ("sagittarius", 'I'),
        ("capricorn", '\\'),
        ("aquarius", 'K'),
        ("pisces", 'L'),
        // Aspects
        ("conjunction", '!'),
        ("sextile", '%'),
        ("square", '#'),
        ("trine", '$'),
        ("opposition", '\"'),
        ("quincunx", '&'),
        ("semi_sextile", '\''),
        ("semi_square", '('),
        ("sesquisquare", ')'),
        ("biquintile", '*'),
        ("quintile", '+'),
        ("semi_quintile", ','),
        ("quindecile", '.'),
        // Other
        ("retrograde", 'N'),
        ("north_node", 'g'),
        ("south_node", 'i'),
        ("chiron", 'q'),
    ]
    .iter()
    .cloned()
    .collect();

    let mut output = fs::File::create(&output_path)?;

    writeln!(output, "//! Generated glyph paths from Astronomicon.ttf")?;
    writeln!(
        output,
        "//! DO NOT EDIT MANUALLY - Run: cargo run --bin extract_glyphs"
    )?;
    writeln!(output)?;
    writeln!(output, "/// Data for a single glyph")?;
    writeln!(output, "#[derive(Debug, Clone, Copy)]")?;
    writeln!(output, "pub struct GlyphData {{")?;
    writeln!(output, "    pub path: &'static str,")?;
    writeln!(output, "    pub view_box: (f32, f32, f32, f32),")?;
    writeln!(output, "    pub baseline_offset: f32,")?;
    writeln!(output, "}}")?;
    writeln!(output)?;
    writeln!(output, "// Define all glyph constants")?;
    writeln!(output)?;

    for (name, ch) in &glyphs {
        let glyph_id = font
            .glyph_index(*ch)
            .unwrap_or_else(|| panic!("No glyph for {}", ch));
        let mut builder = SvgPathBuilder::new();
        font.outline_glyph(glyph_id, &mut builder)
            .unwrap_or_else(|| panic!("No outline for {}", ch));

        writeln!(
            output,
            "pub const {}: GlyphData = GlyphData {{",
            name.to_uppercase()
        )?;
        writeln!(output, "    path: \"{}\",", builder.path.trim())?;
        writeln!(
            output,
            "    view_box: ({}, {}, {}, {}),",
            builder.min_x,
            builder.min_y,
            builder.width(),
            builder.height()
        )?;
        writeln!(output, "    baseline_offset: {},", builder.min_y.abs())?;
        writeln!(output, "}};")?;
        writeln!(output)?;
    }

    println!("Successfully generated {}", output_path);
    Ok(())
}

struct SvgPathBuilder {
    path: String,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl SvgPathBuilder {
    fn new() -> Self {
        Self {
            path: String::new(),
            min_x: f32::INFINITY,
            min_y: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            max_y: f32::NEG_INFINITY,
        }
    }

    fn update_bounds(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
}

impl ttf_parser::OutlineBuilder for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.update_bounds(x, y);
        self.path.push_str(&format!("M{},{} ", x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.update_bounds(x, y);
        self.path.push_str(&format!("L{},{} ", x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.update_bounds(x1, y1);
        self.update_bounds(x, y);
        self.path.push_str(&format!("Q{},{} {},{} ", x1, y1, x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.update_bounds(x1, y1);
        self.update_bounds(x2, y2);
        self.update_bounds(x, y);
        self.path
            .push_str(&format!("C{},{} {},{} {},{} ", x1, y1, x2, y2, x, y));
    }

    fn close(&mut self) {
        self.path.push_str("Z ");
    }
}
