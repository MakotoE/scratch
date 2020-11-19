use super::*;
use coordinate::{Size, SpriteCoordinate, SpriteRectangle};
use palette::IntoColor;
use pen::Pen;
use savefile::{BlockID, Image, Target};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlImageElement, Url};

#[derive(Debug)]
pub struct SpriteRuntime {
    is_a_clone: bool,
    need_redraw: bool,
    position: SpriteCoordinate,
    costumes: Vec<Costume>,
    current_costume: usize,
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
        let mut runtime = Self {
            need_redraw: true,
            position: SpriteCoordinate {
                x: target.x,
                y: target.y,
            },
            costumes: Vec::new(),
            current_costume: 0,
            text: Text {
                id: BlockID::default(),
                text: None,
            },
            pen: Pen::new(),
            is_a_clone,
            hide: HideStatus::Show,
        };

        for costume in &target.costumes {
            match images.get(&costume.md5ext) {
                Some(file) => runtime.load_costume(file).await?,
                None => return Err(wrap_err!(format!("image not found: {}", costume.md5ext))),
            }
        }

        Ok(runtime)
    }

    pub fn redraw(&mut self, context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        self.need_redraw = false;

        if let HideStatus::Hide = self.hide {
            return Ok(());
        }

        self.pen.draw(context);

        let costume = &self.costumes[self.current_costume];
        SpriteRuntime::draw_costume(context, costume, &self.position)?;

        if let Some(text) = &self.text.text {
            context.save();
            context.translate(
                240.0 + costume.size().width as f64 / 4.0 + self.position.x as f64,
                130.0 - costume.size().length as f64 / 2.0 - self.position.y as f64,
            )?;
            SpriteRuntime::draw_text_bubble(context, text)?;
            context.restore();
        }
        Ok(())
    }

    fn draw_costume(
        context: &web_sys::CanvasRenderingContext2d,
        costume: &Costume,
        position: &SpriteCoordinate,
    ) -> Result<()> {
        context.draw_image_with_html_image_element(
            &costume.image,
            240.0 - costume.size().width as f64 / 2.0 + position.x as f64,
            180.0 - costume.size().length as f64 / 2.0 - position.y as f64,
        )?;
        Ok(())
    }

    fn draw_text_bubble(context: &web_sys::CanvasRenderingContext2d, text: &str) -> Result<()> {
        // Original implementation:
        // https://github.com/LLK/scratch-render/blob/954cfff02b08069a082cbedd415c1fecd9b1e4fb/src/TextBubbleSkin.js#L149
        const CORNER_RADIUS: f64 = 16.0;
        const PADDING: f64 = 10.0;
        const PADDED_HEIGHT: f64 = 16.0 + PADDING * 2.0;

        context.set_font("14px Helvetica, sans-serif");
        let line_width: f64 = context.measure_text(text)?.width();
        let padded_width = line_width.max(50.0) + PADDING * 2.0;

        context.begin_path();

        // Corners
        context.move_to(-16.0 + padded_width, PADDED_HEIGHT);
        context.arc_to(
            padded_width,
            PADDED_HEIGHT,
            padded_width,
            PADDED_HEIGHT - CORNER_RADIUS,
            CORNER_RADIUS,
        )?;
        context.arc_to(padded_width, 0.0, 0.0, 0.0, CORNER_RADIUS)?;
        context.arc_to(0.0, 0.0, 0.0, PADDED_HEIGHT, CORNER_RADIUS)?;
        context.arc_to(
            0.0,
            PADDED_HEIGHT,
            CORNER_RADIUS,
            PADDED_HEIGHT,
            CORNER_RADIUS,
        )?;

        // Tail
        context.bezier_curve_to(
            CORNER_RADIUS,
            4.0 + PADDED_HEIGHT,
            -4.0 + CORNER_RADIUS,
            8.0 + PADDED_HEIGHT,
            -4.0 + CORNER_RADIUS,
            10.0 + PADDED_HEIGHT,
        );
        context.arc_to(
            -4.0 + CORNER_RADIUS,
            12.0 + PADDED_HEIGHT,
            -2.0 + CORNER_RADIUS,
            12.0 + PADDED_HEIGHT,
            2.0,
        )?;
        context.bezier_curve_to(
            1.0 + CORNER_RADIUS,
            12.0 + PADDED_HEIGHT,
            11.0 + CORNER_RADIUS,
            8.0 + PADDED_HEIGHT,
            16.0 + CORNER_RADIUS,
            PADDED_HEIGHT,
        );
        context.close_path();

        context.set_fill_style(&"white".into());
        context.set_stroke_style(&"rgba(0, 0, 0, 0.15)".into());
        context.set_line_width(4.0);
        context.stroke();
        context.fill();

        context.set_fill_style(&"#575E75".into());
        context.fill_text(text, PADDING, PADDING + 0.9 * 15.0)?;
        Ok(())
    }

    async fn load_costume(&mut self, file: &Image) -> Result<()> {
        let parts = js_sys::Array::new_with_length(1);
        let arr: js_sys::Uint8Array = match file {
            Image::SVG(b) => b.as_slice().into(),
            Image::PNG(b) => b.as_slice().into(),
        };
        parts.set(0, arr.unchecked_into());

        let mut properties = BlobPropertyBag::new();
        let image_type = match file {
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

        self.costumes.push(Costume::new(image));
        Ok(())
    }

    pub fn need_redraw(&self) -> bool {
        self.need_redraw
    }

    pub fn set_costume_index(&mut self, index: usize) -> Result<()> {
        if index >= self.costumes.len() {
            return Err(wrap_err!(format!(
                "current_costume is out of range: {}",
                self.current_costume
            )));
        }

        self.need_redraw = true;
        self.current_costume = index;
        Ok(())
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
            size: self.costumes[self.current_costume].size,
        }
    }

    /// Can't do scaling yet
    pub fn set_rectangle(&mut self, rectangle: SpriteRectangle) {
        self.need_redraw = true;
        self.position = rectangle.center;
        self.pen().set_position(&rectangle.center);
    }

    pub fn set_hide(&mut self, hide: HideStatus) {
        self.hide = hide;
    }
}

#[derive(Debug, Clone)]
pub struct Costume {
    image: HtmlImageElement,
    size: Size,
}

impl Costume {
    pub fn new(image: HtmlImageElement) -> Self {
        let size = Size {
            width: image.width() as f64,
            length: image.height() as f64,
        };
        Self { image, size }
    }

    pub fn size(&self) -> Size {
        self.size
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
/// Text only be hidden by the thread that posted it. It can be replaced with new text by any
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
