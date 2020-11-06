use super::*;
use blocks::{BlockInfo, BlockInputs};
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use runtime::{Global, Runtime};
use savefile::ScratchFile;
use sprite::Sprite;
use sprite_runtime::SpriteRuntime;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct VM {
    control_sender: Sender<Control>,
}

impl VM {
    pub async fn start(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: &ScratchFile,
    ) -> Result<(Self, Receiver<DebugInfo>)> {
        let sprites = VM::sprites(scratch_file).await?;

        let (control_sender, control_receiver) = channel(1);
        let (debug_sender, debug_receiver) = channel(1);

        wasm_bindgen_futures::spawn_local(async move {
            match VM::run(sprites, control_receiver, &context, debug_sender).await {
                Ok(_) => {}
                Err(e) => log::error!("{}", e),
            }
        });

        Ok((Self { control_sender }, debug_receiver))
    }

    pub async fn block_inputs(scratch_file: &ScratchFile) -> Result<Vec<Vec<BlockInputs>>> {
        Ok(VM::sprites(scratch_file)
            .await?
            .iter()
            .map(|s| s.block_inputs())
            .collect())
    }

    async fn sprites(scratch_file: &ScratchFile) -> Result<Vec<Sprite>> {
        let global = Global::new(&scratch_file.project.targets[0].variables);

        let mut sprites: Vec<Sprite> = Vec::with_capacity(scratch_file.project.targets[1..].len());
        for target in &scratch_file.project.targets[1..] {
            let runtime = SpriteRuntime::new(&target.costumes, &scratch_file.images).await?;

            let runtime = Runtime {
                sprite: Rc::new(RwLock::new(runtime)),
                global: global.clone(),
            };

            sprites.push(Sprite::new(runtime, target).await?);
        }

        Ok(sprites)
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
        sprites: Vec<Sprite>,
        control_chan: Receiver<Control>,
        context: &web_sys::CanvasRenderingContext2d,
        debug_sender: Sender<DebugInfo>,
    ) -> Result<()> {
        const REDRAW_INTERVAL_MILLIS: f64 = 33.0;

        let performance = web_sys::window().unwrap().performance().unwrap();

        let mut last_redraw = performance.now();

        let control_chan = ReceiverCell::new(control_chan);
        let mut futures: FuturesUnordered<LocalBoxFuture<Event>> = FuturesUnordered::new();
        futures.push(Box::pin(control_chan.recv()));
        futures.push(Box::pin(
            TimeoutFuture::new(REDRAW_INTERVAL_MILLIS as u32).map(|_| Event::Redraw),
        ));

        let mut paused_threads: Vec<ThreadID> = Vec::new();
        for (sprite_id, sprite) in sprites.iter().enumerate() {
            for thread_id in 0..sprite.number_of_threads() {
                let id = ThreadID {
                    sprite_id,
                    thread_id,
                };
                paused_threads.push(id);

                debug_sender
                    .send(DebugInfo {
                        thread_id: id,
                        block_info: sprites[sprite_id].block_info(thread_id),
                    })
                    .await?;
            }
        }

        let mut current_state = Control::Pause;

        loop {
            // Not having this causes unresponsive UI
            if performance.now() - last_redraw > REDRAW_INTERVAL_MILLIS {
                VM::redraw(&sprites, context).await?;
                TimeoutFuture::new(0).await; // Yield to render
                last_redraw = performance.now();
            }

            match futures.next().await.unwrap() {
                Event::Thread(thread_id) => match current_state {
                    Control::Continue => {
                        futures.push(VM::step_sprite(&sprites[thread_id.sprite_id], thread_id))
                    }
                    Control::Step | Control::Pause => {
                        paused_threads.push(thread_id);
                        debug_sender
                            .send(DebugInfo {
                                thread_id,
                                block_info: sprites[thread_id.sprite_id]
                                    .block_info(thread_id.thread_id),
                            })
                            .await?;
                        current_state = Control::Pause;
                    }
                    Control::Stop => unreachable!(),
                },
                Event::Error(e) => return Err(e),
                Event::Control(control) => {
                    if let Some(c) = control {
                        current_state = c;
                        match c {
                            Control::Continue | Control::Step => {
                                for thread_id in paused_threads.drain(..) {
                                    futures.push(VM::step_sprite(
                                        &sprites[thread_id.sprite_id],
                                        thread_id,
                                    ));
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
                    VM::redraw(&sprites, context).await?;
                    TimeoutFuture::new(0).await; // Yield to render
                    last_redraw = performance.now();
                    futures.push(Box::pin(
                        TimeoutFuture::new(REDRAW_INTERVAL_MILLIS as u32).map(|_| Event::Redraw),
                    ));
                }
            };
        }
    }

    fn step_sprite(sprite: &Sprite, thread_id: ThreadID) -> LocalBoxFuture<Event> {
        Box::pin(
            sprite
                .step(thread_id.thread_id)
                .map(move |result| match result {
                    Ok(_) => Event::Thread(thread_id),
                    Err(e) => Event::Error(e),
                }),
        )
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
    Thread(ThreadID),
    Error(Error),
    Control(Option<Control>),
    Redraw,
}

#[derive(Debug, Copy, Clone)]
pub struct ThreadID {
    pub sprite_id: usize,
    pub thread_id: usize,
}

/// Resolves a lifetime issue with Receiver and FuturesUnordered.
#[derive(Debug)]
struct ReceiverCell {
    receiver: RefCell<Receiver<Control>>,
}

impl ReceiverCell {
    fn new(receiver: Receiver<Control>) -> Self {
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
