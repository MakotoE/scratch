use super::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, HtmlImageElement, Url};

#[derive(Debug)]
pub struct SpriteRuntime {
    context: web_sys::CanvasRenderingContext2d,
    pub x: f64,
    pub y: f64,
    pub variables: HashMap<String, serde_json::Value>,
    pub costumes: Vec<HtmlImageElement>,
}

impl SpriteRuntime {
    pub fn new(context: web_sys::CanvasRenderingContext2d) -> Self {
        Self {
            context,
            x: 0.0,
            y: 0.0,
            variables: HashMap::new(),
            costumes: Vec::new(),
        }
    }

    pub async fn load_costume(&mut self, file: &str) -> Result<()> {
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

        self.costumes.push(image);
        Ok(())
    }

    pub fn change_costume(&mut self, index: usize) -> Result<()> {
        let image = match self.costumes.get(index) {
            Some(i) => i,
            None => return Err(format!("index is out of range: {}", index).into()),
        };
        self.context
            .draw_image_with_html_image_element(&image, 0.0, 0.0)?;
        Ok(())
    }

    pub fn say(&self, s: &str) -> Result<()> {
        js_sys::Reflect::set(&self.context, &"font".into(), &"10px sans-serif".into())?;
        self.context.fill_text(s, 150.0 + self.x, 150.0 + self.y)?;
        Ok(())
    }
}
