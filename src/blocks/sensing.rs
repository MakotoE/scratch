use super::*;
use std::fmt::Display;
use std::str::FromStr;
use wasm_bindgen::__rt::core::fmt::Formatter;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "keypressed" => Box::new(KeyPressed::new(id, runtime)),
        "keyoptions" => Box::new(KeyOptions::new(id, runtime)),
        "coloristouchingcolor" => Box::new(ColorIsTouchingColor::new(id, runtime)),
        "touchingcolor" => Box::new(TouchingColor::new(id)),
        "touchingobject" => Box::new(TouchingObject::new(id)),
        "touchingobjectmenu" => Box::new(TouchingObjectMenu::new(id)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct KeyPressed {
    id: BlockID,
}

impl KeyPressed {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for KeyPressed {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "KeyPressed",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct KeyOptions {
    id: BlockID,
}

impl KeyOptions {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for KeyOptions {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "KeyOptions",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct ColorIsTouchingColor {
    id: BlockID,
}

impl ColorIsTouchingColor {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for ColorIsTouchingColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ColorIsTouchingColor",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct TouchingColor {
    id: BlockID,
}

impl TouchingColor {
    pub fn new(id: BlockID) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for TouchingColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingColor",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct TouchingObject {
    id: BlockID,
    menu: Box<dyn Block>,
}

impl TouchingObject {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            menu: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for TouchingObject {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingObject",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "TOUCHINGOBJECTMENU" {
            self.menu = block;
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        Err(wrap_err!("this block does not return a value"))
    }
}

#[derive(Debug)]
pub struct TouchingObjectMenu {
    id: BlockID,
    option: TouchingObjectOption,
}

impl TouchingObjectMenu {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            option: TouchingObjectOption::MousePointer,
        }
    }
}

#[async_trait(?Send)]
impl Block for TouchingObjectMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingObjectMenu",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "TOUCHINGOBJECTMENU" {
            self.option = TouchingObjectOption::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn value(&self) -> Result<serde_json::Value> {
        Ok(self.option.to_string().into())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum TouchingObjectOption {
    MousePointer,
    Edge,
}

impl FromStr for TouchingObjectOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "_mouse_" => Self::MousePointer,
            "_edge_" => Self::Edge,
            _ => unimplemented!(),
        })
    }
}

impl Display for TouchingObjectOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TouchingObjectOption::MousePointer => "_mouse_",
            TouchingObjectOption::Edge => "_edge_",
        })
    }
}
