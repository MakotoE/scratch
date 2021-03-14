use super::*;
use crate::blocks::*;
use crate::coordinate::SpriteRectangle;
use crate::file::{BlockID, Image, Target};
use crate::runtime::{Global, Runtime};
use crate::sprite_runtime::{GraphicsCostumeTexture, SpriteRuntime};
use crate::thread::{BlockInputs, Thread};
use crate::vm::ThreadID;
use graphics::character::CharacterCache;
use graphics::Context;
use piston_window::G2dTextureContext;
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<RwLock<Thread>>,
    runtime: Runtime,
    target: Target,
}

impl Sprite {
    pub async fn new(
        sprite_id: SpriteID,
        sprite_runtime: SpriteRuntime,
        global: Arc<Global>,
        target: Target,
    ) -> Result<Self> {
        let mut threads: Vec<RwLock<Thread>> = Vec::new();

        let sprite_runtime_ref = Arc::new(RwLock::new(sprite_runtime));

        for hat_id in find_hats(&target.blocks) {
            let runtime = Runtime::new(
                sprite_runtime_ref.clone(),
                global.clone(),
                ThreadID {
                    sprite_id,
                    thread_id: threads.len(),
                },
            );

            let thread = Thread::start(hat_id, runtime, &target.blocks)?;
            threads.push(RwLock::new(thread));
        }

        Ok(Self {
            threads,
            runtime: Runtime::new(
                sprite_runtime_ref,
                global,
                ThreadID {
                    sprite_id,
                    thread_id: 0,
                },
            ),
            target,
        })
    }

    pub async fn add_costumes(
        &mut self,
        texture_context: &mut G2dTextureContext,
        costumes: &[file::Costume],
        images: &HashMap<String, Image>,
    ) -> Result<()> {
        self.runtime
            .sprite
            .write()
            .await
            .add_costumes(texture_context, costumes, images)
            .await
    }

    pub fn number_of_threads(&self) -> usize {
        self.threads.len()
    }

    pub async fn block_info(&self, thread_id: usize) -> Result<BlockInfo> {
        if let Some(thread) = self.threads.get(thread_id) {
            Ok(thread.read().await.block_info())
        } else {
            Error::msg(format!("thread_id does not exist: {}", thread_id))
        }
    }

    pub async fn step(&self, thread_id: usize) -> Result<()> {
        self.threads[thread_id].write().await.step().await
    }

    pub async fn need_redraw(&self) -> bool {
        self.runtime.sprite.read().await.need_redraw()
    }

    pub async fn redraw<G, C>(
        &self,
        context: &Context,
        graphics: &mut G,
        character_cache: &mut C,
    ) -> Result<()>
    where
        G: GraphicsCostumeTexture<C>,
        C: CharacterCache,
    {
        self.runtime
            .sprite
            .write()
            .await
            .redraw_frame(context, graphics, character_cache)
    }

    pub async fn block_inputs(&self) -> Vec<BlockInputs> {
        let mut result: Vec<BlockInputs> = Vec::with_capacity(self.threads.len());
        for thread in &self.threads {
            result.push(thread.read().await.block_inputs());
        }
        result
    }

    pub async fn clone_sprite(&self, new_sprite_id: SpriteID) -> Result<Sprite> {
        let sprite_runtime = self.runtime.sprite.read().await.clone_sprite_runtime();
        Sprite::new(
            new_sprite_id,
            sprite_runtime,
            self.runtime.global.clone(),
            self.target.clone(),
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
        write!(f, "{:x}", self.hash as u32)
    }
}
