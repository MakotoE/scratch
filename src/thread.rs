use super::*;
use blocks::{Block, Next};
use controller::{PauseState, ThreadController};
use gloo_timers::future::TimeoutFuture;
use runtime::SpriteRuntime;

#[derive(Debug)]
pub struct Thread {
    controller: Rc<ThreadController>,
}

impl Thread {
    pub fn start(
        hat: Box<dyn Block>,
        runtime: Rc<RwLock<SpriteRuntime>>,
        start_state: vm::VMState,
    ) -> Self {
        let thread = Thread {
            controller: Rc::new(ThreadController::new()),
        };

        let controller_clone = thread.controller.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match start_state {
                vm::VMState::Paused => controller_clone.pause().await,
                vm::VMState::Running => controller_clone.continue_().await,
            }
            Thread::run(Rc::new(RefCell::new(hat)), runtime, controller_clone)
                .await
                .unwrap_or_else(|e| log::error!("{}", e));
        });
        thread
    }

    async fn run(
        hat: Rc<RefCell<Box<dyn Block>>>,
        runtime: Rc<RwLock<SpriteRuntime>>,
        controller: Rc<ThreadController>,
    ) -> Result<()> {
        {
            let debug_info = if controller.state().await == PauseState::Paused {
                let block = hat.borrow();
                DebugInfo {
                    show: true,
                    block_name: block.block_info().name.to_string(),
                    block_id: block.block_info().id,
                }
            } else {
                DebugInfo::default()
            };
            runtime.write().await.set_debug_info(&debug_info);
        }

        let mut curr_block = hat.clone();
        let mut loop_start: Vec<Rc<RefCell<Box<dyn Block>>>> = Vec::new();
        let performance = web_sys::window().unwrap().performance().unwrap();
        let mut last_redraw = performance.now();

        for i in 0usize.. {
            let debug_info = if controller.state().await == PauseState::Paused {
                let block = curr_block.borrow();
                DebugInfo {
                    show: true,
                    block_name: block.block_info().name.to_string(),
                    block_id: block.block_info().id.to_string(),
                }
            } else {
                DebugInfo::default()
            };
            runtime.write().await.set_debug_info(&debug_info);

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

            if i % 0x1000 == 0 && performance.now() - last_redraw > 100.0 {
                // Yield to render loop
                TimeoutFuture::new(0).await;
                last_redraw = performance.now();
            }
        }

        Ok(())
    }

    pub async fn continue_(&mut self) {
        self.controller.continue_().await;
    }

    pub async fn pause(&mut self) {
        self.controller.pause().await;
    }

    pub fn step(&self) {
        self.controller.step();
    }
}

#[derive(Debug, Default, Clone)]
pub struct DebugInfo {
    pub show: bool,
    pub block_id: String,
    pub block_name: String,
}
