pub mod block;
pub mod savefile;
pub mod sprite;

use wasm_bindgen::prelude::*;

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Zip(zip::result::ZipError);
        JSON(serde_json::error::Error);
    }
}

impl std::convert::From<wasm_bindgen::JsValue> for Error {
    fn from(v: JsValue) -> Self {
        v.into_serde::<String>().unwrap().into()
    }
}

impl std::convert::Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> JsValue {
        self.to_string().into()
    }
}

#[allow(unused_macros)]
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<()> {
    use wasm_bindgen::JsCast;
    console_error_panic_hook::set_once();
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: web_sys::HtmlCanvasElement = document.create_element("canvas")?.dyn_into().unwrap();
    canvas.set_attribute("width", "400");
    canvas.set_attribute("height", "300");
    canvas.set_attribute("style", "border: 1px solid black");
    let ctx: web_sys::CanvasRenderingContext2d =
        canvas.get_context("2d")?.unwrap().dyn_into().unwrap();
    js_sys::Reflect::set(&ctx, &"font".into(), &"20px sans-serif".into());
    ctx.fill_text("Hello world", 10.0, 50.0)?;

    document.body().unwrap().append_with_node_1(&canvas)?;
    Ok(())
}
