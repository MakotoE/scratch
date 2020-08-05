use super::*;
use web_sys::{HtmlImageElement, Blob, BlobPropertyBag, Url};

#[derive(Debug)]
pub struct SpriteRuntime {
    context: web_sys::CanvasRenderingContext2d,
    pub x: f64,
    pub y: f64,
    pub variables: HashMap<String, serde_json::Value>,
    pub costumes: Vec<RefCell<HtmlImageElement>>,
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

    pub fn load_costume(&mut self, file: &str) -> Result<()> {
        let parts = js_sys::Array::new_with_length(1);
        parts.set(0, file.into());
        let mut properties = BlobPropertyBag::new();
        properties.type_("image/svg+xml");
        let blob = Blob::new_with_str_sequence_and_options(&parts, &properties)?;
        let url = Url::create_object_url_with_blob(&blob)?;

        let image = HtmlImageElement::new()?;

        let error_cb =
            Closure::once_into_js(
                Box::new(move || log::error!("context failed to load image")) as Box<dyn Fn()>,
            );
        image.set_onerror(Some(error_cb.as_ref().unchecked_ref()));

        let url_clone = url.clone();
        let context_clone = self.context.clone();
        let image_clone = image.clone();
        let cb = Closure::once_into_js(Box::new(move || {
            Url::revoke_object_url(&url_clone).unwrap();
            // TODO move draw image to new method
            context_clone.draw_image_with_html_image_element(&image_clone, 0.0, 0.0).unwrap();
        }) as Box<dyn Fn()>);
        image.set_onload(Some(cb.as_ref().unchecked_ref()));

        image.set_src(&url);

        self.costumes.push(RefCell::new(image));

        Ok(())
    }

    pub fn say(&self, s: &str) -> Result<()> {
        js_sys::Reflect::set(
            &self.context,
            &"font".into(),
            &"10px sans-serif".into(),
        )?;
        self.context.fill_text(s, 150.0 + self.x, 150.0 + self.y)?;
        Ok(())
    }
}
