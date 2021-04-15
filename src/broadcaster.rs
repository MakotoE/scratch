use super::*;
use crate::blocks::test::BlockStubMsg;
use crate::coordinate::{CanvasCoordinate, SpriteRectangle};
use crate::file::BlockID;
use crate::sprite::SpriteID;
use crate::vm::ThreadID;
use graphics_buffer::RenderBuffer;
use input::Key;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Broadcaster {
    sender: Sender<BroadcastMsg>,
}

impl Broadcaster {
    pub fn send(&self, m: BroadcastMsg) -> Result<()> {
        self.sender.send(m)?;
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<BroadcastMsg> {
        self.sender.subscribe()
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self {
            sender: channel(64).0,
        }
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
    PressedKeys(HashSet<Key>),
    RequestSpriteRectangle(SpriteID),
    SpriteRectangle {
        sprite: SpriteID,
        rectangle: SpriteRectangle,
    },
    /// Requests image but with sprite removed
    RequestCanvasImage(SpriteID),
    CanvasImage(RenderBuffer),
    BlockStub(BlockID, BlockStubMsg),
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
}
