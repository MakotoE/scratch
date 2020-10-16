use super::*;
use blocks::*;
use runtime::{Coordinate, SpriteRuntime};
use thread::Thread;

#[derive(Debug)]
pub struct Sprite {
    threads: Vec<Thread>,
    closure: ClosureRef,
    request_animation_frame_id: Rc<RefCell<i32>>,
    runtime: Rc<RwLock<SpriteRuntime>>,
}

impl Sprite {
    pub fn new(
        mut runtime: SpriteRuntime,
        target: &savefile::Target,
        start_state: interface::VMState,
    ) -> Result<Self> {
        runtime.set_position(&Coordinate::new(target.x, target.y));

        let runtime_ref = Rc::new(RwLock::new(runtime));
        let mut threads: Vec<Thread> = Vec::new();

        for (thread_id, hat_id) in find_hats(&target.blocks).iter().enumerate() {
            let block = match block_tree(hat_id.to_string(), runtime_ref.clone(), &target.blocks) {
                Ok(b) => b,
                Err(e) => return Err(ErrorKind::Initialization(Box::new(e)).into()),
            };
            let thread = Thread::start(block, runtime_ref.clone(), start_state, thread_id);
            threads.push(thread);
        }

        let closure: ClosureRef = Rc::new(RefCell::new(None));
        let request_animation_frame_id = Rc::new(RefCell::new(0));
        Sprite::start_redraw_loop(
            closure.clone(),
            request_animation_frame_id.clone(),
            runtime_ref.clone(),
        )?;

        Ok(Self {
            threads,
            closure,
            request_animation_frame_id,
            runtime: runtime_ref,
        })
    }

    fn start_redraw_loop(
        closure: ClosureRef,
        request_animation_frame_id: Rc<RefCell<i32>>,
        runtime: Rc<RwLock<SpriteRuntime>>,
    ) -> Result<()> {
        let window = web_sys::window().unwrap();
        let closure_clone = closure.clone();
        let request_animation_frame_id_clone = request_animation_frame_id.clone();
        *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let runtime_clone = runtime.clone();
            let closure_clone = closure_clone.clone();
            let request_animation_frame_id_clone = request_animation_frame_id_clone.clone();
            let window = window.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = runtime_clone.write().await.redraw() {
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
        Ok(())
    }

    pub async fn continue_(&mut self) {
        self.runtime.write().await.set_draw_debug_info(false);
        for thread in &mut self.threads {
            thread.continue_().await;
        }
    }

    pub async fn pause(&mut self) {
        for thread in &mut self.threads {
            thread.pause().await;
        }
        let mut runtime = self.runtime.write().await;
        runtime.set_draw_debug_info(true);
        runtime.redraw().unwrap_or_else(|e| log::error!("{}", e));
    }

    pub fn step(&self) {
        for thread in &self.threads {
            thread.step();
        }
    }
}

impl Drop for Sprite {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap()
            .cancel_animation_frame(*self.request_animation_frame_id.borrow())
            .unwrap();
    }
}

type ClosureRef = Rc<RefCell<Option<Closure<dyn Fn()>>>>;

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
