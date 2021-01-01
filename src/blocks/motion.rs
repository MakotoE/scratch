use super::*;
use crate::broadcaster::BroadcastMsg;
use crate::coordinate::{canvas_const, CanvasRectangle, SpriteCoordinate};
use crate::sprite::SpriteID;
use rand::distributions::{DistIter, Uniform};
use rand::prelude::*;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "movesteps" => Box::new(MoveSteps::new(id, runtime)),
        "gotoxy" => Box::new(GoToXY::new(id, runtime)),
        "changexby" => Box::new(ChangeXBy::new(id, runtime)),
        "changeyby" => Box::new(ChangeYBy::new(id, runtime)),
        "setx" => Box::new(SetX::new(id, runtime)),
        "sety" => Box::new(SetY::new(id, runtime)),
        "xposition" => Box::new(XPosition::new(id, runtime)),
        "yposition" => Box::new(YPosition::new(id, runtime)),
        "direction" => Box::new(Direction::new(id, runtime)),
        "pointindirection" => Box::new(PointingDirection::new(id, runtime)),
        "goto" => Box::new(GoTo::new(id, runtime)),
        "goto_menu" => Box::new(GoToMenu::new(id, runtime)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct MoveSteps {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    steps: Box<dyn Block>,
}

impl MoveSteps {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            steps: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for MoveSteps {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "MoveSteps",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("steps", self.steps.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "STEPS" {
            self.steps = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let steps: f64 = self.steps.value().await?.try_into()?;
        let mut runtime = self.runtime.sprite.write().await;
        let position = runtime.center().add(&SpriteCoordinate { x: steps, y: 0.0 });
        runtime.set_center(position);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct GoToXY {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    x: Box<dyn Block>,
    y: Box<dyn Block>,
}

impl GoToXY {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            x: Box::new(EmptyInput {}),
            y: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for GoToXY {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoToXY",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("x", self.x.as_ref()), ("y", self.y.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "X" => self.x = block,
            "Y" => self.y = block,
            _ => {}
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let x: f64 = self.x.value().await?.try_into()?;
        let y: f64 = self.y.value().await?.try_into()?;

        self.runtime
            .sprite
            .write()
            .await
            .set_center(SpriteCoordinate { x, y });
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct ChangeXBy {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    dx: Box<dyn Block>,
}

impl ChangeXBy {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            dx: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeXBy {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeXBy",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("dx", self.dx.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "DX" {
            self.dx = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let x: f64 = self.dx.value().await?.try_into()?;
        let mut runtime = self.runtime.sprite.write().await;
        let position = runtime.center().add(&SpriteCoordinate { x, y: 0.0 });
        runtime.set_center(position);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct ChangeYBy {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    dy: Box<dyn Block>,
}

impl ChangeYBy {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            dy: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for ChangeYBy {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ChangeYBy",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("dy", self.dy.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "DY" {
            self.dy = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let y: f64 = self.dy.value().await?.try_into()?;

        let mut runtime = self.runtime.sprite.write().await;
        let position = runtime.center().add(&SpriteCoordinate { x: 0.0, y });
        runtime.set_center(position);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SetX {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    x: Box<dyn Block>,
}

impl SetX {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            x: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for SetX {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetX",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("x", self.x.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "X" {
            self.x = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let x: f64 = self.x.value().await?.try_into()?;
        let mut runtime = self.runtime.sprite.write().await;
        let mut position = runtime.center();
        position.x = x;
        runtime.set_center(position);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SetY {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    y: Box<dyn Block>,
}

impl SetY {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            y: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for SetY {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetY",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("y", self.y.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "Y" {
            self.y = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let y: f64 = self.y.value().await?.try_into()?;
        let mut runtime = self.runtime.sprite.write().await;
        let mut position = runtime.center();
        position.y = y;
        runtime.set_center(position);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct XPosition {
    id: BlockID,
    runtime: Runtime,
}

impl XPosition {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for XPosition {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "XPosition",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<Value> {
        let runtime = self.runtime.sprite.read().await;
        Ok(runtime.rectangle().center.x.into())
    }
}

#[derive(Debug)]
pub struct YPosition {
    id: BlockID,
    runtime: Runtime,
}

impl YPosition {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for YPosition {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "YPosition",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<Value> {
        let runtime = self.runtime.sprite.read().await;
        Ok(runtime.rectangle().center.y.into())
    }
}

#[derive(Debug)]
pub struct Direction {
    id: BlockID,
    runtime: Runtime,
}

impl Direction {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for Direction {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Direction",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<Value> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct PointingDirection {
    id: BlockID,
    runtime: Runtime,
}

impl PointingDirection {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self { id, runtime }
    }
}

#[async_trait(?Send)]
impl Block for PointingDirection {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "PointingDirection",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(self.block_info(), vec![], vec![], vec![])
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn value(&self) -> Result<Value> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct GoTo {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    option: Box<dyn Block>,
    rng: RandomCoordinateGenerator,
}

impl GoTo {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            option: Box::new(EmptyInput {}),
            rng: RandomCoordinateGenerator::new(),
        }
    }
}

#[async_trait(?Send)]
impl Block for GoTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoTo",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("TO", self.option.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "TO" {
            self.option = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Next {
        let option: GoToOption = self.option.value().await?.try_into()?;
        let new_coordinate = match option {
            GoToOption::RandomPosition => self.rng.next().unwrap(),
            GoToOption::MousePointer => {
                self.runtime
                    .global
                    .broadcaster
                    .send(BroadcastMsg::RequestMousePosition)?;
                let mut channel = self.runtime.global.broadcaster.subscribe();
                loop {
                    if let BroadcastMsg::MousePosition(position) = channel.recv().await? {
                        break position.into();
                    }
                }
            }
            GoToOption::Sprite(_) => todo!(),
        };
        self.runtime.sprite.write().await.set_center(new_coordinate);

        Next::continue_(self.next)
    }
}

#[derive(Debug)]
struct RandomCoordinateGenerator {
    x_iter: DistIter<Uniform<f64>, SmallRng, f64>,
    y_iter: DistIter<Uniform<f64>, SmallRng, f64>,
}

impl RandomCoordinateGenerator {
    fn new() -> Self {
        let mut seeder = thread_rng();
        Self {
            x_iter: Uniform::new_inclusive(-canvas_const::X_MAX / 2.0, canvas_const::X_MAX / 2.0)
                .sample_iter(SmallRng::seed_from_u64(seeder.next_u64())),
            y_iter: Uniform::new_inclusive(-canvas_const::Y_MAX / 2.0, canvas_const::Y_MAX / 2.0)
                .sample_iter(SmallRng::seed_from_u64(seeder.next_u64())),
        }
    }
}

impl Iterator for RandomCoordinateGenerator {
    type Item = SpriteCoordinate;

    fn next(&mut self) -> Option<SpriteCoordinate> {
        Some(SpriteCoordinate {
            x: self.x_iter.next().unwrap(),
            y: self.y_iter.next().unwrap(),
        })
    }
}

#[derive(Debug)]
pub struct GoToMenu {
    id: BlockID,
    runtime: Runtime,
    option: GoToOption,
}

impl GoToMenu {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            option: GoToOption::RandomPosition,
        }
    }
}

#[async_trait(?Send)]
impl Block for GoToMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "GoToMenu",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("TO", format!("{}", self.option))],
            vec![],
            vec![],
        )
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "TO" {
            self.option = GoToOption::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn value(&self) -> Result<Value> {
        Ok(Value::GoToOption(self.option))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GoToOption {
    RandomPosition,
    MousePointer,
    Sprite(SpriteID),
}

impl FromStr for GoToOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "_random_" => Self::RandomPosition,
            "_mouse_" => Self::MousePointer,
            _ => Self::Sprite(SpriteID::from_sprite_name(s)),
        })
    }
}

impl Display for GoToOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::RandomPosition => "_random_",
            Self::MousePointer => "_mouse_",
            Self::Sprite(id) => return Display::fmt(id, f),
        })
    }
}

try_from_value!(GoToOption);
