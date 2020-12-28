use super::*;
use crate::blocks::value::Value;
use crate::broadcaster::Broadcaster;
use crate::canvas::{CanvasContext, Corner, Direction};
use crate::coordinate::{CanvasCoordinate, CanvasRectangle, Size, Transformation};
use crate::file::Monitor;
use crate::sprite_runtime::SpriteRuntime;
use crate::vm::ThreadID;

#[derive(Debug, Clone)]
pub struct Runtime {
    pub sprite: Rc<RwLock<SpriteRuntime>>,
    pub global: Rc<Global>,
    thread_id: ThreadID,
}

impl Runtime {
    pub fn new(sprite: Rc<RwLock<SpriteRuntime>>, global: Rc<Global>, thread_id: ThreadID) -> Self {
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

    pub async fn redraw(&self, context: &CanvasContext<'_>) -> Result<()> {
        for variable in self.variables.variables.borrow().values() {
            if variable.monitored {
                Global::draw_monitor(
                    context,
                    &variable.position,
                    &variable.name,
                    &variable.value.clone().to_string(),
                )?;
            }
        }
        Ok(())
    }

    fn draw_monitor(
        context: &CanvasContext,
        position: &CanvasCoordinate,
        variable_name: &str,
        value_str: &str,
    ) -> Result<()> {
        const NAME_FONT: &str = "12px Helvetica, sans-serif";
        const VALUE_FONT: &str = "12px Helvetica, sans-serif";

        context.set_font(NAME_FONT);
        let name_width = context.measure_text(variable_name)?;

        context.set_font(VALUE_FONT);
        let value_width = context.measure_text(value_str)?;

        let orange_rectangle_width = f64::max(39.0 - value_width, value_width + 4.0);

        Global::draw_rectangle(
            context,
            &CanvasRectangle {
                top_left: *position,
                size: Size {
                    width: name_width + orange_rectangle_width + 24.0,
                    height: 20.0,
                },
            },
            3.5,
        )?;
        context.set_fill_style("#e6f0ff");
        context.fill();
        context.set_stroke_style("#c4ccd9");
        context.set_line_width(1.0);
        context.stroke();

        context.set_fill_style("#575e75");
        context.set_font(NAME_FONT);
        context.fill_text(
            variable_name,
            &position.add(&CanvasCoordinate { x: 7.0, y: 14.0 }),
        )?;

        let orange_position = position.add(&CanvasCoordinate {
            x: name_width + 16.0,
            y: 3.0,
        });
        Global::draw_rectangle(
            context,
            &CanvasRectangle {
                top_left: orange_position,
                size: Size {
                    width: orange_rectangle_width,
                    height: 14.0,
                },
            },
            3.5,
        )?;
        context.set_fill_style("#ff8c1a");
        context.fill();

        context.set_fill_style("#ffffff");
        context.set_font(VALUE_FONT);
        context.fill_text(
            value_str,
            &orange_position.add(&CanvasCoordinate {
                x: (orange_rectangle_width - value_width) / 2.0,
                y: 11.5,
            }),
        )?;
        Ok(())
    }

    fn draw_rectangle(
        context: &CanvasContext,
        rectangle: &CanvasRectangle,
        corner_radius: f64,
    ) -> Result<()> {
        let context = context.with_transformation(Transformation::translate(rectangle.top_left));
        context.begin_path();
        context.move_to(&CanvasCoordinate {
            x: corner_radius,
            y: 0.0,
        });
        context.rounded_corner(
            &CanvasCoordinate {
                x: rectangle.size.width - corner_radius,
                y: corner_radius,
            },
            corner_radius,
            Corner::TopRight,
            Direction::Clockwise,
        )?;
        context.rounded_corner(
            &CanvasCoordinate {
                x: rectangle.size.width - corner_radius,
                y: rectangle.size.height - corner_radius,
            },
            corner_radius,
            Corner::BottomRight,
            Direction::Clockwise,
        )?;
        context.rounded_corner(
            &CanvasCoordinate {
                x: corner_radius,
                y: rectangle.size.height - corner_radius,
            },
            corner_radius,
            Corner::BottomLeft,
            Direction::Clockwise,
        )?;
        context.rounded_corner(
            &CanvasCoordinate {
                x: corner_radius,
                y: corner_radius,
            },
            corner_radius,
            Corner::TopLeft,
            Direction::Clockwise,
        )?;
        context.close_path();
        Ok(())
    }
}

#[derive(Debug)]
pub struct Variables {
    variables: RefCell<HashMap<String, Variable>>,
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
            variables: RefCell::new(variables),
        }
    }

    pub async fn get(&self, key: &str) -> Result<Value> {
        match self.variables.borrow().get(key) {
            Some(v) => Ok(v.value.clone()),
            None => Err(wrap_err!(format!("key does not exist: {}", key))),
        }
    }

    pub async fn set(&self, key: &str, value: Value) -> Result<()> {
        let mut variables = self.variables.borrow_mut();
        let variable = match variables.get_mut(key) {
            Some(v) => v,
            None => return Err(wrap_err!(format!("key does not exist: {}", key))),
        };

        variable.value = value;
        Ok(())
    }

    pub async fn set_with<F>(&self, key: &str, function: F) -> Result<()>
    where
        F: FnOnce(&Value) -> Value,
    {
        let mut variables = self.variables.borrow_mut();
        let mut variable = match variables.get_mut(key) {
            Some(v) => v,
            None => return Err(wrap_err!(format!("key does not exist: {}", key))),
        };

        variable.value = function(&variable.value);
        Ok(())
    }

    pub async fn set_monitored(&self, key: &str, monitored: bool) -> Result<()> {
        let mut variables = self.variables.borrow_mut();
        match variables.get_mut(key) {
            Some(v) => {
                v.monitored = monitored;
                Ok(())
            }
            None => Err(wrap_err!(format!("key does not exist: {}", key))),
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
