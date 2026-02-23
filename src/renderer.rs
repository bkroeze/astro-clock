use std::path::Path;
use tiny_skia::{Canvas, Paint, Pixmap, Transform};

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
}

pub struct Renderer {
    canvas: Canvas,
    paint: Paint,
    size: Size,
}

impl Renderer {
    pub fn new(size: Size) -> Self {
        let pixmap = Pixmap::new(size.width as u32, size.height as u32).unwrap();
        let canvas = Canvas::from(pixmap);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);

        Self {
            canvas,
            paint,
            size,
        }
    }

    pub fn render_chart(
        &mut self,
        chart_data: &crate::chart::ChartData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Clear background
        self.clear(Color::gray(0.95));

        // Calculate center and radius
        let center = self.size.center();
        let radius = f32::min(self.size.width, self.size.height) * 0.4;

        // Draw zodiac wheel
        self.draw_zodiac_wheel(center, radius)?;

        // Draw house cusps
        self.draw_house_cusps(center, radius, &chart_data.houses)?;

        // Draw planets
        self.draw_planets(center, radius, &chart_data.planets)?;

        // Draw house labels
        self.draw_house_labels(center, radius)?;

        Ok(())
    }

    fn draw_zodiac_wheel(
        &mut self,
        center: Point,
        radius: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Draw outer circle
        self.draw_circle(center, radius, Color::black());

        // Draw zodiac degree lines
        for degree in 0..360 {
            let angle = (degree as f32).to_radians();
            let length = if degree % 30 == 0 {
                radius * 0.9
            } else {
                radius * 0.95
            };

            let start = Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            let end = Point::new(
                center.x + length * angle.cos(),
                center.y + length * angle.sin(),
            );

            self.draw_line(start, end, Color::gray(0.7), 1.0);
        }

        Ok(())
    }

    fn draw_house_cusps(
        &mut self,
        center: Point,
        radius: f32,
        houses: &crate::chart::HouseCusps,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Draw house cusp lines
        let house_cusps = houses.houses;
        for (i, &cusp) in house_cusps.iter().enumerate() {
            let angle = (cusp as f32).to_radians();
            let start = Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            );
            let end = Point::new(
                center.x + (radius * 0.8) * angle.cos(),
                center.y + (radius * 0.8) * angle.sin(),
            );

            // Different colors for different house systems
            let color = match i {
                0 | 3 | 6 | 9 => Color::red(),
                1 | 4 | 7 | 10 => Color::blue(),
                _ => Color::black(),
            };

            self.draw_line(start, end, color, 2.0);
        }

        Ok(())
    }

    fn draw_planets(
        &mut self,
        center: Point,
        radius: f32,
        planets: &[crate::chart::PlanetPosition],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for planet in planets {
            let position = planet.position.longitude;
            let angle = (position as f32).to_radians();
            let planet_radius = radius * 0.7;

            let planet_pos = Point::new(
                center.x + planet_radius * angle.cos(),
                center.y + planet_radius * angle.sin(),
            );

            // Different sizes for different planets
            let size = match planet.name.as_str() {
                crate::chart::planet::SUN => 12.0,
                crate::chart::planet::MOON => 10.0,
                crate::chart::planet::MERCURY => 6.0,
                crate::chart::planet::VENUS => 8.0,
                crate::chart::planet::MARS => 7.0,
                crate::chart::planet::JUPITER => 9.0,
                crate::chart::planet::SATURN => 8.0,
                crate::chart::planet::URANUS => 7.0,
                crate::chart::planet::NEPTUNE => 7.0,
                crate::chart::planet::PLUTO => 6.0,
                crate::chart::planet::CHIRON => 6.0,
                _ => 5.0,
            };

            // Color based on planet type
            let color = match planet.name.as_str() {
                crate::chart::planet::SUN => Color::new(1.0, 0.8, 0.0, 1.0), // Yellow
                crate::chart::planet::MOON => Color::new(0.8, 0.8, 1.0, 1.0), // Light blue
                crate::chart::planet::MERCURY => Color::new(0.8, 0.8, 0.8, 1.0), // Gray
                crate::chart::planet::VENUS => Color::new(1.0, 0.5, 0.5, 1.0), // Pink
                crate::chart::planet::MARS => Color::new(1.0, 0.0, 0.0, 1.0), // Red
                crate::chart::planet::JUPITER => Color::new(0.8, 0.6, 0.2, 1.0), // Orange
                crate::chart::planet::SATURN => Color::new(0.6, 0.5, 0.4, 1.0), // Brown
                crate::chart::planet::URANUS => Color::new(0.0, 0.8, 1.0, 1.0), // Cyan
                crate::chart::planet::NEPTUNE => Color::new(0.3, 0.3, 1.0, 1.0), // Blue
                crate::chart::planet::PLUTO => Color::new(0.5, 0.3, 0.7, 1.0), // Purple
                crate::chart::planet::CHIRON => Color::new(0.8, 0.4, 0.2, 1.0), // Orange-brown
                _ => Color::black(),
            };

            self.draw_circle(planet_pos, size, color);

            // Draw planet name
            let text_pos = Point::new(planet_pos.x + size + 3.0, planet_pos.y + 4.0);
            self.draw_text(&planet.name, text_pos, Color::black(), 10.0);
        }

        Ok(())
    }

    fn draw_house_labels(
        &mut self,
        center: Point,
        radius: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for house in 1..=12 {
            let angle = ((house as f32 - 0.5) * 30.0).to_radians();
            let label_pos = Point::new(
                center.x + (radius * 0.6) * angle.cos(),
                center.y + (radius * 0.6) * angle.sin(),
            );

            let text = format!("House {}", house);
            self.draw_text(&text, label_pos, Color::black(), 8.0);
        }

        Ok(())
    }

    pub fn clear(&mut self, color: Color) {
        self.paint.set_color(tiny_skia::Color::from_rgba8(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        ));
        self.canvas.fill_rect(
            tiny_skia::Rect::from_xywh(0.0, 0.0, self.size.width, self.size.height),
            &self.paint,
        );
    }

    pub fn draw_circle(&mut self, center: Point, radius: f32, color: Color) {
        self.paint.set_color(tiny_skia::Color::from_rgba8(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        ));

        self.canvas
            .fill_circle(center.x, center.y, radius, &self.paint);
    }

    pub fn draw_line(&mut self, start: Point, end: Point, color: Color, stroke_width: f32) {
        self.paint.set_color(tiny_skia::Color::from_rgba8(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        ));

        self.canvas
            .stroke_line(start.x, start.y, end.x, end.y, &self.paint, stroke_width);
    }

    pub fn draw_text(&mut self, text: &str, position: Point, color: Color, font_size: f32) {
        self.paint.set_color(tiny_skia::Color::from_rgba8(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        ));

        // Note: For simplicity, this example doesn't include font loading
        // In a real implementation, you would load a font and use text drawing methods
        // self.canvas.draw_text(text, position.x, position.y, font, &self.paint);
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_renderer_creation() {
        let size = Size::new(800.0, 800.0);
        let renderer = Renderer::new(size);
        assert_eq!(renderer.size.width, 800.0);
        assert_eq!(renderer.size.height, 800.0);
    }

    #[test]
    fn test_chart_rendering() {
        let size = Size::new(800.0, 800.0);
        let mut renderer = Renderer::new(size);

        // Create sample chart data
        let chart_data = ChartData {
            geo_pos: GeoPos::new(40.7128, -74.0060, 0.0),
            julian_day: 2451545.0,
            planets: vec![
                PlanetPosition::new("Sun", Position::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0), false),
                PlanetPosition::new("Moon", Position::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0), false),
            ],
            houses: HouseCusps {
                asc: 0.0,
                mc: 0.0,
                dc: 0.0,
                ic: 0.0,
                houses: [0.0; 12],
                system: HouseSystem::Placidus,
            },
            sidereal_time: 0.0,
        };

        // Test rendering
        let result = renderer.render_chart(&chart_data);
        assert!(result.is_ok());

        // Test saving
        renderer.save("test_chart.png");
        let path = std::path::Path::new("test_chart.png");
        assert!(path.exists());

        // Clean up test file
        std::fs::remove_file(path).unwrap();
    }
}

