use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use ttf_parser::{Face, OutlineBuilder};

#[derive(Parser, Debug)]
#[command(about = "Export mapped font glyphs to SVG files and optional embedded Rust data")]
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

#[derive(Debug)]
struct GlyphMapping {
    character: char,
    basename: String,
    line_number: usize,
}

#[derive(Default)]
struct SvgPathBuilder {
    commands: Vec<String>,
}

impl OutlineBuilder for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands
            .push(format!("M{} {}", fmt_num(x), fmt_num(-y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.commands
            .push(format!("L{} {}", fmt_num(x), fmt_num(-y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.commands.push(format!(
            "Q{} {} {} {}",
            fmt_num(x1),
            fmt_num(-y1),
            fmt_num(x),
            fmt_num(-y)
        ));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.commands.push(format!(
            "C{} {} {} {} {} {}",
            fmt_num(x1),
            fmt_num(-y1),
            fmt_num(x2),
            fmt_num(-y2),
            fmt_num(x),
            fmt_num(-y)
        ));
    }

    fn close(&mut self) {
        self.commands.push("Z".to_string());
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mappings = read_map_csv(&args.map_csv)?;
    let font_data = fs::read(&args.ttf_file)
        .with_context(|| format!("failed to read TTF file {}", args.ttf_file.display()))?;
    let face = Face::parse(&font_data, 0).context("failed to parse TTF face")?;

    fs::create_dir_all(&args.output_dir).with_context(|| {
        format!(
            "failed to create output directory {}",
            args.output_dir.display()
        )
    })?;

    let mut written = 0usize;
    let mut embedded = Vec::new();
    for mapping in &mappings {
        let glyph = extract_glyph(&face, mapping)?;
        export_svg(mapping, &glyph, &args.output_dir)?;
        embedded.push((mapping, glyph));
        written += 1;
    }

    if let Some(rust_output) = &args.rust_output {
        write_embedded_rust(&embedded, rust_output)?;
    }

    println!("Wrote {written} SVG files to {}", args.output_dir.display());
    Ok(())
}

fn read_map_csv(path: &Path) -> Result<Vec<GlyphMapping>> {
    let csv = fs::read_to_string(path)
        .with_context(|| format!("failed to read map CSV {}", path.display()))?;
    let mut mappings = Vec::new();
    let mut basenames = HashSet::new();

    for (line_index, raw_line) in csv.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim_end_matches('\r');

        if line.trim().is_empty() {
            continue;
        }

        let (glyph_field, basename_field) = line.split_once(',').ok_or_else(|| {
            anyhow!("line {line_number}: expected exactly two comma-separated fields")
        })?;

        if basename_field.contains(',') {
            bail!("line {line_number}: strict CSV must not contain extra commas");
        }

        let mut chars = glyph_field.chars();
        let character = chars
            .next()
            .ok_or_else(|| anyhow!("line {line_number}: first field must be one character"))?;
        if chars.next().is_some() {
            bail!("line {line_number}: first field must be one character");
        }

        let basename = basename_field.trim().to_string();
        if basename.is_empty() {
            bail!("line {line_number}: output basename must not be empty");
        }
        if basename.contains('/') || basename.contains('\\') {
            bail!("line {line_number}: output basename must not contain path separators");
        }
        if !basenames.insert(basename.clone()) {
            bail!("line {line_number}: duplicate output basename {basename:?}");
        }

        mappings.push(GlyphMapping {
            character,
            basename,
            line_number,
        });
    }

    Ok(mappings)
}

#[derive(Debug)]
struct ExtractedGlyph {
    path_data: String,
    view_box: (i16, i16, i16, i16),
    baseline_offset: i16,
}

fn extract_glyph(face: &Face<'_>, mapping: &GlyphMapping) -> Result<ExtractedGlyph> {
    let glyph_id = face.glyph_index(mapping.character).ok_or_else(|| {
        anyhow!(
            "line {}: no glyph found for character {:?}",
            mapping.line_number,
            mapping.character
        )
    })?;

    let bbox = face.glyph_bounding_box(glyph_id).ok_or_else(|| {
        anyhow!(
            "line {}: glyph {:?} has no bounding box",
            mapping.line_number,
            mapping.character
        )
    })?;

    let mut builder = SvgPathBuilder::default();
    face.outline_glyph(glyph_id, &mut builder).ok_or_else(|| {
        anyhow!(
            "line {}: glyph {:?} has no outline",
            mapping.line_number,
            mapping.character
        )
    })?;

    if builder.commands.is_empty() {
        bail!(
            "line {}: glyph {:?} produced an empty outline",
            mapping.line_number,
            mapping.character
        );
    }

    let view_min_x = bbox.x_min;
    let view_min_y = -bbox.y_max;
    let width = bbox.x_max - bbox.x_min;
    let height = bbox.y_max - bbox.y_min;
    if width <= 0 || height <= 0 {
        bail!(
            "line {}: glyph {:?} has invalid bounds",
            mapping.line_number,
            mapping.character
        );
    }

    Ok(ExtractedGlyph {
        path_data: builder.commands.join(" "),
        view_box: (view_min_x, view_min_y, width, height),
        baseline_offset: -view_min_y,
    })
}

fn export_svg(mapping: &GlyphMapping, glyph: &ExtractedGlyph, output_dir: &Path) -> Result<()> {
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">
  <path d="{}" fill="currentColor"/>
</svg>
"#,
        glyph.view_box.0, glyph.view_box.1, glyph.view_box.2, glyph.view_box.3, glyph.path_data
    );

    let output_path = output_dir.join(format!("{}.svg", mapping.basename));
    fs::write(&output_path, svg)
        .with_context(|| format!("failed to write SVG {}", output_path.display()))?;

    Ok(())
}

fn write_embedded_rust(entries: &[(&GlyphMapping, ExtractedGlyph)], path: &Path) -> Result<()> {
    let mut output = String::from(
        r#"// Generated glyph paths from fonts/astronomicon.csv and Astronomicon.ttf
// DO NOT EDIT MANUALLY - Run: just make-svg

/// Data for a single glyph
#[derive(Debug, Clone, Copy)]
pub struct GlyphData {
    pub path: &'static str,
    pub view_box: (f32, f32, f32, f32),
    pub baseline_offset: f32,
}

"#,
    );

    for (mapping, glyph) in entries {
        output.push_str(&format!(
            "pub const {}: GlyphData = GlyphData {{\n    path: {:?},\n    view_box: ({}.0, {}.0, {}.0, {}.0),\n    baseline_offset: {}.0,\n}};\n\n",
            const_name(&mapping.basename),
            glyph.path_data,
            glyph.view_box.0,
            glyph.view_box.1,
            glyph.view_box.2,
            glyph.view_box.3,
            glyph.baseline_offset
        ));
    }

    output.push_str("pub static GLYPHS: &[(&str, &GlyphData)] = &[\n");
    for (mapping, _) in entries {
        output.push_str(&format!(
            "    ({:?}, &{}),\n",
            registry_key(&mapping.basename),
            const_name(&mapping.basename)
        ));
    }
    output.push_str("];\n");

    fs::write(path, output)
        .with_context(|| format!("failed to write embedded Rust {}", path.display()))?;
    Ok(())
}

fn registry_key(basename: &str) -> String {
    normalize_name(basename).to_lowercase()
}

fn const_name(basename: &str) -> String {
    normalize_name(basename).to_uppercase()
}

fn normalize_name(name: &str) -> String {
    let stripped = name.strip_prefix("The ").unwrap_or(name);
    let mut normalized = String::new();
    let mut last_was_separator = false;

    for character in stripped.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        }
    }

    normalized.trim_matches('_').to_string()
}

fn fmt_num(value: f32) -> String {
    let rounded = value.round();
    if (value - rounded).abs() < 0.001 {
        return (rounded as i32).to_string();
    }

    let mut formatted = format!("{value:.3}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    formatted
}
