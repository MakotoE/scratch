use super::*;

pub fn get_block(
    name: &str,
    id: &str,
    runtime: Rc<RefCell<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "penDown" => Box::new(PenDown::new(id, runtime)),
        "penUp" => Box::new(PenUp::new(id, runtime)),
        "setPenColorToColor" => Box::new(SetPenColorToColor::new(id, runtime)),
        "setPenSizeTo" => Box::new(SetPenSizeTo::new(id, runtime)),
        _ => return Err(format!("{} does not exist", name).into()),
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

#[derive(Debug)]
pub struct SetPenColorToColor {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    color: Option<Box<dyn Block>>,
}

impl SetPenColorToColor {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            color: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetPenColorToColor {
    fn block_name(&self) -> &'static str {
        "SetPenColorToColor"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "COLOR" => self.color = Some(block),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        let color_value = match &self.color {
            Some(b) => b.value()?,
            None => return Err("color is None".into()),
        };
        let color = color_value
            .as_str()
            .ok_or_else(|| Error::from("color is not a string"))?;
        self.runtime
            .borrow_mut()
            .set_pen_color(&runtime::Color::from_hex(color)?);
        Ok(())
    }
}

#[derive(Debug)]
pub struct SetPenSizeTo {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    size: Option<Box<dyn Block>>,
}

impl SetPenSizeTo {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            size: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetPenSizeTo {
    fn block_name(&self) -> &'static str {
        "SetPenSizeTo"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "SIZE" => self.size = Some(block),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        let size = match &self.size {
            Some(b) => value_to_float(&b.value()?)?,
            None => return Err("color is None".into()),
        };

        self.runtime.borrow_mut().set_pen_size(size);
        Ok(())
    }
}
