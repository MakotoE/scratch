use super::*;
use crate::blocks::BlockInfo;
use crate::coordinate::{CanvasCoordinate, SpriteCoordinate};
use crate::fileinput::FileInput;
use crate::savefile::ScratchFile;
use crate::sprite::SpriteID;
use crate::vm::{DebugInfo, VM};
use tokio::sync::mpsc;
use yew::prelude::*;

pub struct ScratchInterface {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    vm_state: VMState,
    file: Option<ScratchFile>,
    vm: Option<Rc<VM>>,
    debug_info: HashMap<SpriteID, Vec<Option<BlockInfo>>>,
    canvas_position: Option<SpriteCoordinate>,
}

impl ScratchInterface {
    fn debug_output(debug_info: &HashMap<SpriteID, Vec<Option<BlockInfo>>>) -> String {
        let mut result = String::new();
        for (sprite_id, sprite) in debug_info {
            result.push_str(&format!("sprite: {}\n", sprite_id));
            for (thread_id, block_info) in sprite.iter().enumerate() {
                result.push_str(&format!("\t{}: ", thread_id));
                match block_info {
                    Some(info) => result.push_str(&format!("name: {}, id: {}", info.name, info.id)),
                    None => result.push_str("None"),
                }
                result.push('\n');
            }
        }
        result
    }

    fn canvas_position(&mut self) -> &SpriteCoordinate {
        let canvas: web_sys::Element = self.canvas_ref.cast().unwrap();
        self.canvas_position.get_or_insert_with(|| {
            let rect = canvas.get_bounding_client_rect();
            SpriteCoordinate {
                x: rect.left(),
                y: rect.top(),
            }
        })
    }
}

pub enum Msg {
    SetFile(ScratchFile),
    SetVM(VM),
    Run,
    ContinuePause,
    Step,
    SetDebug(DebugInfo),
    OnCanvasClick(MouseEvent),
    Start,
    Stop,
}

#[derive(Debug, Copy, Clone)]
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
            debug_info: HashMap::new(),
            canvas_position: None,
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
                let ctx = canvas.get_context("2d").unwrap().unwrap().unchecked_into();

                let scratch_file = self.file.as_ref().unwrap().clone();
                let set_vm = self.link.callback(Msg::SetVM);
                let set_debug = self.link.callback(Msg::SetDebug);
                wasm_bindgen_futures::spawn_local(async move {
                    let (debug_sender, mut debug_receiver) = mpsc::channel(1);
                    let vm = match VM::start(ctx, scratch_file, debug_sender).await {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("{}", e);
                            return;
                        }
                    };
                    set_vm.emit(vm);

                    loop {
                        match debug_receiver.recv().await {
                            Some(d) => set_debug.emit(d),
                            None => return,
                        }
                    }
                });
                false
            }
            Msg::SetVM(vm) => {
                self.vm = Some(Rc::new(vm));
                false
            }
            Msg::ContinuePause => {
                let state = self.vm_state;
                match state {
                    VMState::Paused => self.vm_state = VMState::Running,
                    VMState::Running => self.vm_state = VMState::Paused,
                }

                if let Some(vm) = self.vm.clone() {
                    wasm_bindgen_futures::spawn_local(async move {
                        match state {
                            VMState::Paused => vm.continue_().await,
                            VMState::Running => vm.pause().await,
                        }
                    });
                }
                true
            }
            Msg::Step => {
                if let Some(vm) = self.vm.clone() {
                    wasm_bindgen_futures::spawn_local(async move {
                        vm.step().await;
                    })
                }
                false
            }
            Msg::SetDebug(debug) => {
                let id = &debug.thread_id;
                let thread = self.debug_info.entry(id.sprite_id).or_default();
                if thread.len() <= id.thread_id {
                    thread.resize(id.thread_id + 1, None);
                }
                thread[id.thread_id] = Some(debug.block_info);
                true
            }
            Msg::OnCanvasClick(e) => {
                let canvas_position = *self.canvas_position();
                if let Some(vm) = &self.vm {
                    vm.click(CanvasCoordinate {
                        x: e.client_x() as f64 - canvas_position.x,
                        y: e.client_y() as f64 - canvas_position.y,
                    });
                }
                false
            }
            Msg::Start => {
                if let Some(vm) = self.vm.clone() {
                    wasm_bindgen_futures::spawn_local(async move {
                        vm.stop().await;
                        vm.continue_().await;
                    });
                }
                false
            }
            Msg::Stop => {
                if let Some(vm) = self.vm.clone() {
                    wasm_bindgen_futures::spawn_local(async move {
                        vm.stop().await;
                    });
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
            <div style="font-family: sans-serif; display: flex;">
                <div>
                    <div style="margin-bottom: 5px;">
                        <a style="cursor: pointer;" onclick={self.link.callback(|_| Msg::Start)}>
                            <img
                                src="/static/green_flag.svg"
                                style="width: 30px; height: 30px; margin-left: 8px; margin-right: 20px;"
                                title="Go"
                            />
                        </a>
                        <a style="cursor: pointer;" onclick={self.link.callback(|_| Msg::Stop)}>
                            <img
                                src="/static/stop.svg"
                                style="width: 30px; height: 30px; cursor: pointer;"
                                title="Stop"
                            />
                        </a>
                    </div>
                    <canvas
                        ref={self.canvas_ref.clone()}
                        width="960"
                        height="720"
                        style="width: 480px; height: 360px; border: 1px solid black;"
                        onclick={self.link.callback(Msg::OnCanvasClick)}
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
                </div>
                <div style="margin-left: 20px; font-family: monospace;">
                    <pre style="margin: 5px 0; tab-size: 2; -moz-tab-size: 2;">
                        {
                            match self.vm_state {
                                VMState::Paused => ScratchInterface::debug_output(&self.debug_info),
                                VMState::Running => String::new(),
                            }
                        }
                    </pre>
                </div>
            </div>
        }
    }
}
