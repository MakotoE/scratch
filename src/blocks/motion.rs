use super::*;

pub fn get_block(name: &str, id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Result<Box<dyn Block>>  {
    Ok(match name {
        "movesteps" => Box::new(MoveSteps::new(id, runtime)),
        "gotoxy" => Box::new(GoToXY::new(id, runtime)),
        "changexby" => Box::new(ChangeXBy::new(id, runtime)),
        "changeyby" => Box::new(ChangeYBy::new(id, runtime)),
        _ => return Err(format!("block \"{}\": name {} does not exist", id, name).into()),
    })
}

#[derive(Debug)]
pub struct MoveSteps {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    steps: Option<Box<dyn Block>>,
}

impl MoveSteps {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            steps: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for MoveSteps {
    fn block_name(&self) -> &'static str {
        "MoveSteps"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "STEPS" => self.steps = Some(block),
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        return self.next.clone().into();
    }

    async fn execute(&mut self) -> Result<()> {
        let steps_value = match &self.steps {
            Some(block) => block.value()?,
            None => return Err("steps is None".into()),
        };

        let steps = steps_value
            .as_f64()
            .ok_or_else(|| wrong_type_err(&steps_value))?;
        self.runtime
            .borrow_mut()
            .add_coordinate(&Coordinate::new(steps, 0.0));
        self.runtime.borrow().redraw()
    }
}

#[derive(Debug)]
pub struct GoToXY {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    x: Option<Box<dyn Block>>,
    y: Option<Box<dyn Block>>,
}

impl GoToXY {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            x: None,
            y: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for GoToXY {
    fn block_name(&self) -> &'static str {
        "GoToXY"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "X" => self.x = Some(block),
            "Y" => self.y = Some(block),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        let x = match &self.x {
            Some(b) => value_to_float(&b.value()?)?,
            None => return Err("x is None".into()),
        };
        let y = match &self.y {
            Some(b) => value_to_float(&b.value()?)?,
            None => return Err("y is None".into()),
        };

        self.runtime
            .borrow_mut()
            .set_position(&Coordinate::new(x, y));
        self.runtime.borrow().redraw()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChangeXBy {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    dx: Option<Box<dyn Block>>,
}

impl ChangeXBy {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            dx: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeXBy {
    fn block_name(&self) -> &'static str {
        "ChangeXBy"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "DX" => self.dx = Some(block),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        let x = match &self.dx {
            Some(b) => value_to_float(&b.value()?)?,
            None => return Err("x is None".into()),
        };

        self.runtime
            .borrow_mut()
            .add_coordinate(&Coordinate::new(x, 0.0));
        self.runtime.borrow().redraw()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ChangeYBy {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    dy: Option<Box<dyn Block>>,
}

impl ChangeYBy {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            dy: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeYBy {
    fn block_name(&self) -> &'static str {
        "ChangeYBy"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "DY" => self.dy = Some(block),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        let y = match &self.dy {
            Some(b) => value_to_float(&b.value()?)?,
            None => return Err("y is None".into()),
        };

        self.runtime
            .borrow_mut()
            .add_coordinate(&Coordinate::new(0.0, y));
        self.runtime.borrow().redraw()?;
        Ok(())
    }
}
