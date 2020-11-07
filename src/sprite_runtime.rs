use super::*;
use palette::IntoColor;
use pen::Pen;
use savefile::Image;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlImageElement, Url};

#[derive(Debug)]
pub struct SpriteRuntime {
    sprite_id: usize,
    is_a_clone: bool,
    need_redraw: bool,
    position: Coordinate,
    costumes: Vec<Costume>,
    current_costume: usize,
    text: Option<String>,
    pen: Pen,
}

#[allow(dead_code)]
impl SpriteRuntime {
    pub async fn new(
        costumes: &[savefile::Costume],
        images: &HashMap<String, Image>,
        sprite_id: usize,
        is_a_clone: bool,
    ) -> Result<Self> {
        let mut runtime = Self {
            need_redraw: true,
            position: Coordinate::default(),
            costumes: Vec::new(),
            current_costume: 0,
            text: None,
            pen: Pen::new(),
            sprite_id,
            is_a_clone,
        };

        for costume in costumes {
            match images.get(&costume.md5ext) {
                Some(file) => {
                    let rotation_center =
                        Coordinate::new(costume.rotation_center_x, costume.rotation_center_y);
                    runtime.load_costume(file, rotation_center).await?
                }
                None => return Err(wrap_err!(format!("image not found: {}", costume.md5ext))),
            }
        }

        Ok(runtime)
    }

    pub fn redraw(&mut self, context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        self.pen.draw(context);

        let costume = match self.costumes.get(self.current_costume) {
            Some(c) => c,
            None => {
                return Err(wrap_err!(format!(
                    "current_costume is out of range: {}",
                    self.current_costume
                )));
            }
        };
        SpriteRuntime::draw_costume(context, costume, &self.position)?;

        if let Some(text) = &self.text {
            context.save();
            context.translate(
                240.0 + costume.rotation_center.x / 2.0 + self.position.x,
                130.0 - costume.rotation_center.y - self.position.y,
            )?;
            SpriteRuntime::draw_text_bubble(context, text)?;
            context.restore();
        }

        self.need_redraw = false;
        Ok(())
    }

    fn draw_costume(
        context: &web_sys::CanvasRenderingContext2d,
        costume: &Costume,
        position: &Coordinate,
    ) -> Result<()> {
        context.draw_image_with_html_image_element(
            &costume.image,
            240.0 - costume.rotation_center.x + position.x,
            180.0 - costume.rotation_center.y - position.y,
        )?;
        Ok(())
    }

    //noinspection RsBorrowChecker
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

        // Flip text bubble
        context.save();
        context.scale(-1.0, 1.0)?;
        context.translate(-1.0 * padded_width, 0.0)?;

        // Corners
        context.move_to(16.0, PADDED_HEIGHT);
        context.arc_to(
            0.0,
            PADDED_HEIGHT,
            0.0,
            PADDED_HEIGHT - CORNER_RADIUS,
            CORNER_RADIUS,
        )?;
        context.arc_to(0.0, 0.0, padded_width, 0.0, CORNER_RADIUS)?;
        context.arc_to(
            padded_width,
            0.0,
            padded_width,
            PADDED_HEIGHT,
            CORNER_RADIUS,
        )?;
        context.arc_to(
            padded_width,
            PADDED_HEIGHT,
            padded_width - CORNER_RADIUS,
            PADDED_HEIGHT,
            CORNER_RADIUS,
        )?;

        // Tail
        context.save();
        context.translate(padded_width - CORNER_RADIUS, PADDED_HEIGHT)?;
        context.bezier_curve_to(0.0, 4.0, 4.0, 8.0, 4.0, 10.0);
        context.arc_to(4.0, 12.0, 2.0, 12.0, 2.0)?;
        context.bezier_curve_to(-1.0, 12.0, -11.0, 8.0, -16.0, 0.0);
        context.restore();

        context.restore(); // Un-flip text bubble

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

    async fn load_costume(&mut self, file: &Image, rotation_center: Coordinate) -> Result<()> {
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

        self.costumes.push(Costume {
            image,
            rotation_center,
        });
        Ok(())
    }

    pub fn need_redraw(&self) -> bool {
        self.need_redraw
    }

    pub fn position(&self) -> &Coordinate {
        &self.position
    }

    pub fn set_position(&mut self, position: &Coordinate) {
        self.need_redraw = true;
        self.position = *position;
        self.pen().set_position(position);
    }

    pub fn set_costume_index(&mut self, index: usize) {
        self.need_redraw = true;
        self.current_costume = index;
    }

    pub fn say(&mut self, text: Option<&str>) {
        self.need_redraw = true;
        self.text = text.map(|s| s.to_string());
    }

    pub fn pen(&mut self) -> &mut Pen {
        self.need_redraw = true;
        &mut self.pen
    }

    pub fn sprite_id(&self) -> usize {
        self.sprite_id
    }

    pub fn is_a_clone(&self) -> bool {
        self.is_a_clone
    }
}

#[derive(Copy, Clone, Default, Debug, PartialOrd, PartialEq)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Debug)]
pub struct Costume {
    image: HtmlImageElement,
    rotation_center: Coordinate,
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
