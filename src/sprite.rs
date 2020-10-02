use super::*;
use async_trait::async_trait;
use blocks::*;
use controller::DebugController;
use maplit::hashmap;
use runtime::{Coordinate, SpriteRuntime};

#[derive(Debug, Default)]
pub struct Sprite {
    controllers: Vec<Rc<DebugController>>,
    closure: ClosureRef,
    request_animation_frame_id: Rc<RefCell<i32>>,
}

impl Sprite {
    pub fn new(
        mut runtime: SpriteRuntime,
        target: &savefile::Target,
        start_state: vm::VMState,
    ) -> Result<Self> {
        runtime.set_position(&Coordinate::new(target.x, target.y));

        let runtime_ref = Rc::new(RwLock::new(runtime));
        let mut controllers: Vec<Rc<DebugController>> = Vec::new();

        for hat_id in find_hats(&target.blocks) {
            let controller = Rc::new(DebugController::new());
            controllers.push(controller.clone());

            let block = block_tree(hat_id, runtime_ref.clone(), &target.blocks)
                .map_err(|e| ErrorKind::Initialization(Box::new(e)))?;
            let thread = Thread::new(block, runtime_ref.clone(), controller.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match start_state {
                    vm::VMState::Paused => controller.pause().await,
                    vm::VMState::Running => controller.continue_(controller::Speed::Normal).await,
                }
                match thread.start().await {
                    Ok(_) => {}
                    Err(e) => log::error!("{}", e),
                }
            });
        }

        let cb_ref: ClosureRef = Rc::new(RefCell::new(None));
        let cb_ref_clone = cb_ref.clone();
        let request_animation_frame_id = Rc::new(RefCell::new(0));
        let request_animation_frame_id_clone = request_animation_frame_id.clone();
        *cb_ref.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let runtime_arc = runtime_ref.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match runtime_arc.write().await.redraw() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("error occurred on redraw: {}", e);
                        return;
                    }
                };
            });

            let cb = cb_ref_clone.borrow();
            let f = cb.as_ref().unwrap();
            *request_animation_frame_id_clone.borrow_mut() = web_sys::window()
                .unwrap()
                .request_animation_frame(f.as_ref().unchecked_ref())
                .unwrap();
        }) as Box<dyn Fn()>));
        let cb = cb_ref.borrow();
        let f = cb.as_ref().unwrap();
        *request_animation_frame_id.borrow_mut() = web_sys::window()
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
            let block = self.hat.borrow();
            DebugInfo {
                show: true,
                block_name: block.block_info().name.to_string(),
                block_id: block.block_info().id,
            }
        } else {
            DebugInfo::default()
        };
        self.runtime.write().await.set_debug_info(&debug_info);

        let mut curr_block = self.hat.clone();
        let mut loop_start: Vec<Rc<RefCell<Box<dyn Block>>>> = Vec::new();

        loop {
            let debug_info = if self.controller.display_debug().await {
                let block = curr_block.borrow();
                DebugInfo {
                    show: true,
                    block_name: block.block_info().name.to_string(),
                    block_id: block.block_info().id.to_string(),
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
                        block.block_info().name,
                        block.block_info().id,
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
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "DummyBlock",
            id: String::new(),
        }
    }

    fn block_inputs(&self) -> BlockInputs {
        BlockInputs {
            info: self.block_info(),
            fields: HashMap::new(),
            inputs: HashMap::new(),
            stacks: hashmap! {"next" => self.next.borrow().block_inputs()},
        }
    }

    fn set_input(&mut self, _: &str, _: Box<dyn Block>) {}

    async fn execute(&mut self) -> Next {
        Next::Continue(self.next.clone())
    }
}
