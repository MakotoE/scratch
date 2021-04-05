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
        _ => return Err(Error::msg(format!("{} does not exist", name))),
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
                ("OPERAND1", self.operand1.as_ref()),
                ("OPERAND2", self.operand2.as_ref()),
            ],
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
            vec![("NUM1", self.num1.as_ref()), ("NUM2", self.num2.as_ref())],
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
            vec![("NUM1", self.num1.as_ref()), ("NUM2", self.num2.as_ref())],
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
            vec![("NUM1", self.num1.as_ref()), ("NUM2", self.num2.as_ref())],
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
            vec![("NUM1", self.num1.as_ref()), ("NUM2", self.num2.as_ref())],
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
                ("OPERAND1", self.operand1.as_ref()),
                ("OPERAND2", self.operand2.as_ref()),
            ],
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
                ("OPERAND1", self.operand1.as_ref()),
                ("OPERAND2", self.operand2.as_ref()),
            ],
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
            vec![("OPERAND", self.operand.as_ref())],
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
                ("OPERAND1", self.operand1.as_ref()),
                ("OPERAND2", self.operand2.as_ref()),
            ],
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
                ("OPERAND1", self.operand1.as_ref()),
                ("OPERAND2", self.operand2.as_ref()),
            ],
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

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::value::{ValueBool, ValueNumber};

    #[rstest]
    #[case(0.0, 0.0, true)]
    #[case(0.0, 1.0, false)]
    #[tokio::test]
    async fn equals(#[case] a: f64, #[case] b: f64, #[case] expected: bool) {
        let operand1 = Box::new(ValueNumber::new(a));
        let operand2 = Box::new(ValueNumber::new(b));
        let mut equals = Equals::new(BlockID::default());
        equals.set_input("OPERAND1", operand1);
        equals.set_input("OPERAND2", operand2);
        assert_eq!(equals.value().await.unwrap(), Value::Bool(expected));
    }

    #[tokio::test]
    async fn add() {
        let num1 = Box::new(ValueNumber::new(1.0));
        let num2 = Box::new(ValueNumber::new(2.0));
        let mut add = Add::new(BlockID::default());
        add.set_input("NUM1", num1);
        add.set_input("NUM2", num2);
        assert_eq!(add.value().await.unwrap(), Value::Number(3.0));
    }

    #[tokio::test]
    async fn subtract() {
        let num1 = Box::new(ValueNumber::new(3.0));
        let num2 = Box::new(ValueNumber::new(1.0));
        let mut subtract = Subtract::new(BlockID::default());
        subtract.set_input("NUM1", num1);
        subtract.set_input("NUM2", num2);
        assert_eq!(subtract.value().await.unwrap(), Value::Number(2.0));
    }

    #[tokio::test]
    async fn multiply() {
        let num1 = Box::new(ValueNumber::new(2.0));
        let num2 = Box::new(ValueNumber::new(3.0));
        let mut multiply = Multiply::new(BlockID::default());
        multiply.set_input("NUM1", num1);
        multiply.set_input("NUM2", num2);
        assert_eq!(multiply.value().await.unwrap(), Value::Number(6.0));
    }

    #[tokio::test]
    async fn divide() {
        let num1 = Box::new(ValueNumber::new(6.0));
        let num2 = Box::new(ValueNumber::new(3.0));
        let mut divide = Divide::new(BlockID::default());
        divide.set_input("NUM1", num1);
        divide.set_input("NUM2", num2);
        assert_eq!(divide.value().await.unwrap(), Value::Number(2.0));
    }

    #[rstest]
    #[case(false, false, false)]
    #[case(false, true, false)]
    #[case(true, true, true)]
    #[tokio::test]
    async fn and(#[case] a: bool, #[case] b: bool, #[case] expected: bool) {
        let operand1 = Box::new(ValueBool::new(a));
        let operand2 = Box::new(ValueBool::new(b));
        let mut and = And::new(BlockID::default());
        and.set_input("OPERAND1", operand1);
        and.set_input("OPERAND2", operand2);
        assert_eq!(and.value().await.unwrap(), Value::Bool(expected));
    }

    #[rstest]
    #[case(false, false, false)]
    #[case(false, true, true)]
    #[case(true, true, true)]
    #[tokio::test]
    async fn or(#[case] a: bool, #[case] b: bool, #[case] expected: bool) {
        let operand1 = Box::new(ValueBool::new(a));
        let operand2 = Box::new(ValueBool::new(b));
        let mut or = Or::new(BlockID::default());
        or.set_input("OPERAND1", operand1);
        or.set_input("OPERAND2", operand2);
        assert_eq!(or.value().await.unwrap(), Value::Bool(expected));
    }

    #[rstest]
    #[case(false, true)]
    #[case(true, false)]
    #[tokio::test]
    async fn not(#[case] a: bool, #[case] expected: bool) {
        let operand = Box::new(ValueBool::new(a));
        let mut or = Not::new(BlockID::default());
        or.set_input("OPERAND", operand);
        assert_eq!(or.value().await.unwrap(), Value::Bool(expected));
    }

    #[rstest]
    #[case(0.0, 0.0, false)]
    #[case(0.0, 1.0, true)]
    #[tokio::test]
    async fn less_than(#[case] a: f64, #[case] b: f64, #[case] expected: bool) {
        let operand1 = Box::new(ValueNumber::new(a));
        let operand2 = Box::new(ValueNumber::new(b));
        let mut less_than = LessThan::new(BlockID::default());
        less_than.set_input("OPERAND1", operand1);
        less_than.set_input("OPERAND2", operand2);
        assert_eq!(less_than.value().await.unwrap(), Value::Bool(expected));
    }

    #[rstest]
    #[case(0.0, 0.0, false)]
    #[case(1.0, 0.0, true)]
    #[tokio::test]
    async fn greater_than(#[case] a: f64, #[case] b: f64, #[case] expected: bool) {
        let operand1 = Box::new(ValueNumber::new(a));
        let operand2 = Box::new(ValueNumber::new(b));
        let mut greater_than = GreaterThan::new(BlockID::default());
        greater_than.set_input("OPERAND1", operand1);
        greater_than.set_input("OPERAND2", operand2);
        assert_eq!(greater_than.value().await.unwrap(), Value::Bool(expected));
    }
}
