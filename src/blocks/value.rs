use super::*;
use palette::{Hsv, IntoColor};
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
            color: str_to_color(value)?,
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

fn str_to_color(s: &str) -> Result<Hsv> {
    if s.len() != 7 || s.bytes().next() != Some(b'#') {
        return Err(wrap_err!(format!("string is invalid: {}", s)));
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
            Self::Color(c) => HsvDisplay(*c).fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_to_color() {
        struct Test {
            s: &'static str,
            expected: Hsv,
            expect_err: bool,
        }

        let tests = vec![
            Test {
                s: "",
                expected: Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
            Test {
                s: "#",
                expected: Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
            Test {
                s: "#000000",
                expected: Hsv::new(0.0, 0.0, 0.0),
                expect_err: false,
            },
            Test {
                s: "#ffffff",
                expected: Hsv::new(0.0, 0.0, 1.0),
                expect_err: false,
            },
            Test {
                s: "#ffffffa",
                expected: Hsv::new(0.0, 0.0, 0.0),
                expect_err: true,
            },
        ];

        for (i, test) in tests.iter().enumerate() {
            let result: Result<Hsv> = str_to_color(test.s);
            assert_eq!(result.is_err(), test.expect_err, "{}", i);
            if !test.expect_err {
                assert_eq!(result.unwrap(), test.expected, "{}", i);
            }
        }
    }

    #[test]
    fn test_fmt_hsv() {
        let result = format!("{}", HsvDisplay(Hsv::new(0.0, 1.0, 1.0)));
        assert_eq!(result, "#ff0000")
    }

    mod value {
        use super::*;

        #[test]
        fn test_to_string() {
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
                assert_eq!(test.value.to_string(), test.expected, "{}", i);
            }
        }
    }
}
