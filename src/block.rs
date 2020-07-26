use super::*;

use runtime::SpriteRuntime;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use std::sync::Mutex;

pub trait BlockOrValue<'r>: std::fmt::Debug {
    fn set_arg(&mut self, key: &str, block: Box<dyn Value<'r> + 'r>);
    fn set_field(&mut self, key: &str, value_id: &str);
}

pub trait Block<'r>: BlockOrValue<'r> {
    fn set_input(&mut self, key: &str, block: Rc<RefCell<dyn Block<'r> + 'r>>);
    fn next(&self) -> Option<Rc<RefCell<dyn Block<'r> + 'r>>>;
    fn execute(&mut self) -> Result<()>;
}

pub trait Value<'r>: BlockOrValue<'r> {
    fn value(&self) -> Result<serde_json::Value>;
}

#[derive(Debug)]
pub struct WhenFlagClicked<'r> {
    id: String,
    runtime: &'r Mutex<SpriteRuntime>,
    next: Option<Rc<RefCell<dyn Block<'r> + 'r>>>,
}

impl<'r> WhenFlagClicked<'r> {
    fn new(id: &str, runtime: &'r Mutex<SpriteRuntime>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

impl<'r> BlockOrValue<'r> for WhenFlagClicked<'r> {
    fn set_arg(&mut self, _: &str, _: Box<dyn Value<'r> + 'r>) {}
    fn set_field(&mut self, _: &str, _: &str) {}
}

impl<'r> Block<'r> for WhenFlagClicked<'r> {
    fn set_input(&mut self, key: &str, block: Rc<RefCell<dyn Block<'r> + 'r>>) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn next(&self) -> Option<Rc<RefCell<dyn Block<'r> + 'r>>> {
        self.next.clone()
    }

    fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Say<'r> {
    id: String,
    runtime: &'r Mutex<SpriteRuntime>,
    message: Option<Box<dyn Value<'r> + 'r>>,
    next: Option<Rc<RefCell<dyn Block<'r> + 'r>>>,
}

impl<'r> Say<'r> {
    fn new(id: &str, runtime: &'r Mutex<SpriteRuntime>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            message: None,
            next: None,
        }
    }
}

impl<'r> BlockOrValue<'r> for Say<'r> {
    fn set_arg(&mut self, key: &str, block: Box<dyn Value<'r> + 'r>) {
        if key == "MESSAGE" {
            self.message = Some(block);
        }
    }

    fn set_field(&mut self, _: &str, _: &str) {}
}

impl<'r> Block<'r> for Say<'r> {
    fn set_input(&mut self, key: &str, block: Rc<RefCell<dyn Block<'r> + 'r>>) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn next(&self) -> Option<Rc<RefCell<dyn Block<'r> + 'r>>> {
        self.next.clone()
    }

    fn execute(&mut self) -> Result<()> {
        match &self.message {
            Some(value) => {
                let v = value.value()?;
                let message = v.as_str().ok_or_else(|| Error::from("invalid type"))?;
                self.runtime.lock()?.say(message);
                Ok(())
            }
            None => Err("message is None".into()),
        }
    }
}

#[derive(Debug)]
pub struct SetVariable<'r> {
    id: String,
    runtime: &'r Mutex<SpriteRuntime>,
    variable_id: Option<String>,
    value: Option<Box<dyn Value<'r> + 'r>>,
    next: Option<Rc<RefCell<dyn Block<'r> + 'r>>>,
}

impl<'r> SetVariable<'r> {
    pub fn new(id: &str, runtime: &'r Mutex<SpriteRuntime>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            variable_id: None,
            value: None,
            next: None,
        }
    }
}

impl<'r> BlockOrValue<'r> for SetVariable<'r> {
    fn set_arg(&mut self, key: &str, value: Box<dyn Value<'r> + 'r>) {
        if key == "VALUE" {
            self.value = Some(value);
        }
    }

    fn set_field(&mut self, key: &str, value_id: &str) {
        if key == "VARIABLE" {
            self.variable_id = Some(value_id.to_string());
        }
    }
}

impl<'r> Block<'r> for SetVariable<'r> {
    fn set_input(&mut self, key: &str, block: Rc<RefCell<dyn Block<'r> + 'r>>) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn next(&self) -> Option<Rc<RefCell<dyn Block<'r> + 'r>>> {
        self.next.clone()
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
            .lock()?
            .variables
            .insert(variable_id.clone(), value.clone());
        Ok(())
    }
}

#[derive(Debug)]
pub struct Variable<'r> {
    id: String,
    runtime: &'r Mutex<SpriteRuntime>,
}

impl<'r> Variable<'r> {
    pub fn new(id: &str, runtime: &'r Mutex<SpriteRuntime>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
        }
    }
}

impl<'r> BlockOrValue<'r> for Variable<'r> {
    fn set_arg(&mut self, _: &str, _: Box<dyn Value<'r> + 'r>) {}
    fn set_field(&mut self, _: &str, _: &str) {}
}

impl<'r> Value<'r> for Variable<'r> {
    fn value(&self) -> Result<serde_json::Value> {
        match self.runtime.lock()?.variables.get(&self.id) {
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

impl<'r> BlockOrValue<'r> for Number {
    fn set_arg(&mut self, _: &str, _: Box<dyn Value<'r> + 'r>) {}

    fn set_field(&mut self, _: &str, _: &str) {}
}

impl<'r> Value<'r> for Number {
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

impl<'r> BlockOrValue<'r> for BlockString {
    fn set_arg(&mut self, _: &str, _: Box<dyn Value<'r> + 'r>) {}
    fn set_field(&mut self, _: &str, _: &str) {}
}

impl<'r> Value<'r> for BlockString {
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
pub struct Equals<'r> {
    operand1: Option<Box<dyn Value<'r> + 'r>>,
    operand2: Option<Box<dyn Value<'r> + 'r>>,
}

impl<'r> BlockOrValue<'r> for Equals<'r> {
    fn set_arg(&mut self, key: &str, value: Box<dyn Value<'r> + 'r>) {
        match key {
            "OPERAND1" => self.operand1 = Some(value),
            "OPERAND2" => self.operand2 = Some(value),
            _ => {},
        }
    }

    fn set_field(&mut self, _: &str, _: &str) {}
}

impl<'r> Value<'r> for Equals<'r> {
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

pub fn new_block<'r>(
    block_id: &str,
    runtime: &'r Mutex<SpriteRuntime>,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Rc<RefCell<dyn Block<'r> + 'r>>> {
    let info = infos.get(block_id).unwrap();
    let block = get_block(block_id, runtime, &info)?;
    if let Some(next_id) = &info.next {
        block
            .borrow_mut()
            .set_input("next", new_block(next_id, runtime, infos)?);
    }
    for (k, input) in &info.inputs {
        let input_err_cb = || Error::from(format!("block \"{}\": invalid {}", block_id, k.as_str()));
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
                block.borrow_mut().set_arg(k, value);
            }
            2 | 3 => {
                let input_info = input_arr.get(1).ok_or_else(input_err_cb)?;
                match input_info {
                    serde_json::Value::String(id) => {
                        let new_block = new_block(id, runtime, infos)?;
                        block.borrow_mut().set_input(k, new_block);
                    }
                    serde_json::Value::Array(arr) => {
                        let id = arr
                            .get(2)
                            .and_then(|v| v.as_str())
                            .ok_or_else(input_err_cb)?;
                        let variable = Box::new(Variable::new(id, runtime));
                        block.borrow_mut().set_arg(k, variable);
                    }
                    _ => return Err(input_err_cb()),
                }
            }
            _ => return Err(format!("block \"{}\": invalid input_type {}", block_id, input_type).into()),
        };
    }
    for (k, field) in &info.fields {
        match field.get(1) {
            Some(value_id) => {
                block.borrow_mut().set_field(k, value_id);
            }
            None => return Err(format!("block \"{}\": invalid field {}", block_id, k).into()),
        }
    }
    Ok(block)
}

pub fn new_value<'r>(value_type: i64, value: serde_json::Value) -> Result<Box<dyn Value<'r> + 'r>> {
    Ok(match value_type {
        4 => Box::new(Number::try_from(value)?),
        10 => Box::new(BlockString::try_from(value)?),
        _ => return Err(format!("value_type {} does not exist", value_type).into()),
    })
}

pub fn get_block<'r>(
    id: &str,
    runtime: &'r Mutex<SpriteRuntime>,
    info: &savefile::Block,
) -> Result<Rc<RefCell<dyn Block<'r> + 'r>>> {
    Ok(match info.opcode.as_str() {
        "event_whenflagclicked" => Rc::new(RefCell::new(WhenFlagClicked::new(id, runtime))),
        "looks_say" => Rc::new(RefCell::new(Say::new(id, runtime))),
        "data_setvariableto" => Rc::new(RefCell::new(SetVariable::new(id, runtime))),
        _ => return Err(format!("block \"{}\": opcode {} does not exist", id, info.opcode).into()),
    })
}
