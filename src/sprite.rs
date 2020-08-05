use super::*;
use block::*;

#[derive(Debug)]
pub struct Sprite<'r> {
    threads: Vec<Thread<'r>>,
    runtime: &'r Mutex<runtime::SpriteRuntime>,
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
                &runtime,
                new_block(hat_id.to_string(), &runtime, &target.blocks)?,
            ));
        }
        Ok(Self { threads, runtime })
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
    hat: Rc<RefCell<Box<dyn Block<'r> + 'r>>>,
}

impl<'r> Thread<'r> {
    pub fn new(runtime: &'r Mutex<runtime::SpriteRuntime>, hat: Box<dyn Block<'r> + 'r>) -> Self {
        Self {
            runtime,
            hat: Rc::new(RefCell::new(hat)),
        }
    }

    pub fn execute(&self) -> Result<()> {
        let mut iter = self.iter();
        while let Some(next) = iter.next()? {
            next.borrow_mut().execute()?;
        }

        Ok(())
    }

    fn iter(&self) -> ThreadIterator<'r> {
        ThreadIterator::new(self.hat.clone())
    }
}

#[derive(Debug)]
pub struct ThreadIterator<'r> {
    curr: Rc<RefCell<Box<dyn Block<'r> + 'r>>>,
}

impl<'r> ThreadIterator<'r> {
    fn new(hat: Rc<RefCell<Box<dyn Block<'r> + 'r>>>) -> Self {
        Self {
            curr: Rc::new(RefCell::new(Box::new(DummyBlock { next: hat }))),
        }
    }

    fn next(&mut self) -> Result<Option<Rc<RefCell<Box<dyn Block<'r> + 'r>>>>> {
        let next = self.curr.borrow().next()?;
        Ok(match next {
            Some(b) => {
                self.curr = b.clone();
                Some(b)
            }
            None => None,
        })
    }
}

#[derive(Debug)]
pub struct DummyBlock<'r> {
    next: Rc<RefCell<Box<dyn Block<'r> + 'r>>>,
}

impl<'r> Block<'r> for DummyBlock<'r> {
    fn set_input(&mut self, _: &str, _: Box<dyn Block<'r> + 'r>) {}
    fn set_field(&mut self, _: &str, _: String) {}

    fn next(&self) -> Result<Option<Rc<RefCell<Box<dyn Block<'r> + 'r>>>>> {
        Ok(Some(self.next.clone()))
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
            fn set_input(&mut self, _: &str, _: Box<dyn Block<'r> + 'r>) {}
            fn set_field(&mut self, _: &str, _: String) {}
        }

        #[test]
        fn into_iter() {
            {
                let block_0: Rc<RefCell<Box<dyn Block<'_> + '_>>> =
                    Rc::new(RefCell::new(Box::new(LastBlock {})));
                let mut iter = ThreadIterator::new(block_0);
                assert!(iter.next().unwrap().is_some());
                assert!(iter.next().unwrap().is_none());
            }
            {
                let block_0: Rc<RefCell<Box<dyn Block<'_> + '_>>> =
                    Rc::new(RefCell::new(Box::new(LastBlock {})));
                let block_1: Rc<RefCell<Box<dyn Block<'_> + '_>>> =
                    Rc::new(RefCell::new(Box::new(DummyBlock { next: block_0 })));
                let mut iter = ThreadIterator::new(block_1);
                assert!(iter.next().unwrap().is_some());
                assert!(iter.next().unwrap().is_some());
                assert!(iter.next().unwrap().is_none());
            }
        }
    }
}
