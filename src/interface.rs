use super::*;
use crate::blocks::BlockInfo;
use crate::broadcaster::{BroadcastMsg, Broadcaster};
use crate::coordinate::CanvasCoordinate;
use crate::file::ScratchFile;
use crate::fileinput::FileInput;
use crate::sprite::SpriteID;
use crate::vm::{DebugInfo, VM};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use yew::prelude::*;

pub struct ScratchInterface {
    link: ComponentLink<Self>,
    canvas_ref: NodeRef,
    vm_state: VMState,
    file: Option<ScratchFile>,
    vm: Option<Rc<VM>>,
    debug_info: HashMap<SpriteID, Vec<Option<BlockInfo>>>,
    canvas_top_left: Option<CanvasCoordinate>,
    event_sender: EventSender,
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

    fn event_coordinate(
        canvas_top_left: &CanvasCoordinate,
        event: &yew::events::MouseEvent,
    ) -> CanvasCoordinate {
        CanvasCoordinate {
            x: event.client_x() as f64 - canvas_top_left.x,
            y: event.client_y() as f64 - canvas_top_left.y,
        }
    }
}

pub enum Msg {
    SetFile(ScratchFile),
    SetVM(VM),
    Run,
    ContinuePause,
    Step,
    SetDebug(DebugInfo),
    OnMouseClick(yew::events::MouseEvent),
    OnMouseMove(yew::events::MouseEvent),
    OnKeyDown(yew::events::KeyboardEvent),
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
            canvas_top_left: None,
            event_sender: EventSender::new(Broadcaster::new()),
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

                let broadcaster = Broadcaster::new();
                self.event_sender = EventSender::new(broadcaster.clone());

                wasm_bindgen_futures::spawn_local({
                    let scratch_file = self.file.as_ref().unwrap().clone();
                    let set_vm = self.link.callback(Msg::SetVM);
                    let set_debug = self.link.callback(Msg::SetDebug);
                    let (debug_sender, mut debug_receiver) = mpsc::channel(1);

                    async move {
                        let vm = match VM::start(ctx, scratch_file, debug_sender, broadcaster).await
                        {
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
            Msg::OnMouseClick(e) => {
                let coordinate =
                    ScratchInterface::event_coordinate(&self.canvas_top_left.unwrap(), &e);
                self.event_sender.mouse_click(coordinate).unwrap(); // TODO handle errors
                false
            }
            Msg::OnMouseMove(e) => {
                let coordinate =
                    ScratchInterface::event_coordinate(&self.canvas_top_left.unwrap(), &e);
                self.event_sender.mouse_move(coordinate).unwrap();
                false
            }
            Msg::OnKeyDown(e) => false, // TODO
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
                <div onmousemove={self.link.callback(Msg::OnMouseMove)}>
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
                        onclick={self.link.callback(Msg::OnMouseClick)}
                        onkeydown={self.link.callback(Msg::OnKeyDown)}
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

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let canvas: web_sys::Element = self.canvas_ref.cast().unwrap();
            let rect = canvas.get_bounding_client_rect();
            self.canvas_top_left = Some(CanvasCoordinate {
                x: rect.left(),
                y: rect.top(),
            });
        }
    }
}

#[derive(Debug)]
struct EventSender {
    broadcaster: Broadcaster,
    requests: Rc<RefCell<Requests>>,
}

#[derive(Debug)]
struct Requests {
    requesting_mouse_position: bool,
}

impl EventSender {
    fn new(broadcaster: Broadcaster) -> Self {
        let requests: Rc<RefCell<Requests>> = Rc::new(RefCell::new(Requests {
            requesting_mouse_position: false,
        }));
        wasm_bindgen_futures::spawn_local({
            let requests = requests.clone();
            let broadcaster = broadcaster.clone();
            async move {
                EventSender::listen_loop(broadcaster.subscribe(), requests)
                    .await
                    .unwrap_or_else(|e| log::error!("{}", wrap_err!(e)));
            }
        });
        Self {
            broadcaster,
            requests,
        }
    }

    async fn listen_loop(
        mut receiver: Receiver<BroadcastMsg>,
        requests: Rc<RefCell<Requests>>,
    ) -> Result<()> {
        loop {
            match receiver.recv().await? {
                BroadcastMsg::RequestMousePosition => {
                    requests.borrow_mut().requesting_mouse_position = true;
                }
                _ => {}
            }
        }
    }

    fn mouse_click(&self, coordinate: CanvasCoordinate) -> Result<()> {
        self.broadcaster.send(BroadcastMsg::MouseClick(coordinate))
    }

    fn mouse_move(&mut self, coordinate: CanvasCoordinate) -> Result<()> {
        let mut requests = self.requests.borrow_mut();
        if requests.requesting_mouse_position {
            self.broadcaster
                .send(BroadcastMsg::MousePosition(coordinate))?;
        }
        requests.requesting_mouse_position = false;
        Ok(())
    }
}
