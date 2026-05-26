//! SVG Chart Renderer
//! Generates standalone SVG charts using embedded glyph paths from the glyph registry.
//! This mirrors the functionality of the PNG renderer but outputs scalable SVG instead.

use std::collections::HashMap;

use crate::aspects::{AspectConfig, AspectType, find_aspects};
use crate::chart::{ChartData, HouseCusps, PlanetPosition};
use crate::svg_glyphs::{GlyphData, GlyphRegistry};

/// SVG Renderer for astrological charts
pub struct SvgRenderer {
    width: u32,
    height: u32,
    glyph_registry: GlyphRegistry,
}

impl SvgRenderer {
    /// Create a new SVG renderer with the specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            glyph_registry: GlyphRegistry::new(),
        }
    }

    /// Render a chart to SVG string
    pub fn render_chart(
        &self,
        chart_data: &ChartData,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let radius = f32::min(center_x, center_y) * 0.35;

        let mut svg_elements = Vec::new();

        // SVG header
        svg_elements.push(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}" style="background-color: white;">"#,
            self.width, self.height, self.width, self.height
        ));

        // Defs for reusable glyphs
        svg_elements.push(self.generate_glyph_defs());

        // Draw chart components
        let house_anchor = chart_data.houses.houses[0] as f32;

        self.draw_zodiac_wheel(&mut svg_elements, center_x, center_y, radius, house_anchor)?;
        self.draw_zodiac_signs(&mut svg_elements, center_x, center_y, radius, house_anchor)?;
        self.draw_house_cusps(
            &mut svg_elements,
            center_x,
            center_y,
            radius,
            &chart_data.houses,
        )?;
        self.draw_aspects(
            &mut svg_elements,
            center_x,
            center_y,
            radius,
            house_anchor,
            &chart_data.planets,
        )?;
        self.draw_planets(
            &mut svg_elements,
            center_x,
            center_y,
            radius,
            house_anchor,
            &chart_data.planets,
        )?;
        self.draw_house_labels(
            &mut svg_elements,
            center_x,
            center_y,
            radius,
            &chart_data.houses,
        )?;

        // Close SVG
        svg_elements.push("</svg>".to_string());

        Ok(svg_elements.join("\n"))
    }

    /// Generate SVG defs section with embedded glyph paths
    fn generate_glyph_defs(&self) -> String {
        let mut defs = vec!["<defs>".to_string()];

        // Define all glyphs as reusable paths
        let glyph_names = [
            "sun",
            "moon",
            "mercury",
            "venus",
            "mars",
            "jupiter",
            "saturn",
            "uranus",
            "neptune",
            "pluto",
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
            "retrograde",
            "chiron",
            "north_node",
            "south_node",
            "conjunction",
            "sextile",
            "square",
            "trine",
            "opposition",
        ];

        for name in &glyph_names {
            if let Some(glyph) = self.glyph_registry.get(name) {
                defs.push(format!(r#"<path id="{name}" d="{}" />"#, glyph.path));
            }
        }

        defs.push("</defs>".to_string());
        defs.join("\n")
    }

    /// Draw the zodiac wheel (outer circle and degree markers)
    fn draw_zodiac_wheel(
        &self,
        svg: &mut Vec<String>,
        center_x: f32,
        center_y: f32,
        radius: f32,
        house_anchor: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Outer circle
        svg.push(format!(
            r#"<circle cx="{}" cy="{}" r="{}" fill="none" stroke="black" stroke-width="2"/>"#,
            center_x, center_y, radius
        ));

        // Degree markers
        for degree in (0..360).step_by(5) {
            let length = if degree % 30 == 0 {
                radius * 0.88
            } else if degree % 10 == 0 {
                radius * 0.92
            } else {
                radius * 0.95
            };

            let angle_rad = Self::zodiac_angle(degree as f32, house_anchor);
            let x1 = center_x + radius * angle_rad.cos();
            let y1 = center_y + radius * angle_rad.sin();
            let x2 = center_x + length * angle_rad.cos();
            let y2 = center_y + length * angle_rad.sin();

            let stroke_width = if degree % 30 == 0 { 2.0 } else { 1.0 };

            svg.push(format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="{}"/>"#,
                x1, y1, x2, y2, stroke_width
            ));
        }

        Ok(())
    }

    /// Draw zodiac sign symbols around the wheel
    fn draw_zodiac_signs(
        &self,
        svg: &mut Vec<String>,
        center_x: f32,
        center_y: f32,
        radius: f32,
        house_anchor: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        let label_radius = radius * 1.15;
        let symbol_size = 18.0;

        for (i, sign) in signs.iter().enumerate() {
            let angle_rad = Self::zodiac_angle((i * 30 + 15) as f32, house_anchor);

            let x = center_x + label_radius * angle_rad.cos();
            let y = center_y + label_radius * angle_rad.sin();

            if let Some(glyph) = self.glyph_registry.get(sign) {
                let transform = self.calculate_glyph_transform(glyph, x, y, symbol_size);
                let use_element =
                    "<use href=\"#".to_string() + *sign + "\" transform=\"" + &transform + "\"/>";
                svg.push(use_element);
            }
        }

        Ok(())
    }

    /// Draw house cusp lines
    fn draw_house_cusps(
        &self,
        svg: &mut Vec<String>,
        center_x: f32,
        center_y: f32,
        radius: f32,
        houses: &HouseCusps,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let inner_radius = radius * 0.7;
        let house_anchor = houses.houses[0] as f32;

        for &cusp in &houses.houses {
            let angle_rad = Self::zodiac_angle(cusp as f32, house_anchor);

            let x1 = center_x + radius * angle_rad.cos();
            let y1 = center_y + radius * angle_rad.sin();
            let x2 = center_x + inner_radius * angle_rad.cos();
            let y2 = center_y + inner_radius * angle_rad.sin();

            svg.push(format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="1.5"/>"#,
                x1, y1, x2, y2
            ));
        }

        Ok(())
    }

    /// Draw aspect lines between planets
    fn draw_aspects(
        &self,
        svg: &mut Vec<String>,
        center_x: f32,
        center_y: f32,
        radius: f32,
        house_anchor: f32,
        planets: &[PlanetPosition],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = AspectConfig::new(3.0);
        let aspects = find_aspects(planets, config);

        let aspect_radius = radius * 0.5;

        for aspect in aspects {
            let planet1 = planets.iter().find(|p| p.name == aspect.planet1);
            let planet2 = planets.iter().find(|p| p.name == aspect.planet2);

            if let (Some(p1), Some(p2)) = (planet1, planet2) {
                let angle1 = Self::zodiac_angle(p1.position.longitude as f32, house_anchor);
                let angle2 = Self::zodiac_angle(p2.position.longitude as f32, house_anchor);

                let x1 = center_x + aspect_radius * angle1.cos();
                let y1 = center_y + aspect_radius * angle1.sin();
                let x2 = center_x + aspect_radius * angle2.cos();
                let y2 = center_y + aspect_radius * angle2.sin();

                let color = match aspect.aspect_type {
                    AspectType::Conjunction => "rgb(204, 204, 204)",
                    AspectType::Opposition => "rgb(255, 0, 0)",
                    AspectType::Square => "rgb(255, 0, 0)",
                    AspectType::Trine => "rgb(0, 128, 255)",
                    AspectType::Sextile => "rgb(0, 204, 0)",
                    AspectType::GrandTrine => "rgb(0, 128, 255)",
                };

                let stroke_width = if aspect.orb < 1.0 {
                    2.0
                } else if aspect.orb < 2.0 {
                    1.5
                } else {
                    1.0
                };

                svg.push(format!(
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>"#,
                    x1, y1, x2, y2, color, stroke_width
                ));
            }
        }

        Ok(())
    }

    /// Draw planets on the chart
    fn draw_planets(
        &self,
        svg: &mut Vec<String>,
        center_x: f32,
        center_y: f32,
        radius: f32,
        house_anchor: f32,
        planets: &[PlanetPosition],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Map planet names to glyph registry keys
        let planet_map: HashMap<&str, &str> = [
            ("Sun", "sun"),
            ("Moon", "moon"),
            ("Mercury", "mercury"),
            ("Venus", "venus"),
            ("Mars", "mars"),
            ("Jupiter", "jupiter"),
            ("Saturn", "saturn"),
            ("Uranus", "uranus"),
            ("Neptune", "neptune"),
            ("Pluto", "pluto"),
            ("True Node", "north_node"),
            ("Mean Node", "north_node"),
            ("Chiron", "chiron"),
        ]
        .iter()
        .cloned()
        .collect();

        let planet_radius = radius * 0.75;
        let symbol_size = 14.0;
        let retrograde_size = 8.0;

        for planet in planets {
            let angle_rad = Self::zodiac_angle(planet.position.longitude as f32, house_anchor);

            let x = center_x + planet_radius * angle_rad.cos();
            let y = center_y + planet_radius * angle_rad.sin();

            // Draw planet symbol
            if let Some(&glyph_name) = planet_map.get(planet.name.as_str())
                && let Some(glyph) = self.glyph_registry.get(glyph_name)
            {
                let transform = self.calculate_glyph_transform(glyph, x, y, symbol_size);
                let use_element = "<use href=\"#".to_string()
                    + glyph_name
                    + "\" transform=\""
                    + &transform
                    + "\"/>";
                svg.push(use_element);
            }

            // Draw retrograde indicator if applicable
            if planet.retrograde {
                let retro_x = x + symbol_size / 2.0 + 2.0;
                let retro_y = y - symbol_size / 2.0;
                svg.push(format!(
                    r#"<text x="{}" y="{}" font-size="{}" fill="black" font-family="sans-serif">r</text>"#,
                    retro_x, retro_y + retrograde_size / 2.0, retrograde_size
                ));
            }
        }

        Ok(())
    }

    /// Draw house degree labels
    fn draw_house_labels(
        &self,
        svg: &mut Vec<String>,
        center_x: f32,
        center_y: f32,
        radius: f32,
        houses: &HouseCusps,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let label_radius = radius * 0.6;
        let font_size = 10.0;
        let house_anchor = houses.houses[0] as f32;

        let zodiac_order = [
            ("aries", "♈"),
            ("taurus", "♉"),
            ("gemini", "♊"),
            ("cancer", "♋"),
            ("leo", "♌"),
            ("virgo", "♍"),
            ("libra", "♎"),
            ("scorpio", "♏"),
            ("sagittarius", "♐"),
            ("capricorn", "♑"),
            ("aquarius", "♒"),
            ("pisces", "♓"),
        ];

        for &cusp in &houses.houses {
            let angle_rad = Self::zodiac_angle(cusp as f32, house_anchor);

            let x = center_x + label_radius * angle_rad.cos();
            let y = center_y + label_radius * angle_rad.sin();

            let degree_in_sign = cusp % 30.0;
            let sign_index = (cusp / 30.0) as usize % 12;
            let sign_symbol = zodiac_order[sign_index].1;
            let deg = degree_in_sign as i32;
            let min = ((degree_in_sign - deg as f64) * 60.0) as i32;

            svg.push(format!(
                r#"<text x="{}" y="{}" font-size="{}" fill="black" font-family="sans-serif" text-anchor="middle" dominant-baseline="middle">{}{:02}°{:02}'</text>"#,
                x, y, font_size, sign_symbol, deg, min
            ));
        }

        Ok(())
    }

    /// Calculate SVG transform for positioning a glyph
    fn calculate_glyph_transform(
        &self,
        glyph: &GlyphData,
        x: f32,
        y: f32,
        target_size: f32,
    ) -> String {
        let glyph_width = glyph.width();
        let glyph_height = glyph.height();

        // Calculate scale to fit target size
        let scale = target_size / f32::max(glyph_width, glyph_height);

        // Center the glyph at the target position
        let offset_x = x - (glyph_width * scale) / 2.0;
        let offset_y = y - (glyph_height * scale) / 2.0 + glyph.baseline_offset * scale;

        format!("translate({}, {}) scale({})", offset_x, offset_y, scale)
    }

    fn zodiac_angle(longitude: f32, house_anchor: f32) -> f32 {
        (180.0 - (longitude - house_anchor))
            .rem_euclid(360.0)
            .to_radians()
    }

    /// Save SVG to file
    pub fn save(
        &self,
        chart_data: &ChartData,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let svg_content = self.render_chart(chart_data)?;
        std::fs::write(path, svg_content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::{GeoPos, HouseSystem, Position};

    fn create_test_chart() -> ChartData {
        ChartData {
            geo_pos: GeoPos::new(40.7128, -74.0060, 0.0),
            julian_day: 2451545.0,
            planets: vec![
                PlanetPosition::new("Sun", Position::new(280.0, 0.0, 0.0, 0.0, 0.0, 0.0), false),
                PlanetPosition::new("Moon", Position::new(45.0, 0.0, 0.0, 0.0, 0.0, 0.0), true),
                PlanetPosition::new(
                    "Mercury",
                    Position::new(275.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                    false,
                ),
                PlanetPosition::new(
                    "Venus",
                    Position::new(300.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                    false,
                ),
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
        }
    }

    #[test]
    fn test_svg_renderer_creation() {
        let renderer = SvgRenderer::new(800, 800);
        assert_eq!(renderer.width, 800);
        assert_eq!(renderer.height, 800);
    }

    #[test]
    fn test_svg_rendering() {
        let renderer = SvgRenderer::new(800, 800);
        let chart_data = create_test_chart();

        let svg = renderer.render_chart(&chart_data).unwrap();

        // Check that SVG contains expected elements
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("<line"));
        assert!(svg.contains("<use"));
    }

    #[test]
    fn test_glyph_transform() {
        let renderer = SvgRenderer::new(800, 800);
        let glyph = GlyphData {
            path: "M0,0 L100,100",
            view_box: (0.0, 0.0, 100.0, 100.0),
            baseline_offset: 10.0,
        };

        let transform = renderer.calculate_glyph_transform(&glyph, 400.0, 400.0, 20.0);

        // Transform should contain translate and scale
        assert!(transform.contains("translate("));
        assert!(transform.contains("scale("));
    }

    #[test]
    fn test_svg_save() {
        let renderer = SvgRenderer::new(800, 800);
        let chart_data = create_test_chart();

        let temp_path = "/tmp/test_chart.svg";
        renderer.save(&chart_data, temp_path).unwrap();

        assert!(std::path::Path::new(temp_path).exists());

        let content = std::fs::read_to_string(temp_path).unwrap();
        assert!(content.contains("<svg"));

        std::fs::remove_file(temp_path).unwrap();
    }
}
