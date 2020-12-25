use super::*;
use crate::blocks::*;
use crate::canvas::CanvasContext;
use crate::coordinate::SpriteRectangle;
use crate::file::{BlockID, Image, Target};
use crate::runtime::{Global, Runtime};
use crate::sprite_runtime::SpriteRuntime;
use crate::thread::{BlockInputs, Thread};
use crate::vm::ThreadID;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<RefCell<Thread>>,
    runtime: Runtime,
    target: Rc<Target>,
    images: Rc<HashMap<String, Image>>,
}

impl Sprite {
    pub async fn new(
        global: Rc<Global>,
        target: Rc<Target>,
        images: Rc<HashMap<String, Image>>,
        is_a_clone: bool,
    ) -> Result<(SpriteID, Self)> {
        let mut sprite_name = target.name.to_string();
        if is_a_clone {
            sprite_name += "-clone";
        };
        let sprite_id = SpriteID::from_sprite_name(&sprite_name);

        let mut threads: Vec<RefCell<Thread>> = Vec::new();

        let sprite_runtime = Rc::new(RwLock::new(
            SpriteRuntime::new(&target, &images, is_a_clone, sprite_name).await?,
        ));

        for hat_id in find_hats(&target.blocks) {
            let runtime = Runtime::new(
                sprite_runtime.clone(),
                global.clone(),
                ThreadID {
                    sprite_id,
                    thread_id: threads.len(),
                },
            );

            let thread = Thread::start(hat_id, runtime, &target.blocks)?;
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

    pub async fn redraw(&self, context: &CanvasContext<'_>) -> Result<()> {
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

    pub async fn rectangle(&self) -> SpriteRectangle {
        self.runtime.sprite.read().await.rectangle()
    }
}

pub fn find_hats(block_infos: &HashMap<BlockID, file::Block>) -> Vec<BlockID> {
    let mut hats: Vec<BlockID> = Vec::new();
    for (id, block_info) in block_infos {
        // Blocks without event watcher (has rounded top in editor) are ignored
        if (block_info.opcode == "control_start_as_clone" || block_info.opcode.contains("_when"))
            && block_info.top_level
        {
            hats.push(*id);
        }
    }
    hats.sort_unstable();

    hats
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct SpriteID {
    hash: u64,
}

impl SpriteID {
    pub fn from_sprite_name(sprite_name: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        sprite_name.hash(&mut hasher);
        Self {
            hash: hasher.finish(),
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
        write!(f, "{:x}", self.hash)
    }
}
