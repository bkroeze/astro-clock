use crate::chart::{planet, ChartData, HouseCusps};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Png,
    Webp,
    Markdown,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "webp" => OutputFormat::Webp,
            "md" | "markdown" => OutputFormat::Markdown,
            _ => OutputFormat::Png,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::Webp => "webp",
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

    fn save_markdown(
        chart_data: &ChartData,
        output_path: &Path,
    ) -> Result<(), crate::errors::Error> {
        let content = Self::format_markdown_table(chart_data);
        std::fs::write(output_path, content).map_err(|e| crate::errors::Error::Io(e))?;
        Ok(())
    }

    fn format_markdown_table(chart_data: &ChartData) -> String {
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
        md.push_str("| Planet | Deg | Sign | Position | House |\n");
        md.push_str("|--------|-----|------|----------|-------|\n");

        for planet in sorted_planets {
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
        assert_eq!(OutputFormat::from_str("md"), OutputFormat::Markdown);
        assert_eq!(OutputFormat::from_str("markdown"), OutputFormat::Markdown);
        assert_eq!(OutputFormat::from_str("unknown"), OutputFormat::Png);
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Png.extension(), "png");
        assert_eq!(OutputFormat::Webp.extension(), "webp");
        assert_eq!(OutputFormat::Markdown.extension(), "md");
    }
}
