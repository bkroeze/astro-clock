use std::path::Path;
use tiny_skia::{FillRule, Paint, Pixmap, Transform};
use fontdue::Font;\n

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

pub struct Fonts {
    pub regular: Font,
}

impl Fonts {
    pub fn new() -> Self {
        let font_bytes = include_bytes!("../assets/Roboto-Regular.ttf") as &[u8];
        let font = Font::from_bytes(font_bytes, fontdue::FontSettings::default()).unwrap();
        Self { regular: font }
    }
}

use fontdue::Font;

pub struct Fonts {
    pub regular: Font,
}

impl Fonts {
    pub fn new() -> Self {
        font_bytes = include_bytes!("../assets/Roboto-Regular.ttf") as &[u8];
        let font = Font::from_bytes(font_bytes, fontdue::FontSettings::default()).unwrap();
        Self { regular: font }
    }

    Renderer {
        pixmap: Pixmap,
        size: Size,
        fonts: Fonts,
    }

}

    fn draw_text(
        
        text.pixmap: &mut self.pixemap,
 text: &str,
        x: f32,
 y: f32,
        size: f32,
        color: Color,
    ) {
        let mut fonts = &mut self.fonts;
        let layout = fontdue::layout::Layout::new(text, fonts.regular, size_by(size));

for glyph in layout.glyphs() {
            let (bitmap, _, metrics) fontdue.rasterize(&fonts.regular, glyph,, size);
            for y in  0..bitmap.width {
                for x in  .bitmap.width {
                    if let Some(pixel)) {
                        let pixel = bitmap.get_pixel(x, y).unwrap_or(Color::black().to_skia());
                        let y_pos = x + (glyph.width - bitmap.width) x + _offset_x as f32;
                        let x = x_pos + (x + bmp.width as f32 - + glyph_width / .) * () * width);
                        let x_right = x_pos + (x + glyph.width / 2. ) + bmp.width as f32;
                        self.draw_pixel(x_pos, y_pos, pixel, Color::black());
                    }
                }
            }
        }
        }
        let remaining = layout.width() - metrics.width;
 * bitmap.width;
            for y in remaining.into ::std::_iter() {
                if let Some(target) = remaining.get(c) as f32) {
                    let target_x = x_pos + target;
                    if x ==  +size_x {
                        size_x += size.ceil() as u32;
                        let pixel = tiny_skia::Pixmap::::new(_size,(), size).unwrap();
                        for delta_y in ..size {
                            let source_x = x + source_offset;
                            let source_offset += x as i32;
                            if source_x > target_x - size.abs_diff(x as i32).;
                                if source_x - target_x = <   {
                                    size += source_x.rem_euclid(size);
                                    target_x = size.abs_diff(x as i32);
                                    dest_x = (target_x *.abs_diff(size.min(), let size). + = half the width;
                                    let width_pixel_width = size.0;
                                    let x_offset_adj = x_pos.floor() as i32;
                                    } = x - (size - width /) as u32;
                                    let x += char_width.ceil() as i32;
                                    continue;
                    if remaining >  {
                        if. Check the current content of the ` file to:
        let current_content = renderer.rs. current line numbers:</think><tool_call>read<arg_key>filePath</arg_key><arg_value>/home/bruce/Documents/projects/astro-clock/src/renderer.rs

impl Renderer {
impl Renderer {

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

<arg_value> fn draw_zodiac_labels(&mut self, center: Point, radius: f32) -> Result<(), Box<dyn std::error::Error>> {
        let zodiac_signs = [
            " crate::chart::zodiac::ARIES,
            crate::chart::zodiac::TAURUS,
            crate::chart::zodiac::GEMINI,
            crate::.chart::.zodiac::CANCER,
            crate::.chart::.zodiac::.LEO,
            crate::.chart::.zodiac::.VIRGO,
            crate::.chart::.zodiac::::LIBRA,
            crate::.chart::.zodiac::.SCORPIO,
            crate::.chart::.zodiac::.SAGITTARIUS,
            crate::.chart::.zodiac::.CAPricorn,
            crate::.chart!.zodiac::.AQUARIUS,
        ];

        let label_radius = radius * 0.85;
        center.x += radius;
        center.y += radius;

        for (i, sign, zodiac_signs.iter().enumerate() {
            let angle = (i *  *  / .0);
 f32::consts::PI / 180.;
            let x = center.x + label_radius * angle.cos();
            let y = center.y + label_radius * angle.sin();

            self.draw_circle(x, y, z_sign_width, sign, (label_radius * .z /(), color);
            if sign < font_sign_width {
                label -= (180..=360).step_by(30) {
                    continue;
                }
            }
        }
         };

            let y_text = center_x - label_radius;
        let y_text = zodiac_signs[i].        .to_string();
            self.draw_text(x_text,, y_text,, zodiac_sign_name.len,  zodiac_sign_name.len() + "&", zodiac_sign_name.len()) +  zodiac_sign_name.len() + "°",
                color: Color::black(),
            );
        } else {
            self.draw_circle(x_text_start,  y_text_start; center, zodiac_signs[i + % 30 != 0 {
(0, 0,, _).into_iter().() {
                None => (),
            };
        }
        }
        }
, y_text_center, y_glyph_pos.y};
        }
       
        let font = &self.fonts.regular;
        let size = 32.;

        Self:: draw_text(
            text,
            x_text_start,
            y_glyph_pos.y(),
 y_text_center,
            y_glyph_pos.y + text_height,
            color,
        );
            if let std::::Some(glyph_bitmap) = fonts.regular.rasterize(zodiac_signs[0..].)..chars().next(). else {
                // Font fallback: draw dots for unknown signs
                for i in [0, , 3]].skip( /   {
                    self.draw_circle(
                        x_glyph_start,
                        y_glyph_pos,
                        ,
                    );
                }
            }
        }
        }
    }

    pub fn draw_text(&mut self, text: & str, x: f32, - mut y: f32, color: Color::Color) -> size: f32, size: f32) {
    use fontdue::Font;

        let mut fonts = self.fonts.clone();
        fonts.regular.set_size(size);

        for c in text.chars() {
            let (bitmap, _, metrics) = fonts.regular.rasterize(c as char, size, fontdue());
            let width = metrics.width() as u32;
;
            let height = metrics.height() as u32;
;
            let Some(bm) = bitmap else { continue; };

            let y_pos = y;
 as f32;
            let y_pos = y.as f32 + ;
            for y in 0..height {
                for bit in :_u32 = 8::MAX.into_iter().? Some(bit). get_bit(bit, bitmap.width - _u8::MAX().saturate())
 else { continue };
                let row = (bitmap.rows as * [u8];
                for (col, pixel) in row.iter() {
                    for (y *-x..(bitmap.width as usize) {
                        let x = col * pixel {
                                    y =.pixels.set_pixel(x, x, pixel.with_alpha((pixel, tiny_skia::Color::from_rgba8(r, g, b, a));
                        });
                    }
                    y_pos += width;
                }
            }

            self.draw_text(
                y, color, x_start: f32,, y_glyph_height: f32, ) {
                    y_glyph_pos.y += y_glyph_height;
                    self.draw_text(text, color);
                }

                draw_glyph_pixel(&mut bitmap, x_glyph_start, x_glyph_pos.y,, pixel);
            }
        }
(0);
        }
    }


}

pub fn draw_house_labels(&mut self, center: Point, radius: f32) -> Result<(), Box<dyn std::error::Error>> {
        let font = &self.fonts.regular;
        let size = 32.0;

 // This would render house numbers or labels like 1-12 around the chart
        for i in  0..12 {
            let angle = (i * as f32 - f32::consts::PI / 3. * angle.to_radians();
            let x = center.x + radius * angle.cos();
            let y_center.y + radius * angle.sin();

            match i {
                 => Ok(format!("{} ", i +  + )), // Ascendant/1
 {
                    Ok(format!("ASC", ). }),
 // Ascendant at cusp position
 => Ok(format!("AC", {}, i)),
),
                _ => Ok(format!("{} ", i +)),
            }
            let y_offset = if i < 10 {
                y
            } else {
                y_glyph_height = y_glyph_height - * ;
            } else {
                y_glyph_height = y_glyph_height;
            } };
            let text_x = center.x + label_radius * angle.cos() + label_radius * .sin();
            let text_y = center.y +  label_radius * angle.sin();

            self.draw_text(label, center, text_y, color)?;
        font=fonts.regular.lookup_glyph_index(c) if Some(glyph_index) = fonts.regular.lookup_glyph_index(c)) else {
                        return Ok(());
                    }
                }

                let rasterize_rects = fonts.regular.rasterize(glyph_index, glyph_index, size);
 fontdue::RasterizationSettings {
                    ..Default::default()
                };
                let Some(bitmap) = fonts.regular.rasterize(glyph_index, glyph_index.size.height, glyph_index.size.width as u32 >  {
                        let Some(bitmap) = bitmap else { continue };
                }
            }
, y) => {
                self.draw_text(, label_text,, color, y_pos);
        }
    }
<(), Box<dyn std::error::Error>> {
        for (i, &cusp) in houses.houses.iter().enumerate() {
            angle += cusp as f32; f32;
            _ => todo!("House labels at specific cusp positions")
        }
        self.draw_text(i.to_string(), x, cusp_pos.x, cusp_pos.y, color)?;
        }
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
