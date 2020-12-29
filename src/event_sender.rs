use super::*;
use crate::broadcaster::{BroadcastMsg, Broadcaster};
use crate::coordinate::CanvasCoordinate;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::EnumString;

#[derive(Debug)]
pub struct EventSender {
    broadcaster: Broadcaster,
    data: Rc<RefCell<Data>>,
}

impl EventSender {
    pub fn new(broadcaster: Broadcaster) -> Self {
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

    pub fn mouse_click(&self, coordinate: CanvasCoordinate) -> Result<()> {
        self.broadcaster.send(BroadcastMsg::MouseClick(coordinate))
    }

    pub fn mouse_move(&mut self, coordinate: CanvasCoordinate) {
        self.data.borrow_mut().mouse_position = coordinate;
    }

    pub fn key_down(&mut self, key: KeyboardKey) {
        self.data.borrow_mut().pressed_keys.insert(key);
    }

    pub fn key_up(&mut self, key: &KeyboardKey) {
        self.data.borrow_mut().pressed_keys.remove(key);
    }
}

#[derive(Debug, Clone)]
struct Data {
    mouse_position: CanvasCoordinate,
    pressed_keys: HashSet<KeyboardKey>,
}

impl Data {
    pub fn respond(&self, msg: BroadcastMsg) -> Option<BroadcastMsg> {
        Some(match msg {
            BroadcastMsg::RequestMousePosition => BroadcastMsg::MousePosition(self.mouse_position),
            BroadcastMsg::RequestPressedKeys => {
                BroadcastMsg::PressedKeys(self.pressed_keys.clone())
            }
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum KeyboardKey {
    #[strum(serialize = "space")]
    Space,
    #[strum(serialize = "up arrow")]
    Up,
    #[strum(serialize = "down arrow")]
    Down,
    #[strum(serialize = "right arrow")]
    Right,
    #[strum(serialize = "left arrow")]
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
    #[strum(serialize = "0")]
    N0,
    #[strum(serialize = "1")]
    N1,
    #[strum(serialize = "2")]
    N2,
    #[strum(serialize = "3")]
    N3,
    #[strum(serialize = "4")]
    N4,
    #[strum(serialize = "5")]
    N5,
    #[strum(serialize = "6")]
    N6,
    #[strum(serialize = "7")]
    N7,
    #[strum(serialize = "8")]
    N8,
    #[strum(serialize = "9")]
    N9,
}

impl KeyboardKey {
    pub fn from_key(key: &str) -> Option<Self> {
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

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum KeyOption {
    Any,
    Key(KeyboardKey),
}

impl FromStr for KeyOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "Any" => KeyOption::Any,
            _ => KeyOption::Key(KeyboardKey::from_str(s)?),
        })
    }
}

impl Display for KeyOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyOption::Any => f.write_str("Any"),
            KeyOption::Key(k) => Display::fmt(k, f),
        }
    }
}
