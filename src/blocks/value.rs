use super::*;

#[derive(Debug)]
pub struct Variable {
    id: String,
    runtime: Runtime,
}

impl Variable {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for Variable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Variable",
            id: BlockID::default(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        self.runtime.global.variables.get(&self.id).await
    }
}

#[derive(Debug)]
pub struct Value {
    value: serde_json::Value,
}

#[async_trait(?Send)]
impl Block for Value {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Value",
            id: BlockID::default(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![("value", self.value.to_string())],
            vec![],
            vec![],
        )
    }

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.value.clone())
    }
}

impl std::convert::From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        Self { value }
    }
}
