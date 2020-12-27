use super::*;
use crate::color::{color_to_hex, hex_to_color};
use palette::Hsv;
use std::convert::TryFrom;
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

    async fn value(&self) -> Result<serde_json::Value> {
        self.runtime.global.variables.get(&self.id).await
    }
}

pub fn value_block_from_input_arr(arr: &[serde_json::Value]) -> Result<Box<dyn Block>> {
    let err = || wrap_err!("invalid input");
    let value_type = arr.get(0).ok_or_else(err)?.as_i64().ok_or_else(err)?;
    let value = arr.get(1).ok_or_else(err)?;
    Ok(match value_type {
        9 => Box::new(ValueColor::new(value)?),
        _ => Box::new(Value {
            value: value.clone(),
        }), // TODO add more values
    })
}

#[derive(Debug)]
pub struct Value {
    value: serde_json::Value,
}

#[async_trait(?Send)]
impl Block for Value {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Value",
            id: BlockID::default(),
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("value", self.value.to_string())],
            vec![],
            vec![],
        )
    }

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.value.clone())
    }
}

#[derive(Debug)]
pub struct ValueColor {
    color: Hsv,
}

impl ValueColor {
    fn new(value: &serde_json::Value) -> Result<Self> {
        Ok(Self {
            color: hex_to_color(&value_to_string(value.clone()))?,
        })
    }
}

#[async_trait(?Send)]
impl Block for ValueColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ColorValue",
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

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(color_to_hex(&self.color).into())
    }
}
