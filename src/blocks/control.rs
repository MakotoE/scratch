use super::*;
use gloo_timers::future::TimeoutFuture;
use maplit::hashmap;
use runtime::BroadcastMsg;
use std::str::FromStr;
use vm::ThreadID;

pub fn get_block(name: &str, id: String, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "if" => Box::new(If::new(id)),
        "forever" => Box::new(Forever::new(id)),
        "repeat" => Box::new(Repeat::new(id)),
        "wait" => Box::new(Wait::new(id, runtime)),
        "repeat_until" => Box::new(RepeatUntil::new(id)),
        "if_else" => Box::new(IfElse::new(id)),
        "wait_until" => Box::new(WaitUntil::new(id)),
        "start_as_clone" => Box::new(StartAsClone::new(id, runtime)),
        "delete_this_clone" => Box::new(DeleteThisClone::new(id, runtime)),
        "stop" => Box::new(Stop::new(id, runtime)),
        "create_clone_of" => Box::new(CreateCloneOf::new(id, runtime)),
        "create_clone_of_menu" => Box::new(CreateCloneOfMenu::new(id)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct If {
    id: String,
    condition: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    substack: Option<Rc<RefCell<Box<dyn Block>>>>,
    done: bool,
}

impl If {
    pub fn new(id: String) -> Self {
        Self {
            id,
            condition: None,
            next: None,
            substack: None,
            done: false,
        }
    }
}

#[async_trait(?Send)]
impl Block for If {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "If",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"condition" => &self.condition}),
            stacks: BlockInputs::stacks(
                hashmap! {"next" => &self.next, "substack" => &self.substack},
            ),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "CONDITION" => self.condition = Some(block),
            "SUBSTACK" => self.substack = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        if self.done {
            self.done = false;
            return Next::continue_(self.next.clone());
        }

        let condition = match &self.condition {
            Some(id) => id,
            None => return Next::continue_(self.next.clone()),
        };

        let value = condition.value().await?;
        let value_bool = match value.as_bool() {
            Some(b) => b,
            None => {
                return Next::Err(wrap_err!(format!(
                    "expected boolean type but got {}",
                    value
                )))
            }
        };

        self.done = true;

        if value_bool {
            return Next::loop_(self.substack.clone());
        }

        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct Wait {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    duration: Option<Box<dyn Block>>,
    runtime: Runtime,
}

impl Wait {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            next: None,
            duration: None,
            runtime,
        }
    }
}

#[async_trait(?Send)]
impl Block for Wait {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Wait",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"duration" => &self.duration}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "DURATION" => self.duration = Some(block),
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let duration = match &self.duration {
            Some(block) => value_to_float(&block.value().await?)?,
            None => return Next::Err(wrap_err!("duration is None")),
        };

        TimeoutFuture::new((MILLIS_PER_SECOND * duration).round() as u32).await;
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct Forever {
    id: String,
    substack: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Forever {
    pub fn new(id: String) -> Self {
        Self { id, substack: None }
    }
}

#[async_trait(?Send)]
impl Block for Forever {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Forever",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"substack" => &self.substack}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "SUBSTACK" {
            self.substack = Some(Rc::new(RefCell::new(block)))
        }
    }

    async fn execute(&mut self) -> Next {
        match &self.substack {
            Some(b) => Next::Loop(b.clone()),
            None => Next::None,
        }
    }
}

#[derive(Debug)]
pub struct Repeat {
    id: String,
    times: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    substack: Option<Rc<RefCell<Box<dyn Block>>>>,
    count: usize,
}

impl Repeat {
    pub fn new(id: String) -> Self {
        Self {
            id,
            times: None,
            next: None,
            substack: None,
            count: 0,
        }
    }
}

#[async_trait(?Send)]
impl Block for Repeat {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Repeat",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"times" => &self.times}),
            stacks: BlockInputs::stacks(
                hashmap! {"next" => &self.next, "substack" => &self.substack},
            ),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "TIMES" => self.times = Some(block),
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "SUBSTACK" => self.substack = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let times = match &self.times {
            Some(v) => value_to_float(&v.value().await?)?,
            None => return Next::Err(wrap_err!("times is None")),
        };

        if self.count < times as usize {
            // Loop until count equals times
            self.count += 1;
            return Next::loop_(self.substack.clone());
        }

        self.count = 0;
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct RepeatUntil {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    substack: Option<Rc<RefCell<Box<dyn Block>>>>,
    condition: Option<Box<dyn Block>>,
}

impl RepeatUntil {
    pub fn new(id: String) -> Self {
        Self {
            id,
            next: None,
            substack: None,
            condition: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for RepeatUntil {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "RepeatUntil",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"condition" => &self.condition}),
            stacks: BlockInputs::stacks(
                hashmap! {"next" => &self.next, "substack" => &self.substack},
            ),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "SUBSTACK" => self.substack = Some(Rc::new(RefCell::new(block))),
            "CONDITION" => self.condition = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let condition_value = match &self.condition {
            Some(block) => block.value().await?,
            None => return Next::Err(wrap_err!("condition is None")),
        };

        let condition = match condition_value.as_bool() {
            Some(b) => b,
            None => {
                return Next::Err(wrap_err!(format!(
                    "condition is not boolean: {}",
                    condition_value
                )));
            }
        };

        if condition {
            return Next::continue_(self.next.clone());
        }

        Next::loop_(self.substack.clone())
    }
}

#[derive(Debug)]
pub struct IfElse {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    condition: Option<Box<dyn Block>>,
    substack_true: Option<Rc<RefCell<Box<dyn Block>>>>,
    substack_false: Option<Rc<RefCell<Box<dyn Block>>>>,
    done: bool,
}

impl IfElse {
    pub fn new(id: String) -> Self {
        Self {
            id,
            next: None,
            condition: None,
            substack_true: None,
            substack_false: None,
            done: false,
        }
    }
}

#[async_trait(?Send)]
impl Block for IfElse {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "IfElse",
            id: self.id.to_string(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"condition" => &self.condition}),
            stacks: BlockInputs::stacks(hashmap! {
                "next" => &self.next,
                "substack_true" => &self.substack_true,
                "substack_false" => &self.substack_false,
            }),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "CONDITION" => self.condition = Some(block),
            "SUBSTACK" => self.substack_true = Some(Rc::new(RefCell::new(block))),
            "SUBSTACK2" => self.substack_false = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        if self.done {
            self.done = false;
            return Next::continue_(self.next.clone());
        }

        let condition_value = match &self.condition {
            Some(block) => block.value().await?,
            None => return Next::Err(wrap_err!("condition is None")),
        };

        let condition = match condition_value.as_bool() {
            Some(b) => b,
            None => {
                return Next::Err(wrap_err!(format!(
                    "condition is not boolean: {}",
                    condition_value
                )))
            }
        };

        self.done = true;

        if condition {
            return Next::loop_(self.substack_true.clone());
        }

        Next::loop_(self.substack_false.clone())
    }
}

#[derive(Debug)]
pub struct WaitUntil {
    id: String,
}

impl WaitUntil {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for WaitUntil {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "WaitUntil",
            id: self.id.clone(),
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
}

#[derive(Debug)]
pub struct StartAsClone {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl StartAsClone {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for StartAsClone {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "StartAsClone",
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
        if self.runtime.sprite.read().await.is_a_clone() {
            Next::continue_(self.next.clone())
        } else {
            Next::None
        }
    }
}

#[derive(Debug)]
pub struct DeleteThisClone {
    id: String,
    runtime: Runtime,
}

impl DeleteThisClone {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for DeleteThisClone {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "DeleteThisClone",
            id: self.id.clone(),
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

    async fn execute(&mut self) -> Next {
        let sprite_id = self.runtime.sprite.read().await.sprite_id();
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::DeleteClone(sprite_id))?;
        Next::None
    }
}

#[derive(Debug)]
pub struct Stop {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    stop_option: Option<StopOption>,
}

impl Stop {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            stop_option: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Stop {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Stop",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: match self.stop_option {
                Some(o) => hashmap! {"stop_option" => format!("{:?}", o)},
                None => HashMap::new(),
            },
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "STOP_OPTION" {
            if let Some(o) = field.get(0) {
                if let Some(s) = o {
                    self.stop_option = Some(StopOption::from_str(s)?);
                }
            }
        }
        Ok(())
    }

    async fn execute(&mut self) -> Next {
        match self.stop_option {
            Some(s) => self
                .runtime
                .global
                .broadcaster
                .send(BroadcastMsg::Stop(s.into_stop(self.runtime.thread_id())))?,
            None => return Next::Err(wrap_err!("stop_option is None")),
        };
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug, Copy, Clone)]
enum StopOption {
    All,
    ThisThread,
    OtherThreads,
}

impl StopOption {
    fn into_stop(self, thread_id: ThreadID) -> runtime::Stop {
        match self {
            StopOption::All => runtime::Stop::All,
            StopOption::ThisThread => runtime::Stop::ThisThread(thread_id),
            StopOption::OtherThreads => runtime::Stop::OtherThreads(thread_id),
        }
    }
}

impl FromStr for StopOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "all" => StopOption::All,
            "this script" => StopOption::ThisThread,
            "other scripts in sprite" => StopOption::OtherThreads,
            _ => return Err(wrap_err!(format!("s is invalid: {}", s))),
        })
    }
}

#[derive(Debug)]
pub struct CreateCloneOf {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    clone_option: Option<Box<dyn Block>>,
}

impl CreateCloneOf {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            clone_option: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for CreateCloneOf {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "CreateCloneOf",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: BlockInputs::inputs(hashmap! {"clone_option" => &self.clone_option}),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "CLONE_OPTION" => self.clone_option = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let sprite_id = self.runtime.sprite.read().await.sprite_id();
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Clone(sprite_id))?;
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct CreateCloneOfMenu {
    id: String,
}

impl CreateCloneOfMenu {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for CreateCloneOfMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "CreateCloneOfMenu",
            id: self.id.clone(),
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
}
