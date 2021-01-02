use super::*;
use crate::coordinate::{CanvasCoordinate, SpriteRectangle};
use crate::event_sender::KeyboardKey;
use crate::sprite::SpriteID;
use crate::vm::ThreadID;
use ndarray::Array2;
use palette::Hsv;
use std::collections::HashSet;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Broadcaster {
    sender: Sender<BroadcastMsg>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            sender: channel(8).0,
        }
    }

    pub fn send(&self, m: BroadcastMsg) -> Result<()> {
        self.sender.send(m)?;
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<BroadcastMsg> {
        self.sender.subscribe()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastMsg {
    Start(String),
    Finished(String),
    Clone(SpriteID),
    DeleteClone(SpriteID),
    Stop(Stop),
    ChangeLayer {
        sprite: SpriteID,
        action: LayerChange,
    },
    MouseClick(CanvasCoordinate),
    RequestMousePosition,
    MousePosition(CanvasCoordinate),
    RequestPressedKeys,
    PressedKeys(HashSet<KeyboardKey>),
    RequestSpriteRectangle(SpriteID),
    SpriteRectangle {
        sprite: SpriteID,
        rectangle: SpriteRectangle,
    },
    RequestCanvasImage,
    CanvasImage(Array2<Hsv>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Stop {
    All,
    ThisThread(ThreadID),
    OtherThreads(ThreadID),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LayerChange {
    Front,
    Back,
    ChangeBy(i64),
}
