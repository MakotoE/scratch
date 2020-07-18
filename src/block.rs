use super::*;

use std::collections::HashMap;
use std::convert::TryFrom;

pub trait Block<'runtime>: std::fmt::Debug {
    fn set_input(&mut self, key: &str, block: Box<dyn Block<'runtime> + 'runtime>);
}

#[derive(Debug)]
pub struct Runtime {}

#[derive(Debug)]
pub struct WhenFlagClicked<'runtime> {
    id: String,
    runtime: &'runtime Runtime,
    next: Option<Box<dyn Block<'runtime> + 'runtime>>,
}

impl<'runtime> WhenFlagClicked<'runtime> {
    fn new(id: &str, runtime: &'runtime Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            next: None,
        }
    }
}

impl<'runtime> Block<'runtime> for WhenFlagClicked<'runtime> {
    fn set_input(&mut self, key: &str, block: Box<dyn Block<'runtime> + 'runtime>) {
        if key == "next" {
            self.next = Some(block);
        }
    }
}

#[derive(Debug)]
pub struct Say<'runtime> {
    id: String,
    runtime: &'runtime Runtime,
    message: Option<Box<dyn Block<'runtime> + 'runtime>>,
    next: Option<Box<dyn Block<'runtime> + 'runtime>>,
}

impl<'runtime> Say<'runtime> {
    fn new(id: &str, runtime: &'runtime Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            message: None,
            next: None,
        }
    }
}

impl<'runtime> Block<'runtime> for Say<'runtime> {
    fn set_input(&mut self, key: &str, block: Box<dyn Block<'runtime> + 'runtime>) {
        match key {
            "next" => self.next = Some(block),
            "MESSAGE" => self.message = Some(block),
            _ => return,
        };
    }
}

#[derive(Debug)]
pub struct Variable<'runtime> {
    id: String,
    runtime: &'runtime Runtime,
}

impl<'runtime> Variable<'runtime> {
    pub fn new(id: &str, runtime: &'runtime Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
        }
    }
}

impl<'runtime> Block<'runtime> for Variable<'runtime> {
    fn set_input(&mut self, _: &str, _: Box<dyn Block<'runtime> + 'runtime>) {}
}

fn wrong_type_err(value: &serde_json::Value) -> Error {
    format!("value has wrong type: {:?}", value).into()
}

#[derive(Debug)]
pub struct Number {
    value: f64,
}

impl<'runtime> Block<'runtime> for Number {
    fn set_input(&mut self, _: &str, _: Box<dyn Block<'runtime> + 'runtime>) {}
}

impl TryFrom<serde_json::Value> for Number {
    type Error = Error;

    fn try_from(v: serde_json::Value) -> Result<Self> {
        Ok(Self { value: v.as_f64().chain_err(|| wrong_type_err(&v))? })
    }
}

#[derive(Debug)]
pub struct BlockString {
    value: String,
}

impl<'runtime> Block<'runtime> for BlockString {
    fn set_input(&mut self, _: &str, _: Box<dyn Block<'runtime> + 'runtime>) {}
}

impl TryFrom<serde_json::Value> for BlockString {
    type Error = Error;

    fn try_from(v: serde_json::Value) -> Result<Self> {
        Ok(Self { value: v.as_str().chain_err(|| wrong_type_err(&v))?.to_string() })
    }
}

pub fn new_block<'runtime>(
    id: &str,
    runtime: &'runtime Runtime,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block<'runtime> + 'runtime>> {
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
                let value = value_info
                    .get(1)
                    .chain_err(input_err_cb)?;
                new_value(value_type, value.clone())
                    .map_err(|e| format!("block \"{}\": {}", id, e.to_string()))?
            },
            2 | 3 => {
                let input_info = input_arr
                    .get(1)
                    .chain_err(input_err_cb)?;
                if input_info.is_string() {
                    // block
                    let id = input_info
                        .as_str()
                        .chain_err(input_err_cb)?;
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
            },
            _ => return Err(format!("block \"{}\": invalid input_type {}", id, input_type).into()),
        };
        block.set_input(k, input_block);
    }
    Ok(block)
}

pub fn new_value<'runtime>(
    value_type: i64,
    value: serde_json::Value,
) -> Result<Box<dyn Block<'runtime> + 'runtime>> {
    Ok(match value_type {
        4 => Box::new(Number::try_from(value)?),
        10 => Box::new(BlockString::try_from(value)?),
        _ => return Err(format!("value_type {} does not exist", value_type).into()),
    })
}

pub fn get_block<'runtime>(
    id: &str,
    runtime: &'runtime Runtime,
    info: &savefile::Block,
) -> Result<Box<dyn Block<'runtime> + 'runtime>> {
    Ok(match info.opcode.as_str() {
        "event_whenflagclicked" => Box::new(WhenFlagClicked::new(id, runtime)),
        "looks_say" => Box::new(Say::new(id, runtime)),
        _ => return Err(format!("block \"{}\": opcode {} does not exist", id, info.opcode).into()),
    })
}
