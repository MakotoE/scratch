use super::*;
use runtime::SpriteRuntime;
use savefile::ScratchFile;
use sprite::Sprite;
use yew::prelude::*;
use yew::services::reader::{FileData, ReaderService, ReaderTask};

pub struct Page {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    task: Option<ReaderTask>,
    state: VMState,
    file: Option<ScratchFile>,
    sprite: Arc<RwLock<Sprite>>,
}

pub enum Msg {
    Noop,
    ImportFile(web_sys::File),
    Load(FileData),
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
    async fn runtime(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: &ScratchFile,
    ) -> Result<SpriteRuntime> {
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

        Ok(runtime)
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
            file: None,
            sprite: Arc::new(RwLock::new(Sprite::new())),
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
                self.file = Some(ScratchFile::parse(reader).unwrap());
                self.link.send_message(Msg::Run);
            }
            Msg::Run => {
                let canvas: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
                let ctx: web_sys::CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();

                let sprite = self.sprite.clone();
                let scratch_file = self.file.as_ref().unwrap().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match Page::runtime(ctx, &scratch_file).await {
                        Ok(s) => match sprite
                            .write()
                            .await
                            .start(s, &scratch_file.project.targets[1])
                        {
                            Ok(_) => {}
                            Err(e) => log::error!("{}", e),
                        },
                        Err(e) => log::error!("{}", e),
                    };
                });
            }
            Msg::ContinuePause => {
                let state = self.state;
                match state {
                    VMState::Paused => self.state = VMState::Running,
                    VMState::Running => self.state = VMState::Paused,
                }

                let sprite = self.sprite.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match state {
                        VMState::Paused => sprite.write().await.continue_().await,
                        VMState::Running => sprite.write().await.pause().await,
                    }
                });
            }
            Msg::Slow => {
                let sprite = self.sprite.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    sprite.write().await.slow_speed().await;
                });
            }
            Msg::Step => {
                let sprite = self.sprite.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    sprite.read().await.step();
                });
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
