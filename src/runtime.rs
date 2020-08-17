use super::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlImageElement, Url};

#[derive(Debug)]
pub struct SpriteRuntime {
    context: web_sys::CanvasRenderingContext2d,
    position: Coordinate,
    pub variables: HashMap<String, serde_json::Value>,
    costumes: Vec<Costume>,
    current_costume: usize,
    text: Option<String>,
    pen_path: web_sys::Path2d,
    pen_color: palette::Srgb,
    pen_size: f64,
}

impl SpriteRuntime {
    pub fn new(
        context: web_sys::CanvasRenderingContext2d,
        variables: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            context,
            position: Coordinate::default(),
            variables,
            costumes: Vec::new(),
            current_costume: 0,
            text: None,
            pen_path: web_sys::Path2d::new().unwrap(),
            pen_color: palette::Srgb::new(0.0, 0.0, 1.0),
            pen_size: 1.0,
        }
    }

    pub fn redraw(&self) -> Result<()> {
        self.context.reset_transform().unwrap();
        self.context.clear_rect(0.0, 0.0, 960.0, 720.0);
        self.context.scale(2.0, 2.0).unwrap();

        self.context.set_line_cap("round");
        self.context
            .set_stroke_style(&color_to_hex(&self.pen_color).as_str().into());
        self.context.set_line_width(self.pen_size);
        self.context.stroke_with_path(&self.pen_path);

        let costume = match self.costumes.get(self.current_costume) {
            Some(i) => i,
            None => {
                return Err(
                    format!("current_costume is out of range: {}", self.current_costume).into(),
                )
            }
        };
        self.context.draw_image_with_html_image_element(
            &costume.image,
            240.0 - costume.rotation_center.x + self.position.x,
            180.0 - costume.rotation_center.y - self.position.y,
        )?;

        if let Some(text) = &self.text {
            self.context
                .translate(260.0 + self.position.x, 80.0 - self.position.y)?;
            SpriteRuntime::draw_text_bubble(&self.context, text)?;
        }
        Ok(())
    }

    //noinspection RsBorrowChecker
    fn draw_text_bubble(context: &web_sys::CanvasRenderingContext2d, text: &str) -> Result<()> {
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

    pub async fn load_costume(
        &mut self,
        file: &savefile::Image,
        rotation_center: Coordinate,
    ) -> Result<()> {
        let parts = js_sys::Array::new_with_length(1);
        let arr: js_sys::Uint8Array = file.into();
        parts.set(0, arr.unchecked_into());

        let mut properties = BlobPropertyBag::new();
        let image_type = match file {
            savefile::Image::SVG(_) => "image/svg+xml",
            savefile::Image::PNG(_) => "image/png",
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

    pub fn position(&self) -> &Coordinate {
        &self.position
    }

    pub fn set_position(&mut self, position: &Coordinate) {
        self.position = *position;
        self.pen_path
            .line_to(240.0 + self.position.x, 180.0 - self.position.y);
    }

    pub fn add_coordinate(&mut self, coordinate: &Coordinate) {
        self.position = self.position.add(coordinate);
        self.pen_path
            .line_to(240.0 + self.position.x, 180.0 - self.position.y);
    }

    pub fn set_costume_index(&mut self, index: usize) {
        self.current_costume = index;
    }

    pub fn say(&mut self, text: Option<&str>) {
        self.text = text.map(|s| s.to_string());
    }

    pub fn pen_down(&mut self) {
        self.pen_path = web_sys::Path2d::new().unwrap();
        self.pen_path
            .move_to(240.0 + self.position.x, 180.0 - self.position.y);
    }

    pub fn pen_up(&mut self) {}

    pub fn pen_color(&self) -> &palette::Srgb {
        &self.pen_color
    }

    pub fn set_pen_color(&mut self, color: &palette::Srgb) {
        self.pen_color = *color;
    }

    pub fn set_pen_size(&mut self, size: f64) {
        self.pen_size = size;
    }

    pub fn pen_clear(&mut self) {
        self.pen_path = web_sys::Path2d::new().unwrap();
    }
}

#[derive(Copy, Clone, Default, Debug, PartialOrd, PartialEq)]
pub struct Coordinate {
    x: f64,
    y: f64,
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

pub fn hex_to_color(s: &str) -> Result<palette::Srgb> {
    if s.len() != 7 || s.bytes().next().unwrap() != b'#' {
        return Err(format!("s is invalid: {}", s).into());
    }

    Ok(palette::rgb::Rgb::new(
        u8::from_str_radix(&s[1..3], 16)? as f32 / 255.0,
        u8::from_str_radix(&s[3..5], 16)? as f32 / 255.0,
        u8::from_str_radix(&s[5..7], 16)? as f32 / 255.0,
    ))
}

pub fn color_to_hex(color: &palette::Srgb) -> String {
    format!("#{:02x}", (color.red * 255.0).round() as u8)
        + &format!("{:02x}", (color.green * 255.0).round() as u8)
        + &format!("{:02x}", (color.blue * 255.0).round() as u8)
}

#[cfg(test)]
mod tests {
    #[test]
    fn hex_to_color() {
        struct Test {
            s: &'static str,
            expected: palette::rgb::Rgb,
            expect_err: bool,
        }

        let tests = vec![
            Test {
                s: "",
                expected: palette::rgb::Rgb::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
            Test {
                s: "#",
                expected: palette::rgb::Rgb::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
            Test {
                s: "#000000",
                expected: palette::rgb::Rgb::new(0.0, 0.0, 0.0),
                expect_err: false,
            },
            Test {
                s: "#ffffff",
                expected: palette::rgb::Rgb::new(1.0, 1.0, 1.0),
                expect_err: false,
            },
            Test {
                s: "#ffffffa",
                expected: palette::rgb::Rgb::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let result = super::hex_to_color(test.s);
            assert_eq!(result.is_err(), test.expect_err, "{}", i);
            if !test.expect_err {
                assert_eq!(result.unwrap(), test.expected, "{}", i);
            }
        }
    }

    #[test]
    fn color_to_hex() {
        assert_eq!(super::color_to_hex(&palette::rgb::Rgb::new(0.0, 1.0, 0.0)), "#00ff00");
    }
}
