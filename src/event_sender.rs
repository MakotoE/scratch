use super::*;
use crate::broadcaster::{BroadcastMsg, Broadcaster};
use crate::coordinate::CanvasCoordinate;
use std::collections::HashSet;
use std::str::FromStr;
use strum::{EnumString, IntoStaticStr};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, EnumString, IntoStaticStr)]
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

impl KeyOption {
    pub fn from_scratch_option(s: &str) -> Result<Self> {
        Ok(Self::Key(match s {
            "any" => return Ok(Self::Any),
            "space" => KeyboardKey::Space,
            "up arrow" => KeyboardKey::Up,
            "down arrow" => KeyboardKey::Down,
            "right arrow" => KeyboardKey::Right,
            "left arrow" => KeyboardKey::Left,
            "a" => KeyboardKey::A,
            "b" => KeyboardKey::B,
            "c" => KeyboardKey::C,
            "d" => KeyboardKey::D,
            "e" => KeyboardKey::E,
            "f" => KeyboardKey::F,
            "g" => KeyboardKey::G,
            "h" => KeyboardKey::H,
            "i" => KeyboardKey::I,
            "j" => KeyboardKey::J,
            "k" => KeyboardKey::K,
            "l" => KeyboardKey::L,
            "m" => KeyboardKey::M,
            "n" => KeyboardKey::N,
            "o" => KeyboardKey::O,
            "p" => KeyboardKey::P,
            "q" => KeyboardKey::Q,
            "r" => KeyboardKey::R,
            "s" => KeyboardKey::S,
            "t" => KeyboardKey::T,
            "u" => KeyboardKey::U,
            "v" => KeyboardKey::V,
            "w" => KeyboardKey::W,
            "x" => KeyboardKey::X,
            "y" => KeyboardKey::Y,
            "z" => KeyboardKey::Z,
            "0" => KeyboardKey::N0,
            "1" => KeyboardKey::N1,
            "2" => KeyboardKey::N2,
            "3" => KeyboardKey::N3,
            "4" => KeyboardKey::N4,
            "5" => KeyboardKey::N5,
            "6" => KeyboardKey::N6,
            "7" => KeyboardKey::N7,
            "8" => KeyboardKey::N8,
            "9" => KeyboardKey::N9,
            _ => return Err(wrap_err!(format!("unknown key: {}", s))),
        }))
    }
}

impl From<KeyOption> for &str {
    fn from(o: KeyOption) -> Self {
        match o {
            KeyOption::Any => "Any",
            KeyOption::Key(k) => k.into(),
        }
    }
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
