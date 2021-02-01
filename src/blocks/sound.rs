use super::*;

pub fn get_block(
    name: &str,
    id: BlockID,
    runtime: Runtime,
) -> Result<Box<dyn Block + Send + Sync>> {
    Ok(match name {
        "play" => Box::new(Play::new(id, runtime)),
        "sounds_menu" => Box::new(SoundsMenu::new(id, runtime)),
        "playuntildone" => Box::new(PlayUntilDone::new(id, runtime)),
        "stopallsounds" => Box::new(StopAllSounds::new(id, runtime)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Play {
    id: BlockID,
    next: Option<BlockID>,
}

impl Play {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait]
impl Block for Play {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Play",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SoundsMenu {
    id: BlockID,
}

impl SoundsMenu {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait]
impl Block for SoundsMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SoundsMenu",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block + Send + Sync>) {}
}

#[derive(Debug)]
pub struct PlayUntilDone {
    id: BlockID,
    next: Option<BlockID>,
}

impl PlayUntilDone {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait]
impl Block for PlayUntilDone {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "PlayUntilDone",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct StopAllSounds {
    id: BlockID,
    next: Option<BlockID>,
}

impl StopAllSounds {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait]
impl Block for StopAllSounds {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "StopAllSounds",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        Next::continue_(self.next)
    }
}
