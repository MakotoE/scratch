use super::*;

#[derive(Debug)]
pub struct SpriteRuntime {
    pub canvas: web_sys::CanvasRenderingContext2d,
    pub x: f64,
    pub y: f64,
}

impl SpriteRuntime {
    pub fn new(canvas: web_sys::CanvasRenderingContext2d) -> Self {
        Self{
            canvas,
            x: 0.0,
            y: 0.0,
        }
    }

    pub fn say(&self, s: &str) -> Result<()> {
        js_sys::Reflect::set(&self.canvas, &"font".into(), &"10px sans-serif".into())?;
        self.canvas.fill_text(s, 150.0 + self.x, 150.0 + self.y)?;
        Ok(())
    }
}
