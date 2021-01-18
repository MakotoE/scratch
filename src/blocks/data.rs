use super::*;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "setvariableto" => Box::new(SetVariable::new(id, runtime)),
        "changevariableby" => Box::new(ChangeVariable::new(id, runtime)),
        "hidevariable" => Box::new(HideVariable::new(id, runtime)),
        "showvariable" => Box::new(ShowVariable::new(id, runtime)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct SetVariable {
    id: BlockID,
    runtime: Runtime,
    variable_id: String,
    value: Box<dyn Block>,
    next: Option<BlockID>,
}

impl SetVariable {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            variable_id: String::new(),
            value: Box::new(EmptyInput {}),
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetVariable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetVariable",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("variable_id", self.variable_id.clone())],
            vec![("value", self.value.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "VALUE" {
            self.value = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "VARIABLE" {
            self.variable_id = get_field_value(field, 1)?.to_string();
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        let value = self.value.value().await?;
        self.runtime
            .global
            .variables
            .set(&self.variable_id, value)
            .await?;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct ChangeVariable {
    id: BlockID,
    runtime: Runtime,
    variable_id: String,
    value: Box<dyn Block>,
    next: Option<BlockID>,
}

impl ChangeVariable {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            variable_id: String::new(),
            value: Box::new(EmptyInput {}),
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeVariable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeVariable",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("variable_id", self.variable_id.clone())],
            vec![("value", self.value.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "VALUE" {
            self.value = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "VARIABLE" {
            self.variable_id = get_field_value(field, 1)?.to_string();
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        let value: f64 = self.value.value().await?.try_into()?;
        self.runtime
            .global
            .variables
            .set_with(&self.variable_id, |v| {
                let previous_float: f64 = v.try_into().unwrap_or(0.0);
                (previous_float + value).into()
            })
            .await?;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct HideVariable {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    variable_id: String,
}

impl HideVariable {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            variable_id: String::new(),
        }
    }
}

#[async_trait(?Send)]
impl Block for HideVariable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "HideVariable",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("variable_id", self.variable_id.clone())],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "VARIABLE" {
            self.variable_id = get_field_value(field, 1)?.to_string();
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        self.runtime
            .global
            .variables
            .set_monitored(&self.variable_id, false)
            .await?;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct ShowVariable {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    variable_id: String,
}

impl ShowVariable {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            variable_id: String::new(),
        }
    }
}

#[async_trait(?Send)]
impl Block for ShowVariable {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ShowVariable",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("variable_id", self.variable_id.clone())],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "VARIABLE" {
            self.variable_id = get_field_value(field, 1)?.to_string();
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        self.runtime
            .global
            .variables
            .set_monitored(&self.variable_id, true)
            .await?;
        Next::continue_(self.next)
    }
}
