use super::*;
use block::{new_block, Block, Runtime};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Sprite<'r> {
    threads: Vec<Thread<'r>>,
}

impl<'r> Sprite<'r> {
    pub fn new(
        runtime: &'r Runtime,
        block_infos: &HashMap<String, savefile::Block>,
    ) -> Result<Self> {
        let mut threads: Vec<Thread> = Vec::new();
        for hat_id in find_hats(block_infos) {
            threads.push(Thread::new(
                runtime,
                new_block(hat_id, runtime, block_infos)?,
            ));
        }
        Ok(Self { threads })
    }
}

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
pub struct Thread<'r> {
    runtime: &'r Runtime,
    hat: Box<dyn Block<'r> + 'r>,
}

impl<'r> Thread<'r> {
    pub fn new(runtime: &'r Runtime, hat: Box<dyn Block<'r> + 'r>) -> Self {
        Self { runtime, hat }
    }
}
