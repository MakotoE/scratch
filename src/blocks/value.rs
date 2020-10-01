use super::*;

#[derive(Debug)]
pub struct Variable {
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
}

impl Variable {
    pub fn new(id: String, runtime: Rc<RwLock<SpriteRuntime>>) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for Variable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Variable",
            id: self.id.to_string(),
        }
    }

    fn inputs(&self) -> Inputs {
        Inputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        match self.runtime.read().await.variables.get(&self.id) {
            Some(v) => Ok(v.clone()),
            None => Err(format!("{} does not exist", self.id).into()),
        }
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
            id: String::new(),
        }
    }

    fn inputs(&self) -> Inputs {
        Inputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.value.clone())
    }
}

impl std::convert::From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        Self { value }
    }
}
