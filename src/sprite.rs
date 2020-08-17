use super::*;
use blocks::*;
use runtime::{Coordinate, SpriteRuntime};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<Thread>,
}

impl Sprite {
    pub fn new(mut runtime: SpriteRuntime, target: &savefile::Target) -> Result<Self> {
        runtime.set_position(&Coordinate::new(target.x, target.y));

        let runtime_ref = Rc::new(RefCell::new(runtime));
        let mut threads: Vec<Thread> = Vec::new();
        for hat_id in find_hats(&target.blocks) {
            let block = new_block(hat_id, runtime_ref.clone(), &target.blocks)
                .map_err(|e| ErrorKind::Initialization(Box::new(e)))?;
            threads.push(Thread::new(block, runtime_ref.clone()));
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
pub struct Thread {
    hat: Rc<RefCell<Box<dyn Block>>>,
    runtime: Rc<RefCell<SpriteRuntime>>,
}

impl Thread {
    pub fn new(hat: Box<dyn Block>, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            hat: Rc::new(RefCell::new(hat)),
            runtime,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        let mut next = self.hat.borrow_mut().execute().await;
        self.runtime.borrow().redraw()?;

        while let Next::Continue(b) = &next {
            let result = b.borrow_mut().execute().await;
            next = result;
            self.runtime.borrow().redraw()?;
        }

        Ok(())
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
