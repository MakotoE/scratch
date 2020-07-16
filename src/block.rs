use super::*;

use std::collections::HashMap;

pub trait Block<'runtime>: std::fmt::Debug {
    fn set_arg(&mut self, key: String, function: Function);
    fn set_next(&mut self, key: String, block: Box<dyn Block<'runtime> + 'runtime>);
}

#[derive(Debug)]
pub struct Runtime {}

#[derive(Debug)]
pub struct Function {
    pub args: HashMap<String, Function>,
}

#[derive(Debug)]
pub struct MoveSteps<'runtime> {
    pub id: String,
    pub runtime: &'runtime Runtime,
    pub args: HashMap<String, Function>,
    pub next: HashMap<String, Box<dyn Block<'runtime> + 'runtime>>,
}

impl<'runtime> MoveSteps<'runtime> {
    fn new(id: &str, runtime: &'runtime Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            args: HashMap::new(),
            next: HashMap::new(),
        }
    }
}

impl<'runtime> Block<'runtime> for MoveSteps<'runtime> {
    fn set_arg(&mut self, key: String, function: Function) {
        self.args.insert(key, function);
    }

    fn set_next(&mut self, key: String, block: Box<dyn Block<'runtime> + 'runtime>) {
        self.next.insert(key, block);
    }
}

#[derive(Debug)]
pub struct Thread<'runtime> {
    runtime: &'runtime Runtime,
    hat: Option<Box<dyn Block<'runtime> + 'runtime>>,
}

impl<'runtime> Thread<'runtime> {
    pub fn new(runtime: &'runtime Runtime, block_infos: &HashMap<String, savefile::Block>) -> Self {
        let hat = match find_hat(block_infos) {
            Some(hat_id) => Some(new_block(hat_id, runtime, block_infos)),
            None => None,
        };
        Self { runtime, hat }
    }
}

fn find_hat(block_infos: &HashMap<String, savefile::Block>) -> Option<&str> {
    for (id, block_info) in block_infos {
        if block_info.top_level {
            return Some(id);
        }
    }

    None
}

fn new_block<'runtime>(id: &str, runtime: &'runtime Runtime, block_infos: &HashMap<String, savefile::Block>) -> Box<dyn Block<'runtime> + 'runtime> {
    let block_info = block_infos.get(id).unwrap();
    let mut block = get_block(id, runtime, &block_info);
    if let Some(next_id) = &block_info.next {
        block.set_next("next".to_string(), new_block(next_id, runtime, block_infos));
    }
    block
}

fn get_block<'runtime>(
    id: &str,
    runtime: &'runtime Runtime,
    block_info: &savefile::Block,
) -> Box<dyn Block<'runtime> + 'runtime> {
    let block = match block_info.opcode.as_str() {
        _ => MoveSteps::new(id, runtime),
        // _ => unreachable!(),
    };
    Box::new(block)
}
