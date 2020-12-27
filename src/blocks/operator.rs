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
    operand1: Box<dyn Block>,
    operand2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    num1: Box<dyn Block>,
    num2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("num1", &self.num1), ("num2", &self.num2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    num1: Box<dyn Block>,
    num2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("num1", &self.num1), ("num2", &self.num2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    num1: Box<dyn Block>,
    num2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("num1", &self.num1), ("num2", &self.num2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    num1: Box<dyn Block>,
    num2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("num1", &self.num1), ("num2", &self.num2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    operand1: Box<dyn Block>,
    operand2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    operand1: Box<dyn Block>,
    operand2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    operand: Box<dyn Block>,
}

impl Not {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            operand: Box::new(EmptyFalse {}),
        }
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

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("operand", &self.operand)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    operand1: Box<dyn Block>,
    operand2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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
    operand1: Box<dyn Block>,
    operand2: Box<dyn Block>,
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

#[async_trait(?Send)]
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
            vec![("operand1", &self.operand1), ("operand2", &self.operand2)],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
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

#[async_trait(?Send)]
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

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}
