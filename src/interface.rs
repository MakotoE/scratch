use super::*;
use fileinput::FileInput;
use savefile::ScratchFile;
use vm::VM;
use yew::prelude::*;

pub struct ScratchInterface {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    vm_state: VMState,
    file: Option<ScratchFile>,
    vm: Option<Arc<RwLock<VM>>>,
}

pub enum Msg {
    SetFile(ScratchFile),
    SetVM(VM),
    Run,
    ContinuePause,
    Step,
}

#[derive(Copy, Clone)]
pub enum VMState {
    Running,
    Paused,
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
            vm: None,
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
                let set_vm = self.link.callback(Msg::SetVM);
                wasm_bindgen_futures::spawn_local(async move {
                    let vm = match VM::new(ctx, &scratch_file).await {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        }
                    };
                    match start_state {
                        VMState::Paused => vm.pause().await,
                        VMState::Running => vm.continue_().await,
                    }
                    set_vm.emit(vm);
                });
                false
            }
            Msg::SetVM(vm) => {
                self.vm = Some(Arc::new(RwLock::new(vm)));
                false
            }
            Msg::ContinuePause => {
                let state = self.vm_state;
                match state {
                    VMState::Paused => self.vm_state = VMState::Running,
                    VMState::Running => self.vm_state = VMState::Paused,
                }

                let vm = self.vm.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(vm) = vm {
                        match state {
                            VMState::Paused => vm.write().await.continue_().await,
                            VMState::Running => vm.write().await.pause().await,
                        }
                    }
                });
                true
            }
            Msg::Step => {
                if let Some(vm) = self.vm.clone() {
                    wasm_bindgen_futures::spawn_local(async move {
                        vm.write().await.step();
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
                    style="width: 480px; height: 360px; border: 1px solid black;"
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
