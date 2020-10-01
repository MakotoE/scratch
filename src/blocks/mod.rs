mod control;
mod data;
mod event;
mod looks;
mod motion;
mod operator;
mod pen;
mod value;

use super::*;
use async_trait::async_trait;
use maplit::hashmap;
use runtime::Coordinate;
use runtime::SpriteRuntime;

fn get_block(
    id: &str,
    runtime: Rc<RwLock<SpriteRuntime>>,
    info: &savefile::Block,
) -> Result<Box<dyn Block>> {
    let (category, name) = match info.opcode.split_once('_') {
        Some(s) => s,
        None => {
            return Err(format!("block \"{}\": opcode {} does not exist", id, info.opcode).into())
        }
    };

    match category {
        "control" => {
            control::get_block(name, id, runtime).map_err(|e| add_error_context(id, "control", e))
        }
        "data" => data::get_block(name, id, runtime).map_err(|e| add_error_context(id, "data", e)),
        "event" => {
            event::get_block(name, id, runtime).map_err(|e| add_error_context(id, "event", e))
        }
        "looks" => {
            looks::get_block(name, id, runtime).map_err(|e| add_error_context(id, "looks", e))
        }
        "motion" => {
            motion::get_block(name, id, runtime).map_err(|e| add_error_context(id, "motion", e))
        }
        "operator" => {
            operator::get_block(name, id, runtime).map_err(|e| add_error_context(id, "operator", e))
        }
        "pen" => pen::get_block(name, id, runtime).map_err(|e| add_error_context(id, "pen", e)),
        _ => Err(format!("block id \"{}\": opcode {} does not exist", id, info.opcode).into()),
    }
}

fn add_error_context(id: &str, category: &str, err: Error) -> Error {
    format!("block id \"{}\", category {}: {}", id, category, err).into()
}

#[async_trait(?Send)]
pub trait Block: std::fmt::Debug {
    fn block_info(&self) -> BlockInfo;

    fn block_inputs(&self) -> BlockInputs;

    fn set_input(&mut self, key: &str, block: Box<dyn Block>);

    #[allow(unused_variables)]
    fn set_field(&mut self, key: &str, value_id: String) {}

    async fn value(&self) -> Result<serde_json::Value> {
        Err("this block does not return a value".into())
    }

    async fn execute(&mut self) -> Next {
        Next::Err("this block cannot be executed".into())
    }
}

#[derive(Debug)]
pub enum Next {
    None,
    Err(Error),
    Continue(Rc<RefCell<Box<dyn Block>>>),
    Loop(Rc<RefCell<Box<dyn Block>>>),
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
    pub fn continue_(block: Option<Rc<RefCell<Box<dyn Block>>>>) -> Next {
        match block {
            Some(b) => Next::Continue(b),
            None => Next::None,
        }
    }

    pub fn loop_(block: Option<Rc<RefCell<Box<dyn Block>>>>) -> Next {
        match block {
            Some(b) => Next::Loop(b),
            None => Next::None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockInfo {
    pub name: &'static str,
    pub id: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockInputs {
    pub info: BlockInfo,
    pub fields: HashMap<&'static str, String>,
    pub inputs: HashMap<&'static str, BlockInputs>,
    pub stacks: HashMap<&'static str, BlockInputs>,
}

impl BlockInputs {
    fn inputs<'a>(
        inputs: HashMap<&'static str, &'a Option<Box<dyn Block>>>,
    ) -> HashMap<&'static str, BlockInputs> {
        let mut result: HashMap<&'static str, BlockInputs> = HashMap::new();
        for (id, b) in inputs {
            if let Some(block) = b {
                result.insert(id, block.block_inputs());
            }
        }
        result
    }

    fn stacks<'a>(
        stacks: HashMap<&'static str, &'a Option<Rc<RefCell<Box<dyn Block>>>>>,
    ) -> HashMap<&'static str, BlockInputs> {
        let mut result: HashMap<&'static str, BlockInputs> = HashMap::new();
        for (id, b) in stacks {
            if let Some(block) = b {
                result.insert(id, block.borrow().block_inputs());
            }
        }
        result
    }
}

pub fn new_block(
    block_id: &str,
    runtime: Rc<RwLock<SpriteRuntime>>,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block>> {
    let info = infos.get(block_id).unwrap();
    let mut block = get_block(block_id, runtime.clone(), &info)?;
    if let Some(next_id) = &info.next {
        block.set_input("next", new_block(next_id, runtime.clone(), infos)?);
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
                let js_value = value_info.get(1).ok_or_else(input_err_cb)?;
                let value = Box::new(value::Value::from(js_value.clone()));
                block.set_input(k, value);
            }
            2 | 3 => {
                let input_info = input_arr.get(1).ok_or_else(input_err_cb)?;
                match input_info {
                    serde_json::Value::String(id) => {
                        let new_block = new_block(id, runtime.clone(), infos)?;
                        block.set_input(k, new_block);
                    }
                    serde_json::Value::Array(arr) => {
                        let id = arr
                            .get(2)
                            .and_then(|v| v.as_str())
                            .ok_or_else(input_err_cb)?;
                        let variable =
                            Box::new(value::Variable::new(id.to_string(), runtime.clone()));
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
        _ => return Err(format!("expected String or Number but got: {:?}", value).into()),
    })
}
