use super::*;
use crate::broadcaster::BroadcastMsg;
use crate::coordinate::{canvas_const, CanvasCoordinate};
use crate::sprite::SpriteID;
use graphics::types::Rectangle;
use graphics::Context;
use graphics_buffer::{buffer_glyphs_from_path, BufferGlyphs, RenderBuffer};
use image::{Pixel, Rgba, RgbaImage};
use input::Key;
use itertools::{any, zip_eq};
use palette::Srgb;
use std::fmt::{Display, Formatter};
use std::ops::DerefMut;
use std::str::FromStr;

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "keypressed" => Box::new(KeyPressed::new(id, runtime)),
        "keyoptions" => Box::new(KeyOptions::new(id, runtime)),
        "coloristouchingcolor" => Box::new(ColorIsTouchingColor::new(id, runtime)),
        "touchingcolor" => Box::new(TouchingColor::new(id, runtime)),
        "touchingobject" => Box::new(TouchingObject::new(id, runtime)),
        "touchingobjectmenu" => Box::new(TouchingObjectMenu::new(id)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
    })
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum KeyOption {
    Any,
    Key(Key),
}

impl FromStr for KeyOption {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "any" => KeyOption::Any,
            _ => KeyOption::Key(match s {
                "space" => Key::Space,
                "left arrow" => Key::Left,
                "right arrow" => Key::Right,
                "up arrow" => Key::Up,
                "down arrow" => Key::Down,
                _ => unimplemented!(),
            }),
        })
    }
}

impl Display for KeyOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyOption::Any => write!(f, "any"),
            KeyOption::Key(k) => match serde_json::to_string(k) {
                Ok(s) => write!(f, "{}", &s),
                Err(e) => {
                    log::error!("{}", e);
                    Err(std::fmt::Error {})
                }
            },
        }
    }
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

#[async_trait]
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
        let keys = self.runtime.global.inputs.keys().await;

        Ok(match key_option {
            KeyOption::Any => !keys.is_empty(),
            KeyOption::Key(key) => keys.contains(&key),
        }
        .into())
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
            key: KeyOption::Key(Key::Space),
        }
    }
}

#[async_trait]
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
            vec![("KEY_OPTION", format!("{:?}", self.key))],
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

lazy_static::lazy_static! {
    static ref BUFFER_GLYPHS: RwLock<BufferGlyphs<'static>>
        = RwLock::new(buffer_glyphs_from_path("assets/Roboto-Regular.ttf").unwrap());
}

#[derive(Debug)]
pub struct ColorIsTouchingColor {
    id: BlockID,
    runtime: Runtime,
    sprite_color: Box<dyn Block>,
    canvas_color: Box<dyn Block>,
}

impl ColorIsTouchingColor {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            sprite_color: Box::new(EmptyInput {}),
            canvas_color: Box::new(EmptyInput {}),
        }
    }

    fn sprite_color_touching_canvas_color(
        sprite_image: &RgbaImage,
        sprite_color: &Rgba<u8>,
        canvas_image: &RgbaImage,
        canvas_color: &Rgba<u8>,
    ) -> bool {
        any(
            zip_eq(canvas_image.pixels(), sprite_image.pixels()),
            |(canvas_pixel, sprite_pixel)| {
                let mut canvas_pixel_blended = Rgba::from_channels(255, 255, 255, 255);
                canvas_pixel_blended.blend(&canvas_pixel);
                colors_approximately_equal(sprite_pixel, sprite_color)
                    && colors_approximately_equal(&canvas_pixel_blended, canvas_color)
            },
        )
    }
}

#[async_trait]
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
        let sprite_color = srgb_to_rgba(self.sprite_color.value().await?.try_into()?);
        let canvas_color = srgb_to_rgba(self.canvas_color.value().await?.try_into()?);

        let sprite_image = {
            let mut render_buffer =
                RenderBuffer::new(canvas_const::X_MAX as u32, canvas_const::Y_MAX as u32);
            let mut cache = BUFFER_GLYPHS.write().await;
            self.runtime.sprite.write().await.draw(
                &Context::new(),
                &mut render_buffer,
                cache.deref_mut(),
            )?;
            render_buffer
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
                    &canvas_image,
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
}

impl TouchingColor {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            color: Box::new(EmptyInput {}),
        }
    }

    fn touching_color(
        canvas_image: &RgbaImage,
        sprite_image: &RgbaImage,
        color: &Rgba<u8>,
    ) -> bool {
        any(
            zip_eq(canvas_image.pixels(), sprite_image.pixels()),
            |(canvas_pixel, sprite_pixel)| {
                let mut canvas_pixel_blended = *canvas_pixel;
                canvas_pixel_blended.blend(&Rgba::from_channels(255, 255, 255, 255));

                sprite_pixel.channels4().3 > 0
                    // Check if canvas color is approximately equal
                    && colors_approximately_equal(&canvas_pixel_blended, color)
            },
        )
    }
}

#[async_trait]
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
            let mut render_buffer =
                RenderBuffer::new(canvas_const::X_MAX as u32, canvas_const::Y_MAX as u32);
            let mut cache = BUFFER_GLYPHS.write().await;
            self.runtime.sprite.write().await.draw(
                &Context::new(),
                &mut render_buffer,
                cache.deref_mut(),
            )?;
            render_buffer
        };

        let match_color = srgb_to_rgba(self.color.value().await?.try_into()?);

        let sprite_id = self.runtime.thread_id().sprite_id;
        self.runtime
            .global
            .broadcaster
            .send(BroadcastMsg::RequestCanvasImage(sprite_id))?;
        let mut channel = self.runtime.global.broadcaster.subscribe();
        loop {
            if let BroadcastMsg::CanvasImage(canvas_image) = channel.recv().await? {
                let result =
                    TouchingColor::touching_color(&canvas_image, &sprite_image, &match_color);
                return Ok(result.into());
            }
        }
    }
}

fn srgb_to_rgba(color: Srgb<u8>) -> Rgba<u8> {
    Rgba::from_channels(color.red, color.green, color.blue, 255)
}

fn colors_approximately_equal(a: &Rgba<u8>, b: &Rgba<u8>) -> bool {
    let a_channels = a.channels4();
    let b_channels = b.channels4();

    (if a_channels.0 > b_channels.0 {
        (a_channels.0 - b_channels.0) < 2
    } else {
        (b_channels.0 - a_channels.0) < 2
    }) && if a_channels.1 > b_channels.1 {
        (a_channels.1 - b_channels.1) < 2
    } else {
        (b_channels.1 - a_channels.1) < 2
    } && if a_channels.2 > b_channels.2 {
        (a_channels.2 - b_channels.2) < 2
    } else {
        (b_channels.2 - a_channels.2) < 2
    } && if a_channels.3 > b_channels.3 {
        (a_channels.3 - b_channels.3) < 2
    } else {
        (b_channels.3 - a_channels.3) < 2
    }
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

    fn sprite_on_edge(rectangle: &Rectangle) -> bool {
        rectangle[0] < 0.0
            || rectangle[1] < 0.0
            || rectangle[0] + rectangle[2] > canvas_const::X_MAX
            || rectangle[1] + rectangle[3] > canvas_const::Y_MAX
    }

    fn rectangle_contains(rectangle: &Rectangle, coordinate: &CanvasCoordinate) -> bool {
        coordinate.x >= rectangle[0]
            && coordinate.y >= rectangle[1]
            && coordinate.x <= rectangle[0] + rectangle[2]
            && coordinate.y <= rectangle[1] + rectangle[3]
    }
}

#[async_trait]
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
            vec![("TOUCHINGOBJECTMENU", self.menu.as_ref())],
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
                sleep(Duration::from_secs(0)).await; // Prevents unresponsiveness
                let canvas_rectangle: Rectangle = sprite_rectangle.into();
                let position = self.runtime.global.inputs.mouse_position().await;
                TouchingObject::rectangle_contains(&canvas_rectangle, &position)
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
        Ok(result.into())
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

#[async_trait]
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
            vec![("TOUCHINGOBJECTMENU", self.option.to_string())],
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
