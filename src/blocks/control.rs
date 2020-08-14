use super::*;
use gloo_timers::future::TimeoutFuture;

pub fn get_block(name: &str, id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Result<Box<dyn Block>>  {
    Ok(match name {
        "if" => Box::new(If::new(id, runtime)),
        "forever" => Box::new(Forever::new(id)),
        "repeat" => Box::new(Repeat::new(id)),
        "wait" => Box::new(Wait::new(id)),
        _ => return Err(format!("block \"{}\": name {} does not exist", id, name).into()),
    })
}

#[derive(Debug)]
pub struct If {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    condition: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    substack: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl If {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            condition: None,
            next: None,
            substack: None,
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

    fn next(&mut self) -> Next {
        let condition = match &self.condition {
            Some(id) => id,
            None => return self.next.clone().into(),
        };

        let value = condition.value()?;
        let value_bool = match value.as_bool() {
            Some(b) => b,
            None => return Next::Err(format!("expected boolean type but got {}", value).into()),
        };

        if value_bool {
            return self.substack.clone().into();
        }

        return self.next.clone().into();
    }

    async fn execute(&mut self) -> Result<()> {
        Ok(())
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

    fn next(&mut self) -> Next {
        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        let duration = match &self.duration {
            Some(block) => value_to_float(&block.value()?)?,
            None => return Err("duration is None".into()),
        };

        TimeoutFuture::new((MILLIS_PER_SECOND * duration).round() as u32).await;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Forever {
    id: String,
    substack: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Forever {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string(), substack: None }
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
        match key {
            "SUBSTACK" => self.substack = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    fn next(&mut self) -> Next {
        match &self.substack {
            Some(b) => Next::Loop(b.clone()),
            None => Next::None,
        }
    }

    async fn execute(&mut self) -> Result<()> {
        Ok(())
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

    fn next(&mut self) -> Next {
        let times = match &self.times {
            Some(v) => value_to_float(&v.value()?)?,
            None => return Next::Err("times is None".into()),
        };

        if self.count < times as usize {
            // Loop until count equals times
            self.count += 1;
            return match &self.substack {
                Some(b) => Next::Loop(b.clone()),
                None => Next::None,
            };
        }

        self.next.clone().into()
    }

    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}
