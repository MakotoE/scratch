use super::*;
use crate::blocks::BlockInfo;
use crate::broadcaster::LayerChange;
use crate::coordinate::SpriteRectangle;
use crate::runtime::Global;
use crate::sprite::{Sprite, SpriteID};
use crate::thread::StepStatus;
use crate::vm::ThreadID;
use arrayvec::ArrayVec;
use graphics::Context;
use graphics_buffer::{BufferGlyphs, RenderBuffer};
use piston_window::{G2d, Glyphs};
use std::iter::{once, repeat_with};
use tokio::time::{sleep, Duration};

/// I needed a map that can to add cloned sprites while other sprites are still running.
#[derive(Debug)]
pub struct SpriteMap {
    sprite_groups: SpriteGroups,
    draw_order: RwLock<DrawOrder>,
    removed_sprites: RwLock<HashSet<SpriteID>>,
    stopped_threads: RwLock<HashSet<ThreadID>>,
    global: Arc<Global>,
}

type SpriteGroups = [RwLock<HashMap<SpriteID, Sprite>>; 64];

impl SpriteMap {
    pub fn new(
        sprites: HashMap<SpriteID, Sprite>,
        targets: &[Target],
        global: Arc<Global>,
    ) -> Self {
        let sprite_groups_array: ArrayVec<[RwLock<HashMap<SpriteID, Sprite>>; 64]> =
            once(RwLock::new(sprites))
                .chain(repeat_with(RwLock::default))
                .take(64)
                .collect();

        Self {
            sprite_groups: sprite_groups_array.into_inner().unwrap(),
            draw_order: RwLock::new(DrawOrder::new(targets)),
            removed_sprites: RwLock::default(),
            stopped_threads: RwLock::default(),
            global,
        }
    }

    pub async fn step(&self, thread_id: ThreadID) -> Result<Option<ThreadID>> {
        if self.stopped_threads.write().await.remove(&thread_id)
            || self
                .removed_sprites
                .read()
                .await
                .contains(&thread_id.sprite_id)
        {
            return Ok(None);
        }

        for group in &self.sprite_groups {
            if let Some(sprite) = group.read().await.get(&thread_id.sprite_id) {
                let result = sprite
                    .step(thread_id.thread_id)
                    .await
                    .map(|status| match status {
                        StepStatus::Continue => Some(thread_id),
                        StepStatus::Done => None,
                    });
                // Hacky fix for unresponsive menu screen in Pixel Snake
                // yield_now() did not work
                sleep(Duration::from_millis(0)).await;
                return result;
            }
        }
        Err(Error::msg("thread_id is invalid"))
    }

    pub async fn remove(&self, sprite_id: SpriteID) {
        self.removed_sprites.write().await.insert(sprite_id);
    }

    pub async fn draw(
        &self,
        context: &Context,
        graphics: &mut G2d<'_>,
        character_cache: &mut Glyphs,
    ) -> Result<()> {
        self.global.draw(context, graphics, character_cache).await?;

        let removed_sprites = self.removed_sprites.read().await;
        for id in self.draw_order.read().await.iter() {
            if !removed_sprites.contains(id) {
                let mut found = false;
                for group in &self.sprite_groups {
                    if let Some(sprite) = group.read().await.get(id) {
                        sprite.draw(context, graphics, character_cache).await?;
                        found = true;
                        break;
                    }
                }
                assert!(found);
            }
        }
        Ok(())
    }

    pub async fn draw_to_buffer(
        &self,
        context: &mut Context,
        graphics: &mut RenderBuffer,
        character_cache: &mut BufferGlyphs<'_>,
        removed_sprite: &SpriteID,
    ) -> Result<()> {
        let removed_sprites = self.removed_sprites.read().await;
        for id in self.draw_order.read().await.iter() {
            if !removed_sprites.contains(id) && id != removed_sprite {
                let mut found = false;
                for group in &self.sprite_groups {
                    if let Some(sprite) = group.read().await.get(id) {
                        sprite.draw(context, graphics, character_cache).await?;
                        found = true;
                        break;
                    }
                }
                assert!(found);
            }
        }
        Ok(())
    }

    pub async fn all_thread_ids(&self) -> Vec<ThreadID> {
        let mut result: Vec<ThreadID> = Vec::new();
        for group in &self.sprite_groups {
            for (sprite_id, sprite) in group.read().await.iter() {
                for thread_id in 0..sprite.number_of_threads() {
                    result.push(ThreadID {
                        sprite_id: *sprite_id,
                        thread_id,
                    });
                }
            }
        }
        result
    }

    pub async fn block_info(&self, thread_id: ThreadID) -> Result<BlockInfo> {
        for group in &self.sprite_groups {
            if let Some(sprite) = group.read().await.get(&thread_id.sprite_id) {
                return sprite.block_info(thread_id.thread_id).await;
            }
        }

        Err(Error::msg(format!("thread_id not found: {:?}", thread_id)))
    }

    pub async fn clone_sprite(&self, sprite_id: SpriteID) -> Result<SpriteID> {
        // Race condition - id creation and sprite insertion is not atomic
        let new_sprite_id = SpriteMap::new_sprite_id(&self.sprite_groups, sprite_id).await;
        let cloned_sprite =
            SpriteMap::get_cloned_sprite(&self.sprite_groups, &sprite_id, new_sprite_id).await?;
        SpriteMap::insert_sprite(&self.sprite_groups, new_sprite_id, cloned_sprite).await?;

        let mut draw_order = self.draw_order.write().await;
        let index = draw_order.iter().position(|s| s == &sprite_id).unwrap();
        draw_order.insert(index + 1, new_sprite_id);
        Ok(new_sprite_id)
    }

    async fn new_sprite_id(sprite_groups: &SpriteGroups, sprite_id: SpriteID) -> SpriteID {
        let contains_id = async move |id: SpriteID| {
            for group in sprite_groups {
                if group.read().await.contains_key(&id) {
                    return true;
                }
            }

            false
        };

        let mut clone_number = 0;
        loop {
            let new_sprite_id = SpriteID::from_sprite_name(
                &(format!("{}", sprite_id) + "clone" + &clone_number.to_string()),
            );

            if contains_id(new_sprite_id).await {
                clone_number += 1;
            } else {
                return new_sprite_id;
            }
        }
    }

    async fn get_cloned_sprite(
        sprite_groups: &SpriteGroups,
        sprite_id: &SpriteID,
        new_sprite_id: SpriteID,
    ) -> Result<Sprite> {
        for group in sprite_groups {
            if let Some(sprite) = group.read().await.get(sprite_id) {
                return sprite.clone_sprite(new_sprite_id).await;
            }
        }
        Err(Error::msg("sprite_id does not exist"))
    }

    async fn insert_sprite(
        sprite_groups: &SpriteGroups,
        new_sprite_id: SpriteID,
        sprite: Sprite,
    ) -> Result<()> {
        for group_cell in sprite_groups {
            if let Some(mut group) = group_cell.try_write() {
                if matches!(group.insert(new_sprite_id, sprite), Some(_)) {
                    panic!("new_sprite_id exists: {}", new_sprite_id);
                }
                return Ok(());
            }
        }
        Err(Error::msg("could not acquire a lock from any sprite group"))
    }

    pub async fn number_of_threads(&self, sprite_id: &SpriteID) -> Result<usize> {
        for group in &self.sprite_groups {
            if let Some(sprite) = group.read().await.get(&sprite_id) {
                return Ok(sprite.number_of_threads());
            }
        }

        Err(Error::msg(format!("sprite_id not found: {}", sprite_id)))
    }

    pub async fn stop(&self, thread_id: ThreadID) {
        self.stopped_threads.write().await.insert(thread_id);
    }

    pub async fn change_layer(&self, id: SpriteID, change: LayerChange) -> Result<()> {
        self.draw_order.write().await.change_layer(id, change)
    }

    pub async fn sprite_rectangle(&self, id: &SpriteID) -> Result<SpriteRectangle> {
        for group in &self.sprite_groups {
            if let Some(sprite) = group.read().await.get(id) {
                return Ok(sprite.rectangle().await);
            }
        }

        Err(Error::msg(format!("id not found: {}", id)))
    }
}

#[derive(Debug)]
struct DrawOrder {
    /// Lowest index = back, highest index = front
    ids: Vec<SpriteID>,
}

impl DrawOrder {
    fn new(targets: &[Target]) -> Self {
        let mut id_layer_order: Vec<(SpriteID, usize)> = targets
            .iter()
            .map(|t| (SpriteID::from_sprite_name(&t.name), t.layer_order))
            .collect();

        id_layer_order.sort_unstable_by(|a, b| a.1.cmp(&b.1));

        Self {
            ids: id_layer_order.iter().map(|i| i.0).collect(),
        }
    }

    fn iter(&self) -> std::slice::Iter<SpriteID> {
        self.ids.iter()
    }

    fn change_layer(&mut self, id: SpriteID, change: LayerChange) -> Result<()> {
        match self.ids.iter().position(|sprite_id| sprite_id == &id) {
            Some(index) => self.ids.remove(index),
            None => return Err(Error::msg(format!("id not found: {}", id))),
        };

        match change {
            LayerChange::Front => self.ids.push(id),
            LayerChange::Back => self.ids.insert(0, id),
        }
        Ok(())
    }

    fn insert(&mut self, index: usize, id: SpriteID) {
        self.ids.insert(index, id)
    }
}
