use super::*;
use crate::broadcaster;
use crate::broadcaster::BroadcastMsg;
use crate::vm::ThreadID;
use futures::StreamExt;
use gloo_timers::future::{IntervalStream, TimeoutFuture};

use std::str::FromStr;
use strum::EnumString;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "if" => Box::new(If::new(id)),
        "forever" => Box::new(Forever::new(id)),
        "repeat" => Box::new(Repeat::new(id)),
        "wait" => Box::new(Wait::new(id, runtime)),
        "repeat_until" => Box::new(RepeatUntil::new(id)),
        "if_else" => Box::new(IfElse::new(id)),
        "wait_until" => Box::new(WaitUntil::new(id)),
        "start_as_clone" => Box::new(StartAsClone::new(id, runtime)),
        "delete_this_clone" => Box::new(DeleteThisClone::new(id, runtime)),
        "stop" => Box::new(Stop::new(id, runtime)),
        "create_clone_of" => Box::new(CreateCloneOf::new(id, runtime)),
        "create_clone_of_menu" => Box::new(CreateCloneOfMenu::new(id)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
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

#[async_trait(?Send)]
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
            vec![("condition", &self.condition)],
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

    async fn execute(&mut self) -> Next {
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
    runtime: Runtime,
}

impl Wait {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            next: None,
            duration: Box::new(EmptyInput {}),
            runtime,
        }
    }
}

#[async_trait(?Send)]
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
            vec![("duration", &self.duration)],
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

    async fn execute(&mut self) -> Next {
        let duration: f64 = self.duration.value().await?.try_into()?;
        TimeoutFuture::new((MILLIS_PER_SECOND * duration).round() as u32).await;
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

#[async_trait(?Send)]
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

    async fn execute(&mut self) -> Next {
        match &self.substack {
            Some(b) => Next::Loop(*b),
            None => Next::None,
        }
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

#[async_trait(?Send)]
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
            vec![("times", &self.times)],
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

    async fn execute(&mut self) -> Next {
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

#[async_trait(?Send)]
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
            vec![("condition", &self.condition)],
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

    async fn execute(&mut self) -> Next {
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

#[async_trait(?Send)]
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
            vec![("condition", &self.condition)],
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

    async fn execute(&mut self) -> Next {
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

#[async_trait(?Send)]
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
            vec![("condition", &self.condition)],
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

    async fn execute(&mut self) -> Next {
        let mut interval = IntervalStream::new(100);
        while interval.next().await.is_some() {
            if self.condition.value().await?.try_into()? {
                return Next::continue_(self.next);
            }
        }
        unreachable!()
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

#[async_trait(?Send)]
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

    async fn execute(&mut self) -> Next {
        if self.runtime.sprite.read().await.is_a_clone() {
            Next::continue_(self.next)
        } else {
            Next::None
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

#[async_trait(?Send)]
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

    async fn execute(&mut self) -> Next {
        let sprite_id = self.runtime.thread_id().sprite_id;
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::DeleteClone(sprite_id))?;
        Next::None
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

#[async_trait(?Send)]
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

    async fn execute(&mut self) -> Next {
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

try_from_value!(StopOption);

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

#[async_trait(?Send)]
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
            vec![("CLONE_OPTION", &self.clone_option)],
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

    async fn execute(&mut self) -> Next {
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

#[async_trait(?Send)]
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
