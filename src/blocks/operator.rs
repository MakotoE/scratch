use super::*;

pub fn get_block(
    name: &str,
    id: &str,
    _runtime: Rc<RefCell<SpriteRuntime>>,
) -> Result<Box<dyn Block>> {
    Ok(match name {
        "equals" => Box::new(Equals::new(id)),
        "add" => Box::new(Add::new(id)),
        "subtract" => Box::new(Subtract::new(id)),
        "multiply" => Box::new(Multiply::new(id)),
        "divide" => Box::new(Divide::new(id)),
        _ => return Err(format!("block \"{}\": name {} does not exist", id, name).into()),
    })
}

#[derive(Debug)]
pub struct Equals {
    id: String,
    operand1: Option<Box<dyn Block>>,
    operand2: Option<Box<dyn Block>>,
}

impl Equals {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            operand1: None,
            operand2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Equals {
    fn block_name(&self) -> &'static str {
        "Equals"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "OPERAND1" => self.operand1 = Some(block),
            "OPERAND2" => self.operand2 = Some(block),
            _ => {}
        }
    }

    fn value(&self) -> Result<serde_json::Value> {
        let a = match &self.operand1 {
            Some(a) => a.value()?,
            None => return Err("operand1 is None".into()),
        };
        let b = match &self.operand2 {
            Some(b) => b.value()?,
            None => return Err("operand2 is None".into()),
        };
        Ok((a == b).into())
    }
}

fn get_num1_and_num2(
    num1: &Option<Box<dyn Block>>,
    num2: &Option<Box<dyn Block>>,
) -> Result<(f64, f64)> {
    let a = match num1 {
        Some(a) => value_to_float(&a.value()?)?,
        None => return Err("num1 is None".into()),
    };
    let b = match num2 {
        Some(b) => value_to_float(&b.value()?)?,
        None => return Err("num2 is None".into()),
    };
    Ok((a, b))
}

#[derive(Debug)]
pub struct Add {
    id: String,
    num1: Option<Box<dyn Block>>,
    num2: Option<Box<dyn Block>>,
}

impl Add {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            num1: None,
            num2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Add {
    fn block_name(&self) -> &'static str {
        "Add"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "NUM1" => self.num1 = Some(block),
            "NUM2" => self.num2 = Some(block),
            _ => {}
        }
    }

    fn value(&self) -> Result<serde_json::Value> {
        let (a, b) = get_num1_and_num2(&self.num1, &self.num2)?;
        Ok((a + b).into())
    }
}

#[derive(Debug)]
pub struct Subtract {
    id: String,
    num1: Option<Box<dyn Block>>,
    num2: Option<Box<dyn Block>>,
}

impl Subtract {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            num1: None,
            num2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Subtract {
    fn block_name(&self) -> &'static str {
        "Add"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "NUM1" => self.num1 = Some(block),
            "NUM2" => self.num2 = Some(block),
            _ => {}
        }
    }

    fn value(&self) -> Result<serde_json::Value> {
        let (a, b) = get_num1_and_num2(&self.num1, &self.num2)?;
        Ok((a - b).into())
    }
}

#[derive(Debug)]
pub struct Multiply {
    id: String,
    num1: Option<Box<dyn Block>>,
    num2: Option<Box<dyn Block>>,
}

impl Multiply {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            num1: None,
            num2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Multiply {
    fn block_name(&self) -> &'static str {
        "Multiply"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "NUM1" => self.num1 = Some(block),
            "NUM2" => self.num2 = Some(block),
            _ => {}
        }
    }

    fn value(&self) -> Result<serde_json::Value> {
        let (a, b) = get_num1_and_num2(&self.num1, &self.num2)?;
        Ok((a * b).into())
    }
}

#[derive(Debug)]
pub struct Divide {
    id: String,
    num1: Option<Box<dyn Block>>,
    num2: Option<Box<dyn Block>>,
}

impl Divide {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            num1: None,
            num2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Divide {
    fn block_name(&self) -> &'static str {
        "Divide"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "NUM1" => self.num1 = Some(block),
            "NUM2" => self.num2 = Some(block),
            _ => {}
        }
    }

    fn value(&self) -> Result<serde_json::Value> {
        let (a, b) = get_num1_and_num2(&self.num1, &self.num2)?;
        Ok((a / b).into())
    }
}
