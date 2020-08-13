mod control;
mod data;
mod event;
mod looks;
mod motion;
mod operator;
mod value;

use super::*;
use async_trait::async_trait;
use runtime::Coordinate;
use runtime::SpriteRuntime;

fn get_block(
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    info: &savefile::Block,
) -> Result<Box<dyn Block>> {
    Ok(match info.opcode.as_str() {
        "control_if" => Box::new(control::If::new(id, runtime)),
        "control_forever" => Box::new(control::Forever::new(id)),
        "control_repeat" => Box::new(control::Repeat::new(id)),
        "control_wait" => Box::new(control::Wait::new(id)),

        "data_setvariableto" => Box::new(data::SetVariable::new(id, runtime)),
        "data_changevariableby" => Box::new(data::ChangeVariable::new(id, runtime)),

        "event_whenflagclicked" => Box::new(event::WhenFlagClicked::new(id, runtime)),

        "looks_say" => Box::new(looks::Say::new(id, runtime)),
        "looks_sayforsecs" => Box::new(looks::SayForSecs::new(id, runtime)),

        "motion_movesteps" => Box::new(motion::MoveSteps::new(id, runtime)),
        "motion_gotoxy" => Box::new(motion::GoToXY::new(id, runtime)),
        "motion_changexby" => Box::new(motion::ChangeXBy::new(id, runtime)),
        "motion_changeyby" => Box::new(motion::ChangeYBy::new(id, runtime)),

        "operator_equals" => Box::new(operator::Equals::new(id)),
        "operator_add" => Box::new(operator::Add::new(id)),
        "operator_subtract" => Box::new(operator::Subtract::new(id)),
        "operator_multiply" => Box::new(operator::Multiply::new(id)),
        "operator_divide" => Box::new(operator::Divide::new(id)),
        _ => return Err(format!("block \"{}\": opcode {} does not exist", id, info.opcode).into()),
    })
}

/// https://en.scratch-wiki.info/wiki/Scratch_File_Format
pub fn new_value(value_type: i64, value: serde_json::Value) -> Result<Box<dyn Block>> {
    use std::convert::TryFrom;
    Ok(match value_type {
        4 | 5 | 6 | 7 => Box::new(value::Number::try_from(value)?),
        10 => Box::new(value::BlockString::try_from(value)?),
        _ => return Err(format!("value_type {} does not exist", value_type).into()),
    })
}

#[async_trait(?Send)]
pub trait Block: std::fmt::Debug {
    fn block_name(&self) -> &'static str;

    fn id(&self) -> &str {
        unreachable!()
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>);

    #[allow(unused_variables)]
    fn set_field(&mut self, key: &str, value_id: String) {}

    fn next(&mut self) -> Next {
        unreachable!()
    }

    fn value(&self) -> Result<serde_json::Value> {
        Err("this block does not return a value".into())
    }

    async fn execute(&mut self) -> Result<()> {
        Err("this block cannot be executed".into())
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

impl std::convert::From<Option<Rc<RefCell<Box<dyn Block>>>>> for Next {
    fn from(next: Option<Rc<RefCell<Box<dyn Block>>>>) -> Self {
        match next {
            Some(b) => Next::Continue(b),
            None => Next::None,
        }
    }
}

pub fn new_block(
    block_id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    infos: &HashMap<String, savefile::Block>,
) -> Result<Box<dyn Block>> {
    let info = infos.get(block_id.as_str()).unwrap();
    let mut block = get_block(block_id.clone(), runtime.clone(), &info)?;
    if let Some(next_id) = &info.next {
        block.set_input("next", new_block(next_id.clone(), runtime.clone(), infos)?);
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
                let value_type = value_info
                    .get(0)
                    .and_then(|v| v.as_i64())
                    .ok_or_else(input_err_cb)?;
                let js_value = value_info.get(1).ok_or_else(input_err_cb)?;
                let value = new_value(value_type, js_value.clone())
                    .map_err(|e| format!("block \"{}\": {}", block_id, e.to_string()))?;
                block.set_input(k, value);
            }
            2 | 3 => {
                let input_info = input_arr.get(1).ok_or_else(input_err_cb)?;
                match input_info {
                    serde_json::Value::String(id) => {
                        let new_block = new_block(id.clone(), runtime.clone(), infos)?;
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
        serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| wrong_type_err(&value))?,
        _ => return Err(format!("expected String or Number but got: {:?}", value).into()),
    })
}

fn wrong_type_err(value: &serde_json::Value) -> Error {
    format!("value has wrong type: {:?}", value).into()
}
