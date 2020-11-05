use super::*;

pub fn get_block(name: &str, id: String, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "play" => Box::new(Play::new(id, runtime)),
        "sounds_menu" => Box::new(SoundsMenu::new(id, runtime)),
        "playuntildone" => Box::new(PlayUntilDone::new(id, runtime)),
        "stopallsounds" => Box::new(StopAllSounds::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Play {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Play {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for Play {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Play",
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
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SoundsMenu {
    id: String,
}

impl SoundsMenu {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for SoundsMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SoundsMenu",
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
pub struct PlayUntilDone {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl PlayUntilDone {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for PlayUntilDone {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "PlayUntilDone",
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

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }
}

#[derive(Debug)]
pub struct StopAllSounds {
    id: String,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl StopAllSounds {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for StopAllSounds {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "StopAllSounds",
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

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }

    async fn execute(&mut self) -> Next {
        Next::continue_(self.next.clone())
    }
}
