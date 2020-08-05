#![feature(async_closure)]

pub mod block;
pub mod runtime;
pub mod savefile;
pub mod sprite;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::services::reader::{FileData, ReaderService, ReaderTask};
use savefile::ScratchFile;

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Zip(zip::result::ZipError);
        JSON(serde_json::error::Error);
        IO(std::io::Error);
    }
}

impl std::convert::From<wasm_bindgen::JsValue> for Error {
    fn from(v: JsValue) -> Self {
        v.into_serde::<String>().unwrap().into()
    }
}

impl<T> std::convert::From<std::sync::PoisonError<T>> for Error {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        e.to_string().into()
    }
}

impl std::convert::Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> JsValue {
        self.to_string().into()
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<()> {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    App::<Page>::new().mount_to_body();
    Ok(())
}

struct Page {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    task: Option<ReaderTask>,
}

enum Msg {
    Noop,
    ImportFile(web_sys::File),
    Run(FileData),
}

impl Page {
    async fn run(context: web_sys::CanvasRenderingContext2d, scratch_file: ScratchFile) {
        let mut runtime = runtime::SpriteRuntime::new(context);
        runtime.load_costume(&scratch_file.images[0]).await.unwrap();

        let sprite = sprite::Sprite::new(runtime, &scratch_file.project.targets[1]).unwrap();
        sprite.execute().unwrap();
    }
}

impl Component for Page {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            canvas_ref: NodeRef::default(),
            task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Noop => return false,
            Msg::ImportFile(file) => {
                let mut reader = ReaderService::new();
                let cb = self.link.callback(Msg::Run);
                self.task = Some(reader.read_file(file, cb).unwrap());
            }
            Msg::Run(file) => {
                let reader = std::io::Cursor::new(file.content);
                let scratch_file = ScratchFile::parse(reader).unwrap();
                let canvas: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
                let ctx: web_sys::CanvasRenderingContext2d = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into()
                    .unwrap();
                ctx.scale(2.0, 2.0).unwrap();
                wasm_bindgen_futures::spawn_local(Page::run(ctx, scratch_file));
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let import_cb: fn(yew::events::ChangeData) -> Msg = |event| {
            if let ChangeData::Files(files) = event {
                if let Some(file) = files.get(0) {
                    return Msg::ImportFile(file);
                }
            }
            Msg::Noop
        };

        html! {
            <div>
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="600"
                    height="450"
                    style="border: 1px solid black"
                /><br />
                <input type="file" accept=".sb3" onchange={self.link.callback(import_cb)} />
            </div>
        }
    }

    fn rendered(&mut self, _: bool) {}
}
