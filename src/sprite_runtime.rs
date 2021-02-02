use super::*;
use crate::coordinate::{
    CanvasCoordinate, Size, SpriteCoordinate, SpriteRectangle, Transformation,
};
use crate::coordinate::{CanvasRectangle, Scale};
use crate::file::{BlockID, Image, Target};
use crate::pen::Pen;
use flo_curves::{bezier, BezierCurve, Coord2};
use gfx_device_gl::Resources;
use gfx_texture::{Texture, TextureSettings};
use graphics::character::CharacterCache;
use graphics::math::Matrix2d;
use graphics::types::FontSize;
use graphics::Transformed;
use graphics::{line, CircleArc, Context, DrawState};
use image::png::PngDecoder;
use image::{DynamicImage, ImageBuffer, ImageDecoder, RgbaImage};
use piston_window::{G2d, G2dTextureContext, Glyphs};
use std::f64::consts::TAU;
use std::io::Cursor;

#[derive(Debug)]
pub struct SpriteRuntime {
    /// To make debugging easier
    sprite_name: String,
    is_a_clone: bool,
    need_redraw: bool,
    position: SpriteCoordinate,
    scale: Scale,
    costumes: Costumes,
    /// 0.0 = transparent, 1.0 = opaque
    costume_transparency: f64,
    text: Text,
    pen: Pen,
    hide: HideStatus,
}

#[allow(dead_code)]
impl SpriteRuntime {
    pub async fn new(
        texture_context: &mut G2dTextureContext,
        target: &Target,
        images: &HashMap<String, Image>,
        is_a_clone: bool,
        sprite_name: String,
    ) -> Result<Self> {
        let scale = if target.is_stage {
            1.0
        } else {
            target.size / 100.0
        };
        Ok(Self {
            sprite_name,
            need_redraw: true,
            position: SpriteCoordinate {
                x: target.x,
                y: target.y,
            },
            scale: Scale { x: scale, y: scale },
            costumes: Costumes::new(texture_context, &target.costumes, images).await?,
            costume_transparency: 1.0,
            text: Text {
                id: BlockID::default(),
                text: None,
            },
            pen: Pen::new(),
            is_a_clone,
            hide: if target.is_stage || target.visible {
                HideStatus::Show
            } else {
                HideStatus::Hide
            },
        })
    }

    pub fn redraw(
        &mut self,
        context: &mut Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        self.need_redraw = false;

        if let HideStatus::Hide = self.hide {
            return Ok(());
        }

        self.pen.draw(context, graphics);

        SpriteRuntime::draw_costume(
            context,
            graphics,
            self.costumes.current_costume(),
            &self.position,
            &self.scale,
            self.costume_transparency,
        );

        if let Some(text) = &self.text.text {
            let position: CanvasCoordinate = self.position.into();
            let size = self.costumes.current_costume().image_size;
            context.transform = context.transform.trans(
                position.x - 8.0 + size.width / 8.0,
                position.y - 44.0 - size.height / 4.0,
            );
            SpriteRuntime::draw_text_bubble(text, context, graphics, character_cache)?;
        }
        Ok(())
    }

    fn draw_costume(
        context: &mut Context,
        graphics: &mut G2d,
        costume: &Costume,
        position: &SpriteCoordinate,
        scale: &Scale,
        alpha: f64,
    ) {
        let mut rectangle = CanvasRectangle {
            top_left: CanvasCoordinate::from(*position).add(&CanvasCoordinate {
                x: -costume.center.x * costume.scale * scale.x,
                y: -costume.center.y * costume.scale * scale.y,
            }),
            size: costume.image_size.multiply(scale),
        };
        rectangle.size.width /= 2.0;
        rectangle.size.height /= 2.0;
        graphics::Image {
            color: Some([1.0, 1.0, 1.0, 1.0]),
            source_rectangle: None,
            rectangle: Some([
                rectangle.top_left.x,
                rectangle.top_left.y,
                rectangle.size.width,
                rectangle.size.height,
            ]),
        }
        .draw(
            &costume.texture,
            &context.draw_state,
            context.transform,
            graphics,
        );
    }

    fn draw_text_bubble(
        text: &str,
        context: &mut Context,
        graphics: &mut G2d,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        const COLOR: graphics::types::Color = [0.0, 0.0, 0.0, 0.3];
        const LINE_THICKNESS: f64 = 1.0;
        const FONT_SIZE: FontSize = 14;

        const RIGHT: f64 = 0.0;
        const DOWN: f64 = TAU / 4.0;
        const LEFT: f64 = TAU / 2.0;
        const UP: f64 = 3.0 * TAU / 4.0;
        const RADIUS: f64 = 16.0;
        const PADDING: f64 = 10.0;
        const HEIGHT: f64 = RADIUS + PADDING * 2.0;

        let arc = |start: f64, end: f64, x: f64, y: f64, graphics: &mut G2d| {
            CircleArc {
                color: COLOR,
                radius: LINE_THICKNESS,
                start,
                end,
                resolution: 16,
            }
            .draw(
                [x, y, RADIUS * 2.0, RADIUS * 2.0],
                &context.draw_state,
                context.transform,
                graphics,
            );
        };

        let line = |start_x: f64, start_y: f64, end_x: f64, end_y: f64, graphics: &mut G2d| {
            line::Line {
                color: COLOR,
                radius: LINE_THICKNESS,
                shape: line::Shape::Square,
            }
            .draw(
                [start_x, start_y, end_x, end_y],
                &context.draw_state,
                context.transform,
                graphics,
            );
        };

        let curve = |curve: bezier::Curve<Coord2>, graphics: &mut G2d| {
            const SUBDIVISIONS: usize = 8;
            let Coord2(mut last_x, mut last_y) = curve.start_point();
            for pos in 0..=SUBDIVISIONS {
                let Coord2(x, y) = curve.point_at_pos(pos as f64 / SUBDIVISIONS as f64);
                line(last_x, last_y, x, y, graphics);
                last_x = x;
                last_y = y;
            }
        };

        let width = f64::max(character_cache.width(FONT_SIZE, text)?, 50.0) + PADDING * 2.0;

        arc(
            RIGHT,
            DOWN,
            width - RADIUS * 2.0,
            HEIGHT - RADIUS * 2.0,
            graphics,
        );
        line(width, HEIGHT - RADIUS, width, RADIUS, graphics);
        arc(UP, RIGHT, width - RADIUS * 2.0, 0.0, graphics);
        line(width - RADIUS, 0.0, RADIUS, 0.0, graphics);
        arc(LEFT, UP, 0.0, 0.0, graphics);
        line(0.0, RADIUS, 0.0, HEIGHT - RADIUS, graphics);
        arc(DOWN, LEFT, 0.0, HEIGHT - RADIUS * 2.0, graphics);

        curve(
            bezier::Curve {
                start_point: Coord2(RADIUS, HEIGHT),
                end_point: Coord2(-3.3 + RADIUS, 9.7 + HEIGHT),
                control_points: (
                    Coord2(RADIUS - 2.4, 4.0 + HEIGHT),
                    Coord2(-4.6 + RADIUS, 8.3 + HEIGHT),
                ),
            },
            graphics,
        );
        curve(
            bezier::Curve {
                start_point: Coord2(-3.3 + RADIUS, 9.7 + HEIGHT),
                end_point: Coord2(16.0 + RADIUS, HEIGHT),
                control_points: (
                    Coord2(-0.6 + RADIUS, 11.0 + HEIGHT),
                    Coord2(4.5 + RADIUS, 11.1 + HEIGHT),
                ),
            },
            graphics,
        );

        line(16.0 + RADIUS, HEIGHT, width - RADIUS, HEIGHT, graphics);

        graphics::text::Text {
            color: [0.34, 0.37, 0.46, 1.0],
            font_size: FONT_SIZE,
            round: false,
        }
        .draw(
            text,
            character_cache,
            &context.draw_state,
            context.transform.trans(PADDING, PADDING + 0.9 * 15.0),
            graphics,
        )?;
        Ok(())
    }

    pub fn need_redraw(&self) -> bool {
        self.need_redraw
    }

    pub fn costumes(&mut self) -> &mut Costumes {
        &mut self.costumes
    }

    pub fn say(&mut self, text: Text) {
        self.need_redraw = true;
        self.text.replace(text);
    }

    pub fn pen(&mut self) -> &mut Pen {
        self.need_redraw = true;
        &mut self.pen
    }

    pub fn is_a_clone(&self) -> bool {
        self.is_a_clone
    }

    pub fn rectangle(&self) -> SpriteRectangle {
        SpriteRectangle {
            center: self.position,
            size: self
                .costumes
                .current_costume()
                .image_size
                .multiply(&self.scale),
        }
    }

    pub fn center(&self) -> SpriteCoordinate {
        self.position
    }

    pub fn set_center(&mut self, center: SpriteCoordinate) {
        self.need_redraw = true;
        self.position = center;
        self.pen().set_position(&center);
    }

    pub fn set_scale(&mut self, scale: Scale) {
        self.need_redraw = true;
        self.scale = scale;
    }

    pub fn set_hide(&mut self, hide: HideStatus) {
        self.hide = hide;
    }

    pub fn transparency(&self) -> f64 {
        self.costume_transparency
    }

    /// 0.0 = transparent, 1.0 = opaque
    pub fn set_transparency(&mut self, transparency: f64) {
        self.costume_transparency = transparency;
    }
}

#[derive(Debug, Clone)]
pub struct Costume {
    image_size: Size,
    scale: f64,
    name: String,
    center: SpriteCoordinate,
    texture: Texture<Resources>,
}

impl Costume {
    pub async fn new(
        texture_context: &mut G2dTextureContext,
        costume: &file::Costume,
        image_file: &Image,
    ) -> Result<Self> {
        let (texture, width, height) = match image_file {
            Image::SVG(b) => {
                let mut options = usvg::Options::default();
                options.fontdb.load_system_fonts();

                let tree = usvg::Tree::from_data(b, &options)?;
                let size = tree.svg_node().size.to_screen_size();
                let mut pixmap =
                    tiny_skia::Pixmap::new(size.width() * 2, size.height() * 2).unwrap();

                let width = pixmap.width();
                let height = pixmap.height();

                resvg::render(&tree, usvg::FitTo::Zoom(2.0), pixmap.as_mut())
                    .ok_or(Error::msg("svg error"))?;
                let image = ImageBuffer::from_raw(width, height, pixmap.take())
                    .ok_or(Error::msg("svg error"))?;
                (
                    Texture::from_image(texture_context, &image, &TextureSettings::new())?,
                    width,
                    height,
                )
            }
            Image::PNG(b) => {
                let decoder = PngDecoder::new(Cursor::new(b))?;
                let x = decoder.dimensions().0;
                let y = decoder.dimensions().1;
                let dynamic_image = DynamicImage::from_decoder(decoder)?;
                let image = dynamic_image.as_rgba8().ok_or(Error::msg("png error"))?;
                (
                    Texture::from_image(texture_context, image, &TextureSettings::new())?,
                    x * 2,
                    y * 2,
                )
            }
        };

        Ok(Self {
            image_size: Size {
                width: width as f64 / costume.bitmap_resolution,
                height: height as f64 / costume.bitmap_resolution,
            },
            scale: 1.0 / costume.bitmap_resolution,
            name: costume.name.clone(),
            center: SpriteCoordinate {
                x: costume.rotation_center_x,
                y: costume.rotation_center_y,
            },
            texture,
        })
    }

    pub fn new_blank(costume: &file::Costume) -> Result<Self> {
        Ok(Self {
            image_size: Size {
                width: 1.0,
                height: 1.0,
            },
            name: costume.name.clone(),
            center: SpriteCoordinate {
                x: costume.rotation_center_x,
                y: costume.rotation_center_y,
            },
            scale: 1.0,
            texture: todo!(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Costumes {
    costumes: Vec<Costume>,
    current_costume: usize,
}

impl Costumes {
    async fn new(
        texture_context: &mut G2dTextureContext,
        costume_data: &[file::Costume],
        images: &HashMap<String, Image>,
    ) -> Result<Self> {
        let mut costumes: Vec<Costume> = Vec::with_capacity(costume_data.len());
        for costume in costume_data {
            let costume = if let Some(md5ext) = &costume.md5ext {
                match images.get(md5ext) {
                    Some(file) => Costume::new(texture_context, &costume, file).await?,
                    None => return Err(Error::msg(format!("image not found: {}", md5ext))),
                }
            } else {
                Costume::new_blank(&costume)?
            };
            costumes.push(costume);
        }
        Ok(Self {
            costumes,
            current_costume: 0,
        })
    }

    fn current_costume(&self) -> &Costume {
        &self.costumes[self.current_costume]
    }

    pub fn set_current_costume(&mut self, current_costume: String) -> Result<()> {
        match self
            .costumes
            .iter()
            .position(|costume| costume.name == current_costume)
        {
            Some(n) => {
                self.current_costume = n;
                Ok(())
            }
            None => Err(Error::msg(format!(
                "costume {} does not exist",
                current_costume
            ))),
        }
    }

    pub fn next_costume(&mut self) {
        self.current_costume = (self.current_costume + 1) % self.costumes.len();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HideStatus {
    Hide,
    Show,
}

#[derive(Debug, Clone)]
/// Text can only be hidden by the thread that posted it. It can be replaced with new text by any
/// thread.
pub struct Text {
    pub id: BlockID,
    pub text: Option<String>,
}

impl Text {
    fn replace(&mut self, other: Text) {
        if other.text.is_some() || self.id == other.id {
            *self = other;
        }
    }
}
