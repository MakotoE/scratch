use super::*;
use crate::coordinate::{canvas_const, CanvasCoordinate, CanvasRectangle, Transformation};
use ndarray::{Array1, Array2, Axis};
use palette::Srgba;
use std::f64::consts::TAU;
use std::fmt::{Debug, Formatter};
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

#[derive(Debug, Clone)]
pub struct CanvasContext<'a> {
    context: &'a CanvasRenderingContext2d,
    transformation: Transformation,
}

impl<'a> CanvasContext<'a> {
    pub fn new(context: &'a CanvasRenderingContext2d) -> Self {
        context.reset_transform().unwrap();
        context.scale(2.0, 2.0).unwrap();
        Self {
            context,
            transformation: Transformation::default(),
        }
    }

    pub fn with_transformation(&'a self, transformation: Transformation) -> Self {
        Self {
            context: self.context,
            transformation: self.transformation.apply_transformation(&transformation),
        }
    }

    pub fn begin_path(&self) {
        self.context.begin_path();
    }

    pub fn close_path(&self) {
        self.context.close_path();
    }

    pub fn move_to(&self, position: &CanvasCoordinate) {
        let position = self.transformation.apply_to_coordinate(position);
        self.context.move_to(position.x, position.y);
    }

    pub fn rounded_corner(
        &self,
        arc_center: &CanvasCoordinate,
        radius: f64,
        corner: Corner,
        direction: Direction,
    ) -> Result<()> {
        let arc_center = self.transformation.apply_to_coordinate(arc_center);
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

    pub fn arc_to(
        &self,
        from: &CanvasCoordinate,
        to: &CanvasCoordinate,
        radius: f64,
    ) -> Result<()> {
        let from = self.transformation.apply_to_coordinate(from);
        let to = self.transformation.apply_to_coordinate(to);
        Ok(self.context.arc_to(from.x, from.y, to.x, to.y, radius)?)
    }

    pub fn bezier_curve_to(
        &self,
        cp1: &CanvasCoordinate,
        cp2: &CanvasCoordinate,
        position: &CanvasCoordinate,
    ) {
        let cp1 = self.transformation.apply_to_coordinate(cp1);
        let cp2 = self.transformation.apply_to_coordinate(cp2);
        let position = self.transformation.apply_to_coordinate(position);
        self.context
            .bezier_curve_to(cp1.x, cp1.y, cp2.x, cp2.y, position.x, position.y);
    }

    pub fn line_to(&self, position: &CanvasCoordinate) {
        let position = self.transformation.apply_to_coordinate(position);
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
        self.context.fill();
    }

    pub fn stroke(&self) {
        self.context.stroke()
    }

    pub fn set_line_width(&self, width: f64) {
        self.context.set_line_width(width)
    }

    pub fn fill_text(&self, s: &str, position: &CanvasCoordinate) -> Result<()> {
        let position = self.transformation.apply_to_coordinate(position);
        Ok(self.context.fill_text(s, position.x, position.y)?)
    }

    pub fn set_line_cap(&self, value: &str) {
        self.context.set_line_cap(value);
    }

    pub fn draw_image(&self, image: &HtmlImageElement, rectangle: &CanvasRectangle) -> Result<()> {
        let rectangle = self.transformation.apply_to_rectangle(&rectangle);
        Ok(self
            .context
            .draw_image_with_html_image_element_and_dw_and_dh(
                image,
                rectangle.top_left.x,
                rectangle.top_left.y,
                rectangle.size.width,
                rectangle.size.height,
            )?)
    }

    pub fn clear(&self) {
        self.context.clear_rect(0.0, 0.0, 480.0, 360.0);
    }

    pub fn set_global_alpha(&self, value: f64) {
        self.context.set_global_alpha(value)
    }

    pub fn get_image_data(&self) -> Result<Array2<Srgba<u8>>> {
        let image_data = self
            .context
            .get_image_data(
                0.0,
                0.0,
                canvas_const::X_MAX * 2.0,
                canvas_const::Y_MAX * 2.0,
            )?
            .data()
            .0;
        let flat: Array1<Srgba<u8>> = image_data
            .chunks(4)
            .map(|c: &[u8]| Srgba::new(c[0], c[1], c[2], c[3]))
            .collect();
        let two_dimension = flat.into_shape([
            canvas_const::X_MAX as usize * 2,
            canvas_const::Y_MAX as usize * 2,
        ])?;

        let mut scaled: Array2<Srgba<u8>> =
            Array2::default([canvas_const::X_MAX as usize, canvas_const::Y_MAX as usize]);
        for (x, col) in two_dimension.axis_iter(Axis(0)).step_by(2).enumerate() {
            for (y, p) in col.iter().step_by(2).enumerate() {
                scaled[[x, y]] = *p;
            }
        }
        Ok(scaled)
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

#[derive(Clone, PartialEq)]
pub struct CanvasImage {
    pub image: Array2<Srgba<u8>>,
}

impl Debug for CanvasImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("[...]")
    }
}
