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
        self.controller.wait().await;
        let mut next = self.hat.borrow_mut().execute().await;
        self.runtime.borrow().redraw()?;

        while let Next::Continue(b) = &next {
            self.controller.wait().await;
            let result = b.borrow_mut().execute().await;
            next = result;
            self.runtime.borrow().redraw()?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct DebugController {
    semaphore: tokio::sync::Semaphore,
    blocking: tokio::sync::RwLock<bool>,
}

impl DebugController {
    pub fn new() -> Self {
        Self {
            semaphore: tokio::sync::Semaphore::new(0),
            blocking: tokio::sync::RwLock::new(false),
        }
    }

    pub async fn wait(&self) {
        if *self.blocking.read().await {
            self.semaphore.acquire().await.forget();
        }
    }

    pub async fn run(&self) {
        *self.blocking.write().await = false;
    }

    pub async fn pause(&self) {
        *self.blocking.write().await = true;
    }

    pub async fn step(&self) {
        if *self.blocking.read().await {
            self.semaphore.add_permits(1);
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
