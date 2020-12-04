use super::*;
use crate::blocks::{block_tree, Block, BlockInfo, BlockInputsPartial, Next};
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
        let block_inputs = self
            .blocks
            .get(&self.curr_block)
            .expect(&format!("curr_block not found: {}", &self.curr_block))
            .block_inputs();
        BlockInputs::new(block_inputs, &self.blocks)
    }

    pub fn block_info(&self) -> BlockInfo {
        self.blocks
            .get(&self.curr_block)
            .expect(&format!("curr_block not found: {}", &self.curr_block))
            .block_info()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlockInputs {
    pub info: BlockInfo,
    pub fields: HashMap<&'static str, String>,
    pub inputs: HashMap<&'static str, BlockInputs>,
    pub stacks: HashMap<&'static str, BlockInputs>,
}

impl BlockInputs {
    fn new(
        mut block_inputs: BlockInputsPartial,
        blocks: &HashMap<BlockID, Box<dyn Block>>,
    ) -> Self {
        Self {
            info: block_inputs.info,
            fields: block_inputs.fields,
            inputs: block_inputs
                .inputs
                .drain()
                .map(|(id, inputs)| {
                    // Input blocks should have empty stacks
                    assert_eq!(inputs.stacks.len(), 0);
                    (id, BlockInputs::new(inputs, blocks))
                })
                .collect(),
            stacks: block_inputs
                .stacks
                .iter()
                .map(|(id, block_id)| {
                    let block_inputs = blocks
                        .get(block_id)
                        .expect(&format!("block_id not found: {}", block_id))
                        .block_inputs();
                    (*id, BlockInputs::new(block_inputs, blocks))
                })
                .collect(),
        }
    }
}
