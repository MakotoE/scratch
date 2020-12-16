use crate::broadcaster::BroadcastMsg;

use super::*;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
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
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl WhenFlagClicked {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
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

    async fn execute(&mut self) -> Next {
        if self.runtime.sprite.read().await.is_a_clone() {
            Next::None
        } else {
            Next::continue_(self.next)
        }
    }
}

#[derive(Debug)]
pub struct WhenBroadcastReceived {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    broadcast_id: String,
    started: bool,
}

impl WhenBroadcastReceived {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
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
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("BROADCAST_OPTION", self.broadcast_id.clone())],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "BROADCAST_OPTION" {
            self.broadcast_id = get_field_value(field, 0)?.to_string();
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
                    return Next::loop_(self.next);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Broadcast {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    message: Box<dyn Block>,
}

impl Broadcast {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            message: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for Broadcast {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Broadcast",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("message", &self.message)],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "BROADCAST_INPUT" {
            self.message = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let msg = value_to_string(self.message.value().await?);
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Start(msg))?;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct BroadcastAndWait {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    message: Box<dyn Block>,
}

impl BroadcastAndWait {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            message: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for BroadcastAndWait {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "BroadcastAndWait",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("message", &self.message)],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "BROADCAST_INPUT" {
            self.message = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let msg = value_to_string(self.message.value().await?);
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Start(msg.clone()))?;
        let mut recv = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::Finished(s) = recv.recv().await? {
                if s == msg {
                    return Next::continue_(self.next);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct WhenThisSpriteClicked {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl WhenThisSpriteClicked {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
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

    async fn execute(&mut self) -> Next {
        let mut channel = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::Click(c) = channel.recv().await? {
                if self
                    .runtime
                    .sprite
                    .read()
                    .await
                    .rectangle()
                    .contains(&c.into())
                {
                    return Next::continue_(self.next);
                }
            }
        }
    }
}
