use super::*;
use std::convert::TryFrom;

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
pub struct Number {
    value: f64,
}

#[async_trait(?Send)]
impl Block for Number {
    fn block_name(&self) -> &'static str {
        "Number"
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    fn value(&self) -> Result<serde_json::Value> {
        Ok(self.value.into())
    }
}

impl TryFrom<serde_json::Value> for Number {
    type Error = Error;

    fn try_from(v: serde_json::Value) -> Result<Self> {
        Ok(Self {
            value: value_to_float(&v)?,
        })
    }
}

#[derive(Debug)]
pub struct BlockString {
    value: String,
}

#[async_trait(?Send)]
impl Block for BlockString {
    fn block_name(&self) -> &'static str {
        "BlockString"
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    fn value(&self) -> Result<serde_json::Value> {
        Ok(self.value.clone().into())
    }
}

impl TryFrom<serde_json::Value> for BlockString {
    type Error = Error;

    fn try_from(v: serde_json::Value) -> Result<Self> {
        Ok(Self {
            value: v.as_str().ok_or_else(|| wrong_type_err(&v))?.to_string(),
        })
    }
}
