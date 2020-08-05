use super::*;

use runtime::SpriteRuntime;
use std::convert::TryFrom;

pub trait Block: std::fmt::Debug {
    fn set_input(&mut self, key: &str, block: Box<dyn Block>);
    fn set_field(&mut self, key: &str, value_id: String);

    fn next(&self) -> Result<Option<Rc<RefCell<Box<dyn Block>>>>> {
        Ok(None)
    }

    fn value(&self) -> Result<serde_json::Value> {
        Err("this block does not return a value".into())
    }

    fn execute(&mut self) -> Result<()> {
        Err("this block cannot be executed".into())
    }
}

#[derive(Debug)]
pub struct WhenFlagClicked {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl WhenFlagClicked {
    fn new(id: String, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

impl Block for WhenFlagClicked {
    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "next" {
            self.next = Some(Rc::new(RefCell::new(block)));
        }
    }

    fn set_field(&mut self, _: &str, _: String) {}

    fn next(&self) -> Result<Option<Rc<RefCell<Box<dyn Block>>>>> {
        Ok(self.next.clone())
    }

    fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Say {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    message: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl Say {
    fn new(id: String, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            message: None,
            next: None,
        }
    }
}

impl Block for Say {
    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "MESSAGE" => self.message = Some(block),
            _ => {}
        }
    }

    fn set_field(&mut self, _: &str, _: String) {}

    fn next(&self) -> Result<Option<Rc<RefCell<Box<dyn Block>>>>> {
        Ok(self.next.clone())
    }

    fn execute(&mut self) -> Result<()> {
        match &self.message {
            Some(value) => {
                let v = value.value()?;
                let message = v.as_str().ok_or_else(|| Error::from("invalid type"))?;
                self.runtime.borrow().say(message)?;
                Ok(())
            }
            None => Err("message is None".into()),
        }
    }
}

#[derive(Debug)]
pub struct SetVariable {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    variable_id: Option<String>,
    value: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SetVariable {
    pub fn new(id: String, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id,
            runtime,
            variable_id: None,
            value: None,
            next: None,
        }
    }
}

impl Block for SetVariable {
    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "VALUE" => self.value = Some(block),
            _ => {}
        }
    }

    fn set_field(&mut self, key: &str, value_id: String) {
        if key == "VARIABLE" {
            self.variable_id = Some(value_id.to_string());
        }
    }

    fn next(&self) -> Result<Option<Rc<RefCell<Box<dyn Block>>>>> {
        Ok(self.next.clone())
    }

    fn execute(&mut self) -> Result<()> {
        let variable_id = match &self.variable_id {
            Some(id) => id,
            None => return Err("variable_id is None".into()),
        };
        let value = match &self.value {
            Some(v) => v.value()?,
            None => return Err("value is None".into()),
        };
        self.runtime
            .borrow_mut()
            .variables
            .insert(variable_id.clone(), value.clone());
        Ok(())
    }
}

#[derive(Debug)]
pub struct If {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    condition: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
    substack: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl If {
    pub fn new(id: String, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id,
            runtime,
            condition: None,
            next: None,
            substack: None,
        }
    }
}

impl Block for If {
    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "CONDITION" => self.condition = Some(block),
            "SUBSTACK" => self.substack = Some(Rc::new(RefCell::new(block))),
            _ => {}
        }
    }

    fn set_field(&mut self, _: &str, _: String) {}

    fn next(&self) -> Result<Option<Rc<RefCell<Box<dyn Block>>>>> {
        let condition = match &self.condition {
            Some(id) => id,
            None => return Ok(self.next.clone()),
        };

        let value = condition.value()?;
        let value_bool = match value.as_bool() {
            Some(b) => b,
            None => return Err(format!("expected boolean type but got {}", value).into()),
        };

        if value_bool {
            return Ok(self.substack.clone());
        }

        return Ok(self.next.clone());
    }

    fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

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

impl Block for Variable {
    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
    fn set_field(&mut self, _: &str, _: String) {}

    fn value(&self) -> Result<serde_json::Value> {
        match self.runtime.borrow().variables.get(&self.id) {
            Some(v) => Ok(v.clone()),
            None => Err(format!("{} does not exist", self.id).into()),
        }
    }
}

fn wrong_type_err(value: &serde_json::Value) -> Error {
    format!("value has wrong type: {:?}", value).into()
}

#[derive(Debug)]
pub struct Number {
    value: f64,
}

impl Block for Number {
    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
    fn set_field(&mut self, _: &str, _: String) {}

    fn value(&self) -> Result<serde_json::Value> {
        Ok(self.value.into())
    }
}

impl TryFrom<serde_json::Value> for Number {
    type Error = Error;

    fn try_from(v: serde_json::Value) -> Result<Self> {
        Ok(Self {
            value: v.as_f64().ok_or_else(|| wrong_type_err(&v))?,
        })
    }
}

#[derive(Debug)]
pub struct BlockString {
    value: String,
}

impl Block for BlockString {
    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
    fn set_field(&mut self, _: &str, _: String) {}

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

#[derive(Debug)]
pub struct Equals {
    id: String,
    operand1: Option<Box<dyn Block>>,
    operand2: Option<Box<dyn Block>>,
}

impl Equals {
    fn new(id: String) -> Self {
        Self {
            id,
            operand1: None,
            operand2: None,
        }
    }
}

impl Block for Equals {
    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "OPERAND1" => self.operand1 = Some(block),
            "OPERAND2" => self.operand2 = Some(block),
            _ => {}
        }
    }

    fn set_field(&mut self, _: &str, _: String) {}

    fn value(&self) -> Result<serde_json::Value> {
        let a = match &self.operand1 {
            Some(a) => a.value()?,
            None => return Err("operand1 is None".into()),
        };
        let b = match &self.operand2 {
            Some(b) => b.value()?,
            None => return Err("operand2 is None".into()),
        };
        Ok((a == b).into())
    }
}

pub fn new_block(
    block_id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block>> {
    let info = infos.get(block_id.as_str()).unwrap();
    let mut block = get_block(block_id.clone(), runtime.clone(), &info)?;
    if let Some(next_id) = &info.next {
        block.set_input("next", new_block(next_id.clone(), runtime.clone(), infos)?);
    }
    for (k, input) in &info.inputs {
        let input_err_cb =
            || Error::from(format!("block \"{}\": invalid {}", block_id, k.as_str()));
        let input_arr = input.as_array().ok_or_else(input_err_cb)?;
        let input_type = input_arr
            .get(0)
            .and_then(|v| v.as_i64())
            .ok_or_else(input_err_cb)?;
        match input_type {
            1 => {
                // value
                let value_info = input_arr
                    .get(1)
                    .and_then(|v| v.as_array())
                    .ok_or_else(input_err_cb)?;
                let value_type = value_info
                    .get(0)
                    .and_then(|v| v.as_i64())
                    .ok_or_else(input_err_cb)?;
                let js_value = value_info.get(1).ok_or_else(input_err_cb)?;
                let value = new_value(value_type, js_value.clone())
                    .map_err(|e| format!("block \"{}\": {}", block_id, e.to_string()))?;
                block.set_input(k, value);
            }
            2 | 3 => {
                let input_info = input_arr.get(1).ok_or_else(input_err_cb)?;
                match input_info {
                    serde_json::Value::String(id) => {
                        let new_block = new_block(id.clone(), runtime.clone(), infos)?;
                        block.set_input(k, new_block);
                    }
                    serde_json::Value::Array(arr) => {
                        let id = arr
                            .get(2)
                            .and_then(|v| v.as_str())
                            .ok_or_else(input_err_cb)?;
                        let variable = Box::new(Variable::new(id.to_string(), runtime.clone()));
                        block.set_input(k, variable);
                    }
                    _ => return Err(input_err_cb()),
                }
            }
            _ => {
                return Err(
                    format!("block \"{}\": invalid input_type {}", block_id, input_type).into(),
                )
            }
        };
    }
    for (k, field) in &info.fields {
        match field.get(1) {
            Some(value_id) => {
                block.set_field(k, value_id.clone());
            }
            None => return Err(format!("block \"{}\": invalid field {}", block_id, k).into()),
        }
    }
    Ok(block)
}

pub fn new_value(value_type: i64, value: serde_json::Value) -> Result<Box<dyn Block>> {
    Ok(match value_type {
        4 => Box::new(Number::try_from(value)?),
        10 => Box::new(BlockString::try_from(value)?),
        _ => return Err(format!("value_type {} does not exist", value_type).into()),
    })
}

pub fn get_block(
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    info: &savefile::Block,
) -> Result<Box<dyn Block>> {
    Ok(match info.opcode.as_str() {
        "event_whenflagclicked" => Box::new(WhenFlagClicked::new(id, runtime)),
        "looks_say" => Box::new(Say::new(id, runtime)),
        "data_setvariableto" => Box::new(SetVariable::new(id, runtime)),
        "operator_equals" => Box::new(Equals::new(id)),
        "control_if" => Box::new(If::new(id, runtime)),
        _ => return Err(format!("block \"{}\": opcode {} does not exist", id, info.opcode).into()),
    })
}
