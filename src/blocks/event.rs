use super::*;
use maplit::hashmap;
use runtime::BroadcastMsg;

pub fn get_block(name: &str, id: String, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "whenflagclicked" => Box::new(WhenFlagClicked::new(id, runtime)),
        "whenbroadcastreceived" => Box::new(WhenBroadcastReceived::new(id, runtime)),
        "broadcast" => Box::new(Broadcast::new(id, runtime)),
        "broadcastandwait" => Box::new(BroadcastAndWait::new(id, runtime)),
        "whenthisspriteclicked" => Box::new(WhenThisSpriteClicked::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct WhenFlagClicked {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl WhenFlagClicked {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for WhenFlagClicked {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "WhenFlagClicked",
            id: self.id.to_string(),
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
            Next::None
        } else {
            Next::continue_(self.next.clone())
        }
    }
}

#[derive(Debug)]
pub struct WhenBroadcastReceived {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    broadcast_id: String,
    started: bool,
}

impl WhenBroadcastReceived {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            broadcast_id: String::new(),
            started: false,
        }
    }
}

#[async_trait(?Send)]
impl Block for WhenBroadcastReceived {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "WhenBroadcastReceived",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: hashmap! {"BROADCAST_OPTION" => self.broadcast_id.clone()},
            inputs: HashMap::new(),
            stacks: BlockInputs::stacks(hashmap! {"next" => &self.next}),
        }
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "BROADCAST_OPTION" {
            match field.get(0).cloned().flatten() {
                Some(s) => self.broadcast_id = s,
                None => return Err(wrap_err!("field is invalid")),
            }
        }
        Ok(())
    }

    async fn execute(&mut self) -> Next {
        if self.started {
            self.runtime
                .global
                .broadcaster
                .send(BroadcastMsg::Finished(self.broadcast_id.clone()))?;
            self.started = false;
            return Next::None;
        }

        let mut recv = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::Start(s) = recv.recv().await? {
                if s == self.broadcast_id {
                    self.started = true;
                    return Next::loop_(self.next.clone());
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Broadcast {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    message: Option<Box<dyn Block>>,
}

impl Broadcast {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            message: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Broadcast {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Broadcast",
            id: self.id.clone(),
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
            "BROADCAST_INPUT" => self.message = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let message_block = match &self.message {
            Some(b) => b,
            None => return Next::Err(wrap_err!("message is None")),
        };

        let msg = value_to_string(message_block.value().await?);
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Start(msg))?;
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct BroadcastAndWait {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    message: Option<Box<dyn Block>>,
}

impl BroadcastAndWait {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            message: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for BroadcastAndWait {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "BroadcastAndWait",
            id: self.id.clone(),
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
            "BROADCAST_INPUT" => self.message = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Next {
        let message_block = match &self.message {
            Some(b) => b,
            None => return Next::Err(wrap_err!("message is None")),
        };

        let msg = value_to_string(message_block.value().await?);
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Start(msg.clone()))?;
        let mut recv = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::Finished(s) = recv.recv().await? {
                if s == msg {
                    return Next::continue_(self.next.clone());
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct WhenThisSpriteClicked {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl WhenThisSpriteClicked {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for WhenThisSpriteClicked {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "WhenThisSpriteClicked",
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
        loop {
            let msg = self.runtime.global.broadcaster.subscribe().recv().await?;
            if let BroadcastMsg::Click(c) = msg {
                if self.runtime.sprite.read().await.rectangle().contains(&c) {
                    return Next::continue_(self.next.clone());
                }
            }
        }
    }
}
