use super::*;
use crate::color::{color_to_hex, hex_to_color};
use palette::Hsv;
use serde::Serializer;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::iter::repeat;

#[derive(Debug)]
pub struct Variable {
    id: String,
    runtime: Runtime,
}

impl Variable {
    pub fn new(id: String, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for Variable {
    fn block_info(&self) -> BlockInfo {
        // Start from first dash or zero
        let start_index = self.id.find('-').map_or(0, |n| n + 1);
        let bytes: Vec<u8> = self
            .id
            .bytes()
            .skip(start_index)
            .take(self.id.bytes().len() - start_index - 1) // Truncate last dash
            .chain(repeat(b' ')) // Ensure length
            .take(20)
            .collect();

        let mut b: [u8; 20] = [0; 20];
        b.copy_from_slice(&bytes);
        BlockInfo {
            name: "Variable",
            id: BlockID::try_from(std::str::from_utf8(&b).unwrap()).unwrap(),
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<Value> {
        self.runtime.global.variables.get(&self.id).await
    }
}

pub fn value_block_from_input_arr(arr: &[serde_json::Value]) -> Result<Box<dyn Block>> {
    // https://en.scratch-wiki.info/wiki/Scratch_File_Format#Blocks
    let err = || wrap_err!("invalid input");
    let value_type = arr.get(0).ok_or_else(err)?.as_i64().ok_or_else(err)?;
    let value = arr.get(1).ok_or_else(err)?;
    Ok(match value_type {
        4 => Box::new(ValueNumber {
            number: value.as_f64().unwrap(),
        }),
        9 => Box::new(ValueColor::new(value.as_str().unwrap())?),
        _ => return Err(wrap_err!(format!("unknown value_type: {}", value_type))),
    })
}

#[derive(Debug)]
pub struct ValueNumber {
    number: f64,
}

#[async_trait(?Send)]
impl Block for ValueNumber {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Number",
            id: BlockID::default(),
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("number", self.number.to_string())],
            vec![],
            vec![],
        )
    }

    async fn value(&self) -> Result<Value> {
        Ok(Value::Number(self.number))
    }
}

#[derive(Debug)]
pub struct ValueString {
    string: String,
}

#[async_trait(?Send)]
impl Block for ValueString {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "String",
            id: BlockID::default(),
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("string", self.string.clone())],
            vec![],
            vec![],
        )
    }

    async fn value(&self) -> Result<Value> {
        Ok(Value::String(self.string.clone()))
    }
}

#[derive(Debug)]
pub struct ValueColor {
    color: Hsv,
}

impl ValueColor {
    fn new(value: &str) -> Result<Self> {
        Ok(Self {
            color: hex_to_color(value)?,
        })
    }
}

#[async_trait(?Send)]
impl Block for ValueColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Color",
            id: BlockID::default(),
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("color", color_to_hex(&self.color))],
            vec![],
            vec![],
        )
    }

    async fn value(&self) -> Result<Value> {
        Ok(Value::Color(self.color))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Color(Hsv),
}

impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(f) => Value::Number(f.as_f64().unwrap()),
            serde_json::Value::String(s) => Value::String(s),
            _ => unimplemented!("{}", v),
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::Number(f)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<Hsv> for Value {
    fn from(c: Hsv) -> Self {
        Self::Color(c)
    }
}

impl TryInto<bool> for Value {
    type Error = Error;

    fn try_into(self) -> Result<bool> {
        if let Self::Bool(b) = self {
            Ok(b)
        } else {
            Err(wrap_err!(format!("value is not bool: {}", self)))
        }
    }
}

impl TryInto<f64> for Value {
    type Error = Error;

    fn try_into(self) -> Result<f64> {
        (&self).try_into()
    }
}

impl TryInto<f64> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<f64> {
        Ok(match self {
            Value::Number(f) => {
                if f.is_nan() {
                    0.0
                } else {
                    *f
                }
            }
            Value::String(s) => s.parse()?,
            _ => {
                return Err(wrap_err!(format!(
                    "expected String or Number but got: {:?}",
                    self
                )))
            }
        })
    }
}

impl TryInto<Hsv> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Hsv> {
        Ok(match self {
            Self::String(s) => hex_to_color(&s)?,
            Self::Color(c) => c,
            _ => return Err(wrap_err!(format!("cannot convert {} into color", self))),
        })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => f.serialize_bool(*b),
            Self::Number(n) => f.serialize_f64(*n),
            Self::String(s) => f.write_str(&s),
            Self::Color(c) => f.write_str(&color_to_hex(c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::blocks::*;

    #[test]
    fn test_into_string() {
        struct Test {
            value: Value,
            expected: &'static str,
        }

        let tests = vec![
            Test {
                value: Value::String("a".into()),
                expected: "a",
            },
            Test {
                value: Value::Number(1.0),
                expected: "1",
            },
            Test {
                value: Value::Number(1.1),
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
