use super::*;
use blocks::{Block, Next};
use controller::{PauseState, ThreadController};
use gloo_timers::future::TimeoutFuture;
use runtime::Runtime;

#[derive(Debug)]
pub struct Thread {
    controller: Rc<ThreadController>,
    hat: Rc<RefCell<Box<dyn Block>>>,
}

impl Thread {
    pub fn start(hat: Box<dyn Block>, runtime: Runtime, thread_id: usize) -> Self {
        let thread = Thread {
            controller: Rc::new(ThreadController::new()),
            hat: Rc::new(RefCell::new(hat)),
        };

        let controller_clone = thread.controller.clone();
        let hat_clone = thread.hat.clone();
        wasm_bindgen_futures::spawn_local(async move {
            Thread::run(hat_clone, runtime, controller_clone, thread_id)
                .await
                .unwrap_or_else(|e| log::error!("{}", e));
        });
        thread
    }

    async fn run(
        hat: Rc<RefCell<Box<dyn Block>>>,
        runtime: Runtime,
        controller: Rc<ThreadController>,
        thread_id: usize,
    ) -> Result<()> {
        {
            let debug_info = if controller.state().await == PauseState::Paused {
                let block = hat.borrow();
                DebugInfo {
                    block_name: block.block_info().name.to_string(),
                    block_id: block.block_info().id,
                }
            } else {
                DebugInfo::default()
            };
            runtime
                .sprite
                .write()
                .await
                .set_debug_info(thread_id, debug_info);
        }

        let mut curr_block = hat.clone();
        let mut loop_start: Vec<Rc<RefCell<Box<dyn Block>>>> = Vec::new();
        let performance = web_sys::window().unwrap().performance().unwrap();
        let mut last_redraw = performance.now();

        for i in 0usize.. {
            let debug_info = if controller.state().await == PauseState::Paused {
                let block = curr_block.borrow();
                DebugInfo {
                    block_name: block.block_info().name.to_string(),
                    block_id: block.block_info().id.to_string(),
                }
            } else {
                DebugInfo::default()
            };
            runtime
                .sprite
                .write()
                .await
                .set_debug_info(thread_id, debug_info);

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

    pub async fn continue_(&self) {
        self.controller.continue_().await;
    }

    pub async fn pause(&self) {
        self.controller.pause().await;
    }

    pub fn step(&self) {
        self.controller.step();
    }

    pub fn hat(&self) -> &Rc<RefCell<Box<dyn Block>>> {
        &self.hat
    }
}

#[derive(Debug, Default, Clone)]
pub struct DebugInfo {
    pub block_id: String,
    pub block_name: String,
}
