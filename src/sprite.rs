use super::*;
use blocks::*;
use runtime::{Global, Runtime};
use savefile::{BlockID, Image, Target};
use sprite_runtime::SpriteRuntime;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use thread::Thread;
use vm::ThreadID;

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<RefCell<Thread>>,
    runtime: Runtime,
    target: Rc<Target>,
    images: Rc<HashMap<String, Image>>,
    sprite_removed: RefCell<bool>,
}

impl Sprite {
    pub async fn new(
        global: Rc<Global>,
        target: Rc<Target>,
        images: Rc<HashMap<String, Image>>,
        is_a_clone: bool,
    ) -> Result<(SpriteID, Self)> {
        let mut hasher = DefaultHasher::new();
        target.hash(&mut hasher);
        is_a_clone.hash(&mut hasher);
        let sprite_id = SpriteID::from(hasher);

        let mut threads: Vec<RefCell<Thread>> = Vec::new();

        let sprite_runtime = Rc::new(RwLock::new(
            SpriteRuntime::new(&target, &images, is_a_clone).await?,
        ));

        for hat_id in find_hats(&target.blocks) {
            let runtime = Runtime::new(
                sprite_runtime.clone(),
                global.clone(),
                ThreadID {
                    sprite_id,
                    thread_id: 0,
                },
            );
            let block = match block_tree(hat_id, runtime.clone(), &target.blocks) {
                Ok(b) => b,
                Err(e) => return Err(ErrorKind::Initialization(Box::new(e)).into()),
            };
            let thread = Thread::start(block, runtime.clone(), 0);
            threads.push(RefCell::new(thread));
        }

        Ok((
            sprite_id,
            Self {
                threads,
                runtime: Runtime::new(
                    sprite_runtime,
                    global.clone(),
                    ThreadID {
                        sprite_id,
                        thread_id: 0,
                    },
                ),
                target,
                images,
                sprite_removed: RefCell::new(false),
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
        if *self.sprite_removed.borrow() {
            Ok(())
        } else {
            self.threads[thread_id].borrow_mut().step().await
        }
    }

    pub async fn need_redraw(&self) -> bool {
        self.runtime.sprite.read().await.need_redraw()
    }

    pub async fn redraw(&self, context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        if *self.sprite_removed.borrow() {
            Ok(())
        } else {
            self.runtime.sprite.write().await.redraw(context)
        }
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

    pub fn remove(&self) {
        self.sprite_removed.replace(true);
    }
}

pub fn find_hats(block_infos: &HashMap<BlockID, savefile::Block>) -> Vec<BlockID> {
    let mut hats: Vec<BlockID> = Vec::new();
    for (id, block_info) in block_infos {
        if block_info.top_level {
            hats.push(*id);
        }
    }
    hats.sort_unstable();

    hats
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct SpriteID {
    hash: [u8; 8],
}

impl<H> From<H> for SpriteID
where
    H: Hasher,
{
    fn from(hasher: H) -> Self {
        Self {
            hash: hasher.finish().to_le_bytes(),
        }
    }
}

impl Debug for SpriteID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("SpriteID { ")?;
        Display::fmt(self, f)?;
        f.write_str(" }")
    }
}

impl Display for SpriteID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.hash[..4]))
    }
}
