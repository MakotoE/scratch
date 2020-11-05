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
use async_trait::async_trait;
use maplit::hashmap;
use runtime::Runtime;
use serde_json::Value;

fn get_block(id: String, runtime: Runtime, info: &savefile::Block) -> Result<Box<dyn Block>> {
    let (category, name) = match info.opcode.split_once('_') {
        Some(s) => s,
        None => {
            return Err(wrap_err!(format!(
                "block \"{}\": opcode {} does not exist",
                id, info.opcode
            )));
        }
    };

    let id_clone = id.clone();
    match category {
        "control" => control::get_block(name, id_clone, runtime)
            .map_err(|e| add_error_context(id, "control", e)),
        "data" => {
            data::get_block(name, id.clone(), runtime).map_err(|e| add_error_context(id, "data", e))
        }
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

fn add_error_context(id: String, category: &str, e: Error) -> Error {
    ErrorKind::BlockInitialization(id, category.to_string(), Box::new(e)).into()
}

#[async_trait(?Send)]
pub trait Block: std::fmt::Debug {
    fn block_info(&self) -> BlockInfo;

    fn block_inputs(&self) -> BlockInputs;

    fn set_input(&mut self, key: &str, block: Box<dyn Block>);

    #[allow(unused_variables)]
    fn set_field(&mut self, key: &str, field: &[String]) -> Result<()> {
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
    runtime: Runtime,
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
        let input_arr = match input.as_array() {
            Some(a) => a,
            None => {
                let e = ErrorKind::BlockInput(
                    top_block_id.clone(),
                    k.clone(),
                    Box::new("invalid type".into()),
                );
                return Err(e.into());
            }
        };

        match input_block(input_arr, runtime.clone(), infos) {
            Ok(b) => block.set_input(k, b),
            Err(e) => {
                let e = ErrorKind::BlockInput(top_block_id.clone(), k.clone(), Box::new(e));
                return Err(e.into());
            }
        }
    }
    for (k, field) in &info.fields {
        block.set_field(k, field)?;
    }
    Ok(block)
}

fn input_block(
    input_arr: &[serde_json::Value],
    runtime: Runtime,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block>> {
    let input_arr1 = match input_arr.get(1) {
        Some(v) => v,
        None => return Err(wrap_err!("invalid type")),
    };

    match input_arr1 {
        serde_json::Value::String(block_id) => block_tree(block_id.clone(), runtime, infos),
        serde_json::Value::Array(arr) => {
            let input_type = match input_arr.get(0).and_then(|v| v.as_i64()) {
                Some(n) => n,
                None => return Err(wrap_err!("invalid type")),
            };

            match input_type {
                // Value
                1 => match arr.get(1) {
                    Some(v) => Ok(Box::new(value::Value::from(v.clone()))),
                    None => Err(wrap_err!("invalid input type")),
                },
                // Variable
                2 | 3 => match arr.get(2).and_then(|v| v.as_str()) {
                    Some(id) => Ok(Box::new(value::Variable::new(id.to_string(), runtime))),
                    None => Err(wrap_err!("invalid input type")),
                },
                _ => Err(wrap_err!("invalid input type id")),
            }
        }
        _ => Err(wrap_err!("invalid type")),
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

fn value_to_string(value: Value) -> String {
    match value {
        Value::String(s) => s,
        Value::Number(n) => n.as_f64().unwrap().to_string(),
        _ => value.to_string(),
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
