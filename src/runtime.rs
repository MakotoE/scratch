use super::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SpriteRuntime {
    canvas: web_sys::CanvasRenderingContext2d,
    pub x: f64,
    pub y: f64,
    pub variables: HashMap<String, serde_json::Value>,
}

impl SpriteRuntime {
    pub fn new(canvas: web_sys::CanvasRenderingContext2d) -> Self {
        Self {
            canvas,
            x: 0.0,
            y: 0.0,
            variables: HashMap::new(),
        }
    }

    pub fn say(&self, s: &str) -> Result<()> {
        js_sys::Reflect::set(&self.canvas, &"font".into(), &"10px sans-serif".into())?;
        self.canvas.fill_text(s, 150.0 + self.x, 150.0 + self.y)?;
        Ok(())
    }
}
