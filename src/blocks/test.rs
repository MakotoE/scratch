use super::*;
use crate::broadcaster::BroadcastMsg;

#[derive(Debug, Clone)]
pub struct BlockStub {
    id: BlockID,
    runtime: Runtime,
}

impl BlockStub {
    #[cfg(test)]
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait]
impl Block for BlockStub {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "BlockStub",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    async fn execute(&mut self) -> Result<Next> {
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::BlockStub(self.id, BlockStubMsg::Executed))?;
        Ok(Next::None)
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum BlockStubMsg {
    Executed,
}
