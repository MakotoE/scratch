use super::*;
use coordinate::CanvasCoordinate;
use std::f64::consts::TAU;

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
    ) -> Result<()> {
        let angles = corner.angles();
        Ok(self
            .context
            .arc(arc_center.x, arc_center.y, radius, angles.start, angles.end)?)
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
            Corner::TopRight => Angles {
                start: UP,
                end: RIGHT,
            },
            Corner::BottomRight => Angles {
                start: RIGHT,
                end: DOWN,
            },
            Corner::BottomLeft => Angles {
                start: DOWN,
                end: LEFT,
            },
            Corner::TopLeft => Angles {
                start: LEFT,
                end: UP,
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Angles {
    start: f64,
    end: f64,
}
