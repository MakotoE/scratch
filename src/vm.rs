use super::*;
use blocks::{BlockInfo, BlockInputs};
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use runtime::BroadcastMsg;
use runtime::{Broadcaster, Global};
use savefile::ScratchFile;
use sprite::Sprite;
use std::cell::Ref;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug)]
pub struct VM {
    control_sender: mpsc::Sender<Control>,
}

impl VM {
    pub async fn start(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: &ScratchFile,
    ) -> Result<(Self, mpsc::Receiver<DebugInfo>)> {
        let (sprites, broadcaster) = VM::sprites(scratch_file).await?;

        let (control_sender, control_receiver) = mpsc::channel(1);
        let (debug_sender, debug_receiver) = mpsc::channel(1);

        wasm_bindgen_futures::spawn_local(async move {
            match VM::run(
                sprites,
                control_receiver,
                &context,
                debug_sender,
                broadcaster,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => log::error!("{}", e),
            }
        });

        Ok((Self { control_sender }, debug_receiver))
    }

    pub async fn block_inputs(scratch_file: &ScratchFile) -> Result<Vec<Vec<BlockInputs>>> {
        Ok(VM::sprites(scratch_file)
            .await?
            .0
            .iter()
            .map(|s| s.block_inputs())
            .collect())
    }

    async fn sprites(scratch_file: &ScratchFile) -> Result<(Vec<Sprite>, Broadcaster)> {
        let global = Global::new(&scratch_file.project.targets[0].variables);

        let mut sprites: Vec<Sprite> = Vec::with_capacity(scratch_file.project.targets[1..].len());
        for (sprite_id, target) in scratch_file.project.targets[1..].iter().enumerate() {
            sprites.push(
                Sprite::new(
                    global.clone(),
                    target.clone(),
                    scratch_file.images.clone(),
                    sprite_id,
                    false,
                )
                .await?,
            );
        }

        Ok((sprites, global.broadcaster))
    }

    async fn redraw(sprites: &[Sprite], context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        let mut need_redraw = false;
        for sprite in sprites.iter() {
            if sprite.need_redraw().await {
                need_redraw = true;
                break;
            }
        }

        if !need_redraw {
            return Ok(());
        }

        context.reset_transform().unwrap();
        context.scale(2.0, 2.0).unwrap();
        context.clear_rect(0.0, 0.0, 480.0, 360.0);

        for sprite in sprites {
            sprite.redraw(&context).await?;
        }
        Ok(())
    }

    async fn run(
        sprites_vec: Vec<Sprite>,
        control_chan: mpsc::Receiver<Control>,
        context: &web_sys::CanvasRenderingContext2d,
        debug_sender: mpsc::Sender<DebugInfo>,
        broadcaster: Broadcaster,
    ) -> Result<()> {
        const REDRAW_INTERVAL_MILLIS: f64 = 33.0;

        let performance = web_sys::window().unwrap().performance().unwrap();

        let mut last_redraw = performance.now();

        let control_chan = ControlReceiverCell::new(control_chan);
        let broadcaster_recv = BroadcastCell::new(broadcaster);
        let sprites = SpritesCell::new(sprites_vec);
        let mut futures: FuturesUnordered<LocalBoxFuture<Event>> = FuturesUnordered::new();
        futures.push(Box::pin(control_chan.recv()));
        futures.push(Box::pin(
            TimeoutFuture::new(REDRAW_INTERVAL_MILLIS as u32).map(|_| Event::Redraw),
        ));
        futures.push(Box::pin(broadcaster_recv.recv()));

        let mut paused_threads: Vec<ThreadID> = Vec::new();
        for (sprite_id, sprite) in sprites.sprites().iter().enumerate() {
            for thread_id in 0..sprite.number_of_threads() {
                let id = ThreadID {
                    sprite_id,
                    thread_id,
                };
                paused_threads.push(id);

                debug_sender
                    .send(DebugInfo {
                        thread_id: id,
                        block_info: sprites.sprites()[sprite_id].block_info(thread_id),
                    })
                    .await?;
            }
        }

        let mut current_state = Control::Pause;

        loop {
            // Not having this causes unresponsive UI
            if performance.now() - last_redraw > REDRAW_INTERVAL_MILLIS {
                VM::redraw(&sprites.sprites(), context).await?;
                TimeoutFuture::new(0).await; // Yield to render
                last_redraw = performance.now();
            }

            match futures.next().await.unwrap() {
                Event::None => {}
                Event::Thread(thread_id) => match current_state {
                    Control::Continue => futures.push(Box::pin(sprites.step(thread_id))),
                    Control::Step | Control::Pause => {
                        paused_threads.push(thread_id);
                        debug_sender
                            .send(DebugInfo {
                                thread_id,
                                block_info: sprites.sprites()[thread_id.sprite_id]
                                    .block_info(thread_id.thread_id),
                            })
                            .await?;
                        current_state = Control::Pause;
                    }
                    Control::Stop => unreachable!(),
                },
                Event::Err(e) => return Err(e),
                Event::Control(control) => {
                    if let Some(c) = control {
                        current_state = c;
                        match c {
                            Control::Continue | Control::Step => {
                                for thread_id in paused_threads.drain(..) {
                                    futures.push(Box::pin(sprites.step(thread_id)));
                                }
                            }
                            Control::Stop => return Ok(()),
                            Control::Pause => {}
                        }
                        log::info!("control: {:?}", c);
                    }
                    futures.push(Box::pin(control_chan.recv()));
                }
                Event::Redraw => {
                    VM::redraw(&sprites.sprites(), context).await?;
                    TimeoutFuture::new(0).await; // Yield to render
                    last_redraw = performance.now();
                    futures.push(Box::pin(
                        TimeoutFuture::new(REDRAW_INTERVAL_MILLIS as u32).map(|_| Event::Redraw),
                    ));
                }
                Event::Clone(from_sprite) => {
                    let sprite_id = sprites.sprites().len();
                    let new_sprite = sprites.sprites()[from_sprite]
                        .clone_sprite(sprite_id)
                        .await?;
                    for thread_id in 0..new_sprite.number_of_threads() {
                        let id = ThreadID {
                            sprite_id,
                            thread_id,
                        };

                        match current_state {
                            Control::Continue | Control::Step => {
                                futures.push(Box::pin(sprites.step(id)))
                            }
                            Control::Pause => paused_threads.push(id),
                            Control::Stop => unreachable!(),
                        }
                    }
                    sprites.push(new_sprite);
                    futures.push(Box::pin(broadcaster_recv.recv()));
                }
                Event::DeleteClone(sprite_id) => {
                    sprites.remove(sprite_id);
                }
            };
        }
    }

    pub async fn continue_(&self) {
        self.control_sender.send(Control::Continue).await.unwrap();
    }

    pub async fn pause(&self) {
        self.control_sender.send(Control::Pause).await.unwrap();
    }

    pub async fn step(&self) {
        self.control_sender.send(Control::Step).await.unwrap();
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.control_sender.blocking_send(Control::Stop).unwrap();
    }
}

#[derive(Debug, Copy, Clone)]
enum Control {
    Continue,
    Pause,
    Step,
    Stop,
}

#[derive(Debug)]
enum Event {
    None,
    Thread(ThreadID),
    Err(Error),
    Control(Option<Control>),
    Redraw,
    Clone(usize),
    DeleteClone(usize),
}

#[derive(Debug, Copy, Clone)]
pub struct ThreadID {
    pub sprite_id: usize,
    pub thread_id: usize,
}

/// Resolves a lifetime issue with Receiver and FuturesUnordered.
#[derive(Debug)]
struct ControlReceiverCell {
    receiver: RefCell<mpsc::Receiver<Control>>,
}

impl ControlReceiverCell {
    fn new(receiver: mpsc::Receiver<Control>) -> Self {
        Self {
            receiver: RefCell::new(receiver),
        }
    }

    async fn recv(&self) -> Event {
        Event::Control(self.receiver.borrow_mut().recv().await)
    }
}

#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub thread_id: ThreadID,
    pub block_info: BlockInfo,
}

#[derive(Debug)]
struct BroadcastCell {
    receiver: RefCell<broadcast::Receiver<BroadcastMsg>>,
}

impl BroadcastCell {
    fn new(broadcaster: Broadcaster) -> Self {
        Self {
            receiver: RefCell::new(broadcaster.subscribe()),
        }
    }

    async fn recv(&self) -> Event {
        match self.receiver.borrow_mut().recv().await {
            Ok(msg) => match msg {
                BroadcastMsg::Clone(id) => Event::Clone(id),
                BroadcastMsg::DeleteClone(id) => Event::DeleteClone(id),
                _ => Event::None,
            },
            Err(e) => Event::Err(e.into()),
        }
    }
}

#[derive(Debug)]
struct SpritesCell {
    sprites: RefCell<Vec<Sprite>>,
}

impl SpritesCell {
    fn new(sprites: Vec<Sprite>) -> Self {
        Self {
            sprites: RefCell::new(sprites),
        }
    }

    fn sprites(&self) -> Ref<Vec<Sprite>> {
        self.sprites.borrow()
    }

    async fn step(&self, thread_id: ThreadID) -> Event {
        // TODO out of bounds due to removed sprite
        let result = self.sprites.borrow()[thread_id.sprite_id]
            .step(thread_id.thread_id)
            .await;
        match result {
            Ok(_) => Event::Thread(thread_id),
            Err(e) => Event::Err(e),
        }
    }

    fn push(&self, sprite: Sprite) {
        self.sprites.borrow_mut().push(sprite)
    }

    fn remove(&self, sprite_id: usize) {
        self.sprites.borrow_mut().remove(sprite_id);
    }
}
