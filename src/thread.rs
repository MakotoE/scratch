use super::*;
use crate::blocks::{Block, BlockInfo, BlockInputsPartial, Next};
use crate::file::BlockID;

#[derive(Debug)]
pub struct Thread {
    blocks: HashMap<BlockID, Box<dyn Block>>,
    curr_block: BlockID,
    loop_stack: Vec<BlockID>,
    done: bool,
}

impl Thread {
    pub fn new(hat: BlockID, blocks: HashMap<BlockID, Box<dyn Block>>) -> Self {
        Thread {
            blocks,
            curr_block: hat,
            loop_stack: Vec::new(),
            done: false,
        }
    }

    pub async fn step(&mut self) -> Result<StepStatus> {
        if self.done {
            return Err(Error::msg("this thread already ended"));
        }

        let curr_block = self.curr_block;
        let block = self
            .blocks
            .get_mut(&curr_block)
            .ok_or_else(|| Error::msg(format!("{} does not exist", &curr_block)))?;

        let execute_result = block.execute().await.map_err(|error| ScratchError::Block {
            id: block.block_info().id,
            name: block.block_info().name,
            error,
        })?;

        match execute_result {
            Next::None => match self.loop_stack.pop() {
                None => {
                    self.done = true;
                    return Ok(StepStatus::Done);
                }
                Some(b) => self.curr_block = b,
            },
            Next::Continue(b) => self.curr_block = b,
            Next::Loop(b) => {
                self.loop_stack.push(self.curr_block);
                self.curr_block = b;
            }
        }

        Ok(StepStatus::Continue)
    }

    pub fn block_inputs(&self) -> Result<BlockInputs> {
        let block = self
            .blocks
            .get(&self.curr_block)
            .ok_or_else(|| Error::msg(format!("{} does not exist", &self.curr_block)))?;
        Ok(BlockInputs::new(block.block_inputs(), &self.blocks))
    }

    pub fn block_info(&self) -> Result<BlockInfo> {
        self.blocks
            .get(&self.curr_block)
            .map(|b| b.block_info())
            .ok_or_else(|| Error::msg(format!("{} does not exist", &self.curr_block)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInputs {
    pub info: BlockInfo,
    pub fields: HashMap<&'static str, String>,
    pub inputs: HashMap<&'static str, BlockInputs>,
    pub stacks: HashMap<&'static str, BlockInputs>,
}

impl BlockInputs {
    fn new(
        mut block_inputs: BlockInputsPartial,
        blocks: &HashMap<BlockID, Box<dyn Block>>,
    ) -> Self {
        Self {
            info: block_inputs.info,
            fields: block_inputs.fields,
            inputs: block_inputs
                .inputs
                .drain()
                .map(|(id, inputs)| {
                    // Input blocks should have empty stacks
                    assert_eq!(inputs.stacks.len(), 0);
                    (id, BlockInputs::new(inputs, blocks))
                })
                .collect(),
            stacks: block_inputs
                .stacks
                .iter()
                .map(|(id, block_id)| {
                    let block_inputs = blocks.get(block_id).unwrap().block_inputs();
                    (*id, BlockInputs::new(block_inputs, blocks))
                })
                .collect(),
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum StepStatus {
    Continue,
    Done,
}

#[cfg(test)]
mod test {
    use super::*;

    mod thread {
        use super::*;
        use crate::blocks::block_map;
        use crate::blocks::test::{BlockStub, BlockStubMsg};
        use crate::broadcaster::BroadcastMsg;
        use crate::file::BlockIDGenerator;
        use crate::runtime::Runtime;

        #[tokio::test]
        async fn step() {
            let runtime = Runtime::default();
            let mut receiver = runtime.global.broadcaster.subscribe();
            let mut gen = BlockIDGenerator::new();
            let block0_id = gen.get_id();
            let block1_id = gen.get_id();

            // Empty block map
            {
                let mut thread = Thread::new(BlockID::default(), HashMap::default());
                assert!(thread.step().await.is_err());
            }

            // Next::None result
            {
                let blocks = block_map(vec![(
                    block0_id,
                    Box::new(BlockStub::with_behavior(
                        block0_id,
                        runtime.clone(),
                        None,
                        Arc::new(RwLock::new(Next::None)),
                    )),
                )]);

                let mut thread = Thread::new(block0_id, blocks);
                assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));
                assert!(thread.step().await.is_err());

                assert_eq!(
                    receiver.try_recv().unwrap(),
                    BroadcastMsg::BlockStub(block0_id, BlockStubMsg::Executed)
                );
                assert!(receiver.try_recv().is_err());
            }

            // Continue
            {
                let blocks = block_map(vec![
                    (
                        block0_id,
                        Box::new(BlockStub::with_behavior(
                            block0_id,
                            runtime.clone(),
                            None,
                            Arc::new(RwLock::new(Next::Continue(block1_id))),
                        )),
                    ),
                    (
                        block1_id,
                        Box::new(BlockStub::with_behavior(
                            block1_id,
                            runtime.clone(),
                            None,
                            Arc::new(RwLock::new(Next::None)),
                        )),
                    ),
                ]);

                let mut thread = Thread::new(block0_id, blocks);
                assert!(matches!(thread.step().await.unwrap(), StepStatus::Continue));
                assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));
                assert!(thread.step().await.is_err());

                assert_eq!(
                    receiver.try_recv().unwrap(),
                    BroadcastMsg::BlockStub(block0_id, BlockStubMsg::Executed)
                );
                assert_eq!(
                    receiver.try_recv().unwrap(),
                    BroadcastMsg::BlockStub(block1_id, BlockStubMsg::Executed)
                );
                assert!(receiver.try_recv().is_err());
            }

            // Loop
            {
                let next = Arc::new(RwLock::new(Next::Loop(block1_id)));

                let blocks = block_map(vec![
                    (
                        block0_id,
                        Box::new(BlockStub::with_behavior(
                            block0_id,
                            runtime.clone(),
                            None,
                            next.clone(),
                        )),
                    ),
                    (
                        block1_id,
                        Box::new(BlockStub::with_behavior(
                            block1_id,
                            runtime.clone(),
                            None,
                            Arc::new(RwLock::new(Next::None)),
                        )),
                    ),
                ]);

                let mut thread = Thread::new(block0_id, blocks);
                assert!(matches!(thread.step().await.unwrap(), StepStatus::Continue));
                assert!(matches!(thread.step().await.unwrap(), StepStatus::Continue));
                assert!(matches!(thread.step().await.unwrap(), StepStatus::Continue));
                assert!(matches!(thread.step().await.unwrap(), StepStatus::Continue));

                *next.write().await = Next::None;
                assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

                assert_eq!(
                    receiver.try_recv().unwrap(),
                    BroadcastMsg::BlockStub(block0_id, BlockStubMsg::Executed)
                );
                assert_eq!(
                    receiver.try_recv().unwrap(),
                    BroadcastMsg::BlockStub(block1_id, BlockStubMsg::Executed)
                );
                assert_eq!(
                    receiver.try_recv().unwrap(),
                    BroadcastMsg::BlockStub(block0_id, BlockStubMsg::Executed)
                );
                assert_eq!(
                    receiver.try_recv().unwrap(),
                    BroadcastMsg::BlockStub(block1_id, BlockStubMsg::Executed)
                );
                assert_eq!(
                    receiver.try_recv().unwrap(),
                    BroadcastMsg::BlockStub(block0_id, BlockStubMsg::Executed)
                );
                assert!(receiver.try_recv().is_err());
            }
        }
    }
}
