use super::*;

pub fn get_block(name: &str, id: BlockID, _runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "equals" => Box::new(Equals::new(id)),
        "add" => Box::new(Add::new(id)),
        "subtract" => Box::new(Subtract::new(id)),
        "multiply" => Box::new(Multiply::new(id)),
        "divide" => Box::new(Divide::new(id)),
        "not" => Box::new(Not::new(id)),
        "and" => Box::new(And::new(id)),
        "or" => Box::new(Or::new(id)),
        "lt" => Box::new(LessThan::new(id)),
        "gt" => Box::new(GreaterThan::new(id)),
        "random" => Box::new(Random::new(id)),
        "join" => Box::new(Join::new(id)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Equals {
    id: BlockID,
    operand1: Option<Box<dyn Block>>,
    operand2: Option<Box<dyn Block>>,
}

impl Equals {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: None,
            operand2: None,
        }
    }

    fn equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        if let Some(a_float) = a.as_f64() {
            if let Some(b_float) = b.as_f64() {
                #[allow(clippy::float_cmp)]
                return a_float == b_float;
            }
        }

        a == b
    }
}

#[async_trait(?Send)]
impl Block for Equals {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Equals",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "OPERAND1" => self.operand1 = Some(block),
            "OPERAND2" => self.operand2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let a = match &self.operand1 {
            Some(a) => a.value().await?,
            None => return Err(wrap_err!("operand1 is None")),
        };
        let b = match &self.operand2 {
            Some(b) => b.value().await?,
            None => return Err(wrap_err!("operand2 is None")),
        };

        Ok(Equals::equal(&a, &b).into())
    }
}

async fn get_num1_and_num2(
    num1: &Option<Box<dyn Block>>,
    num2: &Option<Box<dyn Block>>,
) -> Result<(f64, f64)> {
    let a = match num1 {
        Some(a) => value_to_float(&a.value().await?)?,
        None => return Err(wrap_err!("num1 is None")),
    };
    let b = match num2 {
        Some(b) => value_to_float(&b.value().await?)?,
        None => return Err(wrap_err!("num2 is None")),
    };
    Ok((a, b))
}

#[derive(Debug)]
pub struct Add {
    id: BlockID,
    num1: Option<Box<dyn Block>>,
    num2: Option<Box<dyn Block>>,
}

impl Add {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            num1: None,
            num2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Add {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Add",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("num1", &self.num1), ("num2", &self.num2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "NUM1" => self.num1 = Some(block),
            "NUM2" => self.num2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let (a, b) = get_num1_and_num2(&self.num1, &self.num2).await?;
        Ok((a + b).into())
    }
}

#[derive(Debug)]
pub struct Subtract {
    id: BlockID,
    num1: Option<Box<dyn Block>>,
    num2: Option<Box<dyn Block>>,
}

impl Subtract {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            num1: None,
            num2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Subtract {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Subtract",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("num1", &self.num1), ("num2", &self.num2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "NUM1" => self.num1 = Some(block),
            "NUM2" => self.num2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let (a, b) = get_num1_and_num2(&self.num1, &self.num2).await?;
        Ok((a - b).into())
    }
}

#[derive(Debug)]
pub struct Multiply {
    id: BlockID,
    num1: Option<Box<dyn Block>>,
    num2: Option<Box<dyn Block>>,
}

impl Multiply {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            num1: None,
            num2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Multiply {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Multiply",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("num1", &self.num1), ("num2", &self.num2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "NUM1" => self.num1 = Some(block),
            "NUM2" => self.num2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let (a, b) = get_num1_and_num2(&self.num1, &self.num2).await?;
        Ok((a * b).into())
    }
}

#[derive(Debug)]
pub struct Divide {
    id: BlockID,
    num1: Option<Box<dyn Block>>,
    num2: Option<Box<dyn Block>>,
}

impl Divide {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            num1: None,
            num2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Divide {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Divide",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("num1", &self.num1), ("num2", &self.num2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "NUM1" => self.num1 = Some(block),
            "NUM2" => self.num2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let (a, b) = get_num1_and_num2(&self.num1, &self.num2).await?;
        Ok((a / b).into())
    }
}

#[derive(Debug)]
pub struct And {
    id: BlockID,
    operand1: Option<Box<dyn Block>>,
    operand2: Option<Box<dyn Block>>,
}

impl And {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: None,
            operand2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for And {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "And",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "OPERAND1" => self.operand1 = Some(block),
            "OPERAND2" => self.operand2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let left = match &self.operand1 {
            Some(b) => {
                let value = b.value().await?;
                match value {
                    serde_json::Value::Bool(b) => b,
                    _ => return Err(wrap_err!(format!("operand1 is not a boolean: {:?}", value))),
                }
            }
            None => return Err(wrap_err!("operand1 is None")),
        };

        let right = match &self.operand2 {
            Some(b) => {
                let value = b.value().await?;
                match value {
                    serde_json::Value::Bool(b) => b,
                    _ => return Err(wrap_err!(format!("operand2 is not a boolean: {:?}", value))),
                }
            }
            None => return Err(wrap_err!("operand2 is None")),
        };

        Ok((left && right).into())
    }
}

#[derive(Debug)]
pub struct Or {
    id: BlockID,
    operand1: Option<Box<dyn Block>>,
    operand2: Option<Box<dyn Block>>,
}

impl Or {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: None,
            operand2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Or {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Or",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "OPERAND1" => self.operand1 = Some(block),
            "OPERAND2" => self.operand2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let left = match &self.operand1 {
            Some(b) => {
                let value = b.value().await?;
                match value {
                    serde_json::Value::Bool(b) => b,
                    _ => return Err(wrap_err!(format!("operand1 is not a boolean: {:?}", value))),
                }
            }
            None => return Err(wrap_err!("operand1 is None")),
        };

        let right = match &self.operand2 {
            Some(b) => {
                let value = b.value().await?;
                match value {
                    serde_json::Value::Bool(b) => b,
                    _ => return Err(wrap_err!(format!("operand2 is not a boolean: {:?}", value))),
                }
            }
            None => return Err(wrap_err!("operand2 is None")),
        };

        Ok((left || right).into())
    }
}

#[derive(Debug)]
pub struct Not {
    id: BlockID,
    operand: Option<Box<dyn Block>>,
}

impl Not {
    pub fn new(id: BlockID) -> Self {
        Self { id, operand: None }
    }
}

#[async_trait(?Send)]
impl Block for Not {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Not",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("operand", &self.operand)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "OPERAND" {
            self.operand = Some(block);
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let operand_value = match &self.operand {
            Some(b) => b.value().await?,
            None => return Err(wrap_err!("operand is None")),
        };

        let operand = match operand_value {
            serde_json::Value::Bool(b) => b,
            _ => {
                return Err(wrap_err!(format!(
                    "operand is not a boolean: {}",
                    operand_value
                )));
            }
        };

        Ok((!operand).into())
    }
}

#[derive(Debug)]
pub struct LessThan {
    id: BlockID,
    operand1: Option<Box<dyn Block>>,
    operand2: Option<Box<dyn Block>>,
}

impl LessThan {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: None,
            operand2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for LessThan {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "LessThan",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "OPERAND1" => self.operand1 = Some(block),
            "OPERAND2" => self.operand2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let left = match &self.operand1 {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Err(wrap_err!("operand1 is None")),
        };

        let right = match &self.operand2 {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Err(wrap_err!("operand2 is None")),
        };

        Ok((left < right).into())
    }
}

#[derive(Debug)]
pub struct GreaterThan {
    id: BlockID,
    operand1: Option<Box<dyn Block>>,
    operand2: Option<Box<dyn Block>>,
}

impl GreaterThan {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: None,
            operand2: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for GreaterThan {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GreaterThan",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(
            self.block_info(),
            vec![],
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "OPERAND1" => self.operand1 = Some(block),
            "OPERAND2" => self.operand2 = Some(block),
            _ => {}
        }
    }

    async fn value(&self) -> Result<serde_json::Value> {
        let left = match &self.operand1 {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Err(wrap_err!("operand1 is None")),
        };

        let right = match &self.operand2 {
            Some(b) => value_to_float(&b.value().await?)?,
            None => return Err(wrap_err!("operand2 is None")),
        };

        Ok((left > right).into())
    }
}

#[derive(Debug)]
pub struct Random {
    id: BlockID,
}

impl Random {
    pub fn new(id: BlockID) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for Random {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Random",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[derive(Debug)]
pub struct Join {
    id: BlockID,
}

impl Join {
    pub fn new(id: BlockID) -> Self {
        Self { id }
    }
}

#[async_trait(?Send)]
impl Block for Join {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Join",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    mod equals {
        use super::*;

        #[test]
        fn test_equal() {
            struct Test {
                a: serde_json::Value,
                b: serde_json::Value,
                expected: bool,
            }

            let tests = vec![
                Test {
                    a: serde_json::Value::Null,
                    b: serde_json::Value::Null,
                    expected: true,
                },
                Test {
                    a: serde_json::Value::Null,
                    b: false.into(),
                    expected: false,
                },
                Test {
                    a: 0i64.into(),
                    b: 0.0f64.into(),
                    expected: true,
                },
                Test {
                    a: 1.into(),
                    b: 0.into(),
                    expected: false,
                },
            ];

            for (i, test) in tests.iter().enumerate() {
                assert_eq!(Equals::equal(&test.a, &test.b), test.expected, "{}", i);
            }
        }
    }
}
