use crate::broadcaster::BroadcastMsg;

use super::*;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "whenflagclicked" => Box::new(WhenFlagClicked::new(id, runtime)),
        "whenbroadcastreceived" => Box::new(WhenBroadcastReceived::new(id, runtime)),
        "broadcast" => Box::new(Broadcast::new(id, runtime)),
        "broadcastandwait" => Box::new(BroadcastAndWait::new(id, runtime)),
        "whenthisspriteclicked" => Box::new(WhenThisSpriteClicked::new(id, runtime)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
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

#[async_trait]
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

    async fn execute(&mut self) -> Result<Next> {
        if self.runtime.sprite.read().await.is_a_clone() {
            Ok(Next::None)
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

#[async_trait]
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

    async fn execute(&mut self) -> Result<Next> {
        if self.started {
            self.runtime
                .global
                .broadcaster
                .send(BroadcastMsg::Finished(self.broadcast_id.clone()))?;
            self.started = false;
            return Ok(Next::None);
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

#[async_trait]
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
            vec![("BROADCAST_INPUT", self.message.as_ref())],
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

    async fn execute(&mut self) -> Result<Next> {
        let msg = self.message.value().await?.to_string();
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

#[async_trait]
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
            vec![("BROADCAST_INPUT", self.message.as_ref())],
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

    async fn execute(&mut self) -> Result<Next> {
        let msg = self.message.value().await?.to_string();
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

#[async_trait]
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

    async fn execute(&mut self) -> Result<Next> {
        let mut channel = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::MouseClick(c) = channel.recv().await? {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::blocks::test::{BlockStub, BlockStubMsg};
    use crate::blocks::value::ValueString;
    use crate::coordinate::SpriteCoordinate;
    use crate::file::BlockIDGenerator;
    use crate::thread::{StepStatus, Thread};
    use futures::future::FutureExt;

    #[tokio::test]
    async fn when_flag_clicked() {
        let mut gen = BlockIDGenerator::new();
        {
            let runtime = Runtime::default();
            let mut when_flag_clicked = WhenFlagClicked::new(gen.get_id(), runtime.clone());
            when_flag_clicked.set_substack("next", BlockID::default());
            assert_eq!(
                when_flag_clicked.execute().await.unwrap(),
                Next::Continue(BlockID::default())
            );
        }

        {
            let mut runtime = Runtime::default();
            let new_runtime = runtime.sprite.read().await.clone_sprite_runtime();
            runtime.sprite = Arc::new(RwLock::new(new_runtime));
            let mut when_flag_clicked = WhenFlagClicked::new(gen.get_id(), runtime.clone());
            assert_eq!(when_flag_clicked.execute().await.unwrap(), Next::None);
        }
    }

    #[tokio::test]
    async fn when_broadcast_received() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();
        let when_broadcast_received_id = gen.get_id();
        let next_id = gen.get_id();

        let mut when_broadcast_received =
            WhenBroadcastReceived::new(when_broadcast_received_id, runtime.clone());
        const BROADCAST_ID: &str = "broadcast_id";
        when_broadcast_received
            .set_field("BROADCAST_OPTION", &[Some(BROADCAST_ID.to_string())])
            .unwrap();
        when_broadcast_received.set_substack("next", next_id);

        let blocks = block_map(vec![
            (
                when_broadcast_received_id,
                Box::new(when_broadcast_received),
            ),
            (next_id, Box::new(BlockStub::new(next_id, runtime.clone()))),
        ]);

        let mut thread = Thread::new(when_broadcast_received_id, blocks);

        // WhenBroadcastReceived
        let mut step_future = thread.step().boxed_local();
        assert!((&mut step_future).now_or_never().is_none());

        runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Start(BROADCAST_ID.to_string()))
            .unwrap();
        step_future.await.unwrap();

        // BlockStub
        thread.step().await.unwrap();

        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::Start(BROADCAST_ID.to_string())
        );
        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
        );

        // WhenBroadcastReceived
        assert_eq!(thread.step().await.unwrap(), StepStatus::Done);
        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::Finished(BROADCAST_ID.to_string())
        );
    }

    #[tokio::test]
    async fn broadcast() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();

        let mut broadcast = Broadcast::new(gen.get_id(), runtime.clone());
        const MESSAGE: &str = "message";
        broadcast.set_input(
            "BROADCAST_INPUT",
            Box::new(ValueString::new(MESSAGE.to_string())),
        );
        broadcast.execute().await.unwrap();

        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::Start(MESSAGE.to_string())
        );
    }

    #[tokio::test]
    async fn broadcast_and_wait() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();

        let mut broadcast_and_wait = BroadcastAndWait::new(gen.get_id(), runtime.clone());
        const MESSAGE: &str = "message";
        broadcast_and_wait.set_input(
            "BROADCAST_INPUT",
            Box::new(ValueString::new(MESSAGE.to_string())),
        );
        let mut execute_future = broadcast_and_wait.execute().boxed_local();
        assert!((&mut execute_future).now_or_never().is_none());

        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::Start(MESSAGE.to_string())
        );

        runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Finished(MESSAGE.to_string()))
            .unwrap();

        execute_future.await.unwrap();
    }

    #[tokio::test]
    async fn when_this_sprite_clicked() {
        let runtime = Runtime::default();

        let mut gen = BlockIDGenerator::new();
        let mut when_this_sprite_clicked =
            WhenThisSpriteClicked::new(gen.get_id(), runtime.clone());
        let mut execute_future = when_this_sprite_clicked.execute().boxed_local();
        assert!((&mut execute_future).now_or_never().is_none());

        runtime
            .global
            .broadcaster
            .send(BroadcastMsg::MouseClick(
                SpriteCoordinate { x: 0.0, y: 0.0 }.into(),
            ))
            .unwrap();

        execute_future.await.unwrap();
    }
}
