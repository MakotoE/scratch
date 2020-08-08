use super::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlImageElement, Url};

#[derive(Debug)]
pub struct SpriteRuntime {
    context: web_sys::CanvasRenderingContext2d,
    position: Coordinate,
    pub variables: HashMap<String, serde_json::Value>,
    costumes: Vec<Costume>,
}

impl SpriteRuntime {
    pub fn new(context: web_sys::CanvasRenderingContext2d) -> Self {
        Self {
            context,
            position: Coordinate::default(),
            variables: HashMap::new(),
            costumes: Vec::new(),
        }
    }

    pub fn set_position(&mut self, position: Coordinate) {
        self.position = position;
    }

    pub fn add_position(&mut self, coordinate: &Coordinate) {
        self.position = self.position.add(coordinate);
    }

    pub async fn load_costume(
        &mut self,
        file: &str,
        rotation_center_x: f64,
        rotation_center_y: f64,
    ) -> Result<()> {
        let parts = js_sys::Array::new_with_length(1);
        parts.set(0, file.into());
        let mut properties = BlobPropertyBag::new();
        properties.type_("image/svg+xml");
        let blob = Blob::new_with_str_sequence_and_options(&parts, &properties)?;
        let url = Url::create_object_url_with_blob(&blob)?;

        let image = HtmlImageElement::new()?;
        image.set_src(&url);
        JsFuture::from(image.decode()).await?;

        Url::revoke_object_url(&url)?;

        self.costumes.push(Costume {
            image,
            rotation_center_x,
            rotation_center_y,
        });
        Ok(())
    }

    pub fn change_costume(&mut self, index: usize) -> Result<()> {
        let costume = match self.costumes.get(index) {
            Some(i) => i,
            None => return Err(format!("index is out of range: {}", index).into()),
        };
        self.context.draw_image_with_html_image_element(
            &costume.image,
            240.0 - costume.rotation_center_x,
            180.0 - costume.rotation_center_y,
        )?;
        Ok(())
    }

    pub fn say(&self, s: &str) -> Result<()> {
        // https://github.com/LLK/scratch-render/blob/954cfff02b08069a082cbedd415c1fecd9b1e4fb/src/TextBubbleSkin.js#L149
        const CORNER_RADIUS: f64 = 16.0;
        const PADDING: f64 = 10.0;
        const PADDED_HEIGHT: f64 = 16.0 + PADDING * 2.0;

        let ctx = &self.context;

        ctx.set_font("14px Helvetica, sans-serif");
        let line_width: f64 = ctx.measure_text(s)?.width();
        let padded_width = line_width.max(50.0) + PADDING * 2.0;

        ctx.translate(260.0 + self.position.x, 80.0 + self.position.y)?;

        ctx.begin_path();

        // Flip text bubble
        ctx.save();
        ctx.scale(-1.0, 1.0)?;
        ctx.translate(-1.0 * padded_width, 0.0)?;

        // Corners
        ctx.move_to(16.0, PADDED_HEIGHT);
        ctx.arc_to(
            0.0,
            PADDED_HEIGHT,
            0.0,
            PADDED_HEIGHT - CORNER_RADIUS,
            CORNER_RADIUS,
        )?;
        ctx.arc_to(0.0, 0.0, padded_width, 0.0, CORNER_RADIUS)?;
        ctx.arc_to(
            padded_width,
            0.0,
            padded_width,
            PADDED_HEIGHT,
            CORNER_RADIUS,
        )?;
        ctx.arc_to(
            padded_width,
            PADDED_HEIGHT,
            padded_width - CORNER_RADIUS,
            PADDED_HEIGHT,
            CORNER_RADIUS,
        )?;

        // Tail
        ctx.save();
        ctx.translate(padded_width - CORNER_RADIUS, PADDED_HEIGHT)?;
        ctx.bezier_curve_to(0.0, 4.0, 4.0, 8.0, 4.0, 10.0);
        ctx.arc_to(4.0, 12.0, 2.0, 12.0, 2.0)?;
        ctx.bezier_curve_to(-1.0, 12.0, -11.0, 8.0, -16.0, 0.0);
        ctx.restore();

        ctx.restore(); // Un-flip text bubble

        ctx.close_path();

        ctx.set_fill_style(&"white".into());
        ctx.set_stroke_style(&"rgba(0, 0, 0, 0.15)".into());
        ctx.set_line_width(4.0);
        ctx.stroke();
        ctx.fill();

        ctx.set_fill_style(&"#575E75".into());
        ctx.fill_text(s, PADDING, PADDING + 0.9 * 15.0)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Default, Debug, PartialOrd, PartialEq)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Debug)]
pub struct Costume {
    image: HtmlImageElement,
    rotation_center_x: f64,
    rotation_center_y: f64,
}
