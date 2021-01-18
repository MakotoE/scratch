use super::*;
use crate::blocks::BlockInfo;
use crate::broadcaster::{BroadcastMsg, Broadcaster, LayerChange, Stop};
use crate::coordinate::SpriteRectangle;
use crate::file::{ScratchFile, Target};
use crate::runtime::Global;
use crate::sprite::{Sprite, SpriteID};
use crate::thread::BlockInputs;
use crate::traced_rwlock::TracedRwLock;
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use std::collections::HashSet;
use std::thread::sleep;
use tokio::spawn;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug)]
pub struct VM {
    control_sender: mpsc::Sender<Control>,
    broadcaster: Broadcaster,
}

impl VM {
    pub async fn start(
        scratch_file: ScratchFile,
        debug_sender: mpsc::Sender<DebugInfo>,
        broadcaster: Broadcaster,
    ) -> Result<Self> {
        let global = Rc::new(Global::new(
            &scratch_file.project.targets[0].variables,
            &scratch_file.project.monitors,
            broadcaster.clone(),
        ));

        let (control_sender, control_receiver) = mpsc::channel(1);

        // spawn({
        //     let global = global.clone();
        //
        //     let control_receiver = ControlReceiverCell::new(control_receiver);
        //     let broadcaster = BroadcastCell::new(broadcaster);
        //
        //     async move {
        //         loop {
        //             let sprites = match VM::sprites(&scratch_file, global.clone()).await {
        //                 Ok(s) => s,
        //                 Err(e) => {
        //                     log::error!("{}", e);
        //                     break;
        //                 }
        //             };
        //             let sprites =
        //                 SpritesCell::new(sprites, &scratch_file.project.targets, global.clone());
        //
        //             match VM::run(&sprites, &control_receiver, &debug_sender, &broadcaster).await {
        //                 Ok(l) => match l {
        //                     Loop::Restart => continue,
        //                     Loop::Break => break,
        //                 },
        //                 Err(e) => log::error!("{}", e),
        //             }
        //         }
        //     }
        // });

        Ok(Self {
            control_sender,
            broadcaster: global.broadcaster.clone(),
        })
    }

    pub async fn block_inputs(
        scratch_file: &ScratchFile,
    ) -> Result<HashMap<SpriteID, Vec<BlockInputs>>> {
        let global = Rc::new(Global::new(
            &scratch_file.project.targets[0].variables,
            &scratch_file.project.monitors,
            Broadcaster::new(),
        ));
        Ok(VM::sprites(scratch_file, global)
            .await?
            .iter()
            .map(|(id, sprite)| (*id, sprite.block_inputs()))
            .collect())
    }

    async fn sprites(
        scratch_file: &ScratchFile,
        global: Rc<Global>,
    ) -> Result<HashMap<SpriteID, Sprite>> {
        let images = Rc::new(scratch_file.images.clone());

        let mut futures = FuturesUnordered::new();
        for target in &scratch_file.project.targets {
            futures.push(Sprite::new(
                global.clone(),
                Rc::new(target.clone()),
                images.clone(),
                false,
            ));
        }

        let mut sprites: HashMap<SpriteID, Sprite> =
            HashMap::with_capacity(scratch_file.project.targets.len());
        loop {
            match futures.next().await {
                Some(r) => {
                    let sprite = r?;
                    sprites.insert(sprite.0, sprite.1);
                }
                None => return Ok(sprites),
            }
        }
    }

    async fn run(
        sprites: &SpritesCell,
        control_chan: &ControlReceiverCell,
        debug_sender: &mpsc::Sender<DebugInfo>,
        broadcaster: &BroadcastCell,
    ) -> Result<Loop> {
        todo!()
        // const REDRAW_INTERVAL: Duration = Duration::from_millis(33);
        //
        // let canvas_context = &CanvasContext::new(ctx);
        //
        // let mut last_redraw = 0.0;
        //
        // let hidden_canvas = new_hidden_canvas();
        // let hidden_context = CanvasContext::new(&hidden_canvas);
        //
        // let mut futures: FuturesUnordered<LocalBoxFuture<Event>> = FuturesUnordered::new();
        // futures.push(Box::pin(control_chan.recv()));
        // futures.push(Box::pin(
        //     sleep(REDRAW_INTERVAL).await.map(|_| Event::Redraw),
        // ));
        // futures.push(Box::pin(broadcaster.recv()));
        //
        // let mut paused_threads: Vec<ThreadID> = Vec::new();
        // for thread_id in sprites.all_thread_ids().await {
        //     paused_threads.push(thread_id);
        //     debug_sender
        //         .send(DebugInfo {
        //             thread_id,
        //             block_info: sprites.block_info(thread_id).await,
        //         })
        //         .await?;
        // }
        //
        // let mut current_state = Control::Pause;
        //
        // loop {
        //     // Not having this causes unresponsive UI
        //     if performance.now() - last_redraw > REDRAW_INTERVAL_MILLIS {
        //         sprites.redraw(canvas_context).await?;
        //         sleep(Duration::from_secs(0)).await; // Yield to render
        //         last_redraw = performance.now();
        //     }
        //
        //     match futures.next().await.unwrap() {
        //         Event::None => {}
        //         Event::Thread(thread_id) => match current_state {
        //             Control::Continue => futures.push(Box::pin(sprites.step(thread_id))),
        //             Control::Step | Control::Pause => {
        //                 paused_threads.push(thread_id);
        //                 debug_sender
        //                     .send(DebugInfo {
        //                         thread_id,
        //                         block_info: sprites.block_info(thread_id).await,
        //                     })
        //                     .await?;
        //                 current_state = Control::Pause;
        //             }
        //             _ => unreachable!(),
        //         },
        //         Event::Err(e) => return Err(e),
        //         Event::Control(control) => {
        //             if let Some(c) = control {
        //                 log::info!("control: {:?}", &c);
        //                 current_state = c;
        //                 match c {
        //                     Control::Continue | Control::Step => {
        //                         for thread_id in paused_threads.drain(..) {
        //                             futures.push(Box::pin(sprites.step(thread_id)));
        //                         }
        //                     }
        //                     Control::Stop => return Ok(Loop::Restart),
        //                     Control::Drop => return Ok(Loop::Break),
        //                     Control::Pause => {}
        //                 }
        //             }
        //             futures.push(Box::pin(control_chan.recv()));
        //         }
        //         Event::Redraw => {
        //             sprites.redraw(canvas_context).await?;
        //             sleep(Duration::from_secs(0)).await; // Yield to render
        //             last_redraw = performance.now();
        //             futures.push(Box::pin(
        //                 sleep(REDRAW_INTERVAL).await.map(|_| Event::Redraw),
        //             ));
        //         }
        //         Event::BroadcastMsg(msg) => {
        //             log::info!("broadcast: {:?}", &msg);
        //             match msg {
        //                 BroadcastMsg::Clone(from_sprite) => {
        //                     let new_sprite_id = sprites.clone_sprite(from_sprite).await?;
        //                     for thread_id in 0..sprites.number_of_threads(new_sprite_id).await {
        //                         let id = ThreadID {
        //                             sprite_id: new_sprite_id,
        //                             thread_id,
        //                         };
        //                         match current_state {
        //                             Control::Continue | Control::Step => {
        //                                 futures.push(Box::pin(sprites.step(id)))
        //                             }
        //                             Control::Pause => paused_threads.push(id),
        //                             _ => unreachable!(),
        //                         }
        //                     }
        //                 }
        //                 BroadcastMsg::DeleteClone(sprite_id) => {
        //                     sprites.remove(sprite_id);
        //                     sprites.force_redraw(canvas_context).await?;
        //                     sleep(Duration::from_secs(0)).await;
        //                     last_redraw = performance.now();
        //                 }
        //                 BroadcastMsg::Stop(s) => match s {
        //                     Stop::All => {
        //                         for thread_id in sprites.all_thread_ids().await {
        //                             sprites.stop(thread_id);
        //                         }
        //                     }
        //                     Stop::ThisThread(thread_id) => {
        //                         sprites.stop(thread_id);
        //                     }
        //                     Stop::OtherThreads(thread_id) => {
        //                         for id in sprites.all_thread_ids().await {
        //                             if id.sprite_id == thread_id.sprite_id
        //                                 && id.thread_id != thread_id.thread_id
        //                             {
        //                                 sprites.stop(thread_id);
        //                             }
        //                         }
        //                     }
        //                 },
        //                 BroadcastMsg::ChangeLayer { sprite, action } => {
        //                     sprites.change_layer(sprite, action)?;
        //                 }
        //                 BroadcastMsg::RequestSpriteRectangle(sprite_id) => {
        //                     let rectangle = sprites.sprite_rectangle(&sprite_id).await?;
        //                     broadcaster.send(BroadcastMsg::SpriteRectangle {
        //                         sprite: sprite_id,
        //                         rectangle,
        //                     })?;
        //                 }
        //                 BroadcastMsg::RequestCanvasImage(sprite_id) => {
        //                     sprites
        //                         .draw_without_sprite(&hidden_context, &sprite_id)
        //                         .await?;
        //                     broadcaster.send(BroadcastMsg::CanvasImage(CanvasImage {
        //                         image: hidden_context.get_image_data()?,
        //                     }))?;
        //                 }
        //                 _ => {}
        //             }
        //             futures.push(Box::pin(broadcaster.recv()));
        //         }
        //     };
        // }
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

    pub async fn stop(&self) {
        self.control_sender.send(Control::Stop).await.unwrap();
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        let _ = self.control_sender.blocking_send(Control::Drop);
    }
}

#[derive(Debug, Copy, Clone)]
enum Control {
    Continue,
    Pause,
    Step,
    Stop,
    Drop,
}

#[derive(Debug)]
enum Event {
    None,
    Thread(ThreadID),
    Err(Error),
    Control(Option<Control>),
    Redraw,
    BroadcastMsg(BroadcastMsg),
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
    broadcaster: Broadcaster,
    receiver: RefCell<broadcast::Receiver<BroadcastMsg>>,
}

impl BroadcastCell {
    fn new(broadcaster: Broadcaster) -> Self {
        Self {
            receiver: RefCell::new(broadcaster.subscribe()),
            broadcaster,
        }
    }

    async fn recv(&self) -> Event {
        match self.receiver.borrow_mut().recv().await {
            Ok(msg) => Event::BroadcastMsg(msg),
            Err(e) => Event::Err(Error::msg(e)),
        }
    }

    fn send(&self, msg: BroadcastMsg) -> Result<()> {
        self.broadcaster.send(msg)
    }
}

#[derive(Debug)]
struct SpritesCell {
    sprites: TracedRwLock<HashMap<SpriteID, Sprite>>,
    draw_order: RefCell<DrawOrder>,
    removed_sprites: RefCell<HashSet<SpriteID>>,
    stopped_threads: RefCell<HashSet<ThreadID>>,
    global: Rc<Global>,
}

impl SpritesCell {
    fn new(sprites: HashMap<SpriteID, Sprite>, targets: &[Target], global: Rc<Global>) -> Self {
        Self {
            sprites: TracedRwLock::new(sprites),
            draw_order: RefCell::new(DrawOrder::new(targets)),
            removed_sprites: RefCell::default(),
            stopped_threads: RefCell::default(),
            global,
        }
    }

    async fn step(&self, thread_id: ThreadID) -> Event {
        if self.stopped_threads.borrow_mut().remove(&thread_id)
            || self.removed_sprites.borrow().contains(&thread_id.sprite_id)
        {
            return Event::None;
        }

        match self
            .sprites
            .read(file!(), line!())
            .await
            .get(&thread_id.sprite_id)
        {
            Some(sprite) => match sprite.step(thread_id.thread_id).await {
                Ok(_) => Event::Thread(thread_id),
                Err(e) => Event::Err(e),
            },
            None => Event::None,
        }
    }

    fn remove(&self, sprite_id: SpriteID) {
        self.removed_sprites.borrow_mut().insert(sprite_id);
    }

    // async fn redraw(&self, context: &CanvasContext<'_>) -> Result<()> {
    //     let mut need_redraw = false;
    //     if self.global.need_redraw() {
    //         need_redraw = true;
    //     } else {
    //         for sprite in self.sprites.read(file!(), line!()).await.values() {
    //             if sprite.need_redraw().await {
    //                 need_redraw = true;
    //                 break;
    //             }
    //         }
    //     }
    //
    //     if !need_redraw {
    //         return Ok(());
    //     }
    //
    //     self.force_redraw(&context).await
    // }

    // async fn force_redraw(&self, context: &CanvasContext<'_>) -> Result<()> {
    //     context.clear();
    //
    //     self.global.redraw(context).await?;
    //     let sprites = self.sprites.read(file!(), line!()).await;
    //     let removed_sprites = self.removed_sprites.borrow();
    //     for id in self.draw_order.borrow().iter() {
    //         if !removed_sprites.contains(id) {
    //             match sprites.get(id) {
    //                 Some(s) => s.redraw(context).await?,
    //                 None => return Err(Error::msg(format!("id not found: {}", id))),
    //             }
    //         }
    //     }
    //     Ok(())
    // }

    // async fn draw_without_sprite(
    //     &self,
    //     context: &CanvasContext<'_>,
    //     removed_sprite: &SpriteID,
    // ) -> Result<()> {
    //     context.clear();
    //     let sprites = self.sprites.read(file!(), line!()).await;
    //     let removed_sprites = self.removed_sprites.borrow();
    //     for id in self.draw_order.borrow().iter() {
    //         if !removed_sprites.contains(id) && id != removed_sprite {
    //             match sprites.get(id) {
    //                 Some(s) => s.redraw(context).await?,
    //                 None => return Err(Error::msg(format!("id not found: {}", id))),
    //             }
    //         }
    //     }
    //     Ok(())
    // }

    async fn all_thread_ids(&self) -> Vec<ThreadID> {
        let mut result: Vec<ThreadID> = Vec::new();
        for (sprite_id, sprite) in self.sprites.read(file!(), line!()).await.iter() {
            for thread_id in 0..sprite.number_of_threads() {
                result.push(ThreadID {
                    sprite_id: *sprite_id,
                    thread_id,
                });
            }
        }
        result
    }

    async fn block_info(&self, thread_id: ThreadID) -> BlockInfo {
        self.sprites
            .read(file!(), line!())
            .await
            .get(&thread_id.sprite_id)
            .unwrap()
            .block_info(thread_id.thread_id)
    }

    async fn clone_sprite(&self, sprite_id: SpriteID) -> Result<SpriteID> {
        let new_sprite_id = {
            let mut sprites = self.sprites.write(file!(), line!()).await;
            let (new_sprite_id, new_sprite) = match sprites.get(&sprite_id) {
                Some(sprite) => sprite.clone_sprite().await?,
                None => return Err(Error::msg("sprite_id is invalid")),
            };
            sprites.insert(new_sprite_id, new_sprite);
            new_sprite_id
        };

        let mut draw_order = self.draw_order.borrow_mut();
        let index = draw_order.iter().position(|s| s == &sprite_id).unwrap();
        draw_order.insert(index + 1, new_sprite_id);

        Ok(new_sprite_id)
    }

    async fn number_of_threads(&self, sprite_id: SpriteID) -> usize {
        self.sprites
            .read(file!(), line!())
            .await
            .get(&sprite_id)
            .unwrap()
            .number_of_threads()
    }

    fn stop(&self, thread_id: ThreadID) {
        self.stopped_threads.borrow_mut().insert(thread_id);
    }

    fn change_layer(&self, id: SpriteID, change: LayerChange) -> Result<()> {
        self.draw_order.borrow_mut().change_layer(id, change)
    }

    async fn sprite_rectangle(&self, id: &SpriteID) -> Result<SpriteRectangle> {
        match self.sprites.read(file!(), line!()).await.get(id) {
            Some(sprite) => Ok(sprite.rectangle().await),
            None => Err(Error::msg(format!("id not found: {}", id))),
        }
    }
}

#[derive(Debug)]
struct DrawOrder {
    /// Lowest index = back, highest index = Front
    ids: Vec<SpriteID>,
}

impl DrawOrder {
    fn new(targets: &[Target]) -> Self {
        let mut id_layer_order: Vec<(SpriteID, usize)> = targets
            .iter()
            .map(|t| (SpriteID::from_sprite_name(&t.name), t.layer_order))
            .collect();

        id_layer_order.sort_unstable_by(|a, b| a.1.cmp(&b.1));

        Self {
            ids: id_layer_order.iter().map(|i| i.0).collect(),
        }
    }

    fn iter(&self) -> std::slice::Iter<SpriteID> {
        self.ids.iter()
    }

    fn change_layer(&mut self, id: SpriteID, change: LayerChange) -> Result<()> {
        match self.ids.iter().position(|sprite_id| sprite_id == &id) {
            Some(index) => self.ids.remove(index),
            None => return Err(Error::msg(format!("id not found: {}", id))),
        };

        match change {
            LayerChange::Front => self.ids.push(id),
            LayerChange::Back => self.ids.insert(0, id),
            LayerChange::ChangeBy(_) => unimplemented!(),
        }
        Ok(())
    }

    fn insert(&mut self, index: usize, id: SpriteID) {
        self.ids.insert(index, id)
    }
}

#[derive(Debug)]
enum Loop {
    Restart,
    Break,
}
