use super::*;
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
    pub variables: Rc<RwLock<HashMap<String, serde_json::Value>>>,
    pub broadcaster: Broadcaster,
}

impl Global {
    pub fn new(scratch_file_variables: &HashMap<String, savefile::Variable>) -> Self {
        let mut variables: HashMap<String, serde_json::Value> = HashMap::new();
        for (key, v) in scratch_file_variables {
            variables.insert(key.clone(), v.value.clone());
        }

        Self {
            variables: Rc::new(RwLock::new(variables)),
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
