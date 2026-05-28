use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

#[derive(Parser, Debug)]
#[command(about = "Export mapped font glyphs to individual SVG files")]
struct Args {
    /// Strict, unquoted CSV: single-character glyph key,output basename
    map_csv: PathBuf,

    /// TrueType font file to export glyphs from
    ttf_file: PathBuf,

    /// Directory where SVG files will be written
    output_dir: PathBuf,
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
    for mapping in &mappings {
        export_mapping(&face, mapping, &args.output_dir)?;
        written += 1;
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

fn export_mapping(face: &Face<'_>, mapping: &GlyphMapping, output_dir: &Path) -> Result<()> {
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

    let path_data = builder.commands.join(" ");
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">
  <path d="{}" fill="currentColor"/>
</svg>
"#,
        view_min_x, view_min_y, width, height, path_data
    );

    let output_path = output_dir.join(format!("{}.svg", mapping.basename));
    fs::write(&output_path, svg)
        .with_context(|| format!("failed to write SVG {}", output_path.display()))?;

    Ok(())
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

#[allow(dead_code)]
fn _glyph_id_for_debug(glyph_id: GlyphId) -> u16 {
    glyph_id.0
}
