use super::*;
use crate::blocks::value::Value;
use crate::broadcaster::Broadcaster;
use crate::coordinate::{CanvasCoordinate, CanvasRectangle, Size, Transformation};
use crate::file::Monitor;
use crate::sprite_runtime::SpriteRuntime;
use crate::vm::ThreadID;
use graphics::character::CharacterCache;
use graphics::math::Matrix2d;
use graphics::types::FontSize;
use graphics::{rectangle, text, DrawState};
use graphics::{Context, Transformed};
use piston_window::{G2d, Glyphs};

#[derive(Debug, Clone)]
pub struct Runtime {
    pub sprite: Arc<RwLock<SpriteRuntime>>,
    pub global: Arc<Global>,
    thread_id: ThreadID,
}

impl Runtime {
    pub fn new(
        sprite: Arc<RwLock<SpriteRuntime>>,
        global: Arc<Global>,
        thread_id: ThreadID,
    ) -> Self {
        Self {
            sprite,
            global,
            thread_id,
        }
    }
    pub fn thread_id(&self) -> ThreadID {
        self.thread_id
    }
}

#[derive(Debug)]
pub struct Global {
    pub variables: Variables,
    pub broadcaster: Broadcaster,
}

impl Global {
    pub fn new(
        scratch_file_variables: &HashMap<String, file::Variable>,
        monitors: &[Monitor],
        broadcaster: Broadcaster,
    ) -> Self {
        Self {
            variables: Variables::new(scratch_file_variables, monitors),
            broadcaster,
        }
    }

    pub fn need_redraw(&self) -> bool {
        true
    }

    pub async fn redraw(
        &self,
        context: &mut Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        for variable in self.variables.variables.read().await.values() {
            if variable.monitored {
                context.transform = context
                    .transform
                    .trans_pos([variable.position.x, variable.position.y]);
                Global::draw_monitor(
                    context,
                    graphics,
                    character_cache,
                    &variable.name,
                    &variable.value.clone().to_string(),
                )?;
            }
        }
        Ok(())
    }

    fn draw_monitor(
        context: &Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
        variable_name: &str,
        value_str: &str,
    ) -> Result<()> {
        const FONT_SIZE: FontSize = 14;

        let name_width = character_cache.width(FONT_SIZE, variable_name)?;
        let value_width = character_cache.width(FONT_SIZE, value_str)?;

        let orange_rectangle_width = f64::max(39.0 - value_width, value_width + 4.0);

        rectangle::Rectangle {
            color: [0.9, 0.94, 1.0, 1.0],
            shape: rectangle::Shape::Round(3.5, 8),
            border: Some(rectangle::Border {
                color: [0.77, 0.8, 0.85, 1.0],
                radius: 1.0,
            }),
        }
        .draw(
            [0.0, 0.0, name_width + orange_rectangle_width + 24.0, 20.0],
            &context.draw_state,
            context.transform,
            graphics,
        );
        // Global::draw_rectangle(
        //     context,
        //     &CanvasRectangle {
        //         top_left: *position,
        //         size: Size {
        //             width: name_width + orange_rectangle_width + 24.0,
        //             height: 20.0,
        //         },
        //     },
        //     3.5,
        // )?;
        // context.set_fill_style("#e6f0ff");
        // context.fill();
        // context.set_stroke_style("#c4ccd9");
        // context.set_line_width(1.0);
        // context.stroke();

        text::Text {
            color: [0.34, 0.37, 0.46, 1.0],
            font_size: FONT_SIZE,
            round: false,
        }
        .draw(
            variable_name,
            character_cache,
            &context.draw_state,
            context.transform.trans_pos([7.0, 14.0]),
            graphics,
        )?;

        // context.set_fill_style("#575e75");
        // context.set_font(NAME_FONT);
        // context.fill_text(
        //     variable_name,
        //     &position.add(&CanvasCoordinate { x: 7.0, y: 14.0 }),
        // )?;

        let orange_transform = context.transform.trans_pos([name_width + 16.0, 3.0]);
        rectangle::Rectangle {
            color: [1.0, 0.55, 0.1, 1.0],
            shape: rectangle::Shape::Round(3.5, 8),
            border: None,
        }
        .draw(
            [0.0, 0.0, orange_rectangle_width, 14.0],
            &context.draw_state,
            orange_transform,
            graphics,
        );

        // Global::draw_rectangle(
        //     context,
        //     &CanvasRectangle {
        //         top_left: orange_position,
        //         size: Size {
        //             width: orange_rectangle_width,
        //             height: 14.0,
        //         },
        //     },
        //     3.5,
        // )?;
        // context.set_fill_style("#ff8c1a");
        // context.fill();

        text::Text {
            color: [1.0, 1.0, 1.0, 1.0],
            font_size: FONT_SIZE,
            round: false,
        }
        .draw(
            variable_name,
            character_cache,
            &context.draw_state,
            orange_transform.trans_pos([(orange_rectangle_width - value_width) / 2.0, 11.5]),
            graphics,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Variables {
    variables: RwLock<HashMap<String, Variable>>,
}

impl Variables {
    fn new(scratch_file_variables: &HashMap<String, file::Variable>, monitors: &[Monitor]) -> Self {
        let mut variables: HashMap<String, Variable> = HashMap::new();
        for (key, v) in scratch_file_variables {
            let monitor = monitors.iter().find(|m| &m.id == key);
            let variable = match monitor {
                Some(monitor) => Variable {
                    name: v.id.clone(),
                    value: v.value.clone().into(),
                    monitored: monitor.visible,
                    position: CanvasCoordinate {
                        x: monitor.x,
                        y: monitor.y,
                    },
                },
                None => Variable {
                    name: v.id.clone(),
                    value: v.value.clone().into(),
                    monitored: false,
                    position: CanvasCoordinate { x: 0.0, y: 0.0 },
                },
            };
            variables.insert(key.clone(), variable);
        }

        Self {
            variables: RwLock::new(variables),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Value> {
        match self.variables.read().await.get(key) {
            Some(v) => Ok(v.value.clone()),
            None => Err(Error::msg(format!("key does not exist: {}", key))),
        }
    }

    pub async fn set(&self, key: &str, value: Value) -> Result<()> {
        let mut variables = self.variables.write().await;
        let variable = match variables.get_mut(key) {
            Some(v) => v,
            None => return Err(Error::msg(format!("key does not exist: {}", key))),
        };

        variable.value = value;
        Ok(())
    }

    pub async fn set_with<F>(&self, key: &str, function: F) -> Result<()>
    where
        F: FnOnce(&Value) -> Value,
    {
        let mut variables = self.variables.write().await;
        let mut variable = match variables.get_mut(key) {
            Some(v) => v,
            None => return Err(Error::msg(format!("key does not exist: {}", key))),
        };

        variable.value = function(&variable.value);
        Ok(())
    }

    pub async fn set_monitored(&self, key: &str, monitored: bool) -> Result<()> {
        let mut variables = self.variables.write().await;
        match variables.get_mut(key) {
            Some(v) => {
                v.monitored = monitored;
                Ok(())
            }
            None => Err(Error::msg(format!("key does not exist: {}", key))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
    value: Value,
    monitored: bool,
    position: CanvasCoordinate,
}
