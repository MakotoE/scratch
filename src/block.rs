use super::*;

use std::collections::HashMap;
use std::convert::TryFrom;

pub trait Block<'r>: std::fmt::Debug {
    fn set_input(&mut self, key: &str, block: Box<dyn Block<'r> + 'r>);
}

#[derive(Debug)]
pub struct Runtime {
    pub canvas: web_sys::CanvasRenderingContext2d,
}

#[derive(Debug)]
pub struct WhenFlagClicked<'r> {
    id: String,
    runtime: &'r Runtime,
    next: Option<Box<dyn Block<'r> + 'r>>,
}

impl<'r> WhenFlagClicked<'r> {
    fn new(id: &str, runtime: &'r Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

impl<'r> Block<'r> for WhenFlagClicked<'r> {
    fn set_input(&mut self, key: &str, block: Box<dyn Block<'r> + 'r>) {
        if key == "next" {
            self.next = Some(block);
        }
    }
}

#[derive(Debug)]
pub struct Say<'r> {
    id: String,
    runtime: &'r Runtime,
    message: Option<Box<dyn Block<'r> + 'r>>,
    next: Option<Box<dyn Block<'r> + 'r>>,
}

impl<'r> Say<'r> {
    fn new(id: &str, runtime: &'r Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            message: None,
            next: None,
        }
    }
}

impl<'r> Block<'r> for Say<'r> {
    fn set_input(&mut self, key: &str, block: Box<dyn Block<'r> + 'r>) {
        match key {
            "next" => self.next = Some(block),
            "MESSAGE" => self.message = Some(block),
            _ => return,
        };
    }
}

#[derive(Debug)]
pub struct Variable<'r> {
    id: String,
    runtime: &'r Runtime,
}

impl<'r> Variable<'r> {
    pub fn new(id: &str, runtime: &'r Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
        }
    }
}

impl<'r> Block<'r> for Variable<'r> {
    fn set_input(&mut self, _: &str, _: Box<dyn Block<'r> + 'r>) {}
}

fn wrong_type_err(value: &serde_json::Value) -> Error {
    format!("value has wrong type: {:?}", value).into()
}

#[derive(Debug)]
pub struct Number {
    value: f64,
}

impl<'r> Block<'r> for Number {
    fn set_input(&mut self, _: &str, _: Box<dyn Block<'r> + 'r>) {}
}

impl TryFrom<serde_json::Value> for Number {
    type Error = Error;

    fn try_from(v: serde_json::Value) -> Result<Self> {
        Ok(Self {
            value: v.as_f64().chain_err(|| wrong_type_err(&v))?,
        })
    }
}

#[derive(Debug)]
pub struct BlockString {
    value: String,
}

impl<'r> Block<'r> for BlockString {
    fn set_input(&mut self, _: &str, _: Box<dyn Block<'r> + 'r>) {}
}

impl TryFrom<serde_json::Value> for BlockString {
    type Error = Error;

    fn try_from(v: serde_json::Value) -> Result<Self> {
        Ok(Self {
            value: v.as_str().chain_err(|| wrong_type_err(&v))?.to_string(),
        })
    }
}

pub fn new_block<'r>(
    id: &str,
    runtime: &'r Runtime,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block<'r> + 'r>> {
    let info = infos.get(id).unwrap();
    let mut block = get_block(id, runtime, &info)?;
    if let Some(next_id) = &info.next {
        block.set_input("next", new_block(next_id, runtime, infos)?);
    }
    for (k, input) in &info.inputs {
        let input_err_cb = || Error::from(format!("block \"{}\": invalid {}", id, k.as_str()));
        let input_arr = input.as_array().chain_err(input_err_cb)?;
        let input_type = input_arr
            .get(0)
            .and_then(|v| v.as_i64())
            .chain_err(input_err_cb)?;
        let input_block = match input_type {
            1 => {
                // value
                let value_info = input_arr
                    .get(1)
                    .and_then(|v| v.as_array())
                    .chain_err(input_err_cb)?;
                let value_type = value_info
                    .get(0)
                    .and_then(|v| v.as_i64())
                    .chain_err(input_err_cb)?;
                let value = value_info.get(1).chain_err(input_err_cb)?;
                new_value(value_type, value.clone())
                    .map_err(|e| format!("block \"{}\": {}", id, e.to_string()))?
            }
            2 | 3 => {
                let input_info = input_arr.get(1).chain_err(input_err_cb)?;
                if input_info.is_string() {
                    // block
                    let id = input_info.as_str().chain_err(input_err_cb)?;
                    new_block(id, runtime, infos)?
                } else if input_info.is_array() {
                    // variable
                    let id = input_info
                        .as_array()
                        .and_then(|v| v.get(2))
                        .and_then(|v| v.as_str())
                        .chain_err(input_err_cb)?;
                    Box::new(Variable::new(id, runtime))
                } else {
                    return Err(input_err_cb());
                }
            }
            _ => return Err(format!("block \"{}\": invalid input_type {}", id, input_type).into()),
        };
        block.set_input(k, input_block);
    }
    Ok(block)
}

pub fn new_value<'r>(value_type: i64, value: serde_json::Value) -> Result<Box<dyn Block<'r> + 'r>> {
    Ok(match value_type {
        4 => Box::new(Number::try_from(value)?),
        10 => Box::new(BlockString::try_from(value)?),
        _ => return Err(format!("value_type {} does not exist", value_type).into()),
    })
}

pub fn get_block<'r>(
    id: &str,
    runtime: &'r Runtime,
    info: &savefile::Block,
) -> Result<Box<dyn Block<'r> + 'r>> {
    Ok(match info.opcode.as_str() {
        "event_whenflagclicked" => Box::new(WhenFlagClicked::new(id, runtime)),
        "looks_say" => Box::new(Say::new(id, runtime)),
        _ => return Err(format!("block \"{}\": opcode {} does not exist", id, info.opcode).into()),
    })
}
