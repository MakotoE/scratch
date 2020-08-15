use super::*;

pub fn get_block(
    name: &str,
    id: &str,
    runtime: Rc<RefCell<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "penDown" => Box::new(PenDown::new(id, runtime)),
        "penUp" => Box::new(PenUp::new(id, runtime)),
        _ => return Err(format!("block \"{}\": name {} does not exist", id, name).into()),
    })
}

#[derive(Debug)]
pub struct PenDown {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl PenDown {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for PenDown {
    fn block_name(&self) -> &'static str {
        "PenDown"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        self.runtime.borrow_mut().pen_down();
        Ok(())
    }
}

#[derive(Debug)]
pub struct PenUp {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl PenUp {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for PenUp {
    fn block_name(&self) -> &'static str {
        "PenUp"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        self.runtime.borrow_mut().pen_up();
        Ok(())
    }
}
