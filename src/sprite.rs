use super::*;
use async_trait::async_trait;
use blocks::*;
use runtime::{Coordinate, SpriteRuntime};

#[derive(Debug)]
pub struct Sprite<'d> {
    threads: Vec<Thread<'d>>,
}

impl<'d> Sprite<'d> {
    pub fn new(
        mut runtime: SpriteRuntime,
        target: &savefile::Target,
        controller: &'d DebugController,
    ) -> Result<Self> {
        runtime.set_position(&Coordinate::new(target.x, target.y));

        let runtime_ref = Rc::new(RefCell::new(runtime));
        let mut threads: Vec<Thread> = Vec::new();
        for hat_id in find_hats(&target.blocks) {
            let block = new_block(hat_id, runtime_ref.clone(), &target.blocks)
                .map_err(|e| ErrorKind::Initialization(Box::new(e)))?;
            threads.push(Thread::new(block, runtime_ref.clone(), controller));
        }
        Ok(Self { threads })
    }

    pub fn threads(&self) -> &[Thread] {
        self.threads.as_slice()
    }

    pub async fn execute(&self) -> Result<()> {
        for t in &self.threads {
            t.execute().await?;
        }
        Ok(())
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
pub struct Thread<'d> {
    hat: Rc<RefCell<Box<dyn Block>>>,
    runtime: Rc<RefCell<SpriteRuntime>>,
    controller: &'d DebugController,
}

impl<'d> Thread<'d> {
    pub fn new(
        hat: Box<dyn Block>,
        runtime: Rc<RefCell<SpriteRuntime>>,
        controller: &'d DebugController,
    ) -> Self {
        Self {
            hat: Rc::new(RefCell::new(hat)),
            runtime,
            controller,
        }
    }

    pub async fn execute(&self) -> Result<()> {
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
pub struct DebugController {
    semaphore: tokio::sync::Semaphore,
    blocking: tokio::sync::RwLock<bool>,
    display_debug: tokio::sync::RwLock<bool>,
}

impl DebugController {
    pub fn new() -> Self {
        Self {
            semaphore: tokio::sync::Semaphore::new(1),
            blocking: tokio::sync::RwLock::new(false),
            display_debug: tokio::sync::RwLock::new(false),
        }
    }

    pub async fn wait(&self) {
        self.semaphore.acquire().await.forget();
        if !*self.blocking.read().await {
            self.step().await;
        }
    }

    /// This method resets this DebugController's state.
    pub async fn continue_(&self) {
        *self.blocking.write().await = false;
        DebugController::reset_semaphore(&self.semaphore).await;
        self.semaphore.add_permits(1);
        *self.display_debug.write().await = false;
    }

    pub async fn pause(&self) {
        *self.blocking.write().await = true;
        DebugController::reset_semaphore(&self.semaphore).await;
        *self.display_debug.write().await = true;
    }

    pub async fn step(&self) {
        self.semaphore.add_permits(1);
    }

    pub async fn display_debug(&self) -> bool {
        *self.display_debug.read().await
    }

    async fn reset_semaphore(semaphore: &tokio::sync::Semaphore) {
        while semaphore.available_permits() > 0 {
            match semaphore.try_acquire() {
                Ok(p) => p.forget(),
                Err(_) => break,
            }
        }
    }
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
