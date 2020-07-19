use super::*;
use block::{new_block, Block, Runtime};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Sprite<'runtime> {
    threads: Vec<Thread<'runtime>>,
}

impl<'runtime> Sprite<'runtime> {
    pub fn new(
        runtime: &'runtime Runtime,
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
pub struct Thread<'runtime> {
    runtime: &'runtime Runtime,
    hat: Box<dyn Block<'runtime> + 'runtime>,
}

impl<'runtime> Thread<'runtime> {
    pub fn new(runtime: &'runtime Runtime, hat: Box<dyn Block<'runtime> + 'runtime>) -> Self {
        Self { runtime, hat }
    }
}
