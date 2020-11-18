use super::*;
use coordinate::CanvasCoordinate;

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

    pub fn arc(
        &self,
        arc_center: &CanvasCoordinate,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<()> {
        Ok(self
            .context
            .arc(arc_center.x, arc_center.y, radius, start_angle, end_angle)?)
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
