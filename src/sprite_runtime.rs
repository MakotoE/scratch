use super::*;
use crate::canvas::{CanvasContext, Corner, Direction};
use crate::coordinate::{
    CanvasCoordinate, Size, SpriteCoordinate, SpriteRectangle, Transformation,
};
use crate::coordinate::{CanvasRectangle, Scale};
use crate::pen::Pen;
use crate::savefile::{BlockID, Image, Target};
use palette::IntoColor;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlImageElement, Url};

#[derive(Debug)]
pub struct SpriteRuntime {
    is_a_clone: bool,
    need_redraw: bool,
    center: SpriteCoordinate,
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
        target: &Rc<Target>,
        images: &HashMap<String, Image>,
        is_a_clone: bool,
    ) -> Result<Self> {
        Ok(Self {
            need_redraw: true,
            center: SpriteCoordinate {
                x: target.x,
                y: target.y,
            },
            scale: Scale { x: 1.0, y: 1.0 },
            costumes: Costumes::new(&target.costumes, images).await?,
            costume_transparency: 1.0,
            text: Text {
                id: BlockID::default(),
                text: None,
            },
            pen: Pen::new(),
            is_a_clone,
            hide: HideStatus::Show,
        })
    }

    pub fn redraw(&mut self, context: &CanvasContext) -> Result<()> {
        self.need_redraw = false;

        if let HideStatus::Hide = self.hide {
            return Ok(());
        }

        self.pen.draw(context);

        SpriteRuntime::draw_costume(
            context,
            self.costumes.current_costume(),
            &self.center,
            &self.scale,
            self.costume_transparency,
        )?;

        if let Some(text) = &self.text.text {
            let position: CanvasCoordinate = self.center.into();
            let size = self.costumes.current_costume().size;
            let context = context.with_transformation(Transformation::translate(position.add(
                &CanvasCoordinate {
                    x: size.width as f64 / 4.0,
                    y: -50.0 - size.length as f64 / 2.0,
                },
            )));
            SpriteRuntime::draw_text_bubble(&context, text)?;
        }
        Ok(())
    }

    fn draw_costume(
        context: &CanvasContext,
        costume: &Costume,
        center: &SpriteCoordinate,
        scale: &Scale,
        alpha: f64,
    ) -> Result<()> {
        let size = costume.size.multiply(scale);
        let rectangle = CanvasRectangle {
            top_left: CanvasCoordinate::from(*center).add(&CanvasCoordinate {
                x: size.width / -2.0,
                y: size.length / -2.0,
            }),
            size,
        };
        context.set_global_alpha(alpha);
        context.draw_image(&costume.image, &rectangle)?;
        context.set_global_alpha(1.0);
        Ok(())
    }

    fn draw_text_bubble(context: &CanvasContext, text: &str) -> Result<()> {
        // Original implementation:
        // https://github.com/LLK/scratch-render/blob/954cfff02b08069a082cbedd415c1fecd9b1e4fb/src/TextBubbleSkin.js#L149
        const CORNER_RADIUS: f64 = 16.0;
        const PADDING: f64 = 10.0;
        const HEIGHT: f64 = CORNER_RADIUS + PADDING * 2.0;

        context.set_font("14px Helvetica, sans-serif");
        let line_width = context.measure_text(text)?;
        let width = line_width.max(50.0) + PADDING * 2.0;

        context.begin_path();

        // Corners
        context.move_to(&CanvasCoordinate {
            x: width - CORNER_RADIUS,
            y: HEIGHT,
        });
        context.rounded_corner(
            &CanvasCoordinate {
                x: width - CORNER_RADIUS,
                y: HEIGHT - CORNER_RADIUS,
            },
            CORNER_RADIUS,
            Corner::BottomRight,
            Direction::CounterClockwise,
        )?;
        context.rounded_corner(
            &CanvasCoordinate {
                x: width - CORNER_RADIUS,
                y: CORNER_RADIUS,
            },
            CORNER_RADIUS,
            Corner::TopRight,
            Direction::CounterClockwise,
        )?;
        context.rounded_corner(
            &CanvasCoordinate {
                x: CORNER_RADIUS,
                y: CORNER_RADIUS,
            },
            CORNER_RADIUS,
            Corner::TopLeft,
            Direction::CounterClockwise,
        )?;
        context.rounded_corner(
            &CanvasCoordinate {
                x: CORNER_RADIUS,
                y: HEIGHT - CORNER_RADIUS,
            },
            CORNER_RADIUS,
            Corner::BottomLeft,
            Direction::CounterClockwise,
        )?;

        // Tail
        context.bezier_curve_to(
            &CanvasCoordinate {
                x: CORNER_RADIUS,
                y: 4.0 + HEIGHT,
            },
            &CanvasCoordinate {
                x: -4.0 + CORNER_RADIUS,
                y: 8.0 + HEIGHT,
            },
            &CanvasCoordinate {
                x: -4.0 + CORNER_RADIUS,
                y: 10.0 + HEIGHT,
            },
        );
        context.arc_to(
            &CanvasCoordinate {
                x: -4.0 + CORNER_RADIUS,
                y: 12.0 + HEIGHT,
            },
            &CanvasCoordinate {
                x: -2.0 + CORNER_RADIUS,
                y: 12.0 + HEIGHT,
            },
            2.0,
        )?;
        context.bezier_curve_to(
            &CanvasCoordinate {
                x: 1.0 + CORNER_RADIUS,
                y: 12.0 + HEIGHT,
            },
            &CanvasCoordinate {
                x: 11.0 + CORNER_RADIUS,
                y: 8.0 + HEIGHT,
            },
            &CanvasCoordinate {
                x: 16.0 + CORNER_RADIUS,
                y: HEIGHT,
            },
        );
        context.close_path();

        context.set_fill_style("white");
        context.set_stroke_style("rgba(0, 0, 0, 0.15)");
        context.set_line_width(4.0);
        context.stroke();
        context.fill();

        context.set_fill_style("#575E75");
        context.fill_text(
            text,
            &CanvasCoordinate {
                x: PADDING,
                y: PADDING + 0.9 * 15.0,
            },
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
            center: self.center,
            size: self.costumes.current_costume().size.multiply(&self.scale),
        }
    }

    pub fn center(&self) -> SpriteCoordinate {
        self.center
    }

    pub fn set_center(&mut self, center: SpriteCoordinate) {
        self.need_redraw = true;
        self.center = center;
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
    image: HtmlImageElement,
    size: Size,
    name: String,
}

impl Costume {
    pub async fn new(name: String, image_file: &Image) -> Result<Self> {
        let parts = js_sys::Array::new_with_length(1);
        let arr: js_sys::Uint8Array = match image_file {
            Image::SVG(b) => b.as_slice().into(),
            Image::PNG(b) => b.as_slice().into(),
        };
        parts.set(0, arr.unchecked_into());

        let mut properties = BlobPropertyBag::new();
        let image_type = match image_file {
            Image::SVG(_) => "image/svg+xml",
            Image::PNG(_) => "image/png",
        };
        properties.type_(image_type);

        let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &properties)?;
        let url = Url::create_object_url_with_blob(&blob)?;

        let image = HtmlImageElement::new()?;
        image.set_src(&url);
        JsFuture::from(image.decode()).await?;

        Url::revoke_object_url(&url)?;

        Ok(Self {
            size: Size {
                width: image.width() as f64,
                length: image.height() as f64,
            },
            image,
            name,
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
        costume_data: &[savefile::Costume],
        images: &HashMap<String, Image>,
    ) -> Result<Self> {
        let mut costumes: Vec<Costume> = Vec::with_capacity(costume_data.len());
        for costume in costume_data {
            match images.get(&costume.md5ext) {
                Some(file) => {
                    costumes.push(Costume::new(costume.name.clone(), file).await?);
                }
                None => return Err(wrap_err!(format!("image not found: {}", costume.md5ext))),
            }
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
            None => Err(wrap_err!(format!(
                "costume {} does not exist",
                current_costume
            ))),
        }
    }

    pub fn next_costume(&mut self) {
        self.current_costume = (self.current_costume + 1) % self.costumes.len();
    }
}

pub fn hex_to_color(s: &str) -> Result<palette::Hsv> {
    if s.len() != 7 || s.bytes().next() != Some(b'#') {
        return Err(wrap_err!(format!("s is invalid: {}", s)));
    }

    let rgb = palette::Srgb::new(
        u8::from_str_radix(&s[1..3], 16)? as f32 / 255.0,
        u8::from_str_radix(&s[3..5], 16)? as f32 / 255.0,
        u8::from_str_radix(&s[5..7], 16)? as f32 / 255.0,
    );
    Ok(palette::Hsv::from(rgb))
}

pub fn color_to_hex(color: &palette::Hsv) -> String {
    let rgb = palette::Srgb::from_linear(color.into_rgb());
    format!(
        "#{:02x}{:02x}{:02x}",
        (rgb.red * 255.0).round() as u8,
        (rgb.green * 255.0).round() as u8,
        (rgb.blue * 255.0).round() as u8
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_color() {
        struct Test {
            s: &'static str,
            expected: palette::Hsv,
            expect_err: bool,
        }

        let tests = vec![
            Test {
                s: "",
                expected: palette::Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
            Test {
                s: "#",
                expected: palette::Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
            Test {
                s: "#000000",
                expected: palette::Hsv::new(0.0, 0.0, 0.0),
                expect_err: false,
            },
            Test {
                s: "#ffffff",
                expected: palette::Hsv::new(0.0, 0.0, 1.0),
                expect_err: false,
            },
            Test {
                s: "#ffffffa",
                expected: palette::Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let result = hex_to_color(test.s);
            assert_eq!(result.is_err(), test.expect_err, "{}", i);
            if !test.expect_err {
                assert_eq!(result.unwrap(), test.expected, "{}", i);
            }
        }
    }

    #[test]
    fn test_color_to_hex() {
        assert_eq!(color_to_hex(&palette::Hsv::new(0.0, 1.0, 1.0)), "#ff0000");
    }
}
