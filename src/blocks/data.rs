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

#[async_trait]
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
            vec![("VARIABLE", self.variable_id.clone())],
            vec![("VALUE", self.value.as_ref())],
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
            .await;
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

#[async_trait]
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
            vec![("VARIABLE", self.variable_id.clone())],
            vec![("VALUE", self.value.as_ref())],
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

#[async_trait]
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
            vec![("VARIABLE", self.variable_id.clone())],
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

#[async_trait]
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
            vec![("VARIABLE", self.variable_id.clone())],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::value::ValueNumber;
    use crate::file::BlockIDGenerator;

    const KEY: &str = "key";

    #[tokio::test]
    async fn set_variable() {
        let runtime = Runtime::default();

        let mut gen = BlockIDGenerator::new();
        let mut set_variable = SetVariable::new(gen.get_id(), runtime.clone());
        assert!(set_variable.execute().await.is_err());

        set_variable.set_input("VALUE", Box::new(ValueNumber::new(1.0)));
        set_variable
            .set_field("VARIABLE", &[None, Some(KEY.to_string())])
            .unwrap();

        set_variable.execute().await.unwrap();

        assert_eq!(
            runtime.global.variables.get(KEY).await.unwrap(),
            Value::Number(1.0)
        );
    }

    #[tokio::test]
    async fn change_variable() {
        let runtime = Runtime::default();
        runtime.global.variables.set(KEY, Value::Number(1.0)).await;

        let mut gen = BlockIDGenerator::new();
        let mut change_variable = ChangeVariable::new(gen.get_id(), runtime.clone());
        assert!(change_variable.execute().await.is_err());

        change_variable.set_input("VALUE", Box::new(ValueNumber::new(1.0)));
        change_variable
            .set_field("VARIABLE", &[None, Some(KEY.to_string())])
            .unwrap();

        change_variable.execute().await.unwrap();
        assert_eq!(
            runtime.global.variables.get(KEY).await.unwrap(),
            Value::Number(2.0)
        );

        runtime
            .global
            .variables
            .set(KEY, Value::String(String::new()))
            .await;
        change_variable.set_input("VALUE", Box::new(ValueNumber::new(1.0)));
        change_variable.execute().await.unwrap();
        assert_eq!(
            runtime.global.variables.get(KEY).await.unwrap(),
            Value::Number(1.0)
        );
    }

    #[tokio::test]
    async fn hide_variable_and_show_variable() {
        let runtime = Runtime::default();
        runtime.global.variables.set(KEY, Value::Number(1.0)).await;

        let mut gen = BlockIDGenerator::new();
        let mut hide_variable = HideVariable::new(gen.get_id(), runtime.clone());
        hide_variable
            .set_field("VARIABLE", &[None, Some(KEY.to_string())])
            .unwrap();

        hide_variable.execute().await.unwrap();

        assert!(!runtime.global.variables.monitored(KEY).await);

        let mut show_variable = ShowVariable::new(gen.get_id(), runtime.clone());
        show_variable
            .set_field("VARIABLE", &[None, Some(KEY.to_string())])
            .unwrap();

        show_variable.execute().await.unwrap();

        assert!(runtime.global.variables.monitored(KEY).await);
    }
}
