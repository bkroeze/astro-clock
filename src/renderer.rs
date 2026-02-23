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

    pub fn save(&self, path: &str) {
        let pixmap = self.canvas.device().pixmap().clone();
        pixmap.save_png(Path::new(path)).unwrap();
    }
}
