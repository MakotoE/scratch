use super::*;
use sprite_runtime::SpriteRuntime;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Runtime {
    pub sprite: Rc<RwLock<SpriteRuntime>>,
    pub global: Global,
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
        let (sender, _) = channel(1);
        Self { sender }
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

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum BroadcastMsg {
    Start(String),
    Finished(String),
    Clone(usize),
    DeleteClone(usize),
}
