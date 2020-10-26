use super::*;
use runtime::{Global, Runtime, SpriteRuntime};
use savefile::ScratchFile;
use sprite::Sprite;

pub struct VM {
    sprites: Vec<Sprite>,
    runtimes: Vec<Rc<RwLock<SpriteRuntime>>>,
    context: web_sys::CanvasRenderingContext2d,
    #[allow(dead_code)]
    closure: ClosureRef,
    request_animation_frame_id: Rc<RefCell<i32>>,
}

impl VM {
    pub async fn new(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: &ScratchFile,
    ) -> Result<Self> {
        let global = Global::new(&scratch_file.project.targets[0].variables);

        let mut sprites: Vec<Sprite> = Vec::with_capacity(scratch_file.project.targets.len());
        let mut runtimes: Vec<Rc<RwLock<SpriteRuntime>>> =
            Vec::with_capacity(scratch_file.project.targets.len());
        for target in &scratch_file.project.targets {
            let runtime =
                runtime::SpriteRuntime::new(&target.costumes, &scratch_file.images).await?;

            let runtime = Runtime {
                sprite: Rc::new(RwLock::new(runtime)),
                global: global.clone(),
            };

            runtimes.push(runtime.sprite.clone());
            sprites.push(Sprite::new(runtime, target).await?);
        }

        let request_animation_frame_id = Rc::new(RefCell::new(0));
        let closure = VM::start_redraw_loop(
            request_animation_frame_id.clone(),
            context.clone(),
            runtimes.clone(),
        )?;

        Ok(Self {
            sprites,
            runtimes,
            context,
            closure,
            request_animation_frame_id,
        })
    }

    fn start_redraw_loop(
        request_animation_frame_id: Rc<RefCell<i32>>,
        context: web_sys::CanvasRenderingContext2d,
        runtimes: Vec<Rc<RwLock<SpriteRuntime>>>,
    ) -> Result<ClosureRef> {
        let closure: ClosureRef = Rc::new(RefCell::new(None));
        let window = web_sys::window().unwrap();
        let closure_clone = closure.clone();
        let request_animation_frame_id_clone = request_animation_frame_id.clone();
        *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let closure_clone = closure_clone.clone();
            let request_animation_frame_id_clone = request_animation_frame_id_clone.clone();
            let window = window.clone();
            let runtimes = runtimes.clone();
            let context = context.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = VM::redraw(&runtimes, &context).await {
                    log::error!("error occurred on redraw: {}", e);
                    return;
                }

                let cb = closure_clone.borrow();
                let f = cb.as_ref().unwrap();
                *request_animation_frame_id_clone.borrow_mut() = window
                    .request_animation_frame(f.as_ref().unchecked_ref())
                    .unwrap();
            });
        }) as Box<dyn Fn()>));

        let closure_clone = closure.clone();
        let cb = closure_clone.borrow();
        let f = cb.as_ref().unwrap();
        *request_animation_frame_id.borrow_mut() = web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())?;
        Ok(closure)
    }

    async fn redraw(
        runtimes: &[Rc<RwLock<SpriteRuntime>>],
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<()> {
        let mut need_redraw = false;
        for runtime in runtimes {
            if runtime.read().await.need_redraw() {
                need_redraw = true;
                break;
            }
        }

        if !need_redraw {
            return Ok(());
        }

        context.reset_transform().unwrap();
        context.scale(2.0, 2.0).unwrap();

        for (i, runtime) in runtimes.iter().enumerate() {
            runtime.write().await.redraw(&context, i == 0)?;
        }
        Ok(())
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
        VM::redraw(&self.runtimes, &self.context)
            .await
            .unwrap_or_else(|e| log::error!("{}", e));
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

impl Drop for VM {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap()
            .cancel_animation_frame(*self.request_animation_frame_id.borrow())
            .unwrap();
    }
}

type ClosureRef = Rc<RefCell<Option<Closure<dyn Fn()>>>>;
