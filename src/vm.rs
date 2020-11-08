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
use std::iter::FromIterator;
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

    pub async fn block_inputs(
        scratch_file: &ScratchFile,
    ) -> Result<HashMap<u64, Vec<BlockInputs>>> {
        Ok(HashMap::from_iter(
            VM::sprites(scratch_file)
                .await?
                .0
                .iter()
                .map(|(id, sprite)| (*id, sprite.block_inputs())),
        ))
    }

    async fn sprites(scratch_file: &ScratchFile) -> Result<(HashMap<u64, Sprite>, Broadcaster)> {
        let global = Global::new(&scratch_file.project.targets[0].variables);

        // TODO FuturesUnbound
        let mut sprites: HashMap<u64, Sprite> =
            HashMap::with_capacity(scratch_file.project.targets.len() - 1);
        for target in &scratch_file.project.targets[1..] {
            let sprite = Sprite::new(
                global.clone(),
                target.clone(),
                scratch_file.images.clone(),
                false,
            )
            .await?;
            sprites.insert(sprite.0, sprite.1);
        }

        Ok((sprites, global.broadcaster))
    }

    async fn redraw(
        sprites: &HashMap<u64, Sprite>,
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<()> {
        let mut need_redraw = false;
        for (_, sprite) in sprites.iter() {
            if sprite.need_redraw().await {
                need_redraw = true;
                break;
            }
        }

        if !need_redraw {
            return Ok(());
        }

        VM::force_redraw(sprites, context).await
    }

    async fn force_redraw(
        sprites: &HashMap<u64, Sprite>,
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<()> {
        context.reset_transform().unwrap();
        context.scale(2.0, 2.0).unwrap();
        context.clear_rect(0.0, 0.0, 480.0, 360.0);

        for (_, sprite) in sprites {
            sprite.redraw(&context).await?;
        }
        Ok(())
    }

    async fn run(
        sprites_map: HashMap<u64, Sprite>,
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
        let sprites = SpritesCell::new(sprites_map);
        let mut futures: FuturesUnordered<LocalBoxFuture<Event>> = FuturesUnordered::new();
        futures.push(Box::pin(control_chan.recv()));
        futures.push(Box::pin(
            TimeoutFuture::new(REDRAW_INTERVAL_MILLIS as u32).map(|_| Event::Redraw),
        ));
        futures.push(Box::pin(broadcaster_recv.recv()));

        let mut paused_threads: Vec<ThreadID> = Vec::new();
        for (sprite_id, sprite) in sprites.sprites().iter() {
            for thread_id in 0..sprite.number_of_threads() {
                let id = ThreadID {
                    sprite_id: sprite_id.clone(),
                    thread_id,
                };
                paused_threads.push(id.clone());

                debug_sender
                    .send(DebugInfo {
                        thread_id: id,
                        block_info: sprites
                            .sprites()
                            .get(sprite_id)
                            .unwrap()
                            .block_info(thread_id),
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
                        paused_threads.push(thread_id.clone());
                        debug_sender
                            .send(DebugInfo {
                                thread_id: thread_id.clone(),
                                block_info: sprites
                                    .sprites()
                                    .get(&thread_id.sprite_id)
                                    .unwrap()
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
                    let (sprite_id, new_sprite) = sprites
                        .sprites()
                        .get(&from_sprite)
                        .unwrap()
                        .clone_sprite()
                        .await?;
                    for thread_id in 0..new_sprite.number_of_threads() {
                        let id = ThreadID {
                            sprite_id: sprite_id.clone(),
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
                    sprites.insert(sprite_id, new_sprite);
                    futures.push(Box::pin(broadcaster_recv.recv()));
                }
                Event::DeleteClone(sprite_id) => {
                    sprites.remove(sprite_id);
                    VM::force_redraw(&sprites.sprites(), context).await?;
                    TimeoutFuture::new(0).await;
                    last_redraw = performance.now();
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
    Clone(u64),
    DeleteClone(u64),
}

#[derive(Debug, Copy, Clone)]
pub struct ThreadID {
    pub sprite_id: u64,
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
    sprites: RefCell<HashMap<u64, Sprite>>,
}

impl SpritesCell {
    fn new(sprites: HashMap<u64, Sprite>) -> Self {
        Self {
            sprites: RefCell::new(sprites),
        }
    }

    fn sprites(&self) -> Ref<HashMap<u64, Sprite>> {
        self.sprites.borrow()
    }

    async fn step(&self, thread_id: ThreadID) -> Event {
        match self.sprites.borrow().get(&thread_id.sprite_id) {
            Some(sprite) => match sprite.step(thread_id.thread_id).await {
                Ok(_) => Event::Thread(thread_id),
                Err(e) => Event::Err(e),
            },
            None => Event::None,
        }
    }

    fn insert(&self, sprite_id: u64, sprite: Sprite) {
        self.sprites.borrow_mut().insert(sprite_id, sprite);
    }

    fn remove(&self, sprite_id: u64) {
        self.sprites.borrow_mut().remove(&sprite_id);
    }
}
