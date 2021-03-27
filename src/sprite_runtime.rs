use super::*;
use crate::coordinate::{CanvasCoordinate, Scale, Size, SpriteCoordinate, SpriteRectangle};
use crate::file::{BlockID, Image, Target};
use crate::pen::Pen;
use flo_curves::{bezier, BezierCurve, Coord2};
use gfx_device_gl::Resources;
use gfx_graphics::{CreateTexture, Format};
use gfx_texture::{Texture, TextureSettings};
use graphics::character::CharacterCache;
use graphics::types::{FontSize, Rectangle};
use graphics::{line, CircleArc, Context, Graphics, Transformed};
use graphics_buffer::{BufferGlyphs, RenderBuffer};
use image::codecs::png::PngDecoder;
use image::{DynamicImage, ImageBuffer, ImageDecoder, RgbaImage};
use piston_window::{G2d, G2dTextureContext, Glyphs};
use std::f64::consts::TAU;
use std::fs::File;
use std::io::{Cursor, Read};

#[derive(Debug, Default)]
pub struct SpriteRuntime {
    sprite_name: String,
    is_a_clone: bool,
    position: SpriteCoordinate,
    scale: Scale,
    costumes: Costumes,
    /// 0.0 = transparent, 1.0 = opaque
    costume_transparency: f64,
    text: Text,
    pen: Pen,
    hide: HideStatus,
}

impl SpriteRuntime {
    pub fn new(target: &Target) -> Self {
        let scale = if target.is_stage {
            1.0
        } else {
            target.size / 100.0
        };
        Self {
            sprite_name: target.name.clone(),
            position: SpriteCoordinate {
                x: target.x,
                y: target.y,
            },
            scale: Scale { x: scale, y: scale },
            costumes: Costumes::default(),
            costume_transparency: 1.0,
            text: Text::default(),
            pen: Pen::default(),
            is_a_clone: false,
            hide: if target.is_stage || target.visible {
                HideStatus::Show
            } else {
                HideStatus::Hide
            },
        }
    }

    pub fn set_costumes(&mut self, costumes: Costumes) {
        self.costumes = costumes;
    }

    pub fn draw<G, C>(
        &self,
        context: &Context,
        graphics: &mut G,
        character_cache: &mut C,
    ) -> Result<()>
    where
        G: GraphicsCostumeTexture<C>,
        C: CharacterCache,
    {
        if let HideStatus::Hide = self.hide {
            return Ok(());
        }

        self.pen.draw(context, graphics);

        if let Some(c) = self.costumes.current_costume() {
            SpriteRuntime::draw_costume(
                context,
                graphics,
                c,
                &self.position.into(),
                &self.scale,
                self.costume_transparency,
            );
        }

        if let Some(text) = &self.text.text {
            let position: CanvasCoordinate = self.position.into();
            let size = if let Some(c) = self.costumes.current_costume() {
                c.image_size
            } else {
                Size {
                    width: 0.0,
                    height: 0.0,
                }
            };
            let c = Context {
                transform: context.transform.trans(
                    position.x - 8.0 + size.width / 4.0,
                    position.y - 44.0 - size.height / 2.0,
                ),
                ..*context
            };
            SpriteRuntime::draw_text_bubble(text, &c, graphics, character_cache)?;
        }
        Ok(())
    }

    fn draw_costume<G, C>(
        context: &Context,
        graphics: &mut G,
        costume: &Costume,
        position: &CanvasCoordinate,
        scale: &Scale,
        alpha: f64,
    ) where
        G: GraphicsCostumeTexture<C>,
        C: CharacterCache,
    {
        let rectangle: Rectangle = [
            position.x - costume.center.x * costume.scale * scale.x,
            position.y - costume.center.y * costume.scale * scale.y,
            costume.image_size.width * scale.x,
            costume.image_size.height * scale.y,
        ];
        graphics::Image {
            color: Some([1.0, 1.0, 1.0, alpha as f32]),
            source_rectangle: None,
            rectangle: Some(rectangle),
        }
        .draw(
            G::get_costume_texture(costume),
            &context.draw_state,
            context.transform,
            graphics,
        );
    }

    fn draw_text_bubble<G, C>(
        text: &str,
        context: &Context,
        graphics: &mut G,
        character_cache: &mut C,
    ) -> Result<()>
    where
        G: Graphics<Texture = <C as CharacterCache>::Texture>,
        C: CharacterCache,
    {
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

        let arc = |start: f64, end: f64, x: f64, y: f64, graphics: &mut G| {
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

        let line = |start_x: f64, start_y: f64, end_x: f64, end_y: f64, graphics: &mut G| {
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

        let curve = |curve: bezier::Curve<Coord2>, graphics: &mut G| {
            const SUBDIVISIONS: usize = 8;
            let Coord2(mut last_x, mut last_y) = curve.start_point();
            for pos in 0..=SUBDIVISIONS {
                let Coord2(x, y) = curve.point_at_pos(pos as f64 / SUBDIVISIONS as f64);
                line(last_x, last_y, x, y, graphics);
                last_x = x;
                last_y = y;
            }
        };

        let width = f64::max(
            character_cache
                .width(FONT_SIZE, text)
                .map_err(|_e| Error::msg("width calculation error"))?,
            50.0,
        ) + PADDING * 2.0;

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
        )
        .map_err(|_| Error::msg("text draw error"))
    }

    pub fn costumes(&mut self) -> &mut Costumes {
        &mut self.costumes
    }

    pub fn say(&mut self, text: Text) {
        self.text.replace(text);
    }

    pub fn pen(&mut self) -> &mut Pen {
        &mut self.pen
    }

    pub fn is_a_clone(&self) -> bool {
        self.is_a_clone
    }

    pub fn rectangle(&self) -> SpriteRectangle {
        let size = if let Some(c) = self.costumes.current_costume() {
            c.image_size.multiply(&self.scale)
        } else {
            Size {
                width: 0.0,
                height: 0.0,
            }
        };
        SpriteRectangle {
            center: self.position,
            size,
        }
    }

    pub fn center(&self) -> SpriteCoordinate {
        self.position
    }

    pub fn set_center(&mut self, center: SpriteCoordinate) {
        self.position = center;
        self.pen().set_position(&center);
    }

    pub fn set_scale(&mut self, scale: Scale) {
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

    pub fn clone_sprite_runtime(&self) -> SpriteRuntime {
        SpriteRuntime {
            sprite_name: self.sprite_name.clone() + "-clone",
            is_a_clone: true,
            costumes: self.costumes.clone(),
            text: Text::default(),
            pen: Pen::default(),
            ..*self
        }
    }
}

/// This is needed because G2d and RenderBuffer have different texture types.
pub trait GraphicsCostumeTexture<C>: Graphics<Texture = <C as CharacterCache>::Texture>
where
    C: CharacterCache,
{
    fn get_costume_texture(costume: &Costume) -> &Self::Texture;
}

impl GraphicsCostumeTexture<Glyphs> for G2d<'_> {
    fn get_costume_texture(costume: &Costume) -> &Self::Texture {
        &costume.gfx_texture
    }
}

impl GraphicsCostumeTexture<BufferGlyphs<'_>> for RenderBuffer {
    fn get_costume_texture(costume: &Costume) -> &Self::Texture {
        &costume.render_buffer_texture
    }
}

#[derive(Debug, Clone)]
pub struct Costume {
    image_size: Size,
    scale: f64,
    name: String,
    center: SpriteCoordinate,
    gfx_texture: Texture<Resources>,
    render_buffer_texture: RenderBuffer,
}

impl Costume {
    pub async fn new(
        texture_context: &mut G2dTextureContext,
        costume: &file::Costume,
        image_file: &Image,
    ) -> Result<Self> {
        let (gfx_texture, render_buffer_texture, width, height) = match image_file {
            Image::SVG(b) => Costume::svg_texture(b, texture_context)?,
            Image::PNG(b) => Costume::png_texture(b, texture_context)?,
        };

        Ok(Self {
            image_size: Size {
                width: width as f64 / costume.bitmap_resolution / 2.0,
                height: height as f64 / costume.bitmap_resolution / 2.0,
            },
            scale: 1.0
                / if costume.bitmap_resolution == 0.0 {
                    1.0
                } else {
                    costume.bitmap_resolution
                },
            name: costume.name.clone(),
            center: SpriteCoordinate {
                x: costume.rotation_center_x,
                y: costume.rotation_center_y,
            },
            gfx_texture,
            render_buffer_texture,
        })
    }

    fn svg_texture(
        data: &[u8],
        texture_context: &mut G2dTextureContext,
    ) -> Result<(Texture<Resources>, RenderBuffer, u32, u32)> {
        let mut options = usvg::Options::default();
        options.fontdb.load_system_fonts();

        let tree = usvg::Tree::from_data(data, &options)?;
        let size = tree.svg_node().size.to_screen_size();
        let mut pixmap = tiny_skia::Pixmap::new(size.width() * 2, size.height() * 2).unwrap();

        let width = pixmap.width();
        let height = pixmap.height();

        resvg::render(&tree, usvg::FitTo::Zoom(2.0), pixmap.as_mut())
            .ok_or_else(|| Error::msg("svg error"))?;
        let image: RgbaImage = ImageBuffer::from_raw(width, height, pixmap.take())
            .ok_or_else(|| Error::msg("svg error"))?;
        Ok((
            CreateTexture::create(
                texture_context,
                Format::Rgba8,
                &image,
                [width, height],
                &TextureSettings::new(),
            )?,
            CreateTexture::create(
                &mut (),
                Format::Rgba8,
                &image,
                [width, height],
                &TextureSettings::new(),
            )?,
            width,
            height,
        ))
    }

    fn png_texture(
        data: &[u8],
        texture_context: &mut G2dTextureContext,
    ) -> Result<(Texture<Resources>, RenderBuffer, u32, u32)> {
        let decoder = PngDecoder::new(Cursor::new(data))?;
        let x = decoder.dimensions().0;
        let y = decoder.dimensions().1;
        let dynamic_image = DynamicImage::from_decoder(decoder)?;
        let image = dynamic_image
            .as_rgba8()
            .ok_or_else(|| Error::msg("not in RGBA color space"))?;
        Ok((
            CreateTexture::create(
                texture_context,
                Format::Rgba8,
                &image,
                [image.width(), image.height()],
                &TextureSettings::new(),
            )?,
            CreateTexture::create(
                &mut (),
                Format::Rgba8,
                &image,
                [image.width(), image.height()],
                &TextureSettings::new(),
            )?,
            x * 2,
            y * 2,
        ))
    }

    pub fn new_blank(
        texture_context: &mut G2dTextureContext,
        costume: &file::Costume,
    ) -> Result<Self> {
        let mut file = File::open("assets/blank_backdrop.png")?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;
        let (gfx_texture, render_buffer_texture, width, height) =
            Costume::png_texture(&buffer, texture_context)?;
        Ok(Self {
            image_size: Size {
                width: width as f64,
                height: height as f64,
            },
            name: costume.name.clone(),
            center: SpriteCoordinate {
                x: costume.rotation_center_x,
                y: costume.rotation_center_y,
            },
            scale: 1.0,
            gfx_texture,
            render_buffer_texture,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Costumes {
    costumes: Vec<Costume>,
    current_costume: usize,
}

impl Costumes {
    pub async fn new(
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
                // Pre-made Scratch backdrops are not included in the .sb3 file. A blank image is
                // used as a placeholder.
                Costume::new_blank(texture_context, &costume)?
            };
            costumes.push(costume);
        }
        Ok(Self {
            costumes,
            current_costume: 0,
        })
    }

    fn current_costume(&self) -> Option<&Costume> {
        self.costumes.get(self.current_costume)
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

impl Default for HideStatus {
    fn default() -> Self {
        HideStatus::Hide
    }
}

#[derive(Debug, Clone, Default)]
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
