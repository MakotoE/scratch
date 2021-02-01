use super::*;
use crate::broadcaster::{BroadcastMsg, LayerChange};
use crate::coordinate::Scale;
use crate::sprite_runtime::{HideStatus, Text};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn get_block(
    name: &str,
    id: BlockID,
    runtime: Runtime,
) -> Result<Box<dyn Block + Send + Sync>> {
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
        "switchbackdropto" => Box::new(SwitchBackdropTo::new(id, runtime)),
        "backdrops" => Box::new(Backdrops::new(id, runtime)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct Say {
    id: BlockID,
    runtime: Runtime,
    message: Box<dyn Block + Send + Sync>,
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

#[async_trait]
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
            vec![("message", self.message.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        if key == "MESSAGE" {
            self.message = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let message = self.message.value().await?.to_string();
        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: Some(message),
        });
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SayForSecs {
    id: BlockID,
    runtime: Runtime,
    message: Box<dyn Block + Send + Sync>,
    secs: Box<dyn Block + Send + Sync>,
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

#[async_trait]
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
            vec![
                ("message", self.message.as_ref()),
                ("secs", self.secs.as_ref()),
            ],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
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

    async fn execute(&mut self) -> Result<Next> {
        let message = self.message.value().await?.to_string();
        let seconds: f64 = self.secs.value().await?.try_into()?;

        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: Some(message),
        });
        sleep(Duration::from_secs_f64(seconds)).await;
        self.runtime.sprite.write().await.say(Text {
            id: self.id,
            text: None,
        });
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct GoToFrontBack {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
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

#[async_trait]
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
            self.front_or_back = FrontBack::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        let layer_change = match self.front_or_back {
            FrontBack::Front => LayerChange::Front,
            FrontBack::Back => LayerChange::Back,
        };
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::ChangeLayer {
                sprite: self.runtime.thread_id().sprite_id,
                action: layer_change,
            })?;
        Next::continue_(self.next)
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
            _ => return Err(Error::msg(format!("s is invalid: {}", s))),
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

#[async_trait]
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

    async fn execute(&mut self) -> Result<Next> {
        self.runtime.sprite.write().await.set_hide(HideStatus::Hide);
        Next::continue_(self.next)
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

#[async_trait]
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

    async fn execute(&mut self) -> Result<Next> {
        self.runtime.sprite.write().await.set_hide(HideStatus::Show);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SetEffectTo {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    effect: Effect,
    value: Box<dyn Block + Send + Sync>,
}

impl SetEffectTo {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            effect: Effect::Color,
            value: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
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
            vec![("effect", self.effect.to_string())],
            vec![("value", self.value.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
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
        if key == "EFFECT" {
            self.effect = Effect::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        let value: f64 = self.value.value().await?.try_into()?;
        let mut runtime = self.runtime.sprite.write().await;
        match self.effect {
            Effect::Ghost => runtime.set_transparency((100.0 - value) / 100.0),
            _ => unimplemented!(),
        }

        Next::continue_(self.next)
    }
}

#[derive(Debug, Copy, Clone)]
enum Effect {
    Color,
    Fisheye,
    Whirl,
    Pixelate,
    Mosaic,
    Brightness,
    Ghost,
}

impl FromStr for Effect {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "color" => Self::Color,
            "fisheye" => Self::Fisheye,
            "whirl" => Self::Whirl,
            "pixelate" => Self::Pixelate,
            "mosaic" => Self::Mosaic,
            "brightness" => Self::Brightness,
            "ghost" => Self::Ghost,
            _ => return Err(Error::msg(format!("s is invalid: {}", s))),
        })
    }
}

impl Display for Effect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Effect::Color => "color",
            Effect::Fisheye => "fisheye",
            Effect::Whirl => "whirl",
            Effect::Pixelate => "pixelate",
            Effect::Mosaic => "mosaic",
            Effect::Brightness => "brightness",
            Effect::Ghost => "ghost",
        };
        f.write_str(s)
    }
}

#[derive(Debug)]
pub struct NextCostume {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl NextCostume {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait]
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

    async fn execute(&mut self) -> Result<Next> {
        self.runtime.sprite.write().await.costumes().next_costume();
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct ChangeEffectBy {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    effect: Effect,
    change: Box<dyn Block + Send + Sync>,
}

impl ChangeEffectBy {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            effect: Effect::Color,
            change: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
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
            vec![("effect", self.effect.to_string())],
            vec![("change", self.change.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        if key == "CHANGE" {
            self.change = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "EFFECT" {
            self.effect = Effect::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn execute(&mut self) -> Result<Next> {
        let value: f64 = self.change.value().await?.try_into()?;
        let mut runtime = self.runtime.sprite.write().await;
        match self.effect {
            Effect::Ghost => {
                let current_transparency = runtime.transparency();
                runtime.set_transparency(current_transparency - value / 100.0);
            }
            _ => unimplemented!(),
        }

        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SetSizeTo {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    size: Box<dyn Block + Send + Sync>,
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

#[async_trait]
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

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        if key == "SIZE" {
            self.size = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let size: f64 = self.size.value().await?.try_into()?;
        let scale = size / 100.0;

        self.runtime
            .sprite
            .write()
            .await
            .set_scale(Scale { x: scale, y: scale });

        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SwitchCostumeTo {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    costume: Box<dyn Block + Send + Sync>,
}

impl SwitchCostumeTo {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            costume: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
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
            vec![("costume", self.costume.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        if key == "COSTUME" {
            self.costume = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let costume_name = self.costume.value().await?.to_string();
        self.runtime
            .sprite
            .write()
            .await
            .costumes()
            .set_current_costume(costume_name)?;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct Costume {
    id: BlockID,
    next: Option<BlockID>,
    name: String,
}

impl Costume {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self {
            id,
            next: None,
            name: String::new(),
        }
    }
}

#[async_trait]
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
            vec![("name", self.name.clone())],
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
        if key == "COSTUME" {
            self.name = get_field_value(field, 0)?.to_string();
        }
        Ok(())
    }

    async fn value(&self) -> Result<Value> {
        Ok(self.name.clone().into())
    }
}

#[derive(Debug)]
pub struct SwitchBackdropTo {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    backdrop: Box<dyn Block + Send + Sync>,
}

impl SwitchBackdropTo {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            backdrop: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for SwitchBackdropTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SwitchBackdropTo",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("backdrop", self.backdrop.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block + Send + Sync>) {
        if key == "BACKDROP" {
            self.backdrop = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let backdrop = self.backdrop.value().await?.to_string();
        self.runtime
            .sprite
            .write()
            .await
            .costumes()
            .set_current_costume(backdrop)?;
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct Backdrops {
    id: BlockID,
    backdrop: String,
}

impl Backdrops {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self {
            id,
            backdrop: String::new(),
        }
    }
}

#[async_trait]
impl Block for Backdrops {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Backdrops",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("backdrop", self.backdrop.clone())],
            vec![],
            vec![],
        )
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "BACKDROP" {
            self.backdrop = get_field_value(field, 0)?.to_string();
        }
        Ok(())
    }

    async fn value(&self) -> Result<Value> {
        Ok(self.backdrop.clone().into())
    }
}
