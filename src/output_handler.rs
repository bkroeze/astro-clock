use crate::aspects::{self, AspectConfig};
use crate::chart::{planet, ChartData, HouseCusps};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Png,
    Webp,
    Svg,
    Markdown,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "webp" => OutputFormat::Webp,
            "svg" => OutputFormat::Svg,
            "md" | "markdown" => OutputFormat::Markdown,
            _ => OutputFormat::Png,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::Webp => "webp",
            OutputFormat::Svg => "svg",
            OutputFormat::Markdown => "md",
        }
    }
}

pub struct OutputHandler;

impl OutputHandler {
    pub fn save_chart(
        chart_data: &ChartData,
        format: OutputFormat,
        output_path: &Path,
    ) -> Result<(), crate::errors::Error> {
        match format {
            OutputFormat::Png => Self::save_png(chart_data, output_path),
            OutputFormat::Webp => Self::save_webp(chart_data, output_path),
            OutputFormat::Svg => Self::save_svg(chart_data, output_path),
            OutputFormat::Markdown => Self::save_markdown(chart_data, output_path),
        }
    }

    fn save_png(chart_data: &ChartData, output_path: &Path) -> Result<(), crate::errors::Error> {
        let size = crate::renderer::Size::new(800.0, 800.0);
        let mut renderer = crate::renderer::Renderer::new(size)
            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
        renderer
            .render_chart(chart_data)
            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
        renderer
            .save(output_path.to_str().unwrap())
            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
        Ok(())
    }

    fn save_webp(chart_data: &ChartData, output_path: &Path) -> Result<(), crate::errors::Error> {
        let size = crate::renderer::Size::new(800.0, 800.0);
        let mut renderer = crate::renderer::Renderer::new(size)
            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
        renderer
            .render_chart(chart_data)
            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
        renderer
            .save_webp(output_path.to_str().unwrap())
            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
        Ok(())
    }

    fn save_svg(chart_data: &ChartData, output_path: &Path) -> Result<(), crate::errors::Error> {
        use crate::svg_renderer::SvgRenderer;
        let renderer = SvgRenderer::new(800, 800);
        let svg_content = renderer
            .render_chart(chart_data)
            .map_err(|e| crate::errors::Error::Chart(e.to_string()))?;
        std::fs::write(output_path, svg_content).map_err(|e| crate::errors::Error::Io(e))?;
        Ok(())
    }

    fn save_markdown(
        chart_data: &ChartData,
        output_path: &Path,
    ) -> Result<(), crate::errors::Error> {
        let content = Self::format_markdown_table(chart_data, None);
        std::fs::write(output_path, content).map_err(|e| crate::errors::Error::Io(e))?;
        Ok(())
    }

    pub fn save_markdown_with_orb(
        chart_data: &ChartData,
        output_path: &Path,
        orb: f64,
    ) -> Result<(), crate::errors::Error> {
        let content = Self::format_markdown_table(chart_data, Some(orb));
        std::fs::write(output_path, content).map_err(|e| crate::errors::Error::Io(e))?;
        Ok(())
    }

    fn format_markdown_table(chart_data: &ChartData, orb: Option<f64>) -> String {
        // Convert Julian Day to datetime
        let (year, month, day, hour) =
            crate::ephemeris::datetime_from_julian_day(chart_data.julian_day);
        let hour_int = hour as i32;
        let minute_int = ((hour - hour_int as f64) * 60.0) as i32;

        // Format date string
        let date_str = format!(
            "{:02}-{}-{:04}, {:02}:{:02} UT/GMT",
            day,
            match month {
                1 => "Jan",
                2 => "Feb",
                3 => "Mar",
                4 => "Apr",
                5 => "May",
                6 => "Jun",
                7 => "Jul",
                8 => "Aug",
                9 => "Sep",
                10 => "Oct",
                11 => "Nov",
                12 => "Dec",
                _ => "???",
            },
            year,
            hour_int,
            minute_int
        );

        // Zodiac symbols
        let zodiac_symbols = [
            "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
        ];

        // Planet symbols
        let planet_symbols: HashMap<&str, &str> = [
            (planet::SUN, "☉"),
            (planet::MOON, "☽"),
            (planet::MERCURY, "☿"),
            (planet::VENUS, "♀"),
            (planet::MARS, "♂"),
            (planet::JUPITER, "♃"),
            (planet::SATURN, "♄"),
            (planet::URANUS, "♅"),
            (planet::NEPTUNE, "♆"),
            (planet::PLUTO, "♇"),
            (planet::TRUE_NODE, "☊"),
            (planet::CHIRON, "⚷"),
        ]
        .iter()
        .cloned()
        .collect();

        // Define the desired order of planets
        let planet_order: Vec<&str> = vec![
            planet::SUN,
            planet::MOON,
            planet::MERCURY,
            planet::VENUS,
            planet::MARS,
            planet::JUPITER,
            planet::SATURN,
            planet::URANUS,
            planet::NEPTUNE,
            planet::PLUTO,
            planet::TRUE_NODE,
            planet::CHIRON,
        ];

        // Sort planets by the defined order
        let mut sorted_planets: Vec<_> = chart_data.planets.iter().collect();
        sorted_planets.sort_by(|a, b| {
            let a_index = planet_order
                .iter()
                .position(|&p| p == a.name.as_str())
                .unwrap_or(999);
            let b_index = planet_order
                .iter()
                .position(|&p| p == b.name.as_str())
                .unwrap_or(999);
            a_index.cmp(&b_index)
        });

        // Build markdown table
        let mut md = String::new();
        md.push_str("## Current Planets\n\n");
        md.push_str(&format!("**{}**\n\n", date_str));
        md.push_str(&format!("*House System: {}*\n\n", chart_data.houses.system));
        md.push_str("| Planet | Deg | Sign | Position | House |\n");
        md.push_str("|--------|-----|------|----------|-------|\n");

        for planet in &sorted_planets {
            let name = &planet.name;
            let symbol = planet_symbols.get(name.as_str()).unwrap_or(&"");
            let longitude = planet.position.longitude;

            // Calculate sign and degree
            let sign_index = (longitude / 30.0) as usize % 12;
            let degree_in_sign = longitude % 30.0;
            let sign_symbol = zodiac_symbols[sign_index];

            // Format position (degrees, minutes, retrograde)
            let deg = degree_in_sign as i32;
            let min = ((degree_in_sign - deg as f64) * 60.0) as i32;
            let retro = if planet.retrograde { "r" } else { "" };
            let position = format!("{:02}°{:02}'{}", deg, min, retro);

            // Find which house this planet is in
            let house = Self::calculate_house_number(longitude, &chart_data.houses);

            md.push_str(&format!(
                "| {} {} | {} | {} | {} | {} |\n",
                symbol, name, deg, sign_symbol, position, house
            ));
        }

        // Add aspects section
        let config = AspectConfig::new(orb.unwrap_or(3.0));
        let analysis = aspects::analyze_aspects(&chart_data.planets, config);

        md.push_str("\n## Aspects\n\n");

        // Moon void of course
        if let Some(void_info) = &analysis.moon_void_of_course {
            md.push_str(&format!("**{}**\n\n", void_info));
        }

        // Aspects table
        if !analysis.aspects.is_empty() {
            md.push_str("| Aspect | Planet 1 | Planet 2 | Orb |\n");
            md.push_str("|--------|----------|----------|-----|\n");

            for aspect in &analysis.aspects {
                md.push_str(&format!(
                    "| {} {} | {} | {} | {:.2}° |\n",
                    aspect.aspect_type.symbol(),
                    aspect.aspect_type.name(),
                    aspect.planet1,
                    aspect.planet2,
                    aspect.orb
                ));
            }
        } else {
            md.push_str("*No major aspects within orb*\n");
        }

        // Grand Trines
        if !analysis.grand_trines.is_empty() {
            md.push_str("\n### Grand Trines\n\n");
            for gt in &analysis.grand_trines {
                md.push_str(&format!(
                    "- {} △ {} △ {}\n",
                    gt.planet1, gt.planet2, gt.planet3
                ));
            }
        }

        // Lunar Mansion
        if let Some(moon) = sorted_planets.iter().find(|p| p.name == planet::MOON) {
            let mansion = Self::calculate_lunar_mansion(moon.position.longitude);
            md.push_str(&format!(
                "\n## Lunar Mansion\n\n**Lunar Mansion: {}**\n",
                mansion
            ));
        }

        md
    }

    fn calculate_house_number(longitude: f64, houses: &HouseCusps) -> i32 {
        // Find which house cusp is before this longitude
        for i in 0..12 {
            let cusp = houses.houses[i];
            let next_cusp = houses.houses[(i + 1) % 12];

            // Handle wrap-around at 360 degrees
            let in_house = if next_cusp < cusp {
                // House crosses 0° Aries
                longitude >= cusp || longitude < next_cusp
            } else {
                longitude >= cusp && longitude < next_cusp
            };

            if in_house {
                return (i + 1) as i32;
            }
        }
        1 // Default to house 1
    }

    fn calculate_lunar_mansion(moon_longitude: f64) -> i32 {
        // Lunar Mansions are 28 divisions of the zodiac, each 12°51'26" (approx 12.857°)
        // Mansion 1 starts at 0° Aries
        const MANSION_WIDTH: f64 = 12.0 + 51.0 / 60.0 + 26.0 / 3600.0; // 12°51'26" in decimal
        let mansion = (moon_longitude / MANSION_WIDTH) as i32 + 1;
        // Ensure we return 1-28 (wrap around if needed)
        ((mansion - 1) % 28) + 1
    }

    fn format_svg(chart_data: &ChartData) -> String {
        let size = 800.0;
        let center = size / 2.0;
        let radius = size * 0.35;

        let zodiac_symbols = [
            "♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓",
        ];

        let planet_symbols: HashMap<&str, &str> = [
            ("Sun", "☉"),
            ("Moon", "☽"),
            ("Mercury", "☿"),
            ("Venus", "♀"),
            ("Mars", "♂"),
            ("Jupiter", "♃"),
            ("Saturn", "♄"),
            ("Uranus", "♅"),
            ("Neptune", "♆"),
            ("Pluto", "♇"),
            ("True Node", "☊"),
            ("Mean Node", "☊"),
            ("Chiron", "⚷"),
        ]
        .iter()
        .cloned()
        .collect();

        let mut svg = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}">
  <rect width="{size}" height="{size}" fill="white"/>
  <g transform="translate({center}, {center})">
    <!-- Outer circle -->
    <circle cx="0" cy="0" r="{radius}" fill="none" stroke="black" stroke-width="2"/>
"#,
            size = size,
            center = center,
            radius = radius
        );

        // Draw degree ticks
        for degree in 0..360 {
            let length = if degree % 30 == 0 {
                radius * 0.88
            } else if degree % 10 == 0 {
                radius * 0.92
            } else if degree % 5 == 0 {
                radius * 0.95
            } else {
                continue;
            };

            let angle = (degree as f64).to_radians();
            let x1 = radius * angle.cos();
            let y1 = radius * angle.sin();
            let x2 = length * angle.cos();
            let y2 = length * angle.sin();
            let stroke_width = if degree % 30 == 0 { 2.0 } else { 1.0 };

            svg.push_str(&format!(
                r#"    <line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="black" stroke-width="{stroke_width}"/>
"#,
                x1 = x1, y1 = y1, x2 = x2, y2 = y2, stroke_width = stroke_width
            ));
        }

        // Draw zodiac sign labels
        for (i, sign) in zodiac_symbols.iter().enumerate() {
            let sign_mid = ((i * 30 + 15) as f64 + 180.0).to_radians();
            let label_radius = radius * 1.15;
            let x = label_radius * sign_mid.cos();
            let y = label_radius * sign_mid.sin();

            svg.push_str(&format!(
                r#"    <text x="{x:.1}" y="{y:.1}" text-anchor="middle" dominant-baseline="central" font-size="24" font-family="serif">{sign}</text>
"#,
                x = x, y = y, sign = sign
            ));
        }

        // Draw house cusps
        for &cusp in &chart_data.houses.houses {
            let rotated_cusp = (cusp + 180.0) % 360.0;
            let angle = rotated_cusp.to_radians();
            let x1 = radius * angle.cos();
            let y1 = radius * angle.sin();
            let x2 = radius * 0.7 * angle.cos();
            let y2 = radius * 0.7 * angle.sin();

            svg.push_str(&format!(
                r#"    <line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="black" stroke-width="1.5"/>
"#,
                x1 = x1, y1 = y1, x2 = x2, y2 = y2
            ));

            // Draw house degree labels
            let label_radius = radius * 0.55;
            let label_x = label_radius * angle.cos();
            let label_y = label_radius * angle.sin();
            let degree_in_sign = cusp % 30.0;
            let sign_index = (cusp / 30.0) as usize % 12;
            let sign_symbol = zodiac_symbols[sign_index];
            let deg = degree_in_sign as i32;
            let min = ((degree_in_sign - deg as f64) * 60.0) as i32;
            let label = format!("{}{:02}°{:02}'", sign_symbol, deg, min);

            svg.push_str(&format!(
                r#"    <text x="{label_x:.1}" y="{label_y:.1}" text-anchor="middle" dominant-baseline="central" font-size="10" font-family="sans-serif">{label}</text>
"#,
                label_x = label_x, label_y = label_y, label = label
            ));
        }

        // Draw planets
        let planet_radius = radius * 0.75;
        for planet in &chart_data.planets {
            let position = (planet.position.longitude + 180.0) % 360.0;
            let angle = position.to_radians();
            let x = planet_radius * angle.cos();
            let y = planet_radius * angle.sin();

            let symbol = planet_symbols
                .get(planet.name.as_str())
                .copied()
                .unwrap_or_else(|| &planet.name.as_str());

            svg.push_str(&format!(
                r#"    <text x="{x:.1}" y="{y:.1}" text-anchor="middle" dominant-baseline="central" font-size="18" font-family="serif">{symbol}</text>
"#,
                x = x, y = y, symbol = symbol
            ));

            if planet.retrograde {
                svg.push_str(&format!(
                    r#"    <text x="{x:.1}" y="{y:.1}" dx="12" dy="-8" text-anchor="middle" dominant-baseline="central" font-size="10" font-family="sans-serif">r</text>
"#,
                    x = x, y = y
                ));
            }
        }

        svg.push_str("  </g>\n</svg>\n");
        svg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("png"), OutputFormat::Png);
        assert_eq!(OutputFormat::from_str("PNG"), OutputFormat::Png);
        assert_eq!(OutputFormat::from_str("webp"), OutputFormat::Webp);
        assert_eq!(OutputFormat::from_str("WEBP"), OutputFormat::Webp);
        assert_eq!(OutputFormat::from_str("svg"), OutputFormat::Svg);
        assert_eq!(OutputFormat::from_str("SVG"), OutputFormat::Svg);
        assert_eq!(OutputFormat::from_str("md"), OutputFormat::Markdown);
        assert_eq!(OutputFormat::from_str("markdown"), OutputFormat::Markdown);
        assert_eq!(OutputFormat::from_str("unknown"), OutputFormat::Png);
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Png.extension(), "png");
        assert_eq!(OutputFormat::Webp.extension(), "webp");
        assert_eq!(OutputFormat::Svg.extension(), "svg");
        assert_eq!(OutputFormat::Markdown.extension(), "md");
    }
}
