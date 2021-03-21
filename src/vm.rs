use super::*;
use crate::blocks::BlockInfo;
use crate::broadcaster::{BroadcastMsg, Broadcaster, Stop};
use crate::coordinate::canvas_const;
use crate::file::ScratchFile;
use crate::runtime::Global;
use crate::sprite::{Sprite, SpriteID};
use crate::sprite_map::SpriteMap;
use crate::sprite_runtime::{Costumes, SpriteRuntime};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use graphics::Context;
use graphics_buffer::{buffer_glyphs_from_path, RenderBuffer};
use piston_window::{G2d, G2dTextureContext, Glyphs};
use std::fmt::Debug;
use tokio::select;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct VM {
    control_sender: mpsc::Sender<Control>,
    broadcaster: Broadcaster,
    vm_task: JoinHandle<()>,
    sprites: Arc<SpriteMap>,
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

        let sprite_map = Arc::new(SpriteMap::new(
            sprites,
            &scratch_file.project.targets,
            global.clone(),
        ));

        let vm_task = spawn({
            let mut control_receiver = control_receiver;
            let broadcaster = broadcaster.clone();
            let sprite_map = sprite_map.clone();

            async move {
                loop {
                    if let Err(e) =
                        VM::run(sprite_map.clone(), &mut control_receiver, &broadcaster).await
                    {
                        log::error!("{}", e);
                        std::process::exit(1);
                    }
                }
            }
        });

        Ok(Self {
            control_sender,
            broadcaster,
            vm_task,
            sprites: sprite_map,
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
            let costumes = Costumes::new(texture_context, &target.costumes, &images).await?;
            sprite.set_costumes(costumes).await;
            sprites.insert(id, sprite);
        }
        Ok(sprites)
    }

    async fn run(
        sprites: Arc<SpriteMap>,
        control_receiver: &mut mpsc::Receiver<Control>,
        broadcaster: &Broadcaster,
    ) -> Result<()> {
        let mut broadcast_receiver = broadcaster.subscribe();
        let mut futures = FuturesUnordered::new();

        let mut paused_threads: Vec<ThreadID> = Vec::new();
        for thread_id in sprites.all_thread_ids().await {
            paused_threads.push(thread_id);
            log::trace!(
                "{}",
                DebugInfo {
                    thread_id,
                    block_info: sprites.block_info(thread_id).await?,
                }
            );
        }

        let mut buffer_glyphs = buffer_glyphs_from_path("assets/Roboto-Regular.ttf")?;

        let mut current_state = Control::Pause;

        loop {
            select! {
                biased;
                c = control_receiver.recv() => {
                    if let Some(control) = c {
                        log::info!("control: {:?}", &control);
                        current_state = control;
                        match control {
                            Control::Continue | Control::Step => {
                                for thread_id in paused_threads.drain(..) {
                                    futures.push(sprites.step(thread_id));
                                }
                            }
                            Control::Stop => return Ok(()),
                            Control::Pause => {}
                        }
                    }
                },
                recv_result = broadcast_receiver.recv() => {
                    match recv_result {
                        Ok(msg) => {
                            log::info!("broadcast: {:?}", BroadcastMsgDebug(&msg));
                            match msg {
                                BroadcastMsg::Clone(from_sprite) => {
                                    let new_sprite_id = sprites.clone_sprite(from_sprite).await?;
                                    for thread_id in 0..sprites.number_of_threads(&new_sprite_id).await? {
                                        let id = ThreadID {
                                            sprite_id: new_sprite_id,
                                            thread_id,
                                        };
                                        match current_state {
                                            Control::Continue | Control::Step => {
                                                futures.push(sprites.step(id))
                                            }
                                            Control::Pause => paused_threads.push(id),
                                            _ => unreachable!(),
                                        }
                                    }
                                }
                                BroadcastMsg::DeleteClone(sprite_id) => {
                                    sprites.remove(sprite_id).await;
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
                                    let mut render_buffer =
                                        RenderBuffer::new(canvas_const::X_MAX as u32, canvas_const::Y_MAX as u32);
                                    sprites
                                        .draw_to_buffer(&mut Context::new(), &mut render_buffer, &mut buffer_glyphs, &sprite_id)
                                        .await?;
                                    broadcaster.send(BroadcastMsg::CanvasImage(render_buffer))?;
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                },
                futures_result = futures.next() => {
                    if let Some(step_result) = futures_result {
                        if let Some(thread_id) = step_result? {
                            match current_state {
                                Control::Continue => futures.push(sprites.step(thread_id)),
                                Control::Step | Control::Pause => {
                                    paused_threads.push(thread_id);
                                    log::trace!(
                                        "{}",
                                        DebugInfo {
                                            thread_id,
                                            block_info: sprites.block_info(thread_id).await?,
                                        }
                                    );
                                    current_state = Control::Pause;
                                }
                                _ => unreachable!("{:?}", current_state),
                            }
                        }
                    }
                },
            }
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

    pub async fn draw(
        &mut self,
        context: &Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        self.sprites.draw(context, graphics, character_cache).await
    }
}

#[derive(Debug, Copy, Clone)]
enum Control {
    Continue,
    Pause,
    Step,
    Stop,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ThreadID {
    pub sprite_id: SpriteID,
    pub thread_id: usize,
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

struct BroadcastMsgDebug<'a>(&'a BroadcastMsg);

impl Debug for BroadcastMsgDebug<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            BroadcastMsg::CanvasImage(_) => write!(f, "CanvasImage(RgbaImage)"),
            _ => write!(f, "{:?}", self.0),
        }
    }
}
