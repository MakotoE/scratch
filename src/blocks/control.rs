use super::*;
use crate::broadcaster;
use crate::broadcaster::BroadcastMsg;
use crate::vm::ThreadID;
use std::str::FromStr;
use strum::EnumString;
use tokio::time::interval;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "if" => Box::new(If::new(id)),
        "forever" => Box::new(Forever::new(id)),
        "repeat" => Box::new(Repeat::new(id)),
        "wait" => Box::new(Wait::new(id)),
        "repeat_until" => Box::new(RepeatUntil::new(id)),
        "if_else" => Box::new(IfElse::new(id)),
        "wait_until" => Box::new(WaitUntil::new(id)),
        "start_as_clone" => Box::new(StartAsClone::new(id, runtime)),
        "delete_this_clone" => Box::new(DeleteThisClone::new(id, runtime)),
        "stop" => Box::new(Stop::new(id, runtime)),
        "create_clone_of" => Box::new(CreateCloneOf::new(id, runtime)),
        "create_clone_of_menu" => Box::new(CreateCloneOfMenu::new(id)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct If {
    id: BlockID,
    condition: Box<dyn Block>,
    next: Option<BlockID>,
    substack: Option<BlockID>,
    done: bool,
}

impl If {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            condition: Box::new(EmptyFalse {}),
            next: None,
            substack: None,
            done: false,
        }
    }
}

#[async_trait]
impl Block for If {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "If",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("condition", self.condition.as_ref())],
            vec![("next", &self.next), ("substack", &self.substack)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "CONDITION" {
            self.condition = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        match key {
            "next" => self.next = Some(block),
            "SUBSTACK" => self.substack = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        if self.done {
            self.done = false;
            return Next::continue_(self.next);
        }

        self.done = true;

        if self.condition.value().await?.try_into()? {
            return Next::loop_(self.substack);
        }

        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct Wait {
    id: BlockID,
    next: Option<BlockID>,
    duration: Box<dyn Block>,
}

impl Wait {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            next: None,
            duration: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for Wait {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Wait",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("duration", self.duration.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "DURATION" {
            self.duration = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block)
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let duration: f64 = self.duration.value().await?.try_into()?;
        sleep(Duration::from_secs_f64(duration)).await;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct Forever {
    id: BlockID,
    substack: Option<BlockID>,
}

impl Forever {
    pub fn new(id: BlockID) -> Self {
        Self { id, substack: None }
    }
}

#[async_trait]
impl Block for Forever {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Forever",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("substack", &self.substack)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "SUBSTACK" {
            self.substack = Some(block)
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        Ok(match &self.substack {
            Some(b) => Next::Loop(*b),
            None => Next::None,
        })
    }
}

#[derive(Debug)]
pub struct Repeat {
    id: BlockID,
    times: Box<dyn Block>,
    next: Option<BlockID>,
    substack: Option<BlockID>,
    count: usize,
}

impl Repeat {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            times: Box::new(EmptyInput {}),
            next: None,
            substack: None,
            count: 0,
        }
    }
}

#[async_trait]
impl Block for Repeat {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Repeat",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("times", self.times.as_ref())],
            vec![("next", &self.next), ("substack", &self.substack)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "TIMES" {
            self.times = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        match key {
            "next" => self.next = Some(block),
            "SUBSTACK" => self.substack = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let times: f64 = self.times.value().await?.try_into()?;
        if self.count < times as usize {
            // Loop until count equals times
            self.count += 1;
            return Next::loop_(self.substack);
        }

        self.count = 0;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct RepeatUntil {
    id: BlockID,
    next: Option<BlockID>,
    substack: Option<BlockID>,
    condition: Box<dyn Block>,
}

impl RepeatUntil {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            next: None,
            substack: None,
            condition: Box::new(EmptyFalse {}),
        }
    }
}

#[async_trait]
impl Block for RepeatUntil {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "RepeatUntil",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("condition", self.condition.as_ref())],
            vec![("next", &self.next), ("substack", &self.substack)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "CONDITION" {
            self.condition = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        match key {
            "next" => self.next = Some(block),
            "SUBSTACK" => self.substack = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let condition: bool = self.condition.value().await?.try_into()?;

        if condition {
            return Next::continue_(self.next);
        }

        Next::loop_(self.substack)
    }
}

#[derive(Debug)]
pub struct IfElse {
    id: BlockID,
    next: Option<BlockID>,
    condition: Box<dyn Block>,
    substack_true: Option<BlockID>,
    substack_false: Option<BlockID>,
    done: bool,
}

impl IfElse {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            next: None,
            condition: Box::new(EmptyFalse {}),
            substack_true: None,
            substack_false: None,
            done: false,
        }
    }
}

#[async_trait]
impl Block for IfElse {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "IfElse",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("condition", self.condition.as_ref())],
            vec![
                ("next", &self.next),
                ("substack_true", &self.substack_true),
                ("substack_false", &self.substack_false),
            ],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "CONDITION" {
            self.condition = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        match key {
            "next" => self.next = Some(block),
            "SUBSTACK" => self.substack_true = Some(block),
            "SUBSTACK2" => self.substack_false = Some(block),
            _ => {}
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        if self.done {
            self.done = false;
            return Next::continue_(self.next);
        }

        self.done = true;

        if self.condition.value().await?.try_into()? {
            return Next::loop_(self.substack_true);
        }

        Next::loop_(self.substack_false)
    }
}

#[derive(Debug)]
pub struct WaitUntil {
    id: BlockID,
    next: Option<BlockID>,
    condition: Box<dyn Block>,
}

impl WaitUntil {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            next: None,
            condition: Box::new(EmptyFalse {}),
        }
    }
}

#[async_trait]
impl Block for WaitUntil {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "WaitUntil",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("condition", self.condition.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "CONDITION" {
            self.condition = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let mut interval = interval(Duration::from_millis(10));
        loop {
            if self.condition.value().await?.try_into()? {
                return Next::continue_(self.next);
            }
            interval.tick().await;
        }
    }
}

#[derive(Debug)]
pub struct StartAsClone {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl StartAsClone {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait]
impl Block for StartAsClone {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "StartAsClone",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        if self.runtime.sprite.read().await.is_a_clone() {
            Next::continue_(self.next)
        } else {
            Ok(Next::None)
        }
    }
}

#[derive(Debug)]
pub struct DeleteThisClone {
    id: BlockID,
    runtime: Runtime,
}

impl DeleteThisClone {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait]
impl Block for DeleteThisClone {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "DeleteThisClone",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    async fn execute(&mut self) -> Result<Next> {
        let sprite_id = self.runtime.thread_id().sprite_id;
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::DeleteClone(sprite_id))?;
        Ok(Next::None)
    }
}

#[derive(Debug)]
pub struct Stop {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    stop_option: StopOption,
}

impl Stop {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            stop_option: StopOption::All,
        }
    }
}

#[async_trait]
impl Block for Stop {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Stop",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("STOP_OPTION", format!("{}", self.stop_option))],
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
        if key == "STOP_OPTION" {
            self.stop_option = StopOption::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        self.runtime.global.broadcaster.send(BroadcastMsg::Stop(
            self.stop_option.into_stop(self.runtime.thread_id()),
        ))?;
        Next::continue_(self.next)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, EnumString, strum::Display)]
pub enum StopOption {
    #[strum(serialize = "all")]
    All,
    #[strum(serialize = "this script")]
    ThisThread,
    #[strum(serialize = "other scripts in sprite")]
    OtherThreads,
}

impl StopOption {
    fn into_stop(self, thread_id: ThreadID) -> broadcaster::Stop {
        match self {
            StopOption::All => broadcaster::Stop::All,
            StopOption::ThisThread => broadcaster::Stop::ThisThread(thread_id),
            StopOption::OtherThreads => broadcaster::Stop::OtherThreads(thread_id),
        }
    }
}

impl_try_from_value!(StopOption);

#[derive(Debug)]
pub struct CreateCloneOf {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    clone_option: Box<dyn Block>,
}

impl CreateCloneOf {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            clone_option: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for CreateCloneOf {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "CreateCloneOf",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("CLONE_OPTION", self.clone_option.as_ref())],
            vec![("next", &self.next)],
        )
    }
    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "CLONE_OPTION" {
            self.clone_option = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let sprite_id = self.runtime.thread_id().sprite_id;
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::Clone(sprite_id))?;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct CreateCloneOfMenu {
    id: BlockID,
}

impl CreateCloneOfMenu {
    pub fn new(id: BlockID) -> Self {
        Self { id }
    }
}

#[async_trait]
impl Block for CreateCloneOfMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "CreateCloneOfMenu",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::test::{BlockStub, BlockStubMsg};
    use crate::blocks::value::{ValueBool, ValueNumber};
    use crate::file::BlockIDGenerator;
    use crate::runtime::Global;
    use crate::sprite_runtime::SpriteRuntime;
    use crate::thread::{StepStatus, Thread};
    use tokio::time::timeout;

    #[tokio::test]
    async fn if_block() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();

        let if_id = gen.get_id();
        let branch_id = gen.get_id();
        let next_id = gen.get_id();

        {
            let mut if_block = If::new(if_id);
            if_block.set_substack("SUBSTACK", branch_id);
            if_block.set_substack("next", next_id);
            if_block.set_input("CONDITION", Box::new(ValueBool::new(false)));

            let blocks = block_map(vec![
                (
                    branch_id,
                    Box::new(BlockStub::new(branch_id, runtime.clone(), None)),
                ),
                (
                    next_id,
                    Box::new(BlockStub::new(next_id, runtime.clone(), None)),
                ),
                (if_id, Box::new(if_block)),
            ]);

            let mut thread = Thread::new(if_id, blocks);
            thread.step().await.unwrap();
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
            );
            assert!(receiver.try_recv().is_err());
        }
        {
            let if_id = gen.get_id();
            let mut if_block = If::new(if_id);
            if_block.set_substack("SUBSTACK", branch_id);
            if_block.set_substack("next", next_id);
            if_block.set_input("CONDITION", Box::new(ValueBool::new(true)));

            let blocks = block_map(vec![
                (
                    branch_id,
                    Box::new(BlockStub::new(branch_id, runtime.clone(), None)),
                ),
                (
                    next_id,
                    Box::new(BlockStub::new(next_id, runtime.clone(), None)),
                ),
                (if_id, Box::new(if_block)),
            ]);

            let mut thread = Thread::new(if_id, blocks);
            thread.step().await.unwrap(); // If
            thread.step().await.unwrap(); // Substack
            thread.step().await.unwrap(); // If
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Done)); // Next

            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(branch_id, BlockStubMsg::Executed)
            );
            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
            );
            assert!(receiver.try_recv().is_err());
        }
    }

    #[tokio::test]
    async fn forever() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();
        let substack_id = gen.get_id();
        let forever_id = gen.get_id();

        let mut forever = Forever::new(forever_id);
        forever.set_substack("SUBSTACK", substack_id);

        let blocks = block_map(vec![
            (
                substack_id,
                Box::new(BlockStub::new(substack_id, runtime.clone(), None)),
            ),
            (forever_id, Box::new(forever)),
        ]);

        let mut thread = Thread::new(forever_id, blocks);
        for _ in 0..2 {
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Continue));
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Continue));
            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(substack_id, BlockStubMsg::Executed)
            );
        }

        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn repeat() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();

        let substack_id = gen.get_id();
        let next_id = gen.get_id();
        let repeat_id = gen.get_id();

        {
            let mut repeat = Repeat::new(repeat_id);
            repeat.set_input("TIMES", Box::new(ValueNumber::new(0.1)));
            repeat.set_substack("SUBSTACK", substack_id);
            repeat.set_substack("next", next_id);

            let blocks = block_map(vec![
                (
                    substack_id,
                    Box::new(BlockStub::new(substack_id, runtime.clone(), None)),
                ),
                (
                    next_id,
                    Box::new(BlockStub::new(next_id, runtime.clone(), None)),
                ),
                (repeat_id, Box::new(repeat)),
            ]);

            let mut thread = Thread::new(repeat_id, blocks);
            thread.step().await.unwrap();
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
            );
            assert!(receiver.try_recv().is_err());
        }
        {
            let mut repeat = Repeat::new(repeat_id);
            repeat.set_input("TIMES", Box::new(ValueNumber::new(1.0)));
            repeat.set_substack("SUBSTACK", substack_id);
            repeat.set_substack("next", next_id);

            let blocks = block_map(vec![
                (
                    substack_id,
                    Box::new(BlockStub::new(substack_id, runtime.clone(), None)),
                ),
                (
                    next_id,
                    Box::new(BlockStub::new(next_id, runtime.clone(), None)),
                ),
                (repeat_id, Box::new(repeat)),
            ]);

            let mut thread = Thread::new(repeat_id, blocks);
            thread.step().await.unwrap();
            thread.step().await.unwrap();
            thread.step().await.unwrap();
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(substack_id, BlockStubMsg::Executed)
            );
            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
            );
            assert!(receiver.try_recv().is_err());
        }
    }

    #[tokio::test]
    async fn wait() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();
        let next_id = gen.get_id();
        let wait_id = gen.get_id();

        let mut wait = Wait::new(wait_id);
        wait.set_input("DURATION", Box::new(ValueNumber::new(0.0)));
        wait.set_substack("next", next_id);

        let blocks = block_map(vec![
            (
                next_id,
                Box::new(BlockStub::new(next_id, runtime.clone(), None)),
            ),
            (wait_id, Box::new(wait)),
        ]);

        let mut thread = Thread::new(wait_id, blocks);
        thread.step().await.unwrap();
        assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
        );
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn repeat_until() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();
        let substack_id = gen.get_id();
        let next_id = gen.get_id();
        let repeat_until_id = gen.get_id();

        let return_value = Arc::new(RwLock::new(Value::Bool(false)));

        let mut repeat_until = RepeatUntil::new(repeat_until_id);
        repeat_until.set_input(
            "CONDITION",
            Box::new(BlockStub::new(
                gen.get_id(),
                runtime.clone(),
                Some(return_value.clone()),
            )),
        );
        repeat_until.set_substack("next", next_id);
        repeat_until.set_substack("SUBSTACK", substack_id);

        let blocks = block_map(vec![
            (
                substack_id,
                Box::new(BlockStub::new(substack_id, runtime.clone(), None)),
            ),
            (
                next_id,
                Box::new(BlockStub::new(next_id, runtime.clone(), None)),
            ),
            (repeat_until_id, Box::new(repeat_until)),
        ]);

        let mut thread = Thread::new(repeat_until_id, blocks);
        thread.step().await.unwrap();
        thread.step().await.unwrap();

        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::BlockStub(substack_id, BlockStubMsg::Executed)
        );

        *return_value.write().await = Value::Bool(true);

        thread.step().await.unwrap();
        assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
        );
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn if_else() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();
        let if_else_id = gen.get_id();
        let next_id = gen.get_id();
        let substack_true_id = gen.get_id();
        let substack_false_id = gen.get_id();

        let get_blocks = || {
            block_map(vec![
                (
                    next_id,
                    Box::new(BlockStub::new(next_id, runtime.clone(), None)),
                ),
                (
                    substack_true_id,
                    Box::new(BlockStub::new(substack_true_id, runtime.clone(), None)),
                ),
                (
                    substack_false_id,
                    Box::new(BlockStub::new(substack_false_id, runtime.clone(), None)),
                ),
            ])
        };

        {
            let mut if_else = IfElse::new(if_else_id);
            if_else.set_input("CONDITION", Box::new(ValueBool::new(false)));
            if_else.set_substack("next", next_id);
            if_else.set_substack("SUBSTACK", substack_true_id);
            if_else.set_substack("SUBSTACK2", substack_false_id);

            let mut blocks = get_blocks();
            blocks.insert(if_else_id, Box::new(if_else));

            let mut thread = Thread::new(if_else_id, blocks);
            thread.step().await.unwrap(); // IfElse
            thread.step().await.unwrap(); // Substack
            thread.step().await.unwrap(); // IfElse
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Done)); // Next

            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(substack_false_id, BlockStubMsg::Executed)
            );
            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
            );
            assert!(receiver.try_recv().is_err());
        }
        {
            let mut if_else = IfElse::new(if_else_id);
            if_else.set_input("CONDITION", Box::new(ValueBool::new(true)));
            if_else.set_substack("next", next_id);
            if_else.set_substack("SUBSTACK", substack_true_id);
            if_else.set_substack("SUBSTACK2", substack_false_id);

            let mut blocks = get_blocks();
            blocks.insert(if_else_id, Box::new(if_else));

            let mut thread = Thread::new(if_else_id, blocks);
            thread.step().await.unwrap(); // IfElse
            thread.step().await.unwrap(); // Substack
            thread.step().await.unwrap(); // IfElse
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Done)); // Next

            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(substack_true_id, BlockStubMsg::Executed)
            );
            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
            );
            assert!(receiver.try_recv().is_err());
        }
    }

    #[tokio::test]
    async fn wait_until() {
        let runtime = Runtime::default();
        let mut receiver = runtime.global.broadcaster.subscribe();

        let mut gen = BlockIDGenerator::new();
        let wait_until_id = gen.get_id();
        let next_id = gen.get_id();

        let condition = Arc::new(RwLock::new(Value::Bool(false)));

        let mut wait_until = WaitUntil::new(wait_until_id);
        wait_until.set_input(
            "CONDITION",
            Box::new(BlockStub::new(
                gen.get_id(),
                runtime.clone(),
                Some(condition.clone()),
            )),
        );
        wait_until.set_substack("next", next_id);

        let blocks = block_map(vec![
            (wait_until_id, Box::new(wait_until)),
            (
                next_id,
                Box::new(BlockStub::new(next_id, runtime.clone(), None)),
            ),
        ]);

        let timeout_duration = Duration::from_millis(5);
        let mut thread = Thread::new(wait_until_id, blocks);
        assert!(timeout(timeout_duration, thread.step()).await.is_err());

        *condition.write().await = Value::Bool(true);

        timeout(timeout_duration, thread.step())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
        );
        assert!(receiver.try_recv().is_err());
    }

    #[tokio::test]
    async fn start_as_clone() {
        let mut gen = BlockIDGenerator::new();
        let next_id = gen.get_id();
        let start_as_clone_id = gen.get_id();

        {
            let runtime = Runtime::default();
            let mut receiver = runtime.global.broadcaster.subscribe();

            let mut start_as_clone = StartAsClone::new(start_as_clone_id, runtime.clone());
            start_as_clone.set_substack("next", next_id);

            let blocks = block_map(vec![
                (
                    next_id,
                    Box::new(BlockStub::new(next_id, runtime.clone(), None)),
                ),
                (start_as_clone_id, Box::new(start_as_clone)),
            ]);

            let mut thread = Thread::new(start_as_clone_id, blocks);
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

            assert!(receiver.try_recv().is_err());
        }
        {
            let runtime = Runtime::new(
                Arc::new(RwLock::new(SpriteRuntime::default().clone_sprite_runtime())),
                Arc::new(Global::default()),
                ThreadID::default(),
            );
            let mut receiver = runtime.global.broadcaster.subscribe();

            let mut start_as_clone = StartAsClone::new(start_as_clone_id, runtime.clone());
            start_as_clone.set_substack("next", next_id);

            let blocks = block_map(vec![
                (
                    next_id,
                    Box::new(BlockStub::new(next_id, runtime.clone(), None)),
                ),
                (start_as_clone_id, Box::new(start_as_clone)),
            ]);

            let mut thread = Thread::new(start_as_clone_id, blocks);
            thread.step().await.unwrap();
            assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

            assert_eq!(
                receiver.try_recv().unwrap(),
                BroadcastMsg::BlockStub(next_id, BlockStubMsg::Executed)
            );
            assert!(receiver.try_recv().is_err());
        }
    }

    #[tokio::test]
    async fn delete_this_clone() {
        let runtime = Runtime::default();
        let sprite_id = runtime.thread_id().sprite_id;
        let mut receiver = runtime.global.broadcaster.subscribe();

        let delete_this_clone_id = BlockIDGenerator::new().get_id();
        let delete_this_clone = DeleteThisClone::new(delete_this_clone_id, runtime);

        let blocks = block_map(vec![(delete_this_clone_id, Box::new(delete_this_clone))]);

        let mut thread = Thread::new(delete_this_clone_id, blocks);
        assert!(matches!(thread.step().await.unwrap(), StepStatus::Done));

        assert_eq!(
            receiver.try_recv().unwrap(),
            BroadcastMsg::DeleteClone(sprite_id)
        );
        assert!(receiver.try_recv().is_err());
    }
}
