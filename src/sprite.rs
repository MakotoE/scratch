use super::*;
use blocks::*;
use runtime::Runtime;
use sprite_runtime::Coordinate;
use thread::Thread;

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<RefCell<Thread>>,
    runtime: Runtime,
}

impl Sprite {
    pub async fn new(runtime: Runtime, target: &savefile::Target) -> Result<Self> {
        runtime
            .sprite
            .write()
            .await
            .set_position(&Coordinate::new(target.x, target.y));

        let mut threads: Vec<RefCell<Thread>> = Vec::new();

        for (thread_id, hat_id) in find_hats(&target.blocks).iter().enumerate() {
            let block = match block_tree(hat_id.to_string(), runtime.clone(), &target.blocks) {
                Ok(b) => b,
                Err(e) => return Err(ErrorKind::Initialization(Box::new(e)).into()),
            };
            let thread = Thread::start(block, runtime.clone(), thread_id);
            threads.push(RefCell::new(thread));
        }

        Ok(Self { threads, runtime })
    }

    pub fn number_of_threads(&self) -> usize {
        self.threads.len()
    }

    pub fn block_info(&self, thread_id: usize) -> BlockInfo {
        self.threads[thread_id].borrow().block_info()
    }

    pub async fn step(&self, thread_id: usize) -> Result<()> {
        self.threads[thread_id].borrow_mut().step().await
    }

    pub async fn need_redraw(&self) -> bool {
        self.runtime.sprite.read().await.need_redraw()
    }

    pub async fn redraw(&self, context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        self.runtime.sprite.write().await.redraw(context)
    }

    pub fn block_inputs(&self) -> Vec<BlockInputs> {
        self.threads
            .iter()
            .map(|t| t.borrow().block_inputs())
            .collect()
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
