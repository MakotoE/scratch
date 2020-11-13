use super::*;
use blocks::{BlockInfo, BlockInputs};
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use runtime::{BroadcastMsg, Broadcaster, Global, Stop};
use savefile::ScratchFile;
use sprite::{Sprite, SpriteID};
use sprite_runtime::Coordinate;
use std::collections::HashSet;
use std::iter::FromIterator;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug)]
pub struct VM {
    control_sender: mpsc::Sender<Control>,
    broadcaster: Broadcaster,
}

impl VM {
    pub async fn start(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: &ScratchFile,
    ) -> Result<(Self, mpsc::Receiver<DebugInfo>)> {
        let (sprites, broadcaster) = VM::sprites(scratch_file).await?;

        let (control_sender, control_receiver) = mpsc::channel(1);
        let (debug_sender, debug_receiver) = mpsc::channel(1);

        let broadcaster_clone = broadcaster.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match VM::run(
                sprites,
                control_receiver,
                &context,
                debug_sender,
                broadcaster_clone,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => log::error!("{}", e),
            }
        });

        Ok((
            Self {
                control_sender,
                broadcaster,
            },
            debug_receiver,
        ))
    }

    pub async fn block_inputs(
        scratch_file: &ScratchFile,
    ) -> Result<HashMap<SpriteID, Vec<BlockInputs>>> {
        Ok(HashMap::from_iter(
            VM::sprites(scratch_file)
                .await?
                .0
                .iter()
                .map(|(id, sprite)| (*id, sprite.block_inputs())),
        ))
    }

    async fn sprites(
        scratch_file: &ScratchFile,
    ) -> Result<(HashMap<SpriteID, Sprite>, Broadcaster)> {
        let global = Global::new(&scratch_file.project.targets[0].variables);
        let images = Rc::new(scratch_file.images.clone());

        let mut futures = FuturesUnordered::new();
        for target in &scratch_file.project.targets[1..] {
            futures.push(Sprite::new(
                global.clone(),
                Rc::new(target.clone()),
                images.clone(),
                false,
            ));
        }

        let mut sprites: HashMap<SpriteID, Sprite> =
            HashMap::with_capacity(scratch_file.project.targets.len() - 1);
        loop {
            match futures.next().await {
                Some(r) => {
                    let sprite = r?;
                    sprites.insert(sprite.0, sprite.1);
                }
                None => return Ok((sprites, global.broadcaster)),
            }
        }
    }

    async fn run(
        sprites_map: HashMap<SpriteID, Sprite>,
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
        for thread_id in sprites.all_thread_ids() {
            paused_threads.push(thread_id);
            debug_sender
                .send(DebugInfo {
                    thread_id,
                    block_info: sprites.block_info(thread_id),
                })
                .await?;
        }

        let mut current_state = Control::Pause;

        loop {
            // Not having this causes unresponsive UI
            if performance.now() - last_redraw > REDRAW_INTERVAL_MILLIS {
                sprites.redraw(context).await?;
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
                                block_info: sprites.block_info(thread_id),
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
                    sprites.redraw(context).await?;
                    TimeoutFuture::new(0).await; // Yield to render
                    last_redraw = performance.now();
                    futures.push(Box::pin(
                        TimeoutFuture::new(REDRAW_INTERVAL_MILLIS as u32).map(|_| Event::Redraw),
                    ));
                }
                Event::Clone(from_sprite) => {
                    let new_sprite_id = sprites.clone_sprite(from_sprite).await?;
                    for thread_id in 0..sprites.number_of_threads(new_sprite_id) {
                        let id = ThreadID {
                            sprite_id: new_sprite_id,
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
                    futures.push(Box::pin(broadcaster_recv.recv()));
                }
                Event::DeleteClone(sprite_id) => {
                    sprites.remove(&sprite_id);
                    sprites.force_redraw(context).await?;
                    TimeoutFuture::new(0).await;
                    last_redraw = performance.now();
                }
                Event::Stop(s) => match s {
                    Stop::All => {
                        for thread_id in sprites.all_thread_ids() {
                            sprites.stop(thread_id);
                        }
                    }
                    Stop::ThisThread(thread_id) => {
                        sprites.stop(thread_id);
                    }
                    Stop::OtherThreads(thread_id) => {
                        for id in sprites.all_thread_ids() {
                            if id.sprite_id == thread_id.sprite_id
                                && id.thread_id != thread_id.thread_id
                            {
                                sprites.stop(thread_id);
                            }
                        }
                    }
                },
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

    pub fn click(&self, coordinate: Coordinate) {
        self.broadcaster
            .send(BroadcastMsg::Click(coordinate))
            .unwrap_or_else(|e| log::error!("{}", wrap_err!(e)))
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        let _ = self.control_sender.blocking_send(Control::Stop);
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
    Clone(SpriteID),
    DeleteClone(SpriteID),
    Stop(Stop),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ThreadID {
    pub sprite_id: SpriteID,
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
                BroadcastMsg::Stop(s) => Event::Stop(s),
                _ => Event::None,
            },
            Err(e) => Event::Err(e.into()),
        }
    }
}

#[derive(Debug)]
struct SpritesCell {
    sprites: RefCell<HashMap<SpriteID, Sprite>>,
    threads_to_stop: RefCell<HashSet<ThreadID>>,
}

impl SpritesCell {
    fn new(sprites: HashMap<SpriteID, Sprite>) -> Self {
        Self {
            sprites: RefCell::new(sprites),
            threads_to_stop: RefCell::default(),
        }
    }

    async fn step(&self, thread_id: ThreadID) -> Event {
        if self.threads_to_stop.borrow_mut().remove(&thread_id) {
            return Event::None;
        }

        match self.sprites.borrow().get(&thread_id.sprite_id) {
            Some(sprite) => match sprite.step(thread_id.thread_id).await {
                Ok(_) => Event::Thread(thread_id),
                Err(e) => Event::Err(e),
            },
            None => Event::None,
        }
    }

    fn remove(&self, sprite_id: &SpriteID) {
        self.sprites.borrow().get(&sprite_id).map(|s| s.remove());
    }

    async fn redraw(&self, context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        let mut need_redraw = false;
        for sprite in self.sprites.borrow().values() {
            if sprite.need_redraw().await {
                need_redraw = true;
                break;
            }
        }

        if !need_redraw {
            return Ok(());
        }

        self.force_redraw(context).await
    }

    // TODO probably don't need this because remove() is in same scope
    async fn force_redraw(&self, context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        context.reset_transform().unwrap();
        context.scale(2.0, 2.0).unwrap();
        context.clear_rect(0.0, 0.0, 480.0, 360.0);

        let sprites = self.sprites.borrow();
        for sprite in sprites.values() {
            sprite.redraw(&context).await?;
        }
        Ok(())
    }

    fn all_thread_ids(&self) -> Vec<ThreadID> {
        let mut result: Vec<ThreadID> = Vec::new();
        for (sprite_id, sprite) in self.sprites.borrow().iter() {
            for thread_id in 0..sprite.number_of_threads() {
                result.push(ThreadID {
                    sprite_id: *sprite_id,
                    thread_id,
                });
            }
        }
        result
    }

    fn block_info(&self, thread_id: ThreadID) -> BlockInfo {
        self.sprites
            .borrow()
            .get(&thread_id.sprite_id)
            .unwrap()
            .block_info(thread_id.thread_id)
    }

    async fn clone_sprite(&self, sprite_id: SpriteID) -> Result<SpriteID> {
        let mut sprites = self.sprites.borrow_mut();
        let (new_sprite_id, new_sprite) = sprites.get(&sprite_id).unwrap().clone_sprite().await?;
        sprites.insert(new_sprite_id, new_sprite);
        Ok(new_sprite_id)
    }

    fn number_of_threads(&self, sprite_id: SpriteID) -> usize {
        self.sprites
            .borrow()
            .get(&sprite_id)
            .unwrap()
            .number_of_threads()
    }

    fn stop(&self, thread_id: ThreadID) {
        self.threads_to_stop.borrow_mut().insert(thread_id);
    }
}
