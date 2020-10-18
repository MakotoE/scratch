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

        let runtime = runtime::SpriteRuntime::new(
            context,
            &scratch_file.project.targets[1].costumes,
            &scratch_file.images,
        )
        .await?;

        let runtime = Runtime {
            sprite: Rc::new(RwLock::new(runtime)),
            global,
        };

        let sprite = Sprite::new(runtime, &scratch_file.project.targets[1]).await?;

        Ok(Self {
            sprites: vec![sprite],
        })
    }

    pub async fn continue_(&self) {
        // TODO https://rust-lang.github.io/async-book/06_multiple_futures/02_join.html
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
