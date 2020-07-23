use super::*;

#[allow(unused_imports)]
use log::info;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use std::sync::Mutex;

pub trait BlockOrValue<'r>: std::fmt::Debug {
    fn set_arg(&mut self, key: &str, block: Box<dyn Value<'r> + 'r>);
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
pub struct Runtime {
    pub canvas: web_sys::CanvasRenderingContext2d,
}

#[derive(Debug)]
pub struct WhenFlagClicked<'r> {
    id: String,
    runtime: &'r Mutex<Runtime>,
    next: Option<Rc<RefCell<dyn Block<'r> + 'r>>>,
}

impl<'r> WhenFlagClicked<'r> {
    fn new(id: &str, runtime: &'r Mutex<Runtime>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

impl<'r> BlockOrValue<'r> for WhenFlagClicked<'r> {
    fn set_arg(&mut self, _: &str, _: Box<dyn Value<'r> + 'r>) {}
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
    runtime: &'r Mutex<Runtime>,
    message: Option<Box<dyn Value<'r> + 'r>>,
    next: Option<Rc<RefCell<dyn Block<'r> + 'r>>>,
}

impl<'r> Say<'r> {
    fn new(id: &str, runtime: &'r Mutex<Runtime>) -> Self {
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
        match key {
            "MESSAGE" => self.message = Some(block),
            _ => return,
        };
    }
}

impl<'r> Block<'r> for Say<'r> {
    fn set_input(&mut self, key: &str, block: Rc<RefCell<dyn Block<'r> + 'r>>) {
        match key {
            "next" => self.next = Some(block),
            _ => return,
        };
    }

    fn next(&self) -> Option<Rc<RefCell<dyn Block<'r> + 'r>>> {
        self.next.clone()
    }

    fn execute(&mut self) -> Result<()> {
        if let Some(value) = &self.message {
            let ctx = &self.runtime.lock().unwrap().canvas;
            js_sys::Reflect::set(&ctx, &"font".into(), &"20px sans-serif".into()).unwrap();
            let v = value.value()?;
            let message = v.as_str().ok_or_else(|| Error::from("invalid type"))?;
            ctx.fill_text(message, 10.0, 50.0).unwrap();
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Variable<'r> {
    id: String,
    runtime: &'r Mutex<Runtime>,
}

impl<'r> Variable<'r> {
    pub fn new(id: &str, runtime: &'r Mutex<Runtime>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
        }
    }
}

impl<'r> BlockOrValue<'r> for Variable<'r> {
    fn set_arg(&mut self, _: &str, _: Box<dyn Value<'r> + 'r>) {}
}

impl<'r> Value<'r> for Variable<'r> {
    fn value(&self) -> Result<serde_json::Value> {
        unimplemented!()
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

pub fn new_block<'r>(
    id: &str,
    runtime: &'r Mutex<Runtime>,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Rc<RefCell<dyn Block<'r> + 'r>>> {
    let info = infos.get(id).unwrap();
    let block = get_block(id, runtime, &info)?;
    if let Some(next_id) = &info.next {
        block
            .borrow_mut()
            .set_input("next", new_block(next_id, runtime, infos)?);
    }
    for (k, input) in &info.inputs {
        let input_err_cb = || Error::from(format!("block \"{}\": invalid {}", id, k.as_str()));
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
                    .map_err(|e| format!("block \"{}\": {}", id, e.to_string()))?;
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
            _ => return Err(format!("block \"{}\": invalid input_type {}", id, input_type).into()),
        };
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
    runtime: &'r Mutex<Runtime>,
    info: &savefile::Block,
) -> Result<Rc<RefCell<dyn Block<'r> + 'r>>> {
    Ok(match info.opcode.as_str() {
        "event_whenflagclicked" => Rc::new(RefCell::new(WhenFlagClicked::new(id, runtime))),
        "looks_say" => Rc::new(RefCell::new(Say::new(id, runtime))),
        _ => return Err(format!("block \"{}\": opcode {} does not exist", id, info.opcode).into()),
    })
}
