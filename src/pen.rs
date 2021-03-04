use crate::coordinate::{CanvasCoordinate, SpriteCoordinate};
use crate::pen::PenStatus::PenUp;
use crate::sprite_runtime::GraphicsCostumeTexture;
use graphics::character::CharacterCache;
use graphics::{line, Context, Graphics};
use palette::{IntoColor, LinSrgb};
use piston_window::G2d;

#[derive(Debug, Clone)]
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

    pub fn draw<G, C>(&self, context: &mut Context, graphics: &mut G)
    where
        G: GraphicsCostumeTexture<C>,
        C: CharacterCache,
    {
        for line in &self.lines {
            line.draw(context, graphics);
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

    fn draw<G, C>(&self, context: &mut Context, graphics: &mut G)
    where
        G: GraphicsCostumeTexture<C>,
        C: CharacterCache,
    {
        let rgb: LinSrgb = self.color.into_rgb();
        let line = line::Line {
            color: [rgb.red, rgb.green, rgb.blue, 1.0],
            radius: self.size / 2.0,
            shape: line::Shape::Round,
        };

        let mut iter = self.points.iter();
        if let Some(point0) = iter.next() {
            if self.points.len() == 1 {
                let position: CanvasCoordinate = (*point0).into();
                line.draw(
                    [position.x, position.y, position.x, position.y],
                    &context.draw_state,
                    context.transform,
                    graphics,
                );
            } else {
                let mut last_position: CanvasCoordinate = (*point0).into();
                for point in iter {
                    let position: CanvasCoordinate = (*point).into();
                    line.draw(
                        [position.x, position.y, last_position.x, last_position.y],
                        &context.draw_state,
                        context.transform,
                        graphics,
                    );

                    last_position = position;
                }
            }
        } else {
            return;
        };
    }
}
