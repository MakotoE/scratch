use super::*;
use crate::coordinate::{
    CanvasCoordinate, Size, SpriteCoordinate, SpriteRectangle, Transformation,
};
use crate::coordinate::{CanvasRectangle, Scale};
use crate::file::{BlockID, Image, Target};
use crate::pen::Pen;
use gfx_device_gl::Resources;
use gfx_texture::{Texture, TextureSettings};
use graphics::math::Matrix2d;
use graphics::{Context, DrawState};
use piston_window::{G2d, G2dTextureContext, Glyphs};

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

        // self.pen.draw(context);

        SpriteRuntime::draw_costume(
            context,
            graphics,
            self.costumes.current_costume(),
            &self.position,
            &self.scale,
            self.costume_transparency,
        );

        // if let Some(text) = &self.text.text {
        //     let position: CanvasCoordinate = self.position.into();
        //     let size = self.costumes.current_costume().image_size;
        //     let context = context.with_transformation(Transformation::translate(position.add(
        //         &CanvasCoordinate {
        //             x: size.width as f64 / 4.0,
        //             y: -50.0 - size.height as f64 / 2.0,
        //         },
        //     )));
        //     SpriteRuntime::draw_text_bubble(&context, text)?;
        // }
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

    // fn draw_text_bubble(context: &CanvasContext, text: &str) -> Result<()> {
    //     // Original implementation:
    //     // https://github.com/LLK/scratch-render/blob/954cfff02b08069a082cbedd415c1fecd9b1e4fb/src/TextBubbleSkin.js#L149
    //     const CORNER_RADIUS: f64 = 16.0;
    //     const PADDING: f64 = 10.0;
    //     const HEIGHT: f64 = CORNER_RADIUS + PADDING * 2.0;
    //
    //     context.set_font("14px Helvetica, sans-serif");
    //     let line_width = context.measure_text(text)?;
    //     let width = line_width.max(50.0) + PADDING * 2.0;
    //
    //     context.begin_path();
    //
    //     // Corners
    //     context.move_to(&CanvasCoordinate {
    //         x: width - CORNER_RADIUS,
    //         y: HEIGHT,
    //     });
    //     context.rounded_corner(
    //         &CanvasCoordinate {
    //             x: width - CORNER_RADIUS,
    //             y: HEIGHT - CORNER_RADIUS,
    //         },
    //         CORNER_RADIUS,
    //         Corner::BottomRight,
    //         Direction::CounterClockwise,
    //     )?;
    //     context.rounded_corner(
    //         &CanvasCoordinate {
    //             x: width - CORNER_RADIUS,
    //             y: CORNER_RADIUS,
    //         },
    //         CORNER_RADIUS,
    //         Corner::TopRight,
    //         Direction::CounterClockwise,
    //     )?;
    //     context.rounded_corner(
    //         &CanvasCoordinate {
    //             x: CORNER_RADIUS,
    //             y: CORNER_RADIUS,
    //         },
    //         CORNER_RADIUS,
    //         Corner::TopLeft,
    //         Direction::CounterClockwise,
    //     )?;
    //     context.rounded_corner(
    //         &CanvasCoordinate {
    //             x: CORNER_RADIUS,
    //             y: HEIGHT - CORNER_RADIUS,
    //         },
    //         CORNER_RADIUS,
    //         Corner::BottomLeft,
    //         Direction::CounterClockwise,
    //     )?;
    //
    //     // Tail
    //     context.bezier_curve_to(
    //         &CanvasCoordinate {
    //             x: CORNER_RADIUS,
    //             y: 4.0 + HEIGHT,
    //         },
    //         &CanvasCoordinate {
    //             x: -4.0 + CORNER_RADIUS,
    //             y: 8.0 + HEIGHT,
    //         },
    //         &CanvasCoordinate {
    //             x: -4.0 + CORNER_RADIUS,
    //             y: 10.0 + HEIGHT,
    //         },
    //     );
    //     context.arc_to(
    //         &CanvasCoordinate {
    //             x: -4.0 + CORNER_RADIUS,
    //             y: 12.0 + HEIGHT,
    //         },
    //         &CanvasCoordinate {
    //             x: -2.0 + CORNER_RADIUS,
    //             y: 12.0 + HEIGHT,
    //         },
    //         2.0,
    //     )?;
    //     context.bezier_curve_to(
    //         &CanvasCoordinate {
    //             x: 1.0 + CORNER_RADIUS,
    //             y: 12.0 + HEIGHT,
    //         },
    //         &CanvasCoordinate {
    //             x: 11.0 + CORNER_RADIUS,
    //             y: 8.0 + HEIGHT,
    //         },
    //         &CanvasCoordinate {
    //             x: 16.0 + CORNER_RADIUS,
    //             y: HEIGHT,
    //         },
    //     );
    //     context.close_path();
    //
    //     context.set_fill_style("white");
    //     context.set_stroke_style("rgba(0, 0, 0, 0.15)");
    //     context.set_line_width(4.0);
    //     context.stroke();
    //     context.fill();
    //
    //     context.set_fill_style("#575E75");
    //     context.fill_text(
    //         text,
    //         &CanvasCoordinate {
    //             x: PADDING,
    //             y: PADDING + 0.9 * 15.0,
    //         },
    //     )?;
    //     Ok(())
    // }

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
                let image = image::ImageBuffer::from_raw(width, height, pixmap.take())
                    .ok_or(Error::msg("svg error"))?;
                (
                    Texture::from_image(texture_context, &image, &TextureSettings::new())?,
                    width as f64,
                    height as f64,
                )
            }
            Image::PNG(_) => todo!(),
        };

        Ok(Self {
            image_size: Size {
                width: width / costume.bitmap_resolution,
                height: height / costume.bitmap_resolution,
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
