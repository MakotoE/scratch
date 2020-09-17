use super::*;
use async_trait::async_trait;
use blocks::*;
use controller::DebugController;
use runtime::{Coordinate, SpriteRuntime};

#[derive(Debug, Default)]
pub struct Sprite {
    controllers: Vec<Rc<DebugController>>,
    closure: ClosureRef,
    request_animation_frame_id: i32,
}

impl Sprite {
    pub fn new(
        mut runtime: SpriteRuntime,
        target: &savefile::Target,
        start_state: page::VMState,
    ) -> Result<Self> {
        runtime.set_position(&Coordinate::new(target.x, target.y));

        let runtime_ref = Rc::new(RwLock::new(runtime));
        let mut controllers: Vec<Rc<DebugController>> = Vec::new();

        for hat_id in find_hats(&target.blocks) {
            let controller = Rc::new(DebugController::new());
            controllers.push(controller.clone());

            let block = new_block(hat_id, runtime_ref.clone(), &target.blocks)
                .map_err(|e| ErrorKind::Initialization(Box::new(e)))?;
            let thread = Thread::new(block, runtime_ref.clone(), controller.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match start_state {
                    page::VMState::Paused => controller.pause().await,
                    page::VMState::Running => controller.continue_(controller::Speed::Normal).await,
                }
                match thread.start().await {
                    Ok(_) => {}
                    Err(e) => log::error!("{}", e),
                }
            });
        }

        let cb_ref: ClosureRef = Rc::new(RefCell::new(None));
        let cb_ref_clone = cb_ref.clone();
        *cb_ref.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let runtime_arc = runtime_ref.clone();
            wasm_bindgen_futures::spawn_local(async move {
                runtime_arc.write().await.redraw().unwrap();
            });

            let cb = cb_ref_clone.borrow();
            let f = cb.as_ref().unwrap();
            web_sys::window()
                .unwrap()
                .request_animation_frame(f.as_ref().unchecked_ref())
                .unwrap();
        }) as Box<dyn Fn()>));
        let cb = cb_ref.borrow();
        let f = cb.as_ref().unwrap();
        let request_animation_frame_id = web_sys::window()
            .unwrap()
            .request_animation_frame(f.as_ref().unchecked_ref())?;

        Ok(Self {
            controllers,
            closure: cb_ref.clone(),
            request_animation_frame_id,
        })
    }

    pub async fn continue_(&mut self, speed: controller::Speed) {
        for c in &self.controllers {
            c.continue_(speed).await;
        }
    }

    pub async fn pause(&mut self) {
        for c in &self.controllers {
            c.pause().await;
        }
    }

    pub fn step(&self) {
        for c in &self.controllers {
            c.step();
        }
    }
}

impl Drop for Sprite {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap()
            .cancel_animation_frame(self.request_animation_frame_id)
            .unwrap();
    }
}

type ClosureRef = Rc<RefCell<Option<Closure<dyn Fn()>>>>;

fn find_hats(block_infos: &HashMap<String, savefile::Block>) -> Vec<&str> {
    let mut hats: Vec<&str> = Vec::new();
    for (id, block_info) in block_infos {
        if block_info.top_level {
            hats.push(id);
        }
    }

    hats
}

#[derive(Debug)]
pub struct Thread {
    hat: Rc<RefCell<Box<dyn Block>>>,
    runtime: Rc<RwLock<SpriteRuntime>>,
    controller: Rc<DebugController>,
}

impl Thread {
    pub fn new(
        hat: Box<dyn Block>,
        runtime: Rc<RwLock<SpriteRuntime>>,
        controller: Rc<DebugController>,
    ) -> Self {
        Self {
            hat: Rc::new(RefCell::new(hat)),
            runtime,
            controller,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let debug_info = if self.controller.display_debug().await {
            DebugInfo {
                show: true,
                block_name: self.hat.borrow().block_name().to_string(),
                block_id: self.hat.borrow().id().to_string(),
            }
        } else {
            DebugInfo::default()
        };
        self.runtime.write().await.set_debug_info(&debug_info);

        let mut curr_block = self.hat.clone();
        let mut loop_start: Vec<Rc<RefCell<Box<dyn Block>>>> = Vec::new();

        loop {
            let debug_info = if self.controller.display_debug().await {
                DebugInfo {
                    show: true,
                    block_name: curr_block.borrow().block_name().to_string(),
                    block_id: curr_block.borrow().id().to_string(),
                }
            } else {
                DebugInfo::default()
            };
            self.runtime.write().await.set_debug_info(&debug_info);

            let execute_result = curr_block.borrow_mut().execute().await;
            match execute_result {
                Next::None => match loop_start.pop() {
                    None => break,
                    Some(b) => curr_block = b,
                },
                Next::Err(e) => {
                    let block = curr_block.borrow();
                    return Err(ErrorKind::Block(
                        block.block_name(),
                        block.id().to_string(),
                        Box::new(e),
                    )
                    .into());
                }
                Next::Continue(b) => curr_block = b,
                Next::Loop(b) => {
                    loop_start.push(curr_block.clone());
                    curr_block = b;
                }
            }
            self.controller.wait().await;
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct DebugInfo {
    pub show: bool,
    pub block_id: String,
    pub block_name: String,
}

#[derive(Debug)]
pub struct DummyBlock {
    next: Rc<RefCell<Box<dyn Block>>>,
}

#[async_trait(?Send)]
impl Block for DummyBlock {
    fn block_name(&self) -> &'static str {
        "DummyBlock"
    }

    fn id(&self) -> &str {
        ""
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn execute(&mut self) -> Next {
        Next::Continue(self.next.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod thread_iterator {
        use super::*;

        #[derive(Debug)]
        struct LastBlock {}

        impl Block for LastBlock {
            fn block_name(&self) -> &'static str {
                "LastBlock"
            }

            fn id(&self) -> &str {
                ""
            }

            fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}
        }
    }
}
