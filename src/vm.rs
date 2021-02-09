use super::*;
use crate::blocks::BlockInfo;
use crate::broadcaster::{BroadcastMsg, Broadcaster, LayerChange, Stop};
use crate::coordinate::{canvas_const, SpriteRectangle};
use crate::file::{ScratchFile, Target};
use crate::runtime::Global;
use crate::sprite::{Sprite, SpriteID};
use crate::sprite_runtime::SpriteRuntime;
use crate::thread::BlockInputs;
use futures::future::{BoxFuture, LocalBoxFuture};
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use graphics::math::Matrix2d;
use graphics::{rectangle, Context, DrawState};
use piston_window::{G2d, G2dTextureContext, Glyphs};
use std::borrow::Borrow;
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug)]
pub struct VM {
    control_sender: mpsc::Sender<Control>,
    broadcaster: Broadcaster,
    vm_task: JoinHandle<()>,
    sprites: Arc<SpritesCell>,
}

impl VM {
    pub async fn new(
        texture_context: &mut G2dTextureContext,
        scratch_file: ScratchFile,
        broadcaster: Broadcaster,
    ) -> Result<Self> {
        let (control_sender, control_receiver) = mpsc::channel(1);

        let global = Arc::new(Global::new(
            &scratch_file.project.targets[0].variables,
            &scratch_file.project.monitors,
            broadcaster.clone(),
        ));

        let sprites = VM::sprites(texture_context, &scratch_file, global.clone()).await?;

        let sprites_cell = Arc::new(SpritesCell::new(
            sprites,
            &scratch_file.project.targets,
            global.clone(),
        ));

        let vm_task = spawn({
            let control_receiver = ControlReceiverCell::new(control_receiver);
            let broadcaster = broadcaster.clone();
            let sprites_cell = sprites_cell.clone();

            async move {
                loop {
                    match VM::run(
                        sprites_cell.clone(),
                        &control_receiver,
                        &BroadcastCell::new(broadcaster.clone()),
                    )
                    .await
                    {
                        Ok(l) => match l {
                            Loop::Restart => continue,
                            Loop::Break => break,
                        },
                        Err(e) => panic!("{:?}", e),
                    }
                }
            }
        });

        Ok(Self {
            control_sender,
            broadcaster,
            vm_task,
            sprites: sprites_cell,
        })
    }

    async fn sprites(
        texture_context: &mut G2dTextureContext,
        scratch_file: &ScratchFile,
        global: Arc<Global>,
    ) -> Result<HashMap<SpriteID, Sprite>> {
        let images = Arc::new(scratch_file.images.clone());

        let mut sprites: HashMap<SpriteID, Sprite> =
            HashMap::with_capacity(scratch_file.project.targets.len());
        for target in &scratch_file.project.targets {
            let sprite_runtime = SpriteRuntime::new(&target);
            let id = SpriteID::from_sprite_name(&target.name);
            let mut sprite =
                Sprite::new(id, sprite_runtime, global.clone(), target.clone()).await?;
            sprite
                .add_costumes(texture_context, &target.costumes, &images)
                .await?;
            sprites.insert(id, sprite);
        }
        return Ok(sprites);
    }

    async fn run(
        sprites: Arc<SpritesCell>,
        control_chan: &ControlReceiverCell,
        broadcaster: &BroadcastCell,
    ) -> Result<Loop> {
        let mut futures: FuturesUnordered<BoxFuture<Event>> = FuturesUnordered::new();
        futures.push(Box::pin(control_chan.recv()));
        futures.push(Box::pin(broadcaster.recv()));

        let mut paused_threads: Vec<ThreadID> = Vec::new();
        for thread_id in sprites.all_thread_ids().await {
            paused_threads.push(thread_id);
            log::trace!(
                "{}",
                DebugInfo {
                    thread_id,
                    block_info: sprites.block_info(thread_id).await,
                }
            );
        }

        let mut current_state = Control::Pause;

        loop {
            match futures.next().await.unwrap() {
                Event::None => {}
                Event::Thread(thread_id) => match current_state {
                    Control::Continue => futures.push(Box::pin(sprites.step(thread_id))),
                    Control::Step | Control::Pause => {
                        paused_threads.push(thread_id);
                        log::trace!(
                            "{}",
                            DebugInfo {
                                thread_id,
                                block_info: sprites.block_info(thread_id).await,
                            }
                        );
                        current_state = Control::Pause;
                    }
                    _ => unreachable!(),
                },
                Event::Err(e) => return Err(e),
                Event::Control(control) => {
                    if let Some(c) = control {
                        log::info!("control: {:?}", &c);
                        current_state = c;
                        match c {
                            Control::Continue | Control::Step => {
                                for thread_id in paused_threads.drain(..) {
                                    futures.push(Box::pin(sprites.step(thread_id)));
                                }
                            }
                            Control::Stop => return Ok(Loop::Restart),
                            Control::Drop => return Ok(Loop::Break),
                            Control::Pause => {}
                        }
                    }
                    futures.push(Box::pin(control_chan.recv()));
                }
                Event::BroadcastMsg(msg) => {
                    log::info!("broadcast: {:?}", &msg);
                    match msg {
                        BroadcastMsg::Clone(from_sprite) => {
                            let new_sprite_id = sprites.clone_sprite(from_sprite).await?;
                            for thread_id in 0..sprites.number_of_threads(new_sprite_id).await {
                                let id = ThreadID {
                                    sprite_id: new_sprite_id,
                                    thread_id,
                                };
                                match current_state {
                                    Control::Continue | Control::Step => {
                                        futures.push(Box::pin(sprites.step(id)))
                                    }
                                    Control::Pause => paused_threads.push(id),
                                    _ => unreachable!(),
                                }
                            }
                        }
                        BroadcastMsg::DeleteClone(sprite_id) => {
                            sprites.remove(sprite_id).await;
                            // sprites.force_redraw(canvas_context).await?;
                            // last_redraw = performance.now();
                        }
                        BroadcastMsg::Stop(s) => match s {
                            Stop::All => {
                                for thread_id in sprites.all_thread_ids().await {
                                    sprites.stop(thread_id).await;
                                }
                            }
                            Stop::ThisThread(thread_id) => {
                                sprites.stop(thread_id).await;
                            }
                            Stop::OtherThreads(thread_id) => {
                                for id in sprites.all_thread_ids().await {
                                    if id.sprite_id == thread_id.sprite_id
                                        && id.thread_id != thread_id.thread_id
                                    {
                                        sprites.stop(thread_id).await;
                                    }
                                }
                            }
                        },
                        BroadcastMsg::ChangeLayer { sprite, action } => {
                            sprites.change_layer(sprite, action).await?;
                        }
                        BroadcastMsg::RequestSpriteRectangle(sprite_id) => {
                            let rectangle = sprites.sprite_rectangle(&sprite_id).await?;
                            broadcaster.send(BroadcastMsg::SpriteRectangle {
                                sprite: sprite_id,
                                rectangle,
                            })?;
                        }
                        BroadcastMsg::RequestCanvasImage(sprite_id) => {
                            // sprites
                            //     .draw_without_sprite(&hidden_context, &sprite_id)
                            //     .await?;
                            // broadcaster.send(BroadcastMsg::CanvasImage(CanvasImage {
                            //     image: hidden_context.get_image_data()?,
                            // }))?;
                        }
                        _ => {}
                    }
                    futures.push(Box::pin(broadcaster.recv()));
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

    pub async fn stop(&self) {
        self.control_sender.send(Control::Stop).await.unwrap();
    }

    pub async fn redraw(
        &mut self,
        context: &mut Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        self.sprites
            .redraw(context, graphics, character_cache)
            .await
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        self.control_sender.try_send(Control::Drop).unwrap();
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
    receiver: RwLock<mpsc::Receiver<Control>>,
}

impl ControlReceiverCell {
    fn new(receiver: mpsc::Receiver<Control>) -> Self {
        Self {
            receiver: RwLock::new(receiver),
        }
    }

    async fn recv(&self) -> Event {
        Event::Control(self.receiver.write().await.recv().await)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct DebugInfo {
    pub thread_id: ThreadID,
    pub block_info: BlockInfo,
}

impl Display for DebugInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "sprite: {}, thread: {}, block name: {}, block id: {}",
            self.thread_id.sprite_id,
            self.thread_id.thread_id,
            self.block_info.name,
            self.block_info.id
        )
    }
}

#[derive(Debug)]
struct BroadcastCell {
    broadcaster: Broadcaster,
    receiver: RwLock<broadcast::Receiver<BroadcastMsg>>,
}

impl BroadcastCell {
    fn new(broadcaster: Broadcaster) -> Self {
        Self {
            receiver: RwLock::new(broadcaster.subscribe()),
            broadcaster,
        }
    }

    async fn recv(&self) -> Event {
        match self.receiver.write().await.recv().await {
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
    sprites: RwLock<HashMap<SpriteID, Sprite>>,
    draw_order: RwLock<DrawOrder>,
    removed_sprites: RwLock<HashSet<SpriteID>>,
    stopped_threads: RwLock<HashSet<ThreadID>>,
    global: Arc<Global>,
}

impl SpritesCell {
    fn new(sprites: HashMap<SpriteID, Sprite>, targets: &[Target], global: Arc<Global>) -> Self {
        Self {
            sprites: RwLock::new(sprites),
            draw_order: RwLock::new(DrawOrder::new(targets)),
            removed_sprites: RwLock::default(),
            stopped_threads: RwLock::default(),
            global,
        }
    }

    async fn step(&self, thread_id: ThreadID) -> Event {
        if self.stopped_threads.write().await.remove(&thread_id)
            || self
                .removed_sprites
                .read()
                .await
                .contains(&thread_id.sprite_id)
        {
            return Event::None;
        }

        match self.sprites.read().await.get(&thread_id.sprite_id) {
            Some(sprite) => match sprite.step(thread_id.thread_id).await {
                Ok(_) => Event::Thread(thread_id),
                Err(e) => Event::Err(e),
            },
            None => Event::None,
        }
    }

    async fn remove(&self, sprite_id: SpriteID) {
        self.removed_sprites.write().await.insert(sprite_id);
    }

    async fn redraw(
        &self,
        context: &mut Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        let mut need_redraw = false;
        if self.global.need_redraw() {
            need_redraw = true;
        } else {
            for sprite in self.sprites.read().await.values() {
                if sprite.need_redraw().await {
                    need_redraw = true;
                    break;
                }
            }
        }

        if !need_redraw {
            return Ok(());
        }

        self.force_redraw(context, graphics, character_cache).await
    }

    async fn force_redraw(
        &self,
        context: &mut Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        self.global
            .redraw(context, graphics, character_cache)
            .await?;
        let sprites = self.sprites.read().await;
        let removed_sprites = self.removed_sprites.read().await;
        for id in self.draw_order.read().await.iter() {
            if !removed_sprites.contains(id) {
                match sprites.get(id) {
                    Some(s) => s.redraw(context, graphics, character_cache).await?,
                    None => return Err(Error::msg(format!("id not found: {}", id))),
                }
            }
        }
        Ok(())
    }

    // async fn draw_without_sprite(
    //     &self,
    //     context: &CanvasContext<'_>,
    //     removed_sprite: &SpriteID,
    // ) -> Result<()> {
    //     context.clear();
    //     let sprites = self.sprites.read().await;
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
        for (sprite_id, sprite) in self.sprites.read().await.iter() {
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
            .read()
            .await
            .get(&thread_id.sprite_id)
            .unwrap()
            .block_info(thread_id.thread_id)
            .await
    }

    async fn clone_sprite(&self, sprite_id: SpriteID) -> Result<SpriteID> {
        let new_sprite_id = {
            let mut sprites = self.sprites.write().await;
            let new_sprite_id = SpriteID::from_sprite_name(&(format!("{}", sprite_id) + "clone"));
            let new_sprite = match sprites.get(&sprite_id) {
                Some(sprite) => sprite.clone_sprite(new_sprite_id).await?,
                None => return Err(Error::msg("sprite_id is invalid")),
            };
            sprites.insert(new_sprite_id, new_sprite);
            new_sprite_id
        };

        let mut draw_order = self.draw_order.write().await;
        let index = draw_order.iter().position(|s| s == &sprite_id).unwrap();
        draw_order.insert(index + 1, new_sprite_id);

        Ok(new_sprite_id)
    }

    async fn number_of_threads(&self, sprite_id: SpriteID) -> usize {
        self.sprites
            .read()
            .await
            .get(&sprite_id)
            .unwrap()
            .number_of_threads()
    }

    async fn stop(&self, thread_id: ThreadID) {
        self.stopped_threads.write().await.insert(thread_id);
    }

    async fn change_layer(&self, id: SpriteID, change: LayerChange) -> Result<()> {
        self.draw_order.write().await.change_layer(id, change)
    }

    async fn sprite_rectangle(&self, id: &SpriteID) -> Result<SpriteRectangle> {
        match self.sprites.read().await.get(id) {
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
