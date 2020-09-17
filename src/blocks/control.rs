use super::*;
use gloo_timers::future::TimeoutFuture;

pub fn get_block(
    name: &str,
    id: &str,
    _runtime: Rc<RwLock<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "if" => Box::new(If::new(id)),
        "forever" => Box::new(Forever::new(id)),
        "repeat" => Box::new(Repeat::new(id)),
        "wait" => Box::new(Wait::new(id)),
        "repeat_until" => Box::new(RepeatUntil::new(id)),
        "if_else" => Box::new(IfElse::new(id)),
        _ => return Err(format!("{} does not exist", name).into()),
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
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            condition: None,
            next: None,
            substack: None,
            done: false,
        }
    }
}

#[async_trait(?Send)]
impl Block for If {
    fn block_name(&self) -> &'static str {
        "If"
    }

    fn id(&self) -> &str {
        &self.id
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
            None => return Next::Err(format!("expected boolean type but got {}", value).into()),
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
}

impl Wait {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            next: None,
            duration: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Wait {
    fn block_name(&self) -> &'static str {
        "Wait"
    }

    fn id(&self) -> &str {
        &self.id
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
            None => return Next::Err("duration is None".into()),
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
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            substack: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Forever {
    fn block_name(&self) -> &'static str {
        "Forever"
    }

    fn id(&self) -> &str {
        &self.id
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
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            times: None,
            next: None,
            substack: None,
            count: 0,
        }
    }
}

#[async_trait(?Send)]
impl Block for Repeat {
    fn block_name(&self) -> &'static str {
        "Repeat"
    }

    fn id(&self) -> &str {
        &self.id
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
            None => return Next::Err("times is None".into()),
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
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            next: None,
            substack: None,
            condition: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for RepeatUntil {
    fn block_name(&self) -> &'static str {
        "RepeatUntil"
    }

    fn id(&self) -> &str {
        &self.id
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
            None => return Next::Err("condition is None".into()),
        };

        let condition = match condition_value.as_bool() {
            Some(b) => b,
            None => {
                return Next::Err(format!("condition is not boolean: {}", condition_value).into())
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
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
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
    fn block_name(&self) -> &'static str {
        "IfElse"
    }

    fn id(&self) -> &str {
        &self.id
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
            None => return Next::Err("condition is None".into()),
        };

        let condition = match condition_value.as_bool() {
            Some(b) => b,
            None => {
                return Next::Err(format!("condition is not boolean: {}", condition_value).into())
            }
        };

        self.done = true;

        if condition {
            return Next::loop_(self.substack_true.clone());
        }

        Next::loop_(self.substack_false.clone())
    }
}
