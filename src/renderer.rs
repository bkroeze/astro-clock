use std::path::Path;
use tiny_skia::{FillRule, Paint, Pixmap, Transform};

use crate::aspects::{AspectConfig, AspectType, find_aspects};

pub struct FontState {
    pub font: fontdue::Font,
    pub symbol_font: Option<fontdue::Font>,
}

impl FontState {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let font_bytes = include_bytes!("../assets/Roboto-Regular.ttf");
        let font =
            fontdue::Font::from_bytes(font_bytes.as_slice(), fontdue::FontSettings::default())?;

        // Try to load system symbol font
        let symbol_font = Self::load_symbol_font();

        Ok(Self { font, symbol_font })
    }

    fn load_symbol_font() -> Option<fontdue::Font> {
        // Try common system paths for Noto Sans Symbols
        let paths = [
            "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
            "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
        ];

        for path in &paths {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(font) = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
                {
                    return Some(font);
                }
            }
        }

        None
    }

    pub fn has_symbol_support(&self) -> bool {
        self.symbol_font.is_some()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn center(&self) -> Point {
        Point::new(self.width / 2.0, self.height / 2.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }

    pub fn gray(level: f32) -> Self {
        Self::new(level, level, level, 1.0)
    }

    pub fn red() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0)
    }

    pub fn blue() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    fn to_skia(&self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        )
    }
}

pub struct Renderer {
    pixmap: Pixmap,
    size: Size,
    font_state: FontState,
}

impl Renderer {
    pub fn new(size: Size) -> Result<Self, Box<dyn std::error::Error>> {
        let pixmap = Pixmap::new(size.width as u32, size.height as u32).unwrap();
        let font_state = FontState::load()?;
        Ok(Self {
            pixmap,
            size,
            font_state,
        })
    }

    pub fn render_chart(
        &mut self,
        chart_data: &crate::chart::ChartData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.clear(Color::white());

        let center = self.size.center();
        let radius = f32::min(self.size.width, self.size.height) * 0.35;

        let house_anchor = chart_data.houses.houses[0] as f32;

        self.draw_zodiac_wheel(center, radius, house_anchor)?;
        self.draw_zodiac_sign_labels(center, radius, house_anchor)?;
        self.draw_house_cusps(center, radius, &chart_data.houses)?;
        self.draw_aspects(center, radius, house_anchor, &chart_data.planets)?;
        self.draw_planets(center, radius, house_anchor, &chart_data.planets)?;
        self.draw_house_degree_labels(center, radius, &chart_data.houses)?;

        Ok(())
    }

    fn draw_zodiac_wheel(
        &mut self,
        center: Point,
        radius: f32,
        house_anchor: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Draw the outer circle outline (not filled)
        self.draw_circle_outline(center, radius, Color::black(), 2.0);

        for degree in 0..360 {
            let angle = Self::zodiac_angle(degree as f32, house_anchor);
            let length = if degree % 30 == 0 {
                radius * 0.88
            } else if degree % 10 == 0 {
                radius * 0.92
            } else if degree % 5 == 0 {
                radius * 0.95
            } else {
                continue;
            };

            let start = Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            let end = Point::new(
                center.x + length * angle.cos(),
                center.y + length * angle.sin(),
            );

            let stroke_width = if degree % 30 == 0 { 2.0 } else { 1.0 };
            self.draw_line(start, end, Color::black(), stroke_width);
        }

        Ok(())
    }

    fn draw_zodiac_sign_labels(
        &mut self,
        center: Point,
        radius: f32,
        house_anchor: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Zodiac symbols
        let signs = [
            "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
        ];

        for (i, sign) in signs.iter().enumerate() {
            let sign_mid = Self::zodiac_angle((i * 30 + 15) as f32, house_anchor);
            let label_radius = radius * 1.08;
            let label_x = center.x + label_radius * sign_mid.cos();
            let label_y = center.y + label_radius * sign_mid.sin();

            let font_size = 16.0;
            let text_width = self.text_width(sign, font_size);
            let text_x = label_x - text_width / 2.0;
            let text_y = label_y + font_size / 3.0;

            self.draw_text(sign, text_x, text_y, font_size, Color::black());
        }

        Ok(())
    }

    fn draw_house_cusps(
        &mut self,
        center: Point,
        radius: f32,
        houses: &crate::chart::HouseCusps,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let house_cusps = houses.houses;
        let house_anchor = houses.houses[0] as f32;
        for (_i, &cusp) in house_cusps.iter().enumerate() {
            let angle = Self::zodiac_angle(cusp as f32, house_anchor);
            let start = Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            let end = Point::new(
                center.x + (radius * 0.7) * angle.cos(),
                center.y + (radius * 0.7) * angle.sin(),
            );

            // All house cusps are black
            self.draw_line(start, end, Color::black(), 1.5);
        }

        Ok(())
    }

    fn draw_aspects(
        &mut self,
        center: Point,
        radius: f32,
        house_anchor: f32,
        planets: &[crate::chart::PlanetPosition],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Find aspects with default 3° orb
        let config = AspectConfig::new(3.0);
        let aspects = find_aspects(planets, config);

        // Draw aspect lines inside the chart wheel
        let aspect_radius = radius * 0.5; // Draw aspects in the inner circle

        for aspect in aspects {
            // Find the two planets involved
            let planet1 = planets.iter().find(|p| p.name == aspect.planet1);
            let planet2 = planets.iter().find(|p| p.name == aspect.planet2);

            if let (Some(p1), Some(p2)) = (planet1, planet2) {
                // Calculate positions on the aspect circle
                let angle1 = Self::zodiac_angle(p1.position.longitude as f32, house_anchor);
                let angle2 = Self::zodiac_angle(p2.position.longitude as f32, house_anchor);

                let start = Point::new(
                    center.x + aspect_radius * angle1.cos(),
                    center.y + aspect_radius * angle1.sin(),
                );
                let end = Point::new(
                    center.x + aspect_radius * angle2.cos(),
                    center.y + aspect_radius * angle2.sin(),
                );

                // Color based on aspect type
                let color = match aspect.aspect_type {
                    AspectType::Conjunction => Color::new(0.8, 0.8, 0.8, 1.0), // Light gray
                    AspectType::Opposition => Color::new(1.0, 0.0, 0.0, 1.0),  // Red
                    AspectType::Square => Color::new(1.0, 0.0, 0.0, 1.0),      // Red
                    AspectType::Trine => Color::new(0.0, 0.5, 1.0, 1.0),       // Blue
                    AspectType::Sextile => Color::new(0.0, 0.8, 0.0, 1.0),     // Green
                    AspectType::GrandTrine => Color::new(0.0, 0.5, 1.0, 1.0),  // Blue
                };

                // Line width based on orb (tighter orb = thicker line)
                let stroke_width = if aspect.orb < 1.0 {
                    2.0
                } else if aspect.orb < 2.0 {
                    1.5
                } else {
                    1.0
                };

                self.draw_line(start, end, color, stroke_width);
            }
        }

        Ok(())
    }

    fn draw_planets(
        &mut self,
        center: Point,
        radius: f32,
        house_anchor: f32,
        planets: &[crate::chart::PlanetPosition],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Planet symbols mapping
        let planet_symbols: std::collections::HashMap<&str, &str> = [
            (crate::chart::planet::SUN, "☉"),
            (crate::chart::planet::MOON, "☽"),
            (crate::chart::planet::MERCURY, "☿"),
            (crate::chart::planet::VENUS, "♀"),
            (crate::chart::planet::MARS, "♂"),
            (crate::chart::planet::JUPITER, "♃"),
            (crate::chart::planet::SATURN, "♄"),
            (crate::chart::planet::URANUS, "♅"),
            (crate::chart::planet::NEPTUNE, "♆"),
            (crate::chart::planet::PLUTO, "♇"),
            (crate::chart::planet::TRUE_NODE, "☊"),
            (crate::chart::planet::MEAN_NODE, "☊"),
            (crate::chart::planet::CHIRON, "⚷"),
        ]
        .iter()
        .cloned()
        .collect();

        for planet in planets {
            let angle = Self::zodiac_angle(planet.position.longitude as f32, house_anchor);
            let planet_radius = radius * 0.75;

            let planet_pos = Point::new(
                center.x + planet_radius * angle.cos(),
                center.y + planet_radius * angle.sin(),
            );

            // Get the symbol for this planet
            let name_str = planet.name.clone();
            let default_symbol = name_str.as_str();
            let symbol = planet_symbols
                .get(planet.name.as_str())
                .unwrap_or(&default_symbol);

            // Draw the planetary symbol
            let font_size = 14.0;
            let text_width = self.text_width(symbol, font_size);
            let text_x = planet_pos.x - text_width / 2.0;
            let text_y = planet_pos.y + font_size / 3.0;
            self.draw_text(symbol, text_x, text_y, font_size, Color::black());

            // Draw retrograde indicator if applicable
            if planet.retrograde {
                let retro_x = planet_pos.x + text_width / 2.0 + 3.0;
                let retro_y = planet_pos.y - font_size / 3.0;
                self.draw_text("r", retro_x, retro_y, 8.0, Color::black());
            }
        }

        Ok(())
    }

    fn draw_house_degree_labels(
        &mut self,
        center: Point,
        radius: f32,
        houses: &crate::chart::HouseCusps,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let house_cusps = houses.houses;
        let house_anchor = houses.houses[0] as f32;
        // Zodiac symbols
        let zodiac_symbols = [
            "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
        ];

        for (_i, &cusp) in house_cusps.iter().enumerate() {
            let angle = Self::zodiac_angle(cusp as f32, house_anchor);
            let label_radius = radius * 0.6;
            let label_x = center.x + label_radius * angle.cos();
            let label_y = center.y + label_radius * angle.sin();

            let degree_in_sign = cusp % 30.0;
            let sign_index = (cusp / 30.0) as usize % 12;
            let sign_symbol = zodiac_symbols[sign_index];
            let deg = degree_in_sign as i32;
            let min = ((degree_in_sign - deg as f64) * 60.0) as i32;
            let label = format!("{}{:02}°{:02}'", sign_symbol, deg, min);

            let font_size = 8.0;
            let text_width = self.text_width(&label, font_size);
            let text_x = label_x - text_width / 2.0;
            let text_y = label_y + font_size / 3.0;

            self.draw_text(&label, text_x, text_y, font_size, Color::black());
        }

        Ok(())
    }

    fn zodiac_angle(longitude: f32, house_anchor: f32) -> f32 {
        (180.0 - (longitude - house_anchor))
            .rem_euclid(360.0)
            .to_radians()
    }

    pub fn clear(&mut self, color: Color) {
        let mut paint = Paint::default();
        paint.set_color(color.to_skia());
        paint.anti_alias = true;

        let rect = tiny_skia::Rect::from_xywh(0.0, 0.0, self.size.width, self.size.height).unwrap();
        self.pixmap
            .fill_rect(rect, &paint, Transform::identity(), None);
    }

    pub fn draw_circle(&mut self, center: Point, radius: f32, color: Color) {
        let mut paint = Paint::default();
        paint.set_color(color.to_skia());
        paint.anti_alias = true;

        let mut path = tiny_skia::PathBuilder::new();
        path.push_circle(center.x, center.y, radius);
        let path = path.finish().unwrap();

        self.pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    pub fn draw_circle_outline(
        &mut self,
        center: Point,
        radius: f32,
        color: Color,
        stroke_width: f32,
    ) {
        let mut paint = Paint::default();
        paint.set_color(color.to_skia());
        paint.anti_alias = true;

        let mut path = tiny_skia::PathBuilder::new();
        path.push_circle(center.x, center.y, radius);
        let path = path.finish().unwrap();

        let stroke = tiny_skia::Stroke {
            width: stroke_width,
            ..Default::default()
        };

        if let Some(stroked) = path.stroke(&stroke, 1.0) {
            self.pixmap.fill_path(
                &stroked,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    pub fn draw_line(&mut self, start: Point, end: Point, color: Color, stroke_width: f32) {
        let mut paint = Paint::default();
        paint.set_color(color.to_skia());
        paint.anti_alias = true;

        let mut path = tiny_skia::PathBuilder::new();
        path.move_to(start.x, start.y);
        path.line_to(end.x, end.y);
        let path = path.finish().unwrap();

        let stroke = tiny_skia::Stroke {
            width: stroke_width,
            ..Default::default()
        };

        if let Some(stroked) = path.stroke(&stroke, 1.0) {
            self.pixmap.fill_path(
                &stroked,
                &paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        let mut current_x = x;
        let width = self.size.width as usize;
        let height = self.size.height as usize;
        let pixels = self.pixmap.pixels_mut();

        for c in text.chars() {
            // Use symbol font for astrological symbols (U+2600 to U+26FF range)
            let is_symbol = c as u32 >= 0x2600 && c as u32 <= 0x26FF;
            let font = if is_symbol && self.font_state.symbol_font.is_some() {
                self.font_state.symbol_font.as_ref().unwrap()
            } else {
                &self.font_state.font
            };

            let (metrics, bitmap) = font.rasterize(c, size);

            // Skip characters with no bitmap data
            if metrics.width == 0 || bitmap.is_empty() {
                current_x += metrics.advance_width;
                continue;
            }

            let glyph_x = current_x + metrics.xmin as f32;
            let glyph_y = y - metrics.ymin as f32;

            for (row_idx, row) in bitmap.chunks(metrics.width).enumerate() {
                for (col_idx, &alpha) in row.iter().enumerate() {
                    if alpha == 0 {
                        continue;
                    }

                    let px = glyph_x as i32 + col_idx as i32;
                    let py = glyph_y as i32 + row_idx as i32;

                    if px >= 0 && (px as usize) < width && py >= 0 && (py as usize) < height {
                        let idx = py as usize * width + px as usize;
                        let alpha_f = alpha as f32 / 255.0;

                        let old_r = pixels[idx].red() as f32;
                        let old_g = pixels[idx].green() as f32;
                        let old_b = pixels[idx].blue() as f32;

                        let new_r =
                            (color.r * alpha_f * 255.0 + old_r * (1.0 - alpha_f)).min(255.0) as u8;
                        let new_g =
                            (color.g * alpha_f * 255.0 + old_g * (1.0 - alpha_f)).min(255.0) as u8;
                        let new_b =
                            (color.b * alpha_f * 255.0 + old_b * (1.0 - alpha_f)).min(255.0) as u8;

                        pixels[idx] =
                            tiny_skia::PremultipliedColorU8::from_rgba(new_r, new_g, new_b, 255)
                                .unwrap();
                    }
                }
            }

            current_x += metrics.advance_width;
        }
    }

    pub fn text_width(&self, text: &str, size: f32) -> f32 {
        let mut width = 0.0;
        for c in text.chars() {
            let (metrics, _) = self.font_state.font.rasterize(c, size);
            width += metrics.advance_width;
        }
        width
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(path);
        self.pixmap.save_png(path)?;
        Ok(())
    }

    pub fn export_png(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let pixels = self.pixmap.pixels();

        let mut buffer = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buffer, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;

            let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
            for pixel in pixels {
                rgba_data.push(pixel.red());
                rgba_data.push(pixel.green());
                rgba_data.push(pixel.blue());
                rgba_data.push(pixel.alpha());
            }

            writer.write_image_data(&rgba_data)?;
        }
        Ok(buffer)
    }

    pub fn save_webp(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::BufWriter;

        let width = self.pixmap.width();
        let height = self.pixmap.height();
        let pixels = self.pixmap.pixels();

        // Convert to RGBA8 format
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
        for pixel in pixels {
            rgba_data.push(pixel.red());
            rgba_data.push(pixel.green());
            rgba_data.push(pixel.blue());
            rgba_data.push(pixel.alpha());
        }

        // Encode to WebP
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let encoder = image_webp::WebPEncoder::new(writer);
        encoder.encode(&rgba_data, width, height, image_webp::ColorType::Rgba8)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::*;

    #[test]
    fn test_renderer_creation() {
        let size = Size::new(800.0, 800.0);
        let renderer = Renderer::new(size).unwrap();
        assert_eq!(renderer.size.width, 800.0);
        assert_eq!(renderer.size.height, 800.0);
    }

    #[test]
    fn test_text_rendering() {
        let size = Size::new(400.0, 200.0);
        let mut renderer = Renderer::new(size).unwrap();
        renderer.clear(Color::white());
        renderer.draw_text("Hello", 10.0, 50.0, 24.0, Color::black());

        let width = renderer.text_width("Hello", 24.0);
        assert!(width > 0.0);

        renderer.save("test_text.png").unwrap();
        let path = std::path::Path::new("test_text.png");
        assert!(path.exists());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_chart_rendering() {
        let size = Size::new(800.0, 800.0);
        let mut renderer = Renderer::new(size).unwrap();

        let chart_data = ChartData {
            geo_pos: GeoPos::new(40.7128, -74.0060, 0.0),
            julian_day: 2451545.0,
            planets: vec![
                PlanetPosition::new("Sun", Position::new(280.0, 0.0, 0.0, 0.0, 0.0, 0.0), false),
                PlanetPosition::new("Moon", Position::new(45.0, 0.0, 0.0, 0.0, 0.0, 0.0), false),
            ],
            houses: HouseCusps {
                asc: 180.0,
                mc: 90.0,
                dc: 0.0,
                ic: 270.0,
                houses: [
                    180.0, 210.0, 240.0, 270.0, 300.0, 330.0, 0.0, 30.0, 60.0, 90.0, 120.0, 150.0,
                ],
                system: HouseSystem::Placidus,
            },
            sidereal_time: 0.0,
        };

        let result = renderer.render_chart(&chart_data);
        assert!(result.is_ok());

        renderer.save("test_chart.png").unwrap();
        let path = std::path::Path::new("test_chart.png");
        assert!(path.exists());

        std::fs::remove_file(path).unwrap();
    }
}
