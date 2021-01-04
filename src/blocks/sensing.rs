use super::*;
use crate::broadcaster::BroadcastMsg;
use crate::canvas::CanvasContext;
use crate::coordinate::{canvas_const, CanvasRectangle};
use crate::event_sender::{KeyOption, KeyboardKey};
use crate::sprite::SpriteID;
use crate::vm::new_hidden_canvas;
use gloo_timers::future::TimeoutFuture;
use ndarray::{Array2, Zip};

use palette::{Alpha, Hsv, Srgb, Srgba};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "keypressed" => Box::new(KeyPressed::new(id, runtime)),
        "keyoptions" => Box::new(KeyOptions::new(id, runtime)),
        "coloristouchingcolor" => Box::new(ColorIsTouchingColor::new(id, runtime)),
        "touchingcolor" => Box::new(TouchingColor::new(id, runtime)),
        "touchingobject" => Box::new(TouchingObject::new(id, runtime)),
        "touchingobjectmenu" => Box::new(TouchingObjectMenu::new(id)),
        _ => return Err(wrap_err!(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct KeyPressed {
    id: BlockID,
    runtime: Runtime,
    key_option: Box<dyn Block>,
}

impl KeyPressed {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            key_option: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait(?Send)]
impl Block for KeyPressed {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "KeyPressed",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("KEY_OPTION", self.key_option.as_ref())],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "KEY_OPTION" {
            self.key_option = block;
        }
    }

    async fn value(&self) -> Result<Value> {
        let key_option: KeyOption = self.key_option.value().await?.try_into()?;
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::RequestPressedKeys)?;
        let mut receiver = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::PressedKeys(keys) = receiver.recv().await? {
                return Ok(match key_option {
                    KeyOption::Any => true,
                    KeyOption::Key(key) => keys.contains(&key),
                }
                .into());
            }
        }
    }
}

#[derive(Debug)]
pub struct KeyOptions {
    id: BlockID,
    key: KeyOption,
}

impl KeyOptions {
    pub fn new(id: BlockID, _runtime: Runtime) -> Self {
        Self {
            id,
            key: KeyOption::Key(KeyboardKey::Space),
        }
    }
}

#[async_trait(?Send)]
impl Block for KeyOptions {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "KeyOptions",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("KEY_OPTION", format!("{}", self.key))],
            vec![],
            vec![],
        )
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "KEY_OPTION" {
            self.key = KeyOption::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn value(&self) -> Result<Value> {
        Ok(Value::KeyOption(self.key))
    }
}

impl_try_from_value!(KeyOption);

#[derive(Debug)]
pub struct ColorIsTouchingColor {
    id: BlockID,
    runtime: Runtime,
    sprite_color: Box<dyn Block>,
    canvas_color: Box<dyn Block>,
    buffer_canvas: web_sys::CanvasRenderingContext2d,
}

impl ColorIsTouchingColor {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            sprite_color: Box::new(EmptyInput {}),
            canvas_color: Box::new(EmptyInput {}),
            buffer_canvas: new_hidden_canvas(),
        }
    }

    fn sprite_color_touching_canvas_color(
        sprite_image: &Array2<Srgba<u8>>,
        sprite_color: &Srgba<u8>,
        canvas_image: &Array2<Srgba<u8>>,
        canvas_color: &Srgba<u8>,
    ) -> bool {
        !Zip::from(canvas_image)
            .and(sprite_image)
            .all(|canvas_pixel, sprite_pixel| {
                let apparent_color = blend_with_white(canvas_pixel);
                !(sprite_pixel == sprite_color && &apparent_color == canvas_color)
            })
    }
}

#[async_trait(?Send)]
impl Block for ColorIsTouchingColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "ColorIsTouchingColor",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![
                ("COLOR", self.sprite_color.as_ref()),
                ("COLOR2", self.canvas_color.as_ref()),
            ],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        match key {
            "COLOR" => self.sprite_color = block,
            "COLOR2" => self.canvas_color = block,
            _ => {}
        }
    }

    async fn value(&self) -> Result<Value> {
        let sprite_color = hsv_to_srgb(self.sprite_color.value().await?.try_into()?);
        let canvas_color = hsv_to_srgb(self.canvas_color.value().await?.try_into()?);

        let sprite_image = {
            let canvas_context = CanvasContext::new(&self.buffer_canvas);
            self.runtime.sprite.write().await.redraw(&canvas_context)?;
            canvas_context.get_image_data()?
        };

        let sprite_id = self.runtime.thread_id().sprite_id;
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::RequestCanvasImage(sprite_id))?;
        let mut channel = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::CanvasImage(canvas_image) = channel.recv().await? {
                let result = ColorIsTouchingColor::sprite_color_touching_canvas_color(
                    &sprite_image,
                    &sprite_color,
                    &canvas_image.image,
                    &canvas_color,
                );
                return Ok(result.into());
            }
        }
    }
}

#[derive(Debug)]
pub struct TouchingColor {
    id: BlockID,
    runtime: Runtime,
    color: Box<dyn Block>,
    buffer_canvas: web_sys::CanvasRenderingContext2d,
}

impl TouchingColor {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            color: Box::new(EmptyInput {}),
            buffer_canvas: new_hidden_canvas(),
        }
    }

    fn touching_color(
        canvas_image: &Array2<Srgba<u8>>,
        sprite_image: &Array2<Srgba<u8>>,
        color: &Srgba<u8>,
    ) -> bool {
        !Zip::from(canvas_image)
            .and(sprite_image)
            .all(|canvas_pixel, sprite_pixel| {
                let apparent_color = blend_with_white(canvas_pixel);
                !(sprite_pixel.alpha > 0 && &apparent_color == color)
            })
    }
}

#[async_trait(?Send)]
impl Block for TouchingColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingColor",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("COLOR", self.color.as_ref())],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "COLOR" {
            self.color = block;
        }
    }

    async fn value(&self) -> Result<Value> {
        let sprite_image = {
            let canvas_context = CanvasContext::new(&self.buffer_canvas);
            self.runtime.sprite.write().await.redraw(&canvas_context)?; // TODO this sets need_redraw to false
            canvas_context.get_image_data()?
        };

        let match_color = hsv_to_srgb(self.color.value().await?.try_into()?);

        let sprite_id = self.runtime.thread_id().sprite_id;
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::RequestCanvasImage(sprite_id))?;
        let mut channel = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::CanvasImage(canvas_image) = channel.recv().await? {
                let result =
                    TouchingColor::touching_color(&canvas_image.image, &sprite_image, &match_color);
                return Ok(result.into());
            }
        }
    }
}

fn hsv_to_srgb(hsv: Hsv) -> Srgba<u8> {
    let rgb: Srgb = hsv.into();
    Alpha {
        color: Srgb::<u8>::new(
            (rgb.red * 255.0).round() as u8,
            (rgb.green * 255.0).round() as u8,
            (rgb.blue * 255.0).round() as u8,
        ),
        alpha: 255,
    }
}

fn blend_with_white(color: &Srgba<u8>) -> Srgba<u8> {
    Srgba::new(
        color.red * color.alpha + (1 - color.alpha) * 255,
        color.green * color.alpha + (1 - color.alpha) * 255,
        color.blue * color.alpha + (1 - color.alpha) * 255,
        255,
    )
}

#[derive(Debug)]
pub struct TouchingObject {
    id: BlockID,
    runtime: Runtime,
    menu: Box<dyn Block>,
}

impl TouchingObject {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            menu: Box::new(EmptyInput {}),
        }
    }

    fn sprite_on_edge(rectangle: &CanvasRectangle) -> bool {
        rectangle.top_left.x < 0.0
            || rectangle.top_left.y < 0.0
            || rectangle.top_left.x + rectangle.size.width > canvas_const::X_MAX
            || rectangle.top_left.y + rectangle.size.height > canvas_const::Y_MAX
    }
}

#[async_trait(?Send)]
impl Block for TouchingObject {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingObject",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("menu", self.menu.as_ref())],
            vec![],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "TOUCHINGOBJECTMENU" {
            self.menu = block;
        }
    }

    async fn value(&self) -> Result<Value> {
        let option: TouchingObjectOption = self.menu.value().await?.try_into()?;

        let sprite_rectangle = self.runtime.sprite.read().await.rectangle();

        let result = match option {
            TouchingObjectOption::MousePointer => {
                TimeoutFuture::new(0).await; // Prevents unresponsiveness
                self.runtime
                    .global
                    .broadcaster
                    .send(BroadcastMsg::RequestMousePosition)?;

                let canvas_rectangle: CanvasRectangle = sprite_rectangle.into();
                let mut channel = self.runtime.global.broadcaster.subscribe();
                loop {
                    if let BroadcastMsg::MousePosition(position) = channel.recv().await? {
                        break canvas_rectangle.contains(&position);
                    }
                }
            }
            TouchingObjectOption::Edge => TouchingObject::sprite_on_edge(&sprite_rectangle.into()),
            TouchingObjectOption::Sprite(id) => {
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
                            break sprite_rectangle.intersects(&rectangle);
                        }
                    }
                }
            }
        };
        return Ok(result.into());
    }
}

#[derive(Debug)]
pub struct TouchingObjectMenu {
    id: BlockID,
    option: TouchingObjectOption,
}

impl TouchingObjectMenu {
    pub fn new(id: BlockID) -> Self {
        Self {
            id,
            option: TouchingObjectOption::MousePointer,
        }
    }
}

#[async_trait(?Send)]
impl Block for TouchingObjectMenu {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "TouchingObjectMenu",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![("option", self.option.to_string())],
            vec![],
            vec![],
        )
    }

    fn set_field(&mut self, key: &str, field: &[Option<String>]) -> Result<()> {
        if key == "TOUCHINGOBJECTMENU" {
            self.option = TouchingObjectOption::from_str(get_field_value(field, 0)?)?;
        }
        Ok(())
    }

    async fn value(&self) -> Result<Value> {
        Ok(Value::TouchingObjectOption(self.option))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TouchingObjectOption {
    MousePointer,
    Edge,
    Sprite(SpriteID),
}

impl FromStr for TouchingObjectOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "_mouse_" => Self::MousePointer,
            "_edge_" => Self::Edge,
            _ => Self::Sprite(SpriteID::from_sprite_name(s)),
        })
    }
}

impl Display for TouchingObjectOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TouchingObjectOption::MousePointer => "_mouse_",
            TouchingObjectOption::Edge => "_edge_",
            TouchingObjectOption::Sprite(s) => return Display::fmt(s, f),
        })
    }
}

impl_try_from_value!(TouchingObjectOption);
