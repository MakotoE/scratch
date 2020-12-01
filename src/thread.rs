use super::*;
use crate::blocks::{block_tree, Block, BlockInfo, BlockInputs, Next};
use crate::runtime::Runtime;
use crate::savefile::BlockID;

#[derive(Debug)]
pub struct Thread {
    blocks: HashMap<BlockID, Box<dyn Block>>,
    curr_block: BlockID,
    loop_stack: Vec<BlockID>,
    done: bool,
}

impl Thread {
    pub fn start(
        hat: BlockID,
        runtime: Runtime,
        file_blocks: &HashMap<BlockID, savefile::Block>,
    ) -> Result<Self> {
        let (_, blocks) = match block_tree(hat, runtime.clone(), file_blocks) {
            Ok(b) => b,
            Err(e) => return Err(ErrorKind::Initialization(Box::new(e)).into()),
        };
        Ok(Thread {
            blocks,
            curr_block: hat,
            loop_stack: Vec::new(),
            done: false,
        })
    }

    pub async fn step(&mut self) -> Result<()> {
        if self.done {
            return Ok(());
        }

        let block = self
            .blocks
            .get_mut(&self.curr_block)
            .expect(&format!("curr_block not found: {}", self.curr_block));
        let execute_result = block.execute().await;
        match execute_result {
            Next::None => match self.loop_stack.pop() {
                None => {
                    self.done = true;
                    return Ok(());
                }
                Some(b) => self.curr_block = b,
            },
            Next::Err(e) => {
                return Err(ErrorKind::Block(
                    block.block_info().name,
                    block.block_info().id,
                    Box::new(e),
                )
                .into());
            }
            Next::Continue(b) => self.curr_block = b,
            Next::Loop(b) => {
                self.loop_stack.push(self.curr_block.clone());
                self.curr_block = b;
            }
        }

        Ok(())
    }

    pub fn block_inputs(&self) -> BlockInputs {
        self.blocks
            .get(&self.curr_block)
            .expect(&format!("curr_block not found: {}", &self.curr_block))
            .block_inputs()
    }

    pub fn block_info(&self) -> BlockInfo {
        self.blocks
            .get(&self.curr_block)
            .expect(&format!("curr_block not found: {}", &self.curr_block))
            .block_info()
    }
}
