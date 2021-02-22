use super::*;
use crate::broadcaster::{BroadcastMsg, Broadcaster};
use crate::coordinate::CanvasCoordinate;
use crate::interface::CANVAS_TOP_LEFT;
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use input::{ButtonState, Input, Key, Motion, MouseButton};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::EnumString;
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct EventSender {
    handler_loop: JoinHandle<()>,
    sender: Sender<Input>,
    mouse_position: Arc<RwLock<CanvasCoordinate>>,
}

impl EventSender {
    pub fn new(broadcaster: Broadcaster) -> Self {
        let (sender, receiver) = channel(8);
        let mouse_position = Arc::new(RwLock::default());
        let handler_loop = spawn({
            let mouse_position = mouse_position.clone();
            async move {
                if let Err(e) =
                    EventSender::handler_loop(receiver, broadcaster, mouse_position).await
                {
                    log::error!("{}", e);
                }
            }
        });
        Self {
            handler_loop,
            sender,
            mouse_position,
        }
    }

    async fn handler_loop(
        mut input_receiver: Receiver<Input>,
        broadcaster: Broadcaster,
        mouse_position: Arc<RwLock<CanvasCoordinate>>,
    ) -> Result<()> {
        let mut broadcaster_receiver = broadcaster.subscribe();
        let mut pressed_keys: HashSet<Key> = HashSet::new();
        loop {
            select! {
                m = broadcaster_receiver.recv() => match m {
                    Ok(msg) => match msg {
                        BroadcastMsg::RequestMousePosition => {
                            broadcaster
                                .send(BroadcastMsg::MousePosition(*mouse_position.read().await))?;
                        }
                        BroadcastMsg::RequestPressedKeys => {
                            broadcaster.send(BroadcastMsg::PressedKeys(pressed_keys.clone()))?;
                        }
                        _ => {}
                    }
                    Err(e) => return Err(e.into()),
                },
                i = input_receiver.recv() => match i {
                    Some(input) => {
                        match input {
                            Input::Button(button) => match button.button {
                                input::Button::Keyboard(key) => match button.state {
                                    ButtonState::Press => {
                                        pressed_keys.insert(key);
                                    }
                                    ButtonState::Release => {
                                        pressed_keys.remove(&key);
                                    }
                                },
                                input::Button::Mouse(mouse) => {
                                    if matches!(mouse, MouseButton::Left)
                                        && matches!(button.state, ButtonState::Press)
                                    {
                                        broadcaster.send(BroadcastMsg::MouseClick(
                                            *mouse_position.read().await,
                                        ))?;
                                    }
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    None => return Ok(()),
                }
            }
        }
    }

    pub async fn input(&mut self, input: Input) -> Result<()> {
        match &input {
            Input::Button(_) => self.sender.send(input).await?,
            Input::Move(motion) => {
                if let Motion::MouseCursor(position) = motion {
                    *self.mouse_position.write().await = CanvasCoordinate {
                        x: position[0] - CANVAS_TOP_LEFT.x,
                        y: position[1] - CANVAS_TOP_LEFT.y,
                    };
                }
            }
            _ => {}
        }
        Ok(())
    }
}
