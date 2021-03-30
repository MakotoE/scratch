use super::*;
use crate::blocks::*;
use crate::coordinate::SpriteRectangle;
use crate::file::BlockID;
use crate::runtime::{Global, Runtime};
use crate::sprite_runtime::{Costumes, GraphicsCostumeTexture, SpriteRuntime};
use crate::thread::{BlockInputs, StepStatus, Thread};
use crate::vm::ThreadID;
use graphics::character::CharacterCache;
use graphics::Context;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<RwLock<Thread>>,
    global_runtime: Arc<Global>,
    sprite_runtime: Arc<RwLock<SpriteRuntime>>,
    block_infos: HashMap<BlockID, file::Block>,
}

impl Sprite {
    pub async fn new(
        sprite_id: SpriteID,
        sprite_runtime: SpriteRuntime,
        global: Arc<Global>,
        block_infos: HashMap<BlockID, file::Block>,
    ) -> Result<Self> {
        let sprite_runtime_ref = Arc::new(RwLock::new(sprite_runtime));

        let threads: Result<Vec<RwLock<Thread>>> = find_hats(&block_infos)
            .drain(..)
            .enumerate()
            .map(|(thread_id, hat_id)| -> Result<RwLock<Thread>> {
                let runtime = Runtime::new(
                    sprite_runtime_ref.clone(),
                    global.clone(),
                    ThreadID {
                        sprite_id,
                        thread_id,
                    },
                );

                let blocks = block_tree(hat_id, runtime, &block_infos)?;
                Ok(RwLock::new(Thread::new(hat_id, blocks)))
            })
            .collect();

        Ok(Self {
            threads: threads?,
            global_runtime: global,
            sprite_runtime: sprite_runtime_ref,
            block_infos,
        })
    }

    pub async fn set_costumes(&mut self, costumes: Costumes) {
        self.sprite_runtime.write().await.set_costumes(costumes);
    }

    pub fn number_of_threads(&self) -> usize {
        self.threads.len()
    }

    pub async fn block_info(&self, thread_id: usize) -> Result<BlockInfo> {
        if let Some(thread) = self.threads.get(thread_id) {
            Ok(thread.read().await.block_info())
        } else {
            Err(Error::msg(format!(
                "thread_id does not exist: {}",
                thread_id
            )))
        }
    }

    pub async fn step(&self, thread_id: usize) -> Result<StepStatus> {
        self.threads[thread_id].write().await.step().await
    }

    pub async fn draw<G, C>(
        &self,
        context: &Context,
        graphics: &mut G,
        character_cache: &mut C,
    ) -> Result<()>
    where
        G: GraphicsCostumeTexture<C>,
        C: CharacterCache,
    {
        self.sprite_runtime
            .write()
            .await
            .draw(context, graphics, character_cache)
    }

    pub async fn block_inputs(&self) -> Vec<BlockInputs> {
        let mut result: Vec<BlockInputs> = Vec::with_capacity(self.threads.len());
        for thread in &self.threads {
            result.push(thread.read().await.block_inputs());
        }
        result
    }

    pub async fn clone_sprite(&self, new_sprite_id: SpriteID) -> Result<Sprite> {
        let sprite_runtime = self.sprite_runtime.read().await.clone_sprite_runtime();
        Sprite::new(
            new_sprite_id,
            sprite_runtime,
            self.global_runtime.clone(),
            self.block_infos.clone(),
        )
        .await
    }

    pub async fn rectangle(&self) -> SpriteRectangle {
        self.sprite_runtime.read().await.rectangle()
    }
}

fn find_hats(block_infos: &HashMap<BlockID, file::Block>) -> Vec<BlockID> {
    let mut hats: Vec<BlockID> = block_infos
        .iter()
        .filter(|(_, block)| -> bool {
            // Blocks without event watcher (has rounded top in editor) are ignored
            (block.opcode == "control_start_as_clone" || block.opcode.contains("_when"))
                && block.top_level
        })
        .map(|(id, _)| *id)
        .collect();
    hats.sort_unstable();
    hats
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
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
        write!(f, "{:x}", self.hash as u32)
    }
}
