use std::path::Path;
use tiny_skia::{FillRule, Paint, Pixmap, Transform};

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
}

impl Renderer {
    pub fn new(size: Size) -> Self {
        let pixmap = Pixmap::new(size.width as u32, size.height as u32).unwrap();
        Self { pixmap, size }
    }

    pub fn render_chart(
        &mut self,
        chart_data: &crate::chart::ChartData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.clear(Color::gray(0.95));

        let center = self.size.center();
        let radius = f32::min(self.size.width, self.size.height) * 0.4;

        self.draw_zodiac_wheel(center, radius)?;
        self.draw_house_cusps(center, radius, &chart_data.houses)?;
        self.draw_planets(center, radius, &chart_data.planets)?;
        self.draw_house_labels(center, radius)?;

        Ok(())
    }

    fn draw_zodiac_wheel(
        &mut self,
        center: Point,
        radius: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_circle(center, radius, Color::black());

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

            let color = match planet.name.as_str() {
                crate::chart::planet::SUN => Color::new(1.0, 0.8, 0.0, 1.0),
                crate::chart::planet::MOON => Color::new(0.8, 0.8, 1.0, 1.0),
                crate::chart::planet::MERCURY => Color::new(0.8, 0.8, 0.8, 1.0),
                crate::chart::planet::VENUS => Color::new(1.0, 0.5, 0.5, 1.0),
                crate::chart::planet::MARS => Color::new(1.0, 0.0, 0.0, 1.0),
                crate::chart::planet::JUPITER => Color::new(0.8, 0.6, 0.2, 1.0),
                crate::chart::planet::SATURN => Color::new(0.6, 0.5, 0.4, 1.0),
                crate::chart::planet::URANUS => Color::new(0.0, 0.8, 1.0, 1.0),
                crate::chart::planet::NEPTUNE => Color::new(0.3, 0.3, 1.0, 1.0),
                crate::chart::planet::PLUTO => Color::new(0.5, 0.3, 0.7, 1.0),
                crate::chart::planet::CHIRON => Color::new(0.8, 0.4, 0.2, 1.0),
                _ => Color::black(),
            };

            self.draw_circle(planet_pos, size, color);
        }

        Ok(())
    }

    fn draw_house_labels(
        &mut self,
        _center: Point,
        _radius: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
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

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(path);
        self.pixmap.save_png(path)?;
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
        let renderer = Renderer::new(size);
        assert_eq!(renderer.size.width, 800.0);
        assert_eq!(renderer.size.height, 800.0);
    }

    #[test]
    fn test_chart_rendering() {
        let size = Size::new(800.0, 800.0);
        let mut renderer = Renderer::new(size);

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
