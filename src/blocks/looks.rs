use super::*;
use gloo_timers::future::TimeoutFuture;
use sprite_runtime::HideStatus;

pub fn get_block(name: &str, id: String, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "say" => Box::new(Say::new(id, runtime)),
        "sayforsecs" => Box::new(SayForSecs::new(id, runtime)),
        "gotofrontback" => Box::new(GoToFrontBack::new(id, runtime)),
        "hide" => Box::new(Hide::new(id, runtime)),
        "show" => Box::new(Show::new(id, runtime)),
        "seteffectto" => Box::new(SetEffectTo::new(id, runtime)),
        "nextcostume" => Box::new(NextCostume::new(id, runtime)),
        "changeeffectby" => Box::new(ChangeEffectBy::new(id, runtime)),
        "setsizeto" => Box::new(SetSizeTo::new(id, runtime)),
        "switchcostumeto" => Box::new(SwitchCostumeTo::new(id, runtime)),
        "costume" => Box::new(Costume::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Say {
    id: String,
    runtime: Runtime,
    message: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Say {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            message: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Say {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Say",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"message" => &self.message}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "MESSAGE" => self.message = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let message = match &self.message {
            Some(b) => value_to_string(b.value().await?),
            None => return Next::Err(wrap_err!("message is None")),
        };
        self.runtime.sprite.write().await.say(Some(message));
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SayForSecs {
    id: String,
    runtime: Runtime,
    message: Option<Box<dyn Block>>,
    secs: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SayForSecs {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            message: None,
            secs: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SayForSecs {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SayForSecs",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(
                hashmap! {"message" => &self.message, "secs" => &self.secs},
            ),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "MESSAGE" => self.message = Some(block),
            "SECS" => self.secs = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let message = match &self.message {
            Some(b) => value_to_string(b.value().await?),
            None => return Next::Err(wrap_err!("message is None")),
        };

        let seconds = match &self.secs {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err(wrap_err!("secs is None")),
        };

        self.runtime.sprite.write().await.say(Some(message));
        TimeoutFuture::new((MILLIS_PER_SECOND * seconds).round() as u32).await;
        self.runtime.sprite.write().await.say(None);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct GoToFrontBack {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl GoToFrontBack {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for GoToFrontBack {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoToFrontBack",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct Hide {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Hide {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Hide {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Hide",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }

    async fn execute(&mut self) -> Next {
        self.runtime.sprite.write().await.set_hide(HideStatus::Hide);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct Show {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Show {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Show {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Show",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }

    async fn execute(&mut self) -> Next {
        self.runtime.sprite.write().await.set_hide(HideStatus::Show);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetEffectTo {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SetEffectTo {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for SetEffectTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetEffectTo",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct NextCostume {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl NextCostume {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for NextCostume {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "NextCostume",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct ChangeEffectBy {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl ChangeEffectBy {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for ChangeEffectBy {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeEffectBy",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct SetSizeTo {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SetSizeTo {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for SetSizeTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetSizeTo",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct SwitchCostumeTo {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SwitchCostumeTo {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for SwitchCostumeTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SwitchCostumeTo",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct Costume {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Costume {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for Costume {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Costume",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}
