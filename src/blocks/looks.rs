use super::*;
use crate::coordinate::Scale;
use crate::runtime::{BroadcastMsg, LayerChange};
use crate::sprite_runtime::{HideStatus, Text};
use gloo_timers::future::TimeoutFuture;
use std::str::FromStr;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "say" => Box::new(Say::new(id, runtime)),
        "sayforsecs" => Box::new(SayForSecs::new(id, runtime)),
        "gotofrontback" => Box::new(GoToFrontBack::new(id, runtime)),
        "hide" => Box::new(Hide::new(id, runtime)),
        "show" => Box::new(Show::new(id, runtime)),
        "seteffectto" => Box::new(SetEffectTo::new(id, runtime)),
        "nextcostume" => Box::new(NextCostume::new(id, runtime)),
        "changeeffectby" => Box::new(ChangeEffectBy::new(id, runtime)),
        "setsizeto" => Box::new(SetSizeTo::new(id, runtime)),
        "switchcostumeto" => Box::new(SwitchCostumeTo::new(id, runtime)),
        "costume" => Box::new(Costume::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Say {
    id: BlockID,
    runtime: Runtime,
    message: Box<dyn Block>,
    next: Option<BlockID>,
}

impl Say {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            message: Box::new(EmptyInput {}),
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Say {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Say",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("message", &self.message)],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "MESSAGE" {
            self.message = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let message = value_to_string(self.message.value().await?);
        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: Some(message),
        });
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SayForSecs {
    id: BlockID,
    runtime: Runtime,
    message: Box<dyn Block>,
    secs: Box<dyn Block>,
    next: Option<BlockID>,
}

impl SayForSecs {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            message: Box::new(EmptyInput {}),
            secs: Box::new(EmptyInput {}),
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for SayForSecs {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SayForSecs",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("message", &self.message), ("secs", &self.secs)],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "MESSAGE" => self.message = block,
            "SECS" => self.secs = block,
            _ => {}
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let message = value_to_string(self.message.value().await?);
        let seconds = value_to_float(&self.secs.value().await?)?;

        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: Some(message),
        });
        TimeoutFuture::new((MILLIS_PER_SECOND * seconds).round() as u32).await;
        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: None,
        });
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct GoToFrontBack {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>, // TODO store ID instead of block reference
    front_or_back: FrontBack,
}

impl GoToFrontBack {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            front_or_back: FrontBack::Front,
        }
    }
}

#[async_trait(?Send)]
impl Block for GoToFrontBack {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoToFrontBack",
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

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "FRONT_BACK" {
            if let Some(s) = field.get(0).unwrap_or(&None) {
                self.front_or_back = FrontBack::from_str(s)?;
            }
        }
        Ok(())
    }

    async fn execute(&mut self) -> Next {
        let layer_change = match self.front_or_back {
            FrontBack::Front => LayerChange::Front,
            FrontBack::Back => LayerChange::Back,
        };
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::ChangeLayer((
                self.runtime.thread_id().sprite_id,
                layer_change,
            )))?;
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
enum FrontBack {
    Front,
    Back,
}

impl FromStr for FrontBack {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "front" => Self::Front,
            "back" => Self::Back,
            _ => return Err(wrap_err!(format!("s is invalid: {}", s))),
        })
    }
}

#[derive(Debug)]
pub struct Hide {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl Hide {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Hide {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Hide",
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
        self.runtime.sprite.write().await.set_hide(HideStatus::Hide);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct Show {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl Show {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait(?Send)]
impl Block for Show {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Show",
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
        self.runtime.sprite.write().await.set_hide(HideStatus::Show);
        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SetEffectTo {
    id: BlockID,
    next: Option<BlockID>,
}

impl SetEffectTo {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for SetEffectTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetEffectTo",
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
}

#[derive(Debug)]
pub struct NextCostume {
    id: BlockID,
    next: Option<BlockID>,
}

impl NextCostume {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for NextCostume {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "NextCostume",
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
}

#[derive(Debug)]
pub struct ChangeEffectBy {
    id: BlockID,
    next: Option<BlockID>,
}

impl ChangeEffectBy {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for ChangeEffectBy {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeEffectBy",
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
}

#[derive(Debug)]
pub struct SetSizeTo {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    size: Box<dyn Block>,
}

impl SetSizeTo {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            size: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for SetSizeTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetSizeTo",
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

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "SIZE" {
            self.size = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let scale = value_to_float(&self.size.value().await?)? / 100.0;

        self.runtime
            .sprite
            .write()
            .await
            .set_scale(Scale { x: scale, y: scale });

        Next::continue_(self.next.clone())
    }
}

#[derive(Debug)]
pub struct SwitchCostumeTo {
    id: BlockID,
    next: Option<BlockID>,
}

impl SwitchCostumeTo {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for SwitchCostumeTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SwitchCostumeTo",
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
}

#[derive(Debug)]
pub struct Costume {
    id: BlockID,
    next: Option<BlockID>,
}

impl Costume {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self { id, next: None }
    }
}

#[async_trait(?Send)]
impl Block for Costume {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Costume",
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
}
