use super::*;

use std::collections::HashMap;
use std::convert::TryFrom;

pub trait Block<'runtime>: std::fmt::Debug {
    fn set_input(&mut self, key: String, block: Box<dyn Block<'runtime> + 'runtime>);
}

#[derive(Debug)]
pub struct Runtime {}

#[derive(Debug)]
pub struct MoveSteps<'runtime> {
    id: String,
    runtime: &'runtime Runtime,
    inputs: HashMap<String, Box<dyn Block<'runtime> + 'runtime>>,
}

impl<'runtime> MoveSteps<'runtime> {
    pub fn new(id: &str, runtime: &'runtime Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            inputs: HashMap::new(),
        }
    }
}

impl<'runtime> Block<'runtime> for MoveSteps<'runtime> {
    fn set_input(&mut self, key: String, block: Box<dyn Block<'runtime> + 'runtime>) {
        self.inputs.insert(key, block);
    }
}

#[derive(Debug)]
pub struct Number {
    value: f64,
}

impl TryFrom<&str> for Number {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        let value = serde_json::from_str(s)?;
        Ok(Number { value })
    }
}

impl<'runtime> Block<'runtime> for Number {
    fn set_input(&mut self, key: String, block: Box<dyn Block<'runtime> + 'runtime>) {}
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
    fn set_input(&mut self, key: String, block: Box<dyn Block<'runtime> + 'runtime>) {}
}

#[derive(Debug)]
pub struct Thread<'runtime> {
    runtime: &'runtime Runtime,
    hat: Option<Box<dyn Block<'runtime> + 'runtime>>,
}

impl<'runtime> Thread<'runtime> {
    pub fn new(
        runtime: &'runtime Runtime,
        block_infos: &HashMap<String, savefile::Block>,
    ) -> Result<Self> {
        let hat = match find_hat(block_infos) {
            Some(hat_id) => Some(new_block(hat_id, runtime, block_infos)?),
            None => None,
        };
        Ok(Self { runtime, hat })
    }
}

fn find_hat(block_infos: &HashMap<String, savefile::Block>) -> Option<&str> {
    for (id, block_info) in block_infos {
        if block_info.top_level {
            return Some(id);
        }
    }

    None
}

fn new_block<'runtime>(
    id: &str,
    runtime: &'runtime Runtime,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block<'runtime> + 'runtime>> {
    let info = infos.get(id).unwrap();
    let mut block = get_block(id, runtime, &info)?;
    if let Some(next_id) = &info.next {
        block.set_input("next".to_string(), new_block(next_id, runtime, infos)?);
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
                    .and_then(|v| v.as_str())
                    .chain_err(input_err_cb)?;
                new_value(value_type, value)?
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
        block.set_input(k.clone(), input_block);
    }
    Ok(block)
}

fn new_value<'runtime>(
    value_type: i64,
    value: &str,
) -> Result<Box<dyn Block<'runtime> + 'runtime>> {
    Ok(match value_type {
        4 => Box::new(Number::try_from(value)?),
        _ => Box::new(Number::try_from(value)?),
    })
}

fn get_block<'runtime>(
    id: &str,
    runtime: &'runtime Runtime,
    info: &savefile::Block,
) -> Result<Box<dyn Block<'runtime> + 'runtime>> {
    let block = match info.opcode.as_str() {
        "motion_movesteps" => MoveSteps::new(id, runtime),
        _ => MoveSteps::new(id, runtime),
    };
    Ok(Box::new(block))
}
