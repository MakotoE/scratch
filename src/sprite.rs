use super::*;
use blocks::*;
use runtime::Global;
use runtime::Runtime;
use savefile::Image;
use savefile::Target;
use sprite_runtime::{Coordinate, SpriteRuntime};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use thread::Thread;

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<RefCell<Thread>>,
    runtime: Runtime,
    target: Rc<Target>,
    images: Rc<HashMap<String, Image>>,
}

impl Sprite {
    pub async fn new(
        global: Global,
        target: Rc<Target>,
        images: Rc<HashMap<String, Image>>,
        is_a_clone: bool,
    ) -> Result<(SpriteID, Self)> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        target.hash(&mut hasher);
        is_a_clone.hash(&mut hasher);

        let mut sprite_runtime = SpriteRuntime::new(
            &target.costumes,
            &images,
            hasher.finish().into(),
            is_a_clone,
        )
        .await?;

        sprite_runtime.set_position(&Coordinate::new(target.x as i16, target.y as i16));

        let runtime = Runtime {
            sprite: Rc::new(RwLock::new(sprite_runtime)),
            global,
        };

        let mut threads: Vec<RefCell<Thread>> = Vec::new();

        for (thread_id, hat_id) in find_hats(&target.blocks).iter().enumerate() {
            let block = match block_tree(hat_id.to_string(), runtime.clone(), &target.blocks) {
                Ok(b) => b,
                Err(e) => return Err(ErrorKind::Initialization(Box::new(e)).into()),
            };
            let thread = Thread::start(block, runtime.clone(), thread_id);
            threads.push(RefCell::new(thread));
        }

        Ok((
            hasher.finish().into(),
            Self {
                threads,
                runtime,
                target,
                images,
            },
        ))
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

    pub async fn clone_sprite(&self) -> Result<(SpriteID, Sprite)> {
        Sprite::new(
            self.runtime.global.clone(),
            self.target.clone(),
            self.images.clone(),
            true,
        )
        .await
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

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct SpriteID {
    hash: [u8; 8],
}

impl From<u64> for SpriteID {
    fn from(n: u64) -> Self {
        Self {
            hash: n.to_le_bytes(),
        }
    }
}

impl Debug for SpriteID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpriteID {{ ")?;
        Display::fmt(&self, f)?;
        write!(f, " }}")
    }
}

impl Display for SpriteID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for b in &self.hash[..4] {
            write!(f, "{:x}", b)?;
        }
        Ok(())
    }
}
