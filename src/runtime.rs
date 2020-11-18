use super::*;
use crate::coordinate::{CanvasCoordinate, SpriteCoordinate};
use blocks::value_to_string;
use savefile::Monitor;
use serde_json::Value;
use sprite::SpriteID;
use sprite_runtime::SpriteRuntime;
use std::f64::consts::TAU;
use tokio::sync::broadcast::{channel, Receiver, Sender};
use vm::ThreadID;

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
        scratch_file_variables: &HashMap<String, savefile::Variable>,
        monitors: &Vec<Monitor>,
    ) -> Self {
        Self {
            variables: Variables::new(scratch_file_variables, monitors),
            broadcaster: Broadcaster::new(),
        }
    }

    pub fn need_redraw(&self) -> bool {
        true
    }

    pub async fn redraw(&self, context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        for (name, variable) in self.variables.variables.read().await.iter() {
            if variable.monitored {
                Global::draw_monitor(
                    context,
                    &variable.position,
                    name,
                    &value_to_string(variable.value.clone()),
                )?;
            }
        }
        Ok(())
    }

    // TODO display variable name, not id
    fn draw_monitor(
        context: &web_sys::CanvasRenderingContext2d,
        position: &CanvasCoordinate,
        variable_name: &str,
        value_str: &str,
    ) -> Result<()> {
        const NAME_FONT: &str = "12px Helvetica, sans-serif";
        const VALUE_FONT: &str = "12px Helvetica, sans-serif";

        context.set_font(NAME_FONT);
        let name_width: f64 = context.measure_text(variable_name)?.width();

        context.set_font(VALUE_FONT);
        let value_width: f64 = context.measure_text(value_str)?.width();

        let orange_rectangle_width = f64::max(39.0 - value_width, value_width + 4.0);

        Global::draw_rectangle(
            context,
            position,
            name_width + orange_rectangle_width + 24.0,
            20.0,
            3.5,
        )?;
        context.set_fill_style(&"#e6f0ff".into());
        context.fill();
        context.set_stroke_style(&"#c4ccd9".into());
        context.set_line_width(1.0);
        context.stroke();

        context.set_fill_style(&"#575e75".into());
        context.set_font(NAME_FONT);
        context.fill_text(variable_name, position.x + 7.0, position.y + 14.0)?;

        let orange_position = position.add(&CanvasCoordinate::new(name_width + 16.0, 3.0));
        Global::draw_rectangle(context, &orange_position, orange_rectangle_width, 14.0, 3.5)?;
        context.set_fill_style(&"#ff8c1a".into());
        context.fill();

        context.set_fill_style(&"#ffffff".into());
        context.set_font(VALUE_FONT);
        context.fill_text(
            value_str,
            orange_position.x + (orange_rectangle_width - value_width) / 2.0,
            orange_position.y + 11.5,
        )?;
        Ok(())
    }

    fn draw_rectangle(
        context: &web_sys::CanvasRenderingContext2d,
        position: &CanvasCoordinate,
        width: f64,
        height: f64,
        corner_radius: f64,
    ) -> Result<()> {
        context.begin_path();
        context.move_to(position.x + corner_radius, position.y + 0.0);
        context.arc(
            position.x + width - corner_radius,
            position.y + corner_radius,
            corner_radius,
            3.0 / 4.0 * TAU,
            0.0,
        )?;
        context.arc(
            position.x + width - corner_radius,
            position.y + height - corner_radius,
            corner_radius,
            0.0,
            1.0 / 4.0 * TAU,
        )?;
        context.arc(
            position.x + corner_radius,
            position.y + height - corner_radius,
            corner_radius,
            1.0 / 4.0 * TAU,
            2.0 / 4.0 * TAU,
        )?;
        context.arc(
            position.x + corner_radius,
            position.y + corner_radius,
            corner_radius,
            2.0 / 4.0 * TAU,
            3.0 / 4.0 * TAU,
        )?;
        context.close_path();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Broadcaster {
    sender: Sender<BroadcastMsg>,
}

impl Broadcaster {
    fn new() -> Self {
        Self {
            sender: channel(1).0,
        }
    }

    pub fn send(&self, m: BroadcastMsg) -> Result<()> {
        log::info!("broadcast: {:?}", &m);
        self.sender.send(m)?;
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<BroadcastMsg> {
        self.sender.subscribe()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BroadcastMsg {
    Start(String),
    Finished(String),
    Clone(SpriteID),
    DeleteClone(SpriteID),
    Click(SpriteCoordinate),
    Stop(Stop),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Stop {
    All,
    ThisThread(ThreadID),
    OtherThreads(ThreadID),
}

#[derive(Debug)]
pub struct Variables {
    variables: RwLock<HashMap<String, Variable>>,
}

impl Variables {
    fn new(
        scratch_file_variables: &HashMap<String, savefile::Variable>,
        monitors: &[Monitor],
    ) -> Self {
        let mut variables: HashMap<String, Variable> = HashMap::new();
        for (key, v) in scratch_file_variables {
            let monitor = monitors.iter().find(|m| &m.id == key);
            let variable = match monitor {
                Some(monitor) => Variable {
                    value: v.value.clone(),
                    monitored: monitor.visible,
                    position: CanvasCoordinate::new(monitor.x, monitor.y),
                },
                None => Variable {
                    value: v.value.clone(),
                    monitored: false,
                    position: CanvasCoordinate::new(0.0, 0.0),
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
            None => Err(wrap_err!(format!("key does not exist: {}", key))),
        }
    }

    pub async fn set(&self, key: &str, value: Value) -> Result<()> {
        let mut variables = self.variables.write().await;
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
        let mut variables = self.variables.write().await;
        let mut variable = match variables.get_mut(key) {
            Some(v) => v,
            None => return Err(wrap_err!(format!("key does not exist: {}", key))),
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
            None => Err(wrap_err!(format!("key does not exist: {}", key))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    value: Value,
    monitored: bool,
    position: CanvasCoordinate,
}
