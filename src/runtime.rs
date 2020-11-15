use super::*;
use serde_json::Value;
use sprite::SpriteID;
use sprite_runtime::{Coordinate, SpriteRuntime};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use vm::ThreadID;

#[derive(Debug, Clone)]
pub struct Runtime {
    pub sprite: Rc<RwLock<SpriteRuntime>>,
    pub global: Global,
    thread_id: ThreadID,
}

impl Runtime {
    pub fn new(sprite: Rc<RwLock<SpriteRuntime>>, global: Global, thread_id: ThreadID) -> Self {
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

#[derive(Debug, Clone)]
pub struct Global {
    pub variables: Rc<Variables>,
    pub broadcaster: Broadcaster,
}

impl Global {
    pub fn new(scratch_file_variables: &HashMap<String, savefile::Variable>) -> Self {
        Self {
            variables: Rc::new(Variables::new(scratch_file_variables)),
            broadcaster: Broadcaster::new(),
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BroadcastMsg {
    Start(String),
    Finished(String),
    Clone(SpriteID),
    DeleteClone(SpriteID),
    Click(Coordinate),
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
    fn new(scratch_file_variables: &HashMap<String, savefile::Variable>) -> Self {
        let mut variables: HashMap<String, Variable> = HashMap::new();
        for (key, v) in scratch_file_variables {
            let variable = Variable {
                value: v.value.clone(),
                monitored: false,
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
}

#[derive(Debug, Clone)]
pub struct Variable {
    value: Value,
    monitored: bool,
}
