use super::*;
use coordinate::CanvasCoordinate;
use std::f64::consts::TAU;
use web_sys::HtmlImageElement;

pub struct CanvasContext {
    context: web_sys::CanvasRenderingContext2d,
}

impl CanvasContext {
    pub fn new(context: web_sys::CanvasRenderingContext2d) -> Self {
        Self { context }
    }

    pub fn begin_path(&self) {
        self.context.begin_path();
    }

    pub fn close_path(&self) {
        self.context.close_path();
    }

    pub fn move_to(&self, position: &CanvasCoordinate) {
        self.context.move_to(position.x, position.y);
    }

    pub fn rounded_corner(
        &self,
        arc_center: &CanvasCoordinate,
        radius: f64,
        corner: Corner,
        direction: Direction,
    ) -> Result<()> {
        let angles = corner.angles();
        let (start, end) = match direction {
            Direction::Clockwise => (angles.0, angles.1),
            Direction::CounterClockwise => (angles.1, angles.0),
        };

        Ok(self.context.arc_with_anticlockwise(
            arc_center.x,
            arc_center.y,
            radius,
            start,
            end,
            matches! {direction, Direction::CounterClockwise},
        )?)
    }

    pub fn arc_to(&self, x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> Result<()> {
        Ok(self.context.arc_to(x1, y1, x2, y2, radius)?)
    }

    pub fn bezier_curve_to(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.context.bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y);
    }

    pub fn line_to(&self, position: &CanvasCoordinate) {
        self.context.line_to(position.x, position.y);
    }

    pub fn set_font(&self, font: &str) {
        self.context.set_font(font);
    }

    pub fn measure_text(&self, text: &str) -> Result<f64> {
        Ok(self.context.measure_text(text)?.width())
    }

    pub fn set_fill_style(&self, style: &str) {
        self.context.set_fill_style(&style.into())
    }

    pub fn set_stroke_style(&self, style: &str) {
        self.context.set_stroke_style(&style.into())
    }

    pub fn fill(&self) {
        self.context.fill()
    }

    pub fn stroke(&self) {
        self.context.stroke()
    }

    pub fn set_line_width(&self, width: f64) {
        self.context.set_line_width(width)
    }

    pub fn fill_text(&self, s: &str, position: &CanvasCoordinate) -> Result<()> {
        Ok(self.context.fill_text(s, position.x, position.y)?)
    }

    pub fn set_line_cap(&self, value: &str) {
        self.context.set_line_cap(value);
    }

    pub fn draw_image(&self, image: &HtmlImageElement, position: &CanvasCoordinate) -> Result<()> {
        Ok(self
            .context
            .draw_image_with_html_image_element(image, position.x, position.y)?)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Corner {
    TopRight,
    BottomRight,
    BottomLeft,
    TopLeft,
}

impl Corner {
    fn angles(&self) -> Angles {
        const RIGHT: f64 = 0.0 / 4.0 * TAU;
        const DOWN: f64 = 1.0 / 4.0 * TAU;
        const LEFT: f64 = 2.0 / 4.0 * TAU;
        const UP: f64 = 3.0 / 4.0 * TAU;

        match self {
            Corner::TopRight => Angles(UP, RIGHT),
            Corner::BottomRight => Angles(RIGHT, DOWN),
            Corner::BottomLeft => Angles(DOWN, LEFT),
            Corner::TopLeft => Angles(LEFT, UP),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Angles(f64, f64);

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}
