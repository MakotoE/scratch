use super::*;
use maplit::hashmap;
use sprite_runtime::Coordinate;

pub fn get_block(name: &str, id: String, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "movesteps" => Box::new(MoveSteps::new(id, runtime)),
        "gotoxy" => Box::new(GoToXY::new(id, runtime)),
        "changexby" => Box::new(ChangeXBy::new(id, runtime)),
        "changeyby" => Box::new(ChangeYBy::new(id, runtime)),
        "setx" => Box::new(SetX::new(id, runtime)),
        "sety" => Box::new(SetY::new(id, runtime)),
        "xposition" => Box::new(XPosition::new(id, runtime)),
        "yposition" => Box::new(YPosition::new(id, runtime)),
        "direction" => Box::new(Direction::new(id, runtime)),
        "pointindirection" => Box::new(PointingDirection::new(id, runtime)),
        "goto" => Box::new(GoTo::new(id, runtime)),
        "goto_menu" => Box::new(GoToMenu::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct MoveSteps {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    steps: Option<Box<dyn Block>>,
}

impl MoveSteps {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            steps: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for MoveSteps {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "MoveSteps",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"steps" => &self.steps}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
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
            None => return Next::Err(wrap_err!("steps is None")),
        };

        let steps = value_to_float(&steps_value)?;
        let mut runtime = self.runtime.sprite.write().await;
        let position = runtime.position().add(&Coordinate::new(steps, 0.0));
        runtime.set_position(&position);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct GoToXY {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    x: Option<Box<dyn Block>>,
    y: Option<Box<dyn Block>>,
}

impl GoToXY {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            x: None,
            y: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for GoToXY {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoToXY",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"x" => &self.x, "y" => &self.y}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
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
            None => return Next::Err(wrap_err!("x is None")),
        };
        let y = match &self.y {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err(wrap_err!("y is None")),
        };

        self.runtime
            .sprite
            .write()
            .await
            .set_position(&Coordinate::new(x, y));
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct ChangeXBy {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    dx: Option<Box<dyn Block>>,
}

impl ChangeXBy {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            dx: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeXBy {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeXBy",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"dx" => &self.dx}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
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
            None => return Next::Err(wrap_err!("dx is None")),
        };

        let mut runtime = self.runtime.sprite.write().await;
        let position = runtime.position().add(&Coordinate::new(x, 0.0));
        runtime.set_position(&position);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct ChangeYBy {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    dy: Option<Box<dyn Block>>,
}

impl ChangeYBy {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            dy: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeYBy {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeYBy",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"dy" => &self.dy}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
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
            None => return Next::Err(wrap_err!("dy is None")),
        };

        let mut runtime = self.runtime.sprite.write().await;
        let position = runtime.position().add(&Coordinate::new(0.0, y));
        runtime.set_position(&position);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetX {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    x: Option<Box<dyn Block>>,
}

impl SetX {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            x: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetX {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetX",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"x" => &self.x}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
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
            None => return Next::Err(wrap_err!("x is None")),
        };

        let curr_y = self.runtime.sprite.write().await.position().y();

        self.runtime
            .sprite
            .write()
            .await
            .set_position(&Coordinate::new(x, curr_y));
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetY {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    y: Option<Box<dyn Block>>,
}

impl SetY {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            y: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetY {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetY",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"y" => &self.y}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
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
            None => return Next::Err(wrap_err!("y is None")),
        };

        let mut runtime = self.runtime.sprite.write().await;
        let curr_x = runtime.position().x();

        runtime.set_position(&Coordinate::new(curr_x, y));
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct XPosition {
    id: String,
    runtime: Runtime,
}

impl XPosition {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for XPosition {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "XPosition",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.runtime.sprite.read().await.position().x().into())
    }
}

#[derive(Debug)]
pub struct YPosition {
    id: String,
    runtime: Runtime,
}

impl YPosition {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for YPosition {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "YPosition",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.runtime.sprite.read().await.position().y().into())
    }
}

#[derive(Debug)]
pub struct Direction {
    id: String,
    runtime: Runtime,
}

impl Direction {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for Direction {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Direction",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.runtime.sprite.read().await.position().y().into())
    }
}

#[derive(Debug)]
pub struct PointingDirection {
    id: String,
    runtime: Runtime,
}

impl PointingDirection {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for PointingDirection {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "PointingDirection",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.runtime.sprite.read().await.position().y().into())
    }
}

#[derive(Debug)]
pub struct GoTo {
    id: String,
    runtime: Runtime,
}

impl GoTo {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for GoTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoTo",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.runtime.sprite.read().await.position().y().into())
    }
}

#[derive(Debug)]
pub struct GoToMenu {
    id: String,
    runtime: Runtime,
}

impl GoToMenu {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for GoToMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoToMenu",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.runtime.sprite.read().await.position().y().into())
    }
}
