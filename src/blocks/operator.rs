use super::*;

pub fn get_block(
    name: &str,
    id: BlockID,
    _runtime: Runtime,
) -> Result<Box<dyn Block + Send + Sync>> {
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
        _ => return Err(Error::msg(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Equals {
    id: BlockID,
    operand1: Box<dyn Block + Send + Sync>,
    operand2: Box<dyn Block + Send + Sync>,
}

impl Equals {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: Box::new(EmptyInput {}),
            operand2: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for Equals {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Equals",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![
                ("operand1", self.operand1.as_ref()),
                ("operand2", self.operand2.as_ref()),
            ],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "OPERAND1" => self.operand1 = block,
            "OPERAND2" => self.operand2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let a = self.operand1.value().await?;
        let b = self.operand2.value().await?;
        Ok((a == b).into())
    }
}

#[derive(Debug)]
pub struct Add {
    id: BlockID,
    num1: Box<dyn Block + Send + Sync>,
    num2: Box<dyn Block + Send + Sync>,
}

impl Add {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            num1: Box::new(EmptyInput {}),
            num2: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for Add {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Add",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("num1", self.num1.as_ref()), ("num2", self.num2.as_ref())],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "NUM1" => self.num1 = block,
            "NUM2" => self.num2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let a: f64 = self.num1.value().await?.try_into()?;
        let b: f64 = self.num2.value().await?.try_into()?;
        Ok((a + b).into())
    }
}

#[derive(Debug)]
pub struct Subtract {
    id: BlockID,
    num1: Box<dyn Block + Send + Sync>,
    num2: Box<dyn Block + Send + Sync>,
}

impl Subtract {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            num1: Box::new(EmptyInput {}),
            num2: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for Subtract {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Subtract",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("num1", self.num1.as_ref()), ("num2", self.num2.as_ref())],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "NUM1" => self.num1 = block,
            "NUM2" => self.num2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let a: f64 = self.num1.value().await?.try_into()?;
        let b: f64 = self.num2.value().await?.try_into()?;
        Ok((a - b).into())
    }
}

#[derive(Debug)]
pub struct Multiply {
    id: BlockID,
    num1: Box<dyn Block + Send + Sync>,
    num2: Box<dyn Block + Send + Sync>,
}

impl Multiply {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            num1: Box::new(EmptyInput {}),
            num2: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for Multiply {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Multiply",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("num1", self.num1.as_ref()), ("num2", self.num2.as_ref())],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "NUM1" => self.num1 = block,
            "NUM2" => self.num2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let a: f64 = self.num1.value().await?.try_into()?;
        let b: f64 = self.num2.value().await?.try_into()?;
        Ok((a * b).into())
    }
}

#[derive(Debug)]
pub struct Divide {
    id: BlockID,
    num1: Box<dyn Block + Send + Sync>,
    num2: Box<dyn Block + Send + Sync>,
}

impl Divide {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            num1: Box::new(EmptyInput {}),
            num2: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for Divide {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Divide",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("num1", self.num1.as_ref()), ("num2", self.num2.as_ref())],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "NUM1" => self.num1 = block,
            "NUM2" => self.num2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let a: f64 = self.num1.value().await?.try_into()?;
        let b: f64 = self.num2.value().await?.try_into()?;
        Ok((a / b).into())
    }
}

#[derive(Debug)]
pub struct And {
    id: BlockID,
    operand1: Box<dyn Block + Send + Sync>,
    operand2: Box<dyn Block + Send + Sync>,
}

impl And {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: Box::new(EmptyFalse {}),
            operand2: Box::new(EmptyFalse {}),
        }
    }
}

#[async_trait]
impl Block for And {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "And",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![
                ("operand1", self.operand1.as_ref()),
                ("operand2", self.operand2.as_ref()),
            ],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "OPERAND1" => self.operand1 = block,
            "OPERAND2" => self.operand2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let left: bool = self.operand1.value().await?.try_into()?;
        let right = self.operand2.value().await?.try_into()?;
        Ok((left && right).into())
    }
}

#[derive(Debug)]
pub struct Or {
    id: BlockID,
    operand1: Box<dyn Block + Send + Sync>,
    operand2: Box<dyn Block + Send + Sync>,
}

impl Or {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: Box::new(EmptyFalse {}),
            operand2: Box::new(EmptyFalse {}),
        }
    }
}

#[async_trait]
impl Block for Or {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Or",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![
                ("operand1", self.operand1.as_ref()),
                ("operand2", self.operand2.as_ref()),
            ],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "OPERAND1" => self.operand1 = block,
            "OPERAND2" => self.operand2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let left: bool = self.operand1.value().await?.try_into()?;
        let right: bool = self.operand2.value().await?.try_into()?;
        Ok((left || right).into())
    }
}

#[derive(Debug)]
pub struct Not {
    id: BlockID,
    operand: Box<dyn Block + Send + Sync>,
}

impl Not {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand: Box::new(EmptyFalse {}),
        }
    }
}

#[async_trait]
impl Block for Not {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Not",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("operand", self.operand.as_ref())],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        if key == "OPERAND" {
            self.operand = block;
        }
    }

    async fn value(&self) -> Result<Value> {
        let operand: bool = self.operand.value().await?.try_into()?;
        Ok((!operand).into())
    }
}

#[derive(Debug)]
pub struct LessThan {
    id: BlockID,
    operand1: Box<dyn Block + Send + Sync>,
    operand2: Box<dyn Block + Send + Sync>,
}

impl LessThan {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: Box::new(EmptyInput {}),
            operand2: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for LessThan {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "LessThan",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![
                ("operand1", self.operand1.as_ref()),
                ("operand2", self.operand2.as_ref()),
            ],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "OPERAND1" => self.operand1 = block,
            "OPERAND2" => self.operand2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let left: f64 = self.operand1.value().await?.try_into()?;
        let right: f64 = self.operand2.value().await?.try_into()?;
        Ok((left < right).into())
    }
}

#[derive(Debug)]
pub struct GreaterThan {
    id: BlockID,
    operand1: Box<dyn Block + Send + Sync>,
    operand2: Box<dyn Block + Send + Sync>,
}

impl GreaterThan {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand1: Box::new(EmptyInput {}),
            operand2: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for GreaterThan {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GreaterThan",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![
                ("operand1", self.operand1.as_ref()),
                ("operand2", self.operand2.as_ref()),
            ],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        match key {
            "OPERAND1" => self.operand1 = block,
            "OPERAND2" => self.operand2 = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let left: f64 = self.operand1.value().await?.try_into()?;
        let right: f64 = self.operand2.value().await?.try_into()?;
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

#[async_trait]
impl Block for Random {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Random",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block + Send + Sync>) {}
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

#[async_trait]
impl Block for Join {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Join",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block + Send + Sync>) {}
}
