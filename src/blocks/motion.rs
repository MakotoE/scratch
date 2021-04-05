use super::*;
use crate::broadcaster::BroadcastMsg;
use crate::coordinate::{canvas_const, SpriteCoordinate};
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
        "pointindirection" => Box::new(PointInDirection::new(id, runtime)),
        "goto" => Box::new(GoTo::new(id, runtime)),
        "goto_menu" => Box::new(GoToMenu::new(id, runtime)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
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

#[async_trait]
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
            vec![("STEPS", self.steps.as_ref())],
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

    async fn execute(&mut self) -> Result<Next> {
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

#[async_trait]
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
            vec![("X", self.x.as_ref()), ("Y", self.y.as_ref())],
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

    async fn execute(&mut self) -> Result<Next> {
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

#[async_trait]
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
            vec![("DX", self.dx.as_ref())],
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

    async fn execute(&mut self) -> Result<Next> {
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

#[async_trait]
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
            vec![("DY", self.dy.as_ref())],
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

    async fn execute(&mut self) -> Result<Next> {
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

#[async_trait]
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
            vec![("X", self.x.as_ref())],
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

    async fn execute(&mut self) -> Result<Next> {
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

#[async_trait]
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
            vec![("Y", self.y.as_ref())],
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

    async fn execute(&mut self) -> Result<Next> {
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

#[async_trait]
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

#[async_trait]
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

#[async_trait]
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

    async fn value(&self) -> Result<Value> {
        Ok(Value::Number(self.runtime.sprite.read().await.rotation()))
    }
}

#[derive(Debug)]
pub struct PointInDirection {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    direction: Box<dyn Block>,
}

impl PointInDirection {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            direction: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for PointInDirection {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "PointInDirection",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("DIRECTION", self.direction.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "DIRECTION" {
            self.direction = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let direction: f64 = self.direction.value().await?.try_into()?;
        self.runtime.sprite.write().await.set_rotation(direction);
        Next::continue_(self.next)
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

#[async_trait]
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

    async fn execute(&mut self) -> Result<Next> {
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
            GoToOption::Sprite(id) => {
                self.runtime
                    .global
                    .broadcaster
                    .send(BroadcastMsg::RequestSpriteRectangle(id))?;

                let mut channel = self.runtime.global.broadcaster.subscribe();
                loop {
                    if let BroadcastMsg::SpriteRectangle { sprite, rectangle } =
                        channel.recv().await?
                    {
                        if sprite == id {
                            break rectangle.center;
                        }
                    }
                }
            }
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

#[async_trait]
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

impl_try_from_value!(GoToOption);

#[cfg(test)]
mod test {
    use super::*;
    use crate::blocks::value::ValueNumber;
    use crate::coordinate::{CanvasCoordinate, Size, SpriteRectangle};
    use crate::file::BlockIDGenerator;

    #[tokio::test]
    async fn move_steps() {
        let runtime = Runtime::default();
        runtime
            .sprite
            .write()
            .await
            .set_center(SpriteCoordinate { x: 1.0, y: 1.0 });

        let mut gen = BlockIDGenerator::new();
        let mut move_steps = MoveSteps::new(gen.get_id(), runtime.clone());
        move_steps.set_input("STEPS", Box::new(ValueNumber::new(1.0)));
        move_steps.execute().await.unwrap();

        assert_eq!(
            runtime.sprite.read().await.center(),
            SpriteCoordinate { x: 2.0, y: 1.0 }
        );
    }

    #[tokio::test]
    async fn go_to_xy() {
        let mut gen = BlockIDGenerator::new();
        let runtime = Runtime::default();
        let mut go_to_xy = GoToXY::new(gen.get_id(), runtime.clone());
        go_to_xy.set_input("X", Box::new(ValueNumber::new(1.0)));
        go_to_xy.set_input("Y", Box::new(ValueNumber::new(2.0)));

        go_to_xy.execute().await.unwrap();
        assert_eq!(
            runtime.sprite.read().await.center(),
            SpriteCoordinate { x: 1.0, y: 2.0 }
        );
    }

    #[tokio::test]
    async fn change_x_by_and_change_y_by() {
        let runtime = Runtime::default();
        runtime
            .sprite
            .write()
            .await
            .set_center(SpriteCoordinate { x: 1.0, y: 2.0 });

        let mut gen = BlockIDGenerator::new();
        let mut change_x_by = ChangeXBy::new(gen.get_id(), runtime.clone());
        change_x_by.set_input("DX", Box::new(ValueNumber::new(3.0)));
        change_x_by.execute().await.unwrap();

        let mut change_y_by = ChangeYBy::new(gen.get_id(), runtime.clone());
        change_y_by.set_input("DY", Box::new(ValueNumber::new(4.0)));
        change_y_by.execute().await.unwrap();

        assert_eq!(
            runtime.sprite.read().await.center(),
            SpriteCoordinate { x: 4.0, y: 6.0 }
        );
    }

    #[tokio::test]
    async fn set_x_and_set_y() {
        let mut gen = BlockIDGenerator::new();
        let runtime = Runtime::default();

        let mut set_x = SetX::new(gen.get_id(), runtime.clone());
        set_x.set_input("X", Box::new(ValueNumber::new(1.0)));
        set_x.execute().await.unwrap();

        let mut set_y = SetY::new(gen.get_id(), runtime.clone());
        set_y.set_input("Y", Box::new(ValueNumber::new(2.0)));
        set_y.execute().await.unwrap();

        assert_eq!(
            runtime.sprite.read().await.center(),
            SpriteCoordinate { x: 1.0, y: 2.0 }
        );
    }

    #[tokio::test]
    async fn x_position_and_y_position() {
        let runtime = Runtime::default();
        runtime
            .sprite
            .write()
            .await
            .set_center(SpriteCoordinate { x: 1.0, y: 2.0 });

        let mut gen = BlockIDGenerator::new();
        let x_position = XPosition::new(gen.get_id(), runtime.clone());
        assert_eq!(x_position.value().await.unwrap(), Value::Number(1.0));

        let y_position = YPosition::new(gen.get_id(), runtime.clone());
        assert_eq!(y_position.value().await.unwrap(), Value::Number(2.0));
    }

    #[tokio::test]
    async fn point_in_direction_and_direction() {
        let runtime = Runtime::default();
        let mut gen = BlockIDGenerator::new();

        let rotation = 1.0;

        let mut point_in_direction = PointInDirection::new(gen.get_id(), runtime.clone());
        point_in_direction.set_input("DIRECTION", Box::new(ValueNumber::new(rotation)));
        point_in_direction.execute().await.unwrap();

        let direction = Direction::new(gen.get_id(), runtime.clone());
        assert_eq!(direction.value().await.unwrap(), Value::Number(rotation));
    }

    #[tokio::test]
    async fn go_to() {
        let runtime = Runtime::default();
        let mut gen = BlockIDGenerator::new();

        // Random position option
        {
            let mut menu = GoToMenu::new(gen.get_id(), runtime.clone());
            menu.set_field("TO", &[Some("_random_".to_string())])
                .unwrap();
            let mut go_to = GoTo::new(gen.get_id(), runtime.clone());
            go_to.set_input("TO", Box::new(menu));
            go_to.execute().await.unwrap();
        }

        // Mouse position option
        {
            let mut menu = GoToMenu::new(gen.get_id(), runtime.clone());
            menu.set_field("TO", &[Some("_mouse_".to_string())])
                .unwrap();
            let mut go_to = GoTo::new(gen.get_id(), runtime.clone());
            go_to.set_input("TO", Box::new(menu));

            let mut receiver = runtime.global.broadcaster.subscribe();
            let task = spawn(async move { go_to.execute().await.unwrap() });

            // Block requests mouse position
            assert_eq!(
                receiver.recv().await.unwrap(),
                BroadcastMsg::RequestMousePosition
            );

            // Send mouse position
            runtime
                .global
                .broadcaster
                .send(BroadcastMsg::MousePosition(CanvasCoordinate {
                    x: 1.0,
                    y: 1.0,
                }))
                .unwrap();

            task.await.unwrap();

            assert_eq!(
                runtime.sprite.read().await.center(),
                SpriteCoordinate {
                    x: -239.0,
                    y: 179.0
                }
            );
        }

        // Sprite position option
        {
            const SPRITE_NAME: &str = "sprite_id";
            let mut menu = GoToMenu::new(gen.get_id(), runtime.clone());
            menu.set_field("TO", &[Some(SPRITE_NAME.to_string())])
                .unwrap();
            let mut go_to = GoTo::new(gen.get_id(), runtime.clone());
            go_to.set_input("TO", Box::new(menu));

            let mut receiver = runtime.global.broadcaster.subscribe();
            let task = spawn(async move { go_to.execute().await.unwrap() });

            assert_eq!(
                receiver.recv().await.unwrap(),
                BroadcastMsg::RequestSpriteRectangle(SpriteID::from_sprite_name(SPRITE_NAME))
            );

            // Sprite rectangle of different sprite
            runtime
                .global
                .broadcaster
                .send(BroadcastMsg::SpriteRectangle {
                    sprite: SpriteID::from_sprite_name("wrong_id"),
                    rectangle: SpriteRectangle::default(),
                })
                .unwrap();

            let rectangle = SpriteRectangle {
                center: SpriteCoordinate { x: 1.0, y: 2.0 },
                size: Size::default(),
            };
            runtime
                .global
                .broadcaster
                .send(BroadcastMsg::SpriteRectangle {
                    sprite: SpriteID::from_sprite_name(SPRITE_NAME),
                    rectangle,
                })
                .unwrap();

            task.await.unwrap();

            assert_eq!(runtime.sprite.read().await.center(), rectangle.center);
        }
    }
}
