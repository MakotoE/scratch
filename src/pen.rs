use super::*;
use crate::pen::PenStatus::PenUp;
use sprite_runtime::{color_to_hex, SpriteCoordinate};

#[derive(Debug)]
pub struct Pen {
    lines: Vec<Line>,
    pen_status: PenStatus,
}

#[derive(Debug, Copy, Clone)]
enum PenStatus {
    PenUp,
    PenDown,
}

impl Pen {
    pub fn new() -> Self {
        let mut result = Self {
            lines: Vec::new(),
            pen_status: PenUp,
        };
        result.clear();
        result
    }

    pub fn color(&self) -> &palette::Hsv {
        self.lines.last().unwrap().color()
    }

    pub fn set_color(&mut self, color: &palette::Hsv) {
        self.new_line();
        self.lines.last_mut().unwrap().set_color(color);
    }

    pub fn size(&self) -> f64 {
        self.lines.last().unwrap().size()
    }

    pub fn set_size(&mut self, size: f64) {
        self.new_line();
        self.lines.last_mut().unwrap().set_size(size);
    }

    pub fn set_position(&mut self, position: &SpriteCoordinate) {
        match self.pen_status {
            PenStatus::PenDown => self.lines.last_mut().unwrap().add_point(position),
            PenStatus::PenUp => {}
        }
    }

    pub fn pen_down(&mut self, position: &SpriteCoordinate) {
        self.new_line();
        self.pen_status = PenStatus::PenDown;
        self.set_position(position);
    }

    pub fn pen_up(&mut self) {
        self.new_line();
        self.pen_status = PenStatus::PenUp;
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.lines
            .push(Line::new(&palette::Hsv::new(0.0, 1.0, 1.0), 1.0));
    }

    pub fn draw(&self, context: &web_sys::CanvasRenderingContext2d) {
        context.set_line_cap("round");
        for line in &self.lines {
            line.draw(context, None);
        }
    }

    fn new_line(&mut self) {
        let last_point = self.lines.last().unwrap().last_point();
        if let Some(point) = last_point {
            let mut line = Line::new(self.color(), self.size());
            match self.pen_status {
                PenStatus::PenDown => line.add_point(point),
                PenStatus::PenUp => {}
            }
            self.lines.push(line);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Line {
    points: Vec<SpriteCoordinate>,
    color: palette::Hsv,
    size: f64,
}

impl Line {
    fn new(color: &palette::Hsv, size: f64) -> Self {
        Self {
            points: Vec::new(),
            color: *color,
            size,
        }
    }

    fn color(&self) -> &palette::Hsv {
        &self.color
    }

    fn set_color(&mut self, color: &palette::Hsv) {
        self.color = *color;
    }

    fn size(&self) -> f64 {
        self.size
    }

    fn set_size(&mut self, size: f64) {
        self.size = size;
    }

    fn last_point(&self) -> Option<&SpriteCoordinate> {
        self.points.last()
    }

    fn add_point(&mut self, position: &SpriteCoordinate) {
        self.points.push(*position);
    }

    fn draw(
        &self,
        context: &web_sys::CanvasRenderingContext2d,
        extra_point: Option<SpriteCoordinate>,
    ) {
        context.begin_path();
        for (i, point) in self.points.iter().enumerate() {
            if i == 0 {
                context.move_to((240 + point.x) as f64, (180 - point.y) as f64);

                if self.points.len() == 1 {
                    context.line_to((240 + point.x) as f64, (180 - point.y) as f64);
                }
            } else {
                context.line_to((240 + point.x) as f64, (180 - point.y) as f64);
            }
        }

        if let Some(extra_point) = extra_point {
            context.line_to((240 + extra_point.x) as f64, (180 - extra_point.y) as f64);
        }

        let color_hex = color_to_hex(&self.color);
        context.set_stroke_style(&color_hex.as_str().into());
        context.set_line_width(self.size);
        context.stroke();
    }
}
