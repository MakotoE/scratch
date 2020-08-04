use super::*;
use web_sys::HtmlImageElement;

#[derive(Debug)]
pub struct SpriteRuntime {
    context: RefCell<web_sys::CanvasRenderingContext2d>,
    pub x: f64,
    pub y: f64,
    pub variables: HashMap<String, serde_json::Value>,
}

impl SpriteRuntime {
    pub fn new(context: web_sys::CanvasRenderingContext2d) -> Self {
        let context_ref = RefCell::new(context);
        let context_clone = context_ref.clone();
        let image = RefCell::new(HtmlImageElement::new().unwrap());
        let image_clone = image.clone();
        let cb = Closure::once_into_js(Box::new(move || {
            context_clone
                .borrow()
                .draw_image_with_html_image_element(&image_clone.borrow(), 0.0, 0.0)
                .unwrap();
        }) as Box<dyn Fn()>);
        image.borrow().set_onload(Some(cb.as_ref().unchecked_ref()));

        let error_cb = Closure::once_into_js(Box::new(move || {
            log::error!("context failed to load image")
        }) as Box<dyn Fn()>);
        image.borrow().set_onerror(Some(error_cb.as_ref().unchecked_ref()));

        image.borrow().set_src("./b7853f557e4426412e64bb3da6531a99.svg");

        Self {
            context: context_ref,
            x: 0.0,
            y: 0.0,
            variables: HashMap::new(),
        }
    }

    pub fn say(&self, s: &str) -> Result<()> {
        js_sys::Reflect::set(
            &self.context.borrow(),
            &"font".into(),
            &"10px sans-serif".into(),
        )?;
        self.context
            .borrow()
            .fill_text(s, 150.0 + self.x, 150.0 + self.y)?;
        Ok(())
    }
}
