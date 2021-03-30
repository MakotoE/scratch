use super::*;
use crate::broadcaster::BroadcastMsg;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BlockStub {
    id: BlockID,
    runtime: Runtime,
    return_value: Option<Arc<RwLock<Value>>>,
    next: Arc<RwLock<Next>>,
}

impl BlockStub {
    #[cfg(test)]
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            return_value: None,
            next: Arc::new(RwLock::new(Next::None)),
        }
    }

    #[cfg(test)]
    pub fn with_behavior(
        id: BlockID,
        runtime: Runtime,
        return_value: Option<Arc<RwLock<Value>>>,
        next: Arc<RwLock<Next>>,
    ) -> Self {
        Self {
            id,
            runtime,
            return_value,
            next,
        }
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

    async fn value(&self) -> Result<Value> {
        match &self.return_value {
            Some(return_value) => Ok(return_value.read().await.clone()),
            None => Err(Error::msg("unexpected value() call")),
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::BlockStub(self.id, BlockStubMsg::Executed))?;
        Ok(*self.next.read().await)
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum BlockStubMsg {
    Executed,
}
