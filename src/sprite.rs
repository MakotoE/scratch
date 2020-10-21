use super::*;
use blocks::*;
use runtime::{Coordinate, Runtime};
use thread::Thread;

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<Thread>,
    runtime: Runtime,
}

impl Sprite {
    pub async fn new(runtime: Runtime, target: &savefile::Target) -> Result<Self> {
        runtime
            .sprite
            .write()
            .await
            .set_position(&Coordinate::new(target.x, target.y));

        let mut threads: Vec<Thread> = Vec::new();

        for (thread_id, hat_id) in find_hats(&target.blocks).iter().enumerate() {
            let block = match block_tree(hat_id.to_string(), runtime.clone(), &target.blocks) {
                Ok(b) => b,
                Err(e) => return Err(ErrorKind::Initialization(Box::new(e)).into()),
            };
            let thread = Thread::start(block, runtime.clone(), thread_id);
            threads.push(thread);
        }

        Ok(Self { threads, runtime })
    }

    pub async fn continue_(&self) {
        self.runtime.sprite.write().await.set_draw_debug_info(false);
        for thread in &self.threads {
            thread.continue_().await;
        }
    }

    pub async fn pause(&self) {
        for thread in &self.threads {
            thread.pause().await;
        }
        let mut runtime = self.runtime.sprite.write().await;
        runtime.set_draw_debug_info(true);
    }

    pub fn step(&self) {
        for thread in &self.threads {
            thread.step();
        }
    }

    pub fn threads(&self) -> &[Thread] {
        &self.threads
    }
}

pub fn find_hats(block_infos: &HashMap<String, savefile::Block>) -> Vec<&str> {
    let mut hats: Vec<&str> = Vec::new();
    for (id, block_info) in block_infos {
        if block_info.top_level {
            hats.push(id);
        }
    }
    hats.sort_unstable();

    hats
}
