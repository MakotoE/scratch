use super::*;
use block::*;

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<Thread>,
}

impl Sprite {
    pub fn new(mut runtime: runtime::SpriteRuntime, target: &savefile::Target) -> Result<Self> {
        runtime.x = target.x;
        runtime.y = target.y;

        let runtime_ref = Rc::new(RefCell::new(runtime));
        let mut threads: Vec<Thread> = Vec::new();
        for hat_id in find_hats(&target.blocks) {
            threads.push(Thread::new(
                new_block(hat_id.to_string(), runtime_ref.clone(), &target.blocks)?,
            ));
        }
        Ok(Self { threads })
    }

    pub fn threads(&self) -> &[Thread] {
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
pub struct Thread {
    hat: Rc<RefCell<Box<dyn Block>>>,
}

impl Thread {
    pub fn new(hat: Box<dyn Block>) -> Self {
        Self {
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

    fn iter(&self) -> ThreadIterator {
        ThreadIterator::new(self.hat.clone())
    }
}

#[derive(Debug)]
pub struct ThreadIterator {
    curr: Rc<RefCell<Box<dyn Block>>>,
}

impl ThreadIterator {
    fn new(hat: Rc<RefCell<Box<dyn Block>>>) -> Self {
        Self {
            curr: Rc::new(RefCell::new(Box::new(DummyBlock { next: hat }))),
        }
    }

    fn next(&mut self) -> Result<Option<Rc<RefCell<Box<dyn Block>>>>> {
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
pub struct DummyBlock {
    next: Rc<RefCell<Box<dyn Block>>>,
}

impl Block for DummyBlock {
    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
    fn set_field(&mut self, _: &str, _: String) {}

    fn next(&self) -> Result<Option<Rc<RefCell<Box<dyn Block>>>>> {
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

        impl Block for LastBlock {
            fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
            fn set_field(&mut self, _: &str, _: String) {}
        }

        #[test]
        fn into_iter() {
            {
                let block_0: Rc<RefCell<Box<dyn Block>>> =
                    Rc::new(RefCell::new(Box::new(LastBlock {})));
                let mut iter = ThreadIterator::new(block_0);
                assert!(iter.next().unwrap().is_some());
                assert!(iter.next().unwrap().is_none());
            }
            {
                let block_0: Rc<RefCell<Box<dyn Block>>> =
                    Rc::new(RefCell::new(Box::new(LastBlock {})));
                let block_1: Rc<RefCell<Box<dyn Block>>> =
                    Rc::new(RefCell::new(Box::new(DummyBlock { next: block_0 })));
                let mut iter = ThreadIterator::new(block_1);
                assert!(iter.next().unwrap().is_some());
                assert!(iter.next().unwrap().is_some());
                assert!(iter.next().unwrap().is_none());
            }
        }
    }
}
