use super::*;
use fileinput::FileInput;
use runtime::{Global, Runtime};
use savefile::ScratchFile;
use sprite::Sprite;
use yew::prelude::*;

pub struct ScratchInterface {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    vm_state: VMState,
    file: Option<ScratchFile>,
    sprite: Option<Arc<RwLock<Sprite>>>,
}

pub enum Msg {
    SetFile(ScratchFile),
    SetSprite(Sprite),
    Run,
    ContinuePause,
    Step,
}

#[derive(Copy, Clone)]
pub enum VMState {
    Running,
    Paused,
}

impl ScratchInterface {
    async fn runtime(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: &ScratchFile,
    ) -> Result<Runtime> {
        let global = Global::new(&scratch_file.project.targets[0].variables);

        let runtime = runtime::SpriteRuntime::new(
            context,
            &scratch_file.project.targets[1].costumes,
            &scratch_file.images,
        )
        .await?;

        Ok(Runtime {
            sprite: Rc::new(RwLock::new(runtime)),
            global,
        })
    }
}

impl Component for ScratchInterface {
    type Message = Msg;
    type Properties = ();

    fn create(_: (), link: ComponentLink<Self>) -> Self {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .set_title("Scratch VM");

        Self {
            link,
            canvas_ref: NodeRef::default(),
            vm_state: VMState::Running,
            file: None,
            sprite: None,
        }
    }

    fn update(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::SetFile(file) => {
                self.file = Some(file);
                self.link.send_message(Msg::Run);
                false
            }
            Msg::Run => {
                let canvas: web_sys::HtmlCanvasElement = self.canvas_ref.cast().unwrap();
                let ctx: web_sys::CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();

                let scratch_file = self.file.as_ref().unwrap().clone();
                let start_state = self.vm_state;
                let set_sprite = self.link.callback(Msg::SetSprite);
                wasm_bindgen_futures::spawn_local(async move {
                    match ScratchInterface::runtime(ctx, &scratch_file).await {
                        Ok(runtime) => {
                            match Sprite::new(
                                runtime,
                                &scratch_file.project.targets[1],
                                start_state,
                            )
                            .await
                            {
                                Ok(s) => set_sprite.emit(s),
                                Err(e) => log::error!("{}", e),
                            }
                        }
                        Err(e) => log::error!("{}", e),
                    };
                });
                false
            }
            Msg::SetSprite(sprite) => {
                self.sprite = Some(Arc::new(RwLock::new(sprite)));
                false
            }
            Msg::ContinuePause => {
                let state = self.vm_state;
                match state {
                    VMState::Paused => self.vm_state = VMState::Running,
                    VMState::Running => self.vm_state = VMState::Paused,
                }

                let sprite = self.sprite.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(sprite) = sprite {
                        match state {
                            VMState::Paused => sprite.write().await.continue_().await,
                            VMState::Running => sprite.write().await.pause().await,
                        }
                    }
                });
                true
            }
            Msg::Step => {
                if let Some(sprite) = self.sprite.clone() {
                    wasm_bindgen_futures::spawn_local(async move {
                        sprite.write().await.step();
                    })
                }
                false
            }
        }
    }

    fn change(&mut self, _: ()) -> bool {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <div style="font-family: sans-serif;">
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="960"
                    height="720"
                    style="height: 360px; width: 480px; border: 1px solid black;"
                /><br />
                <FileInput onchange={self.link.callback(Msg::SetFile)} /><br />
                <br />
                <button onclick={self.link.callback(|_| Msg::ContinuePause)} style="width: 120px;">
                    {
                        match self.vm_state {
                            VMState::Paused => "Continue",
                            VMState::Running => "Pause",
                        }
                    }
                </button>{"\u{00a0}"}
                {
                    match self.vm_state {
                        VMState::Paused => html! {
                            <button onclick={self.link.callback(|_| Msg::Step)}>
                                {"Step"}
                            </button>
                        },
                        VMState::Running => html! {
                            <select>
                                <option>{"Normal speed"}</option>
                            </select>
                        },
                    }
                }{"\u{00a0}"}
                <button onclick={self.link.callback(|_| Msg::Run)}>{"Restart"}</button>{"\u{00a0}"}
            </div>
        }
    }
}
