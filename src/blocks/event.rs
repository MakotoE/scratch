use super::*;
use maplit::hashmap;

pub fn get_block(name: &str, id: String, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "whenflagclicked" => Box::new(WhenFlagClicked::new(id, runtime)),
        "whenbroadcastreceived" => Box::new(WhenBroadcastReceived::new(id, runtime)),
        "broadcast" => Box::new(Broadcast::new(id, runtime)),
        "broadcastandwait" => Box::new(BroadcastAndWait::new(id, runtime)),
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
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct WhenBroadcastReceived {
    id: String,
    runtime: Runtime,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    broadcast_id: String,
}

impl WhenBroadcastReceived {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            broadcast_id: String::new(),
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

    fn set_field(&mut self, key: &str, value_id: String) {
        if key == "BROADCAST_OPTION" {
            self.broadcast_id = value_id;
        }
    }

    async fn execute(&mut self) -> Next {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Broadcast {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Broadcast {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
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
pub struct BroadcastAndWait {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl BroadcastAndWait {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
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
