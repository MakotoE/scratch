use super::*;
use crate::blocks::BlockInfo;
use crate::broadcaster::{BroadcastMsg, Broadcaster};
use crate::coordinate::CanvasCoordinate;
use crate::file::ScratchFile;
use crate::fileinput::FileInput;
use crate::sprite::SpriteID;
use crate::vm::{DebugInfo, VM};
use std::collections::HashSet;
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
    OnKeyUp(yew::events::KeyboardEvent),
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
                self.event_sender.mouse_move(coordinate);
                false
            }
            Msg::OnKeyDown(e) => {
                if let Some(k) = KeyboardKey::from_key(&e.key()) {
                    self.event_sender.key_down(k);
                }
                false
            }
            Msg::OnKeyUp(e) => {
                if let Some(k) = KeyboardKey::from_key(&e.key()) {
                    self.event_sender.key_up(&k);
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
                        onkeyup={self.link.callback(Msg::OnKeyUp)}
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
    data: Rc<RefCell<Data>>,
}

impl EventSender {
    fn new(broadcaster: Broadcaster) -> Self {
        let data = Rc::new(RefCell::new(Data {
            mouse_position: CanvasCoordinate::default(),
            pressed_keys: HashSet::with_capacity(1),
        }));
        wasm_bindgen_futures::spawn_local({
            let data = data.clone();
            let broadcaster = broadcaster.clone();
            async move {
                if let Err(e) = EventSender::msg_loop(data, broadcaster).await {
                    log::error!("{}", wrap_err!(e));
                }
            }
        });
        Self { broadcaster, data }
    }

    async fn msg_loop(data: Rc<RefCell<Data>>, broadcaster: Broadcaster) -> Result<()> {
        let mut receiver = broadcaster.subscribe();
        loop {
            let msg = receiver.recv().await?;
            if let Some(m) = data.borrow().respond(msg) {
                broadcaster.send(m)?;
            }
        }
    }

    fn mouse_click(&self, coordinate: CanvasCoordinate) -> Result<()> {
        self.broadcaster.send(BroadcastMsg::MouseClick(coordinate))
    }

    fn mouse_move(&mut self, coordinate: CanvasCoordinate) {
        self.data.borrow_mut().mouse_position = coordinate;
    }

    fn key_down(&mut self, key: KeyboardKey) {
        self.data.borrow_mut().pressed_keys.insert(key);
    }

    fn key_up(&mut self, key: &KeyboardKey) {
        self.data.borrow_mut().pressed_keys.remove(key);
    }
}

#[derive(Debug, Clone)]
struct Data {
    mouse_position: CanvasCoordinate,
    pressed_keys: HashSet<KeyboardKey>,
}

impl Data {
    fn respond(&self, msg: BroadcastMsg) -> Option<BroadcastMsg> {
        Some(match msg {
            BroadcastMsg::RequestMousePosition => BroadcastMsg::MousePosition(self.mouse_position),
            BroadcastMsg::RequestPressedKeys => {
                BroadcastMsg::PressedKeys(self.pressed_keys.clone())
            }
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum KeyboardKey {
    Space,
    Up,
    Down,
    Right,
    Left,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    N0,
    N1,
    N2,
    N3,
    N4,
    N5,
    N6,
    N7,
    N8,
    N9,
}

impl KeyboardKey {
    fn from_key(key: &str) -> Option<Self> {
        Some(match key {
            " " => Self::Space,
            "ArrowUp" => Self::Up,
            "ArrowDown" => Self::Down,
            "ArrowRight" => Self::Right,
            "ArrowLeft" => Self::Left,
            "a" => Self::A,
            "b" => Self::B,
            "c" => Self::C,
            "d" => Self::D,
            "e" => Self::E,
            "f" => Self::F,
            "g" => Self::G,
            "h" => Self::H,
            "i" => Self::I,
            "j" => Self::J,
            "k" => Self::K,
            "l" => Self::L,
            "m" => Self::M,
            "n" => Self::N,
            "o" => Self::O,
            "p" => Self::P,
            "q" => Self::Q,
            "r" => Self::R,
            "s" => Self::S,
            "t" => Self::T,
            "u" => Self::U,
            "v" => Self::V,
            "w" => Self::W,
            "x" => Self::X,
            "y" => Self::Y,
            "z" => Self::Z,
            "0" => Self::N0,
            "1" => Self::N1,
            "2" => Self::N2,
            "3" => Self::N3,
            "4" => Self::N4,
            "5" => Self::N5,
            "6" => Self::N6,
            "7" => Self::N7,
            "8" => Self::N8,
            "9" => Self::N9,
            _ => return None,
        })
    }
}
