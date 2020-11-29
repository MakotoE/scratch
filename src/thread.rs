use super::*;
use crate::blocks::{Block, BlockInfo, BlockInputs, Next};

#[derive(Debug)]
pub struct Thread {
    hat: Rc<RefCell<Box<dyn Block>>>,
    curr_block: Rc<RefCell<Box<dyn Block>>>,
    loop_start: Vec<Rc<RefCell<Box<dyn Block>>>>,
    done: bool,
}

impl Thread {
    pub fn start(hat: Box<dyn Block>) -> Self {
        let hat = Rc::new(RefCell::new(hat));
        Thread {
            hat: hat.clone(),
            curr_block: hat,
            loop_start: Vec::new(),
            done: false,
        }
    }

    pub async fn step(&mut self) -> Result<()> {
        if self.done {
            return Ok(());
        }

        let execute_result = self.curr_block.borrow_mut().execute().await;
        match execute_result {
            Next::None => match self.loop_start.pop() {
                None => {
                    self.done = true;
                    return Ok(());
                }
                Some(b) => self.curr_block = b,
            },
            Next::Err(e) => {
                let block = self.curr_block.borrow();
                return Err(ErrorKind::Block(
                    block.block_info().name,
                    block.block_info().id,
                    Box::new(e),
                )
                .into());
            }
            Next::Continue(b) => self.curr_block = b,
            Next::Loop(b) => {
                self.loop_start.push(self.curr_block.clone());
                self.curr_block = b;
            }
        }

        Ok(())
    }

    pub fn block_inputs(&self) -> BlockInputs {
        self.hat.borrow().block_inputs()
    }

    pub fn block_info(&self) -> BlockInfo {
        self.curr_block.borrow().block_info()
    }
}
