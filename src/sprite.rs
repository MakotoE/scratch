use super::*;
use block::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Sprite<'r> {
    threads: Vec<Thread<'r>>,
}

impl<'r> Sprite<'r> {
    pub fn new(
        runtime: &'r Mutex<runtime::SpriteRuntime>,
        target: &savefile::Target,
    ) -> Result<Self> {
        {
            let mut r = runtime.lock()?;
            r.x = target.x;
            r.y = target.y;
        }

        let mut threads: Vec<Thread> = Vec::new();
        for hat_id in find_hats(&target.blocks) {
            threads.push(Thread::new(
                runtime,
                new_block(hat_id, runtime, &target.blocks)?,
            ));
        }
        Ok(Self { threads })
    }

    pub fn threads(&self) -> &[Thread<'r>] {
        self.threads.as_slice()
    }

    pub fn execute(&self) -> Result<()> {
        for t in &self.threads {
            t.execute()?;
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
pub struct Thread<'r> {
    runtime: &'r Mutex<runtime::SpriteRuntime>,
    hat: Rc<RefCell<dyn Block<'r> + 'r>>,
}

impl<'r> Thread<'r> {
    pub fn new(
        runtime: &'r Mutex<runtime::SpriteRuntime>,
        hat: Rc<RefCell<dyn Block<'r> + 'r>>,
    ) -> Self {
        Self { runtime, hat }
    }

    pub fn execute(&self) -> Result<()> {
        for b in self.into_iter() {
            b.borrow_mut().execute()?;
        }

        Ok(())
    }
}

impl<'a, 'r> IntoIterator for &'a Thread<'r> {
    type Item = Rc<RefCell<dyn Block<'r> + 'r>>;
    type IntoIter = ThreadIterator<'r>;

    fn into_iter(self) -> ThreadIterator<'r> {
        ThreadIterator::new(self.hat.clone())
    }
}

#[derive(Debug)]
pub struct ThreadIterator<'r> {
    curr: Rc<RefCell<dyn Block<'r> + 'r>>,
}

impl<'r> ThreadIterator<'r> {
    fn new(hat: Rc<RefCell<dyn Block<'r> + 'r>>) -> Self {
        Self {
            curr: Rc::new(RefCell::new(DummyBlock { next: hat })),
        }
    }
}

impl<'r> Iterator for ThreadIterator<'r> {
    type Item = Rc<RefCell<dyn Block<'r> + 'r>>;

    fn next(&mut self) -> Option<Rc<RefCell<dyn Block<'r> + 'r>>> {
        let next = self.curr.borrow().next();
        match next {
            Some(b) => {
                self.curr = b.clone();
                Some(b)
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct DummyBlock<'r> {
    next: Rc<RefCell<dyn Block<'r> + 'r>>,
}

impl<'r> Block<'r> for DummyBlock<'r> {
    fn set_input(&mut self, _: &str, _: Rc<RefCell<dyn Block<'r> + 'r>>) {}
    fn set_field(&mut self, _: &str, _: &str) {}

    fn next(&self) -> Option<Rc<RefCell<dyn Block<'r> + 'r>>> {
        Some(self.next.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod thread_iterator {
        use super::*;

        #[derive(Debug)]
        struct LastBlock {}

        impl<'r> Block<'r> for LastBlock {
            fn set_input(&mut self, _: &str, _: Rc<RefCell<dyn Block<'r> + 'r>>) {}
            fn set_field(&mut self, _: &str, _: &str) {}

            fn next(&self) -> Option<Rc<RefCell<dyn Block<'r> + 'r>>> {
                None
            }
        }

        #[test]
        fn into_iter() {
            {
                let block_0 = Rc::new(RefCell::new(LastBlock {}));
                let mut iter = ThreadIterator::new(block_0);
                assert!(iter.next().is_some());
                assert!(iter.next().is_none());
            }
            {
                let block_0 = Rc::new(RefCell::new(LastBlock {}));
                let block_1 = Rc::new(RefCell::new(DummyBlock { next: block_0 }));
                let mut iter = ThreadIterator::new(block_1);
                assert!(iter.next().is_some());
                assert!(iter.next().is_some());
                assert!(iter.next().is_none());
            }
        }
    }
}
