use super::*;
use async_trait::async_trait;
use blocks::*;
use controller::DebugController;
use runtime::{Coordinate, SpriteRuntime};

#[derive(Debug, Default)]
pub struct Sprite {
    controllers: Vec<Arc<DebugController>>,
    start_paused: bool,
}

impl Sprite {
    pub fn new() -> Self {
        Self {
            controllers: Vec::new(),
            start_paused: false,
        }
    }

    pub fn start(&mut self, mut runtime: SpriteRuntime, target: &savefile::Target) -> Result<()> {
        self.controllers.clear();

        runtime.set_position(&Coordinate::new(target.x, target.y));

        let runtime_ref = Rc::new(RefCell::new(runtime));
        let controller_ref = Arc::new(DebugController::new());

        let start_paused = self.start_paused;

        for hat_id in find_hats(&target.blocks) {
            self.controllers.push(controller_ref.clone());

            let block = new_block(hat_id, runtime_ref.clone(), &target.blocks)
                .map_err(|e| ErrorKind::Initialization(Box::new(e)))?;
            let thread = Thread::new(block, runtime_ref.clone(), controller_ref.clone());
            let controller = controller_ref.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if start_paused {
                    controller.pause().await;
                }
                match thread.start().await {
                    Ok(_) => {}
                    Err(e) => log::error!("{}", e),
                }
            });
        }
        Ok(())
    }

    pub async fn continue_(&mut self) {
        self.start_paused = false;

        for c in &self.controllers {
            c.continue_().await;
        }
    }

    pub async fn pause(&mut self) {
        self.start_paused = true;

        for c in &self.controllers {
            c.pause().await;
        }
    }

    pub async fn slow_speed(&mut self) {
        for c in &self.controllers {
            c.slow_speed().await;
        }
    }

    pub fn step(&self) {
        for c in &self.controllers {
            c.step();
        }
    }
}

fn find_hats(block_infos: &HashMap<String, savefile::Block>) -> Vec<&str> {
    let mut hats: Vec<&str> = Vec::new();
    for (id, block_info) in block_infos {
        if block_info.top_level {
            hats.push(id);
        }
    }

    hats
}

#[derive(Debug)]
pub struct Thread {
    hat: Rc<RefCell<Box<dyn Block>>>,
    runtime: Rc<RefCell<SpriteRuntime>>,
    controller: Arc<DebugController>,
}

impl Thread {
    pub fn new(
        hat: Box<dyn Block>,
        runtime: Rc<RefCell<SpriteRuntime>>,
        controller: Arc<DebugController>,
    ) -> Self {
        Self {
            hat: Rc::new(RefCell::new(hat)),
            runtime,
            controller,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let debug_info = if self.controller.display_debug().await {
            DebugInfo {
                show: true,
                block_name: self.hat.borrow().block_name().to_string(),
                block_id: self.hat.borrow().id().to_string(),
            }
        } else {
            DebugInfo::default()
        };
        self.runtime.borrow_mut().set_debug_info(&debug_info);
        self.runtime.borrow().redraw()?; // Clears screen on restart

        let mut curr_block = self.hat.clone();
        let mut loop_start: Vec<Rc<RefCell<Box<dyn Block>>>> = Vec::new();

        loop {
            let debug_info = if self.controller.display_debug().await {
                DebugInfo {
                    show: true,
                    block_name: curr_block.borrow().block_name().to_string(),
                    block_id: curr_block.borrow().id().to_string(),
                }
            } else {
                DebugInfo::default()
            };
            self.runtime.borrow_mut().set_debug_info(&debug_info);

            let execute_result = curr_block.borrow_mut().execute().await;
            self.runtime.borrow().redraw()?;
            match execute_result {
                Next::None => match loop_start.pop() {
                    None => break,
                    Some(b) => curr_block = b,
                },
                Next::Err(e) => return Err(e),
                Next::Continue(b) => curr_block = b,
                Next::Loop(b) => {
                    loop_start.push(curr_block.clone());
                    curr_block = b;
                }
            }
            self.controller.wait().await;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct DebugInfo {
    pub show: bool,
    pub block_id: String,
    pub block_name: String,
}

#[derive(Debug)]
pub struct DummyBlock {
    next: Rc<RefCell<Box<dyn Block>>>,
}

#[async_trait(?Send)]
impl Block for DummyBlock {
    fn block_name(&self) -> &'static str {
        "DummyBlock"
    }

    fn id(&self) -> &str {
        ""
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn execute(&mut self) -> Next {
        Next::Continue(self.next.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod thread_iterator {
        use super::*;

        #[derive(Debug)]
        struct LastBlock {}

        impl Block for LastBlock {
            fn block_name(&self) -> &'static str {
                "LastBlock"
            }

            fn id(&self) -> &str {
                ""
            }

            fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
        }
    }
}
