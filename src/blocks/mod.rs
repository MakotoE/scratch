mod control;
mod data;
mod event;
mod looks;
mod motion;
mod operator;
mod pen;
mod sensing;
mod sound;
mod value;

use super::*;
use crate::runtime::Runtime;
use crate::savefile::BlockID;
use async_trait::async_trait;
use serde_json::Value;
use std::convert::TryInto;

fn get_block(id: BlockID, runtime: Runtime, info: &savefile::Block) -> Result<Box<dyn Block>> {
    let (category, name) = match info.opcode.split_once('_') {
        Some(s) => s,
        None => {
            return Err(wrap_err!(format!(
                "block \"{}\": opcode {} does not exist",
                id, info.opcode
            )));
        }
    };

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
        _ => Err(wrap_err!(format!(
            "block id \"{}\": opcode {} does not exist",
            id, info.opcode
        ))),
    }
}

fn add_error_context(id: BlockID, category: &str, e: Error) -> Error {
    ErrorKind::BlockInitialization(id, category.to_string(), Box::new(e)).into()
}

#[async_trait(?Send)]
pub trait Block: std::fmt::Debug {
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

    async fn value(&self) -> Result<serde_json::Value> {
        Err(wrap_err!("this block does not return a value"))
    }

    async fn execute(&mut self) -> Next {
        Next::Err(wrap_err!("this block cannot be executed"))
    }
}

#[derive(Debug)]
pub enum Next {
    None,
    Err(Error),
    Continue(BlockID),
    Loop(BlockID),
}

impl std::ops::Try for Next {
    type Ok = Next;
    type Error = Error;

    fn into_result(self) -> Result<Next> {
        match self {
            Self::Err(e) => Err(e),
            _ => Ok(self),
        }
    }

    fn from_error(v: Error) -> Self {
        Self::Err(v)
    }

    fn from_ok(v: Next) -> Self {
        v
    }
}

impl Next {
    pub fn continue_(block: Option<BlockID>) -> Next {
        match block {
            Some(b) => Next::Continue(b),
            None => Next::None,
        }
    }

    pub fn loop_(block: Option<BlockID>) -> Next {
        match block {
            Some(b) => Next::Loop(b),
            None => Next::None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockInfo {
    pub name: &'static str,
    pub id: BlockID,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockInputsPartial {
    pub info: BlockInfo,
    pub fields: HashMap<&'static str, String>,
    pub inputs: HashMap<&'static str, BlockInputsPartial>,
    pub stacks: HashMap<&'static str, BlockID>,
}

impl BlockInputsPartial {
    #[allow(clippy::borrowed_box)]
    fn new<'a>(
        info: BlockInfo,
        mut fields: Vec<(&'static str, String)>,
        mut inputs: Vec<(&'static str, &'a Box<dyn Block>)>,
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
    infos: &HashMap<BlockID, savefile::Block>,
) -> Result<(BlockID, HashMap<BlockID, Box<dyn Block>>)> {
    let info = match infos.get(&top_block_id) {
        Some(b) => b,
        None => return Err(wrap_err!(format!("could not find block: {}", top_block_id))),
    };

    let mut result: HashMap<BlockID, Box<dyn Block>> = HashMap::new();
    let mut block = get_block(top_block_id, runtime.clone(), &info)?;

    if let Some(next_id) = &info.next {
        let (id, input_blocks) = block_tree(*next_id, runtime.clone(), infos)?;
        block.set_substack("next", id);
        result.extend(input_blocks);
    }

    for (k, input) in &info.inputs {
        let input_arr = match input.as_array() {
            Some(a) => a,
            None => {
                let e =
                    ErrorKind::BlockInput(top_block_id, k.clone(), Box::new("invalid type".into()));
                return Err(e.into());
            }
        };

        let input_arr1 = match input_arr.get(1) {
            Some(v) => v,
            None => return Err(wrap_err!("invalid type")),
        };

        match input_arr1 {
            serde_json::Value::String(block_id) => {
                let (id, mut blocks) =
                    block_tree(block_id.as_str().try_into()?, runtime.clone(), infos)?;

                if k.starts_with("SUBSTACK") {
                    block.set_substack(k, id);
                    result.extend(blocks);
                } else {
                    assert_eq!(blocks.len(), 1);
                    block.set_input(k, blocks.drain().next().unwrap().1);
                }
            }
            serde_json::Value::Array(arr) => {
                let input_type = match input_arr.get(0).and_then(|v| v.as_i64()) {
                    Some(n) => n,
                    None => return Err(wrap_err!("invalid type")),
                };

                let value = match input_type {
                    // Value
                    1 => match arr.get(1) {
                        Some(v) => Box::new(value::Value::from(v.clone())) as Box<dyn Block>,
                        None => return Err(wrap_err!("invalid input type")),
                    },
                    // Variable
                    2 | 3 => match arr.get(2).and_then(|v| v.as_str()) {
                        Some(id) => Box::new(value::Variable::new(id.to_string(), runtime.clone()))
                            as Box<dyn Block>,
                        None => return Err(wrap_err!("invalid input type")),
                    },
                    _ => return Err(wrap_err!("invalid input type id")),
                };
                block.set_input(k, value);
            }
            _ => return Err(wrap_err!("invalid type")),
        };
    }

    for (k, field) in &info.fields {
        block.set_field(k, field)?;
    }

    let id = block.block_info().id;
    result.insert(id, block);
    Ok((id, result))
}

const MILLIS_PER_SECOND: f64 = 1000.0;

fn value_to_float(value: &serde_json::Value) -> Result<f64> {
    Ok(match value {
        serde_json::Value::String(s) => s.parse()?,
        serde_json::Value::Number(n) => match n.as_f64() {
            Some(f) => {
                if f.is_nan() {
                    0.0
                } else {
                    f
                }
            }
            None => unreachable!(),
        },
        _ => {
            return Err(wrap_err!(format!(
                "expected String or Number but got: {:?}",
                value
            )))
        }
    })
}

pub fn value_to_string(value: Value) -> String {
    match value {
        Value::String(s) => s,
        Value::Number(n) => n.as_f64().unwrap().to_string(),
        _ => value.to_string(),
    }
}

#[derive(Debug)]
pub struct EmptyInput {}

#[async_trait(?Send)]
impl Block for EmptyInput {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "EmptyInput",
            id: BlockID::default(),
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

    async fn value(&self) -> Result<serde_json::Value> {
        Err(wrap_err!("input is unconnected"))
    }
}

#[derive(Debug)]
pub struct EmptyFalse {}

#[async_trait(?Send)]
impl Block for EmptyFalse {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "EmptyFalse",
            id: BlockID::default(),
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

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(true.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_string() {
        struct Test {
            value: Value,
            expected: &'static str,
        }

        let tests = vec![
            Test {
                value: Value::Null,
                expected: "null",
            },
            Test {
                value: Value::String("a".into()),
                expected: "a",
            },
            Test {
                value: Value::Number(serde_json::Number::from_f64(1.0).unwrap()),
                expected: "1",
            },
            Test {
                value: Value::Number(serde_json::Number::from_f64(1.1).unwrap()),
                expected: "1.1",
            },
            Test {
                value: Value::Bool(false),
                expected: "false",
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            assert_eq!(value_to_string(test.value.clone()), test.expected, "{}", i);
        }
    }
}
