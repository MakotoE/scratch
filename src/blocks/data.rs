use super::*;

pub fn get_block(
    name: &str,
    id: &str,
    runtime: Rc<RefCell<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "setvariableto" => Box::new(SetVariable::new(id, runtime)),
        "changevariableby" => Box::new(ChangeVariable::new(id, runtime)),
        _ => return Err(format!("{} does not exist", name).into()),
    })
}

#[derive(Debug)]
pub struct SetVariable {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    variable_id: Option<String>,
    value: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl SetVariable {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            variable_id: None,
            value: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SetVariable {
    fn block_name(&self) -> &'static str {
        "SetVariable"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "VALUE" => self.value = Some(block),
            _ => {}
        }
    }

    fn set_field(&mut self, key: &str, value_id: String) {
        if key == "VARIABLE" {
            self.variable_id = Some(value_id.to_string());
        }
    }

    async fn execute(&mut self) -> Next {
        let variable_id = match &self.variable_id {
            Some(id) => id,
            None => return Next::Err("variable_id is None".into()),
        };
        let value = match &self.value {
            Some(v) => v.value()?,
            None => return Next::Err("value is None".into()),
        };
        self.runtime
            .borrow_mut()
            .variables
            .insert(variable_id.clone(), value.clone());
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct ChangeVariable {
    id: String,
    runtime: Rc<RefCell<SpriteRuntime>>,
    variable_id: Option<String>,
    value: Option<Box<dyn Block>>,
    next: Option<Rc<RefCell<Box<dyn Block>>>>,
}

impl ChangeVariable {
    pub fn new(id: &str, runtime: Rc<RefCell<SpriteRuntime>>) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            variable_id: None,
            value: None,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeVariable {
    fn block_name(&self) -> &'static str {
        "ChangeVariable"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "next" => self.next = Some(Rc::new(RefCell::new(block))),
            "VALUE" => self.value = Some(block),
            _ => {}
        }
    }

    fn set_field(&mut self, key: &str, value_id: String) {
        if key == "VARIABLE" {
            self.variable_id = Some(value_id.to_string());
        }
    }

    async fn execute(&mut self) -> Next {
        let variable_id = match &self.variable_id {
            Some(id) => id,
            None => return Next::Err("variable_id is None".into()),
        };

        let previous_value = match self.runtime.borrow_mut().variables.remove(variable_id) {
            Some(v) => v,
            None => return Next::Err(format!("variable {} does not exist", variable_id).into()),
        };

        let previous_float = value_to_float(&previous_value).unwrap_or(0.0);

        let value = match &self.value {
            Some(b) => b.value()?,
            None => return Next::Err("value is None".into()),
        };

        let new_value = previous_float + value_to_float(&value)?;
        self.runtime
            .borrow_mut()
            .variables
            .insert(variable_id.clone(), new_value.into());
        Next::continue_(self.next.clone())
    }
}
