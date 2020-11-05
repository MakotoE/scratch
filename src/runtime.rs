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
    sender: Sender<BroadcastMsg>,
    receiver: Receiver<BroadcastMsg>,
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum BroadcastMsg {
    Start(String),
    Finished(String),
}

impl Broadcaster {
    fn new() -> Self {
        let (sender, receiver) = channel(BroadcastMsg::Start(String::new()));
        Self { sender, receiver }
    }

    pub fn send(&self, m: BroadcastMsg) -> Result<()> {
        log::info!("broadcast: \"{:?}\"", &m);
        Ok(self.sender.send(m)?)
    }

    pub fn receiver(&self) -> Receiver<BroadcastMsg> {
        self.receiver.clone()
    }

    pub async fn wait_until(receiver: &mut Receiver<BroadcastMsg>, msg: &str) -> Result<()> {
        loop {
            receiver.changed().await?;
            if let BroadcastMsg::Start(s) = &*receiver.borrow() {
                if s == msg {
                    return Ok(());
                }
            }
        }
    }

    pub async fn wait_until_finished(
        receiver: &mut Receiver<BroadcastMsg>,
        msg: &str,
    ) -> Result<()> {
        loop {
            receiver.changed().await?;
            if let BroadcastMsg::Finished(s) = &*receiver.borrow() {
                if s == msg {
                    return Ok(());
                }
            }
        }
    }
}
