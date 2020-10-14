mod control;
mod data;
mod event;
mod looks;
mod motion;
mod operator;
mod pen;
mod sound;
mod value;

use super::*;
use async_trait::async_trait;
use maplit::hashmap;
use runtime::Coordinate;
use runtime::SpriteRuntime;

fn get_block(
    id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    info: &savefile::Block,
) -> Result<Box<dyn Block>> {
    let (category, name) = match info.opcode.split_once('_') {
        Some(s) => s,
        None => {
            return Err(wrap_err!(format!(
                "block \"{}\": opcode {} does not exist",
                id, info.opcode
            )));
        }
    };

    match category {
        "control" => control::get_block(name, id.clone(), runtime)
            .map_err(|e| add_error_context(id, "control", e)),
        "data" => {
            data::get_block(name, id.clone(), runtime).map_err(|e| add_error_context(id, "data", e))
        }
        "event" => event::get_block(name, id.clone(), runtime)
            .map_err(|e| add_error_context(id, "event", e)),
        "looks" => looks::get_block(name, id.clone(), runtime)
            .map_err(|e| add_error_context(id, "looks", e)),
        "motion" => motion::get_block(name, id.clone(), runtime)
            .map_err(|e| add_error_context(id, "motion", e)),
        "operator" => operator::get_block(name, id.clone(), runtime)
            .map_err(|e| add_error_context(id, "operator", e)),
        "pen" => {
            pen::get_block(name, id.clone(), runtime).map_err(|e| add_error_context(id, "pen", e))
        }
        "sound" => sound::get_block(name, id.clone(), runtime)
            .map_err(|e| add_error_context(id, "sound", e)),
        _ => Err(wrap_err!(format!(
            "block id \"{}\": opcode {} does not exist",
            id, info.opcode
        ))),
    }
}

fn add_error_context(id: String, category: &str, e: Error) -> Error {
    ErrorKind::BlockInitialization(id, category.to_string(), Box::new(e)).into()
}

#[async_trait(?Send)]
pub trait Block: std::fmt::Debug {
    fn block_info(&self) -> BlockInfo;

    fn block_inputs(&self) -> BlockInputs;

    fn set_input(&mut self, key: &str, block: Box<dyn Block>);

    #[allow(unused_variables)]
    fn set_field(&mut self, key: &str, value_id: String) {}

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

pub fn block_tree(
    top_block_id: String,
    runtime: Rc<RwLock<SpriteRuntime>>,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block>> {
    let info = match infos.get(&top_block_id) {
        Some(b) => b,
        None => return Err(wrap_err!(format!("could not find block: {}", top_block_id))),
    };
    let mut block = get_block(top_block_id.clone(), runtime.clone(), &info)?;
    if let Some(next_id) = &info.next {
        block.set_input("next", block_tree(next_id.clone(), runtime.clone(), infos)?);
    }
    for (k, input) in &info.inputs {
        let input_err = |msg: &str| {
            Err(wrap_err!(format!(
                "block \"{}\", input {}: {}",
                top_block_id,
                k.as_str(),
                msg,
            )))
        };
        let input_arr = match input.as_array() {
            Some(a) => a,
            None => return input_err("invalid type"),
        };
        let input_type = match input_arr.get(0).and_then(|v| v.as_i64()) {
            Some(n) => n,
            None => return input_err("invalid type"),
        };
        let input_block = match input_type {
            // Value
            1 => match value_from_input_arr(input_arr) {
                Some(b) => b,
                None => return input_err("invalid type"),
            },
            // Block or variable
            2 | 3 => block_from_input_arr(input_arr, runtime.clone(), infos)?,
            _ => return input_err("invalid type"),
        };
        block.set_input(k, input_block);
    }
    for (k, field) in &info.fields {
        match field.get(1) {
            Some(value_id) => {
                block.set_field(k, value_id.clone());
            }
            None => {
                return Err(wrap_err!(format!(
                    "block \"{}\": invalid field {}",
                    top_block_id, k
                )))
            }
        }
    }
    Ok(block)
}

fn value_from_input_arr(input_arr: &[serde_json::Value]) -> Option<Box<value::Value>> {
    input_arr
        .get(1)
        .and_then(|v| v.as_array())
        .and_then(|v| v.get(1))
        .map(|v| Box::new(value::Value::from(v.clone())))
}

fn block_from_input_arr(
    input_arr: &[serde_json::Value],
    runtime: Rc<RwLock<SpriteRuntime>>,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block>> {
    match input_arr.get(1) {
        Some(a) => match a {
            serde_json::Value::String(id) => block_tree(id.clone(), runtime, infos),
            serde_json::Value::Array(arr) => match arr.get(2).and_then(|v| v.as_str()) {
                Some(id) => Ok(Box::new(value::Variable::new(id.to_string(), runtime))),
                None => Err(wrap_err!("invalid input type")),
            },
            _ => Err(wrap_err!("invalid input type")),
        },
        None => Err(wrap_err!("invalid input type")),
    }
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
