use super::*;
use runtime::{Global, Runtime};
use savefile::ScratchFile;
use sprite::Sprite;

pub struct VM {
    sprites: Vec<Sprite>,
}

impl VM {
    pub async fn new(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: &ScratchFile,
    ) -> Result<Self> {
        let global = Global::new(&scratch_file.project.targets[0].variables);

        let mut sprites: Vec<Sprite> = Vec::with_capacity(scratch_file.project.targets.len());
        for target in &scratch_file.project.targets {
            let runtime = runtime::SpriteRuntime::new(
                context.clone(),
                &target.costumes,
                &scratch_file.images,
            )
            .await?;

            let runtime = Runtime {
                sprite: Rc::new(RwLock::new(runtime)),
                global: global.clone(),
            };

            sprites.push(Sprite::new(runtime, target).await?);
        }

        Ok(Self { sprites })
    }

    pub async fn continue_(&self) {
        // TODO use FuturesUnordered
        for sprite in &self.sprites {
            sprite.continue_().await;
        }
    }

    pub async fn pause(&self) {
        for sprite in &self.sprites {
            sprite.pause().await;
        }
    }

    pub fn step(&self) {
        for sprite in &self.sprites {
            sprite.step();
        }
    }

    pub fn sprites(&self) -> &[Sprite] {
        &self.sprites
    }
}
