use super::*;

#[derive(Debug)]
pub struct Variable {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
}

impl Variable {
    pub fn new(id: String, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for Variable {
    fn block_name(&self) -> &'static str {
        "Variable"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    fn value(&self) -> Result<serde_json::Value> {
        match self.runtime.borrow().variables.get(&self.id) {
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
    fn block_name(&self) -> &'static str {
        "Value"
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    fn value(&self) -> Result<serde_json::Value> {
        Ok(self.value.clone())
    }
}

impl std::convert::From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        Self { value }
    }
}
