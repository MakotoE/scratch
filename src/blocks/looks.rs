use super::*;
use gloo_timers::future::TimeoutFuture;

pub fn get_block(
    name: &str,
    id: &str,
    runtime: Rc<RwLock<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "say" => Box::new(looks::Say::new(id, runtime)),
        "sayforsecs" => Box::new(looks::SayForSecs::new(id, runtime)),
        _ => return Err(format!("{} does not exist", name).into()),
    })
}

#[derive(Debug)]
pub struct Say {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    message: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Say {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            message: None,
            next: None,
        }
    }

    fn value_to_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            _ => value.to_string(),
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

    fn inputs(&self) -> Inputs {
        Inputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: Inputs::inputs(hashmap! {"message" => &self.message}),
            stacks: Inputs::stacks(hashmap! {"next" => &self.next}),
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
            Some(b) => Say::value_to_string(&b.value().await?),
            None => return Next::Err("message is None".into()),
        };
        self.runtime.write().await.say(Some(&message));
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SayForSecs {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    message: Option<Box<dyn Block>>,
    secs: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SayForSecs {
    pub fn new(id: &str, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
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

    fn inputs(&self) -> Inputs {
        Inputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: Inputs::inputs(hashmap! {"message" => &self.message, "secs" => &self.secs}),
            stacks: Inputs::stacks(hashmap! {"next" => &self.next}),
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
            Some(b) => Say::value_to_string(&b.value().await?),
            None => return Next::Err("message is None".into()),
        };

        let seconds = match &self.secs {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Next::Err("secs is None".into()),
        };

        let mut runtime = self.runtime.write().await;
        runtime.say(Some(&message));
        runtime.redraw()?;
        TimeoutFuture::new((MILLIS_PER_SECOND * seconds).round() as u32).await;
        Next::continue_(self.next.clone())
    }
}
