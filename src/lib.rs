pub mod block;
pub mod savefile;
pub mod sprite;

use log::info;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;

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

#[wasm_bindgen(start)]
pub fn start() -> Result<()> {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Page>::new().mount_to_body();
    Ok(())
}

struct Page {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
}

enum Msg {}

impl Component for Page {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            canvas_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="400"
                    height="300"
                    style="border: 1px solid black"
                />
            </div>
        }
    }

    fn rendered(&mut self, _: bool) {
        let canvas: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
        let ctx: web_sys::CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        js_sys::Reflect::set(&ctx, &"font".into(), &"20px sans-serif".into());
        ctx.fill_text("Hello world", 10.0, 50.0).unwrap();
    }
}
