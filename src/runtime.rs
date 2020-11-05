use super::*;
use sprite_runtime::SpriteRuntime;
use tokio::sync::watch::{channel, Receiver, Sender};

#[derive(Debug, Clone)]
pub struct Runtime {
    pub sprite: Rc<RwLock<SpriteRuntime>>,
    pub global: Global,
}

#[derive(Debug, Clone)]
pub struct Global {
    pub variables: Rc<RwLock<HashMap<String, serde_json::Value>>>,
    pub broadcaster: Rc<Broadcaster>,
}

impl Global {
    pub fn new(scratch_file_variables: &HashMap<String, savefile::Variable>) -> Self {
        let mut variables: HashMap<String, serde_json::Value> = HashMap::new();
        for (key, v) in scratch_file_variables {
            variables.insert(key.clone(), v.value.clone());
        }

        Self {
            variables: Rc::new(RwLock::new(variables)),
            broadcaster: Rc::new(Broadcaster::new()),
        }
    }
}

#[derive(Debug)]
pub struct Broadcaster {
    sender: Sender<String>,
    receiver: Receiver<String>,
}

impl Broadcaster {
    fn new() -> Self {
        let (sender, receiver) = channel(String::new());
        Self { sender, receiver }
    }

    pub fn send(&self, s: String) -> Result<()> {
        Ok(self.sender.send(s)?)
    }

    pub fn receiver(&self) -> Receiver<String> {
        self.receiver.clone()
    }
}
