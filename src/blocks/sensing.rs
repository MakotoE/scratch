use super::*;

pub fn get_block(name: &str, id: String, runtime: Runtime) -> Result<Box<dyn Block>> {
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
    id: String,
}

impl KeyPressed {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for KeyPressed {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "KeyPressed",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct KeyOptions {
    id: String,
}

impl KeyOptions {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for KeyOptions {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "KeyOptions",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct ColorIsTouchingColor {
    id: String,
}

impl ColorIsTouchingColor {
    pub fn new(id: String, _runtime: Runtime) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for ColorIsTouchingColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ColorIsTouchingColor",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct TouchingColor {
    id: String,
}

impl TouchingColor {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for TouchingColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingColor",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct TouchingObject {
    id: String,
}

impl TouchingObject {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for TouchingObject {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingObject",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct TouchingObjectMenu {
    id: String,
}

impl TouchingObjectMenu {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for TouchingObjectMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingObjectMenu",
            id: self.id.clone(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: HashMap::new(),
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}
