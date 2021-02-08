use super::*;
use palette::{Hsv, IntoColor};
use serde::Serializer;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::iter::repeat;
use std::str::FromStr;

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

#[async_trait]
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

    fn set_input(&mut self, _: &str, _: Box<dyn Block + Send + Sync>) {}

    async fn value(&self) -> Result<Value> {
        self.runtime.global.variables.get(&self.id).await
    }
}

pub fn value_block_from_input_arr(
    arr: &[serde_json::Value],
) -> Result<Box<dyn Block + Send + Sync>> {
    // https://en.scratch-wiki.info/wiki/Scratch_File_Format#Blocks
    let err = || Error::msg("invalid input");
    let value_type = arr.get(0).ok_or_else(err)?.as_i64().ok_or_else(err)?;
    let value = arr.get(1).ok_or_else(err)?;
    Ok(match value_type {
        4 | 5 | 6 | 7 | 8 => {
            let number = if let Some(f) = value.as_f64() {
                f
            } else {
                f64::from_str(value.as_str().unwrap())?
            };
            Box::new(ValueNumber { number })
        }
        9 => Box::new(ValueColor::new(value.as_str().unwrap())?),
        10 | 11 => Box::new(ValueString {
            string: if let serde_json::Value::String(s) = value {
                s.clone()
            } else {
                value.to_string()
            },
        }),
        _ => return Err(Error::msg(format!("unknown value_type: {}", value_type))),
    })
}

#[derive(Debug)]
pub struct ValueNumber {
    number: f64,
}

#[async_trait]
impl Block for ValueNumber {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Number",
            id: BlockID::pseudo_id(),
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

#[async_trait]
impl Block for ValueString {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "String",
            id: BlockID::pseudo_id(),
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
            color: str_to_color(value)?,
        })
    }
}

#[async_trait]
impl Block for ValueColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Color",
            id: BlockID::pseudo_id(),
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("color", format!("{}", HsvDisplay(self.color)))],
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
    TouchingObjectOption(sensing::TouchingObjectOption),
    KeyOption(event_sender::KeyOption),
    StopOption(control::StopOption),
    GoToOption(motion::GoToOption),
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
            Err(Error::msg(format!("value is not bool: {}", self)))
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
                return Err(Error::msg(format!(
                    "expected String or Number but got: {:?}",
                    self
                )))
            }
        })
    }
}

fn str_to_color(s: &str) -> Result<Hsv> {
    if s.len() != 7 || s.bytes().next() != Some(b'#') {
        return Err(Error::msg(format!("string is invalid: {}", s)));
    }

    let rgb = palette::Srgb::new(
        u8::from_str_radix(&s[1..3], 16)? as f32 / 255.0,
        u8::from_str_radix(&s[3..5], 16)? as f32 / 255.0,
        u8::from_str_radix(&s[5..7], 16)? as f32 / 255.0,
    );
    Ok(Hsv::from(rgb))
}

pub struct HsvDisplay(pub Hsv);

impl Display for HsvDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let rgb = palette::Srgb::from_linear(self.0.into_rgb());
        write!(
            f,
            "#{:02x}{:02x}{:02x}",
            (rgb.red * 255.0).round() as u8,
            (rgb.green * 255.0).round() as u8,
            (rgb.blue * 255.0).round() as u8
        )
    }
}

impl TryInto<Hsv> for Value {
    type Error = Error;

    fn try_into(self) -> Result<Hsv> {
        Ok(match self {
            Self::String(s) => str_to_color(&s)?,
            Self::Color(c) => c,
            _ => return Err(Error::msg(format!("cannot convert {} into color", self))),
        })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let o: &dyn Display = match self {
            Self::Bool(b) => return f.serialize_bool(*b),
            Self::Number(n) => return f.serialize_f64(*n),
            Self::String(s) => return f.write_str(&s),
            Self::Color(c) => return HsvDisplay(*c).fmt(f),
            Self::TouchingObjectOption(o) => o,
            Self::KeyOption(o) => o,
            Self::StopOption(o) => o,
            Self::GoToOption(o) => o,
        };
        Display::fmt(o, f)
    }
}

#[macro_export]
macro_rules! impl_try_from_value {
    ( $value_name:ident ) => {
        impl std::convert::TryFrom<Value> for $value_name {
            type Error = Error;

            fn try_from(value: Value) -> Result<Self> {
                Ok(match value {
                    Value::String(s) => Self::from_str(&s)?,
                    Value::$value_name(o) => o,
                    _ => return Err(Error::msg(format!("cannot convert value: {}", value))),
                })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        s,
        expected,
        expect_err,
        case("", Hsv::new(0.0, 0.0, 0.0), true),
        case("#", Hsv::new(0.0, 0.0, 0.0), true),
        case("#000000", Hsv::new(0.0, 0.0, 0.0), false),
        case("#ffffff", Hsv::new(0.0, 0.0, 1.0), false),
        case("#ffffffa", Hsv::new(0.0, 0.0, 0.0), true)
    )]
    fn test_str_to_color(s: &'static str, expected: Hsv, expect_err: bool) {
        let result = str_to_color(s);
        assert_eq!(result.is_err(), expect_err);
        if !expect_err {
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    fn test_fmt_hsv() {
        let result = format!("{}", HsvDisplay(Hsv::new(0.0, 1.0, 1.0)));
        assert_eq!(result, "#ff0000")
    }

    #[rstest(
        value,
        expected,
        case (Value::String("a".into()), "a"),
        case (Value::Number(1.0), "1"),
        case (Value::Number(1.1), "1.1"),
        case (Value::Bool(false), "false"),
    )]
    fn test_to_string(value: Value, expected: &'static str) {
        assert_eq!(value.to_string(), expected);
    }
}
