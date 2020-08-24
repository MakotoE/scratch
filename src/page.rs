use super::*;
use controller::DebugController;
use savefile::ScratchFile;
use yew::prelude::*;
use yew::services::reader::{FileData, ReaderService, ReaderTask};
use sprite::Sprite;

pub struct Page {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    task: Option<ReaderTask>,
    state: VMState,
    sprite: Arc<RwLock<Sprite>>
}

pub enum Msg {
    Noop,
    ImportFile(web_sys::File),
    Load(FileData),
    LoadFinished(Sprite),
    Run,
    ContinuePause,
    Slow,
    Step,
}

#[derive(Copy, Clone)]
enum VMState {
    Running,
    Paused,
}

impl Page {
    async fn load(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: ScratchFile,
    ) -> Result<Sprite> {
        let mut variables: HashMap<String, serde_json::Value> = HashMap::new();
        for (key, v) in &scratch_file.project.targets[0].variables {
            variables.insert(key.clone(), v.1.clone());
        }

        let mut runtime = runtime::SpriteRuntime::new(context, variables);
        for costume in &scratch_file.project.targets[1].costumes {
            match scratch_file.images.get(&costume.md5ext) {
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

        Sprite::new(runtime, &scratch_file.project.targets[1], DebugController::new())
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
            state: VMState::Running,
            sprite: Arc::new(RwLock::default()),
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Msg::Noop => return false,
            Msg::ImportFile(file) => {
                let mut reader = ReaderService::new();
                let cb = self.link.callback(Msg::Load);
                self.task = Some(reader.read_file(file, cb).unwrap());
            }
            Msg::Load(file_data) => {
                let reader = std::io::Cursor::new(file_data.content);
                let scratch_file = ScratchFile::parse(reader).unwrap();
                let canvas: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
                let ctx: web_sys::CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();

                let cb = self.link.callback(Msg::LoadFinished);
                wasm_bindgen_futures::spawn_local((async move || {
                    let sprite = match Page::load(ctx, scratch_file).await {
                        Ok(s) => cb.emit(s),
                        Err(e) => log::error!("{}", e), // TODO error Msg
                    };
                })());
            }
            Msg::LoadFinished(sprite) => {
                self.sprite = Arc::new(RwLock::new(sprite));
            }
            Msg::Run => {
                let sprite = self.sprite.clone();
                wasm_bindgen_futures::spawn_local((async move || {
                    sprite.write().await.reset().await;
                    match sprite.read().await.execute().await {
                        Ok(_) => {}
                        Err(e) => log::error!("{}", e),
                    }
                })());
            }
            Msg::ContinuePause => {
                let state = self.state;
                match state {
                    VMState::Paused => self.state = VMState::Running,
                    VMState::Running => self.state = VMState::Paused,
                }

                let sprite = self.sprite.clone();
                wasm_bindgen_futures::spawn_local((async move || match state {
                    VMState::Paused => sprite.write().await.continue_().await,
                    VMState::Running => sprite.write().await.pause().await,
                })());
            }
            Msg::Slow => {
                let sprite = self.sprite.clone();
                wasm_bindgen_futures::spawn_local((async move || {
                    sprite.write().await.slow_speed().await;
                })());
            }
            Msg::Step => {
                let sprite = self.sprite.clone();
                wasm_bindgen_futures::spawn_local((async move || {
                    sprite.read().await.step();
                })());
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
                <input type="file" accept=".sb3" onchange={self.link.callback(import_cb)} /><br />
                <br />
                <button onclick={self.link.callback(|_| Msg::ContinuePause)} style="width: 120px;">
                    {
                        match self.state {
                            VMState::Paused => "Continue",
                            VMState::Running => "Pause",
                        }
                    }
                </button>{"\u{00a0}"}
                {
                    match self.state {
                        VMState::Paused => {
                            html! {
                                <>
                                    <button onclick={self.link.callback(|_| Msg::Slow)}>
                                        {"Normal speed"}
                                    </button>
                                    {"\u{00a0}"}
                                    <button onclick={self.link.callback(|_| Msg::Step)}>
                                        {"Step"}
                                    </button>
                                    {"\u{00a0}"}

                                </>
                            }
                        }
                        VMState::Running => html! {<></>}
                    }
                }
                <button onclick={self.link.callback(|_| Msg::Run)}>{"Restart"}</button>{"\u{00a0}"}
            </div>
        }
    }
}
