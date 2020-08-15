#![feature(async_closure)]
#![feature(try_trait)]
#![feature(str_split_once)]

pub mod blocks;
pub mod runtime;
pub mod savefile;
pub mod sprite;

use savefile::ScratchFile;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::services::reader::{FileData, ReaderService, ReaderTask};

#[wasm_bindgen(start)]
pub fn start() -> Result<()> {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    App::<Page>::new().mount_to_body();
    Ok(())
}

error_chain::error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Zip(zip::result::ZipError);
        JSON(serde_json::error::Error);
        IO(std::io::Error);
        ParseFloatError(std::num::ParseFloatError);
    }

    errors {
        Block(block_name: &'static str, block_id: String, error: Box<Error>) {
            description("block error")
            display(
                r#"block "{}" of type {} returned error during execution: {}"#,
                block_id,
                block_name,
                error.to_string(),
            )
        }
    }
}

impl std::convert::From<wasm_bindgen::JsValue> for Error {
    fn from(v: JsValue) -> Self {
        let mut s = format!("{:?}", v);
        if let Some(stripped_prefix) = s.strip_prefix("JsValue(") {
            s = stripped_prefix.strip_suffix(")").unwrap_or(&s).to_string();
        }
        s.into()
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
    async fn run(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: ScratchFile,
    ) -> Result<()> {
        let mut variables: HashMap<String, serde_json::Value> = HashMap::new();
        for (key, v) in &scratch_file.project.targets[0].variables {
            variables.insert(key.clone(), v.1.clone());
        }

        let mut runtime = runtime::SpriteRuntime::new(context, variables);
        for costume in &scratch_file.project.targets[1].costumes {
            match &scratch_file.images.get(&costume.md5ext) {
                Some(file) => {
                    let rotation_center = runtime::Coordinate::new(
                        costume.rotation_center_x,
                        costume.rotation_center_y,
                    );
                    runtime.load_costume(file, rotation_center).await?
                }
                None => return Err(format!("image not found: {}", costume.md5ext).into()),
            }
        }

        let sprite = sprite::Sprite::new(runtime, &scratch_file.project.targets[1])?;
        sprite.execute().await
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
                let ctx: web_sys::CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();

                let future = (async || match Page::run(ctx, scratch_file).await {
                    Ok(_) => {}
                    Err(e) => log::error!("{}", e),
                })();
                wasm_bindgen_futures::spawn_local(future);
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
                    width="960"
                    height="720"
                    style="height: 360px; width: 480px; border: 1px solid black;"
                /><br />
                <input type="file" accept=".sb3" onchange={self.link.callback(import_cb)} />
            </div>
        }
    }

    fn rendered(&mut self, _: bool) {}
}
