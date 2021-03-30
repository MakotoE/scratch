mod control;
mod data;
mod event;
mod looks;
mod motion;
mod operator;
mod pen;
mod sensing;
mod sound;
pub mod test;
pub mod value;

use super::*;
use crate::blocks::value::value_block_from_input_arr;
use crate::file::BlockID;
use crate::runtime::Runtime;
use async_trait::async_trait;
use std::convert::TryInto;
use std::time::Duration;
use tokio::time::sleep;
use value::Value;

fn get_block(id: BlockID, runtime: Runtime, info: &file::Block) -> Result<Box<dyn Block>> {
    let (category, name) = info.opcode.split_once('_').ok_or_else(|| {
        Error::msg(format!(
            "block \"{}\": opcode {} does not exist",
            id, info.opcode
        ))
    })?;

    let id_clone = id;
    match category {
        "control" => control::get_block(name, id_clone, runtime)
            .map_err(|e| add_error_context(id, "control", e)),
        "data" => data::get_block(name, id, runtime).map_err(|e| add_error_context(id, "data", e)),
        "event" => {
            event::get_block(name, id_clone, runtime).map_err(|e| add_error_context(id, "event", e))
        }
        "looks" => {
            looks::get_block(name, id_clone, runtime).map_err(|e| add_error_context(id, "looks", e))
        }
        "motion" => motion::get_block(name, id_clone, runtime)
            .map_err(|e| add_error_context(id, "motion", e)),
        "operator" => operator::get_block(name, id_clone, runtime)
            .map_err(|e| add_error_context(id, "operator", e)),
        "pen" => {
            pen::get_block(name, id_clone, runtime).map_err(|e| add_error_context(id, "pen", e))
        }
        "sensing" => sensing::get_block(name, id_clone, runtime)
            .map_err(|e| add_error_context(id, "sensing", e)),
        "sound" => {
            sound::get_block(name, id_clone, runtime).map_err(|e| add_error_context(id, "sound", e))
        }
        _ => Err(Error::msg(format!(
            "block id \"{}\": opcode {} does not exist",
            id, info.opcode
        ))),
    }
}

fn add_error_context(id: BlockID, category: &str, error: Error) -> Error {
    ScratchError::BlockInitialization {
        id,
        category: category.to_string(),
        error,
    }
    .into()
}

#[async_trait]
pub trait Block: std::fmt::Debug + Sync + Send {
    fn block_info(&self) -> BlockInfo;

    fn block_inputs(&self) -> BlockInputsPartial;

    #[allow(unused_variables)]
    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {}

    #[allow(unused_variables)]
    fn set_substack(&mut self, key: &str, block: BlockID) {}

    #[allow(unused_variables)]
    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        Ok(())
    }

    async fn value(&self) -> Result<Value> {
        Err(Error::msg("this block does not return a value"))
    }

    async fn execute(&mut self) -> Result<Next> {
        Err(Error::msg("this block cannot be executed"))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Next {
    None,
    Continue(BlockID),
    Loop(BlockID),
}

impl Next {
    pub fn continue_(block: Option<BlockID>) -> Result<Next> {
        Ok(match block {
            Some(b) => Next::Continue(b),
            None => Next::None,
        })
    }

    pub fn loop_(block: Option<BlockID>) -> Result<Next> {
        Ok(match block {
            Some(b) => Next::Loop(b),
            None => Next::None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BlockInfo {
    pub name: &'static str,
    pub id: BlockID,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockInputsPartial {
    pub info: BlockInfo,
    pub fields: HashMap<&'static str, String>,
    pub inputs: HashMap<&'static str, BlockInputsPartial>,
    pub stacks: HashMap<&'static str, BlockID>,
}

impl BlockInputsPartial {
    fn new<'a>(
        info: BlockInfo,
        mut fields: Vec<(&'static str, String)>,
        mut inputs: Vec<(&'static str, &'a dyn Block)>,
        mut stacks: Vec<(&'static str, &'a Option<BlockID>)>,
    ) -> Self {
        Self {
            info,
            fields: fields.drain(..).collect(),
            inputs: inputs
                .drain(..)
                .map(|(id, b)| (id, b.block_inputs()))
                .collect(),
            stacks: stacks
                .drain(..)
                .filter_map(|(id, &b)| {
                    if let Some(block_id) = b {
                        Some((id, block_id))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

pub fn block_tree(
    top_block_id: BlockID,
    runtime: Runtime,
    infos: &HashMap<BlockID, file::Block>,
) -> Result<HashMap<BlockID, Box<dyn Block>>> {
    let info = match infos.get(&top_block_id) {
        Some(b) => b,
        None => {
            return Err(Error::msg(format!(
                "could not find block: {}",
                top_block_id
            )))
        }
    };

    let mut block_map: HashMap<BlockID, Box<dyn Block>> = HashMap::default();
    let mut block = get_block(top_block_id, runtime.clone(), &info)?;

    if let Some(next_id) = info.next {
        let input_blocks = block_tree(next_id, runtime.clone(), infos)?;
        block.set_substack("next", next_id);
        block_map.extend(input_blocks);
    }

    for (k, input) in &info.inputs {
        let wrap_err = |error: Error| -> Error {
            ScratchError::BlockInput {
                block_id: top_block_id,
                input_id: k.clone(),
                error,
            }
            .into()
        };
        let input_err = || wrap_err(Error::msg("invalid type"));

        let input_arr = input.as_array().ok_or_else(input_err)?;
        match input_arr.get(1).ok_or_else(input_err)? {
            serde_json::Value::String(str_id) => {
                let block_id = str_id.as_str().try_into().map_err(wrap_err)?;
                let mut blocks = block_tree(
                    str_id.as_str().try_into().map_err(wrap_err)?,
                    runtime.clone(),
                    infos,
                )?;

                if k.starts_with("SUBSTACK") {
                    block.set_substack(k, block_id);
                    block_map.extend(blocks);
                } else if let Some(b) = blocks.drain().next() {
                    block.set_input(k, b.1);
                }
            }
            serde_json::Value::Array(arr) => {
                let input_type = input_arr
                    .get(0)
                    .ok_or_else(input_err)?
                    .as_i64()
                    .ok_or_else(input_err)?;

                let value = match input_type {
                    // Value
                    1 => value_block_from_input_arr(arr).map_err(wrap_err)?,
                    // Variable
                    2 | 3 => {
                        let id = arr
                            .get(2)
                            .ok_or_else(input_err)?
                            .as_str()
                            .ok_or_else(input_err)?;
                        Box::new(value::Variable::new(id.to_string(), runtime.clone()))
                            as Box<dyn Block>
                    }
                    _ => return Err(input_err()),
                };
                block.set_input(k, value);
            }
            _ => return Err(input_err()),
        };
    }

    for (k, field) in &info.fields {
        match block.set_field(k, field) {
            Ok(_) => {}
            Err(error) => {
                return Err(ScratchError::BlockField {
                    block_id: top_block_id,
                    field_id: k.clone(),
                    error,
                }
                .into());
            }
        }
    }

    block_map.insert(top_block_id, block);
    Ok(block_map)
}

#[derive(Debug)]
pub struct EmptyInput {}

#[async_trait]
impl Block for EmptyInput {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "EmptyInput",
            id: BlockID::pseudo_id(),
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial {
            info: self.block_info(),
            fields: HashMap::default(),
            inputs: HashMap::default(),
            stacks: HashMap::default(),
        }
    }

    async fn value(&self) -> Result<Value> {
        Err(Error::msg("input is unconnected"))
    }
}

#[derive(Debug)]
pub struct EmptyFalse {}

#[async_trait]
impl Block for EmptyFalse {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "EmptyFalse",
            id: BlockID::pseudo_id(),
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial {
            info: self.block_info(),
            fields: HashMap::default(),
            inputs: HashMap::default(),
            stacks: HashMap::default(),
        }
    }

    async fn value(&self) -> Result<Value> {
        Ok(true.into())
    }
}

pub fn get_field_value(field: &[Option<String>], index: usize) -> Result<&str> {
    if let Some(Some(s)) = field.get(index) {
        Ok(s)
    } else {
        Err(Error::msg("invalid field"))
    }
}

#[cfg(test)]
pub fn block_map(mut blocks: Vec<(BlockID, Box<dyn Block>)>) -> HashMap<BlockID, Box<dyn Block>> {
    blocks.drain(..).collect()
}
