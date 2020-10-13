use super::*;
use blocks::*;
use controller::DebugController;
use gloo_timers::future::TimeoutFuture;
use runtime::{Coordinate, SpriteRuntime};

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<Thread>,
    closure: ClosureRef,
    request_animation_frame_id: Rc<RefCell<i32>>,
    runtime: Rc<RwLock<SpriteRuntime>>,
}

impl Sprite {
    pub fn new(
        mut runtime: SpriteRuntime,
        target: &savefile::Target,
        start_state: vm::VMState,
    ) -> Result<Self> {
        runtime.set_position(&Coordinate::new(target.x, target.y));

        let runtime_ref = Rc::new(RwLock::new(runtime));
        let mut threads: Vec<Thread> = Vec::new();

        for hat_id in find_hats(&target.blocks) {
            let block = block_tree(hat_id, runtime_ref.clone(), &target.blocks)
                .map_err(|e| ErrorKind::Initialization(Box::new(e)))?;
            threads.push(Thread::start(block, runtime_ref.clone(), start_state));
        }

        let cb_ref: ClosureRef = Rc::new(RefCell::new(None));
        let cb_ref_clone = cb_ref.clone();
        let request_animation_frame_id = Rc::new(RefCell::new(0));
        let request_animation_frame_id_clone = request_animation_frame_id.clone();
        let runtime_clone = runtime_ref.clone();
        let window = web_sys::window().unwrap();
        *cb_ref.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let runtime_clone = runtime_clone.clone();
            let cb_ref_clone = cb_ref_clone.clone();
            let request_animation_frame_id_clone = request_animation_frame_id_clone.clone();
            let window = window.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = runtime_clone.write().await.redraw() {
                    log::error!("error occurred on redraw: {}", e);
                    return;
                }

                let cb = cb_ref_clone.borrow();
                let f = cb.as_ref().unwrap();
                *request_animation_frame_id_clone.borrow_mut() = window
                    .request_animation_frame(f.as_ref().unchecked_ref())
                    .unwrap();
            });
        }) as Box<dyn Fn()>));
        let cb = cb_ref.borrow();
        let f = cb.as_ref().unwrap();
        *request_animation_frame_id.borrow_mut() = web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())?;

        Ok(Self {
            threads,
            closure: cb_ref.clone(),
            request_animation_frame_id,
            runtime: runtime_ref,
        })
    }

    pub async fn continue_(&mut self) {
        for thread in &mut self.threads {
            thread.continue_().await;
        }
    }

    pub async fn pause(&mut self) {
        for thread in &mut self.threads {
            thread.pause().await;
        }
        self.runtime
            .write()
            .await
            .redraw()
            .unwrap_or_else(|e| log::error!("{}", e));
    }

    pub async fn step(&self) {
        for thread in &self.threads {
            thread.step().await;
        }
    }
}

impl Drop for Sprite {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap()
            .cancel_animation_frame(*self.request_animation_frame_id.borrow())
            .unwrap();
    }
}

type ClosureRef = Rc<RefCell<Option<Closure<dyn Fn()>>>>;

pub fn find_hats(block_infos: &HashMap<String, savefile::Block>) -> Vec<&str> {
    let mut hats: Vec<&str> = Vec::new();
    for (id, block_info) in block_infos {
        if block_info.top_level {
            hats.push(id);
        }
    }
    hats.sort_unstable();

    hats
}

#[derive(Debug)]
pub struct Thread {
    inner: Rc<RwLock<Inner>>,
    controller: Rc<DebugController>,
}

#[derive(Debug)]
struct Inner {
    hat: Rc<RefCell<Box<dyn Block>>>,
    runtime: Rc<RwLock<SpriteRuntime>>,
    display_debug: Rc<RwLock<bool>>,
}

impl Thread {
    pub fn start(
        hat: Box<dyn Block>,
        runtime: Rc<RwLock<SpriteRuntime>>,
        start_state: vm::VMState,
    ) -> Self {
        let inner = Inner {
            hat: Rc::new(RefCell::new(hat)),
            runtime,
            display_debug: Rc::new(RwLock::new(false)),
        };
        let thread = Thread {
            inner: Rc::new(RwLock::new(inner)),
            controller: Rc::new(DebugController::new()),
        };

        let inner_clone = thread.inner.clone();
        let controller_clone = thread.controller.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match start_state {
                vm::VMState::Paused => controller_clone.pause().await,
                vm::VMState::Running => controller_clone.continue_().await,
            }
            Thread::run(inner_clone, controller_clone).await.unwrap();
        });
        thread
    }

    async fn run(inner: Rc<RwLock<Inner>>, controller: Rc<DebugController>) -> Result<()> {
        {
            let thread = inner.write().await;
            let debug_info = if *thread.display_debug.read().await {
                let block = thread.hat.borrow();
                DebugInfo {
                    show: true,
                    block_name: block.block_info().name.to_string(),
                    block_id: block.block_info().id,
                }
            } else {
                DebugInfo::default()
            };
            thread.runtime.write().await.set_debug_info(&debug_info);
        }

        let mut curr_block = inner.read().await.hat.clone();
        let mut loop_start: Vec<Rc<RefCell<Box<dyn Block>>>> = Vec::new();

        for i in 0usize.. {
            let thread = inner.write().await;
            let debug_info = if *thread.display_debug.read().await {
                let block = curr_block.borrow();
                DebugInfo {
                    show: true,
                    block_name: block.block_info().name.to_string(),
                    block_id: block.block_info().id.to_string(),
                }
            } else {
                DebugInfo::default()
            };
            thread.runtime.write().await.set_debug_info(&debug_info);

            let execute_result = curr_block.borrow_mut().execute().await;
            match execute_result {
                Next::None => match loop_start.pop() {
                    None => break,
                    Some(b) => curr_block = b,
                },
                Next::Err(e) => {
                    let block = curr_block.borrow();
                    return Err(ErrorKind::Block(
                        block.block_info().name,
                        block.block_info().id,
                        Box::new(e),
                    )
                    .into());
                }
                Next::Continue(b) => curr_block = b,
                Next::Loop(b) => {
                    loop_start.push(curr_block.clone());
                    curr_block = b;
                }
            }

            controller.wait().await;

            if i % 0x1000 == 0 {
                // TODO record time for variable fps
                // Yield to render loop
                TimeoutFuture::new(0).await;
            }
        }

        Ok(())
    }

    pub async fn continue_(&mut self) {
        // let inner = self.inner.write().await;
        // *inner.display_debug.write().await = false;
        self.controller.continue_().await;
    }

    pub async fn pause(&mut self) {
        // let inner = self.inner.write().await;
        self.controller.pause().await;
        // *inner.display_debug.write().await = true;
    }

    pub async fn step(&self) {
        self.controller.step();
    }
}

#[derive(Debug, Default, Clone)]
pub struct DebugInfo {
    pub show: bool,
    pub block_id: String,
    pub block_name: String,
}
