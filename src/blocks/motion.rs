use super::*;

pub fn get_block(
    name: &str,
    id: &str,
    runtime: Rc<RwLock<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "movesteps" => Box::new(MoveSteps::new(id, runtime)),
        "gotoxy" => Box::new(GoToXY::new(id, runtime)),
        "changexby" => Box::new(ChangeXBy::new(id, runtime)),
        "changeyby" => Box::new(ChangeYBy::new(id, runtime)),
        "setx" => Box::new(SetX::new(id, runtime)),
        "sety" => Box::new(SetY::new(id, runtime)),
        "xposition" => Box::new(XPosition::new(id, runtime)),
        "yposition" => Box::new(YPosition::new(id, runtime)),
        _ => return Err(format!("{} does not exist", name).into()),
    })
}

#[derive(Debug)]
pub struct MoveSteps {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    steps: Option<Box<dyn Block>>,
}

impl MoveSteps {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
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

    async fn execute(&mut self) -> Next {
        let steps_value = match &self.steps {
            Some(block) => block.value().await?,
            None => return Next::Err("steps is None".into()),
        };

        let steps = value_to_float(&steps_value)?;
        let mut runtime = self.runtime.write().await;
        let position = runtime.position().add(&Coordinate::new(steps, 0.0));
        runtime.set_position(&position);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct GoToXY {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    x: Option<Box<dyn Block>>,
    y: Option<Box<dyn Block>>,
}

impl GoToXY {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
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

    async fn execute(&mut self) -> Next {
        let x = match &self.x {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err("x is None".into()),
        };
        let y = match &self.y {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err("y is None".into()),
        };

        self.runtime
            .write()
            .await
            .set_position(&Coordinate::new(x, y));
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct ChangeXBy {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    dx: Option<Box<dyn Block>>,
}

impl ChangeXBy {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
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

    async fn execute(&mut self) -> Next {
        let x = match &self.dx {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err("dx is None".into()),
        };

        let mut runtime = self.runtime.write().await;
        let position = runtime.position().add(&Coordinate::new(x, 0.0));
        runtime.set_position(&position);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct ChangeYBy {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    dy: Option<Box<dyn Block>>,
}

impl ChangeYBy {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
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

    async fn execute(&mut self) -> Next {
        let y = match &self.dy {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err("dy is None".into()),
        };

        let mut runtime = self.runtime.write().await;
        let position = runtime.position().add(&Coordinate::new(0.0, y));
        runtime.set_position(&position);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetX {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    x: Option<Box<dyn Block>>,
}

impl SetX {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            x: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetX {
    fn block_name(&self) -> &'static str {
        "SetX"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "X" => self.x = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let x = match &self.x {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err("x is None".into()),
        };

        let curr_y = self.runtime.write().await.position().y();

        self.runtime
            .write()
            .await
            .set_position(&Coordinate::new(x, curr_y));
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetY {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    y: Option<Box<dyn Block>>,
}

impl SetY {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
            y: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetY {
    fn block_name(&self) -> &'static str {
        "SetY"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "Y" => self.y = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let y = match &self.y {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err("y is None".into()),
        };

        let mut runtime = self.runtime.write().await;
        let curr_x = runtime.position().x();

        runtime.set_position(&Coordinate::new(curr_x, y));
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct XPosition {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
}

impl XPosition {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
        }
    }
}

#[async_trait(?Send)]
impl Block for XPosition {
    fn block_name(&self) -> &'static str {
        "XPosition"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.runtime.read().await.position().x().into())
    }
}

#[derive(Debug)]
pub struct YPosition {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
}

impl YPosition {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
        }
    }
}

#[async_trait(?Send)]
impl Block for YPosition {
    fn block_name(&self) -> &'static str {
        "YPosition"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.runtime.read().await.position().y().into())
    }
}
