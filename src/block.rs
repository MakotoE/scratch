use super::*;

use petgraph::prelude::*;
use std::collections::HashMap;

pub trait Block: std::fmt::Debug {
    fn set_arg(&mut self, key: String, function: Function);
}

#[derive(Debug)]
pub struct Runtime {}

#[derive(Debug)]
pub struct Function {}

#[derive(Debug)]
pub struct MoveSteps<'runtime> {
    pub id: String,
    pub runtime: &'runtime Runtime,
    pub args: HashMap<String, Function>,
}

impl<'runtime> MoveSteps<'runtime> {
    fn new(id: &str, runtime: &'runtime Runtime) -> Self {
        Self {
            id: id.to_string(),
            runtime,
            args: HashMap::new(),
        }
    }
}

impl<'runtime> Block for MoveSteps<'runtime> {
    fn set_arg(&mut self, key: String, function: Function) {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct Thread<'runtime> {
    runtime: &'runtime Runtime,
    graph: DiGraph<Box<dyn Block + 'runtime>, usize>,
}

impl<'runtime> Thread<'runtime> {
    pub fn new(runtime: &'runtime Runtime) -> Self {
        Self {
            runtime,
            graph: DiGraph::new(),
        }
    }

    pub fn add_blocks(&mut self, blocks: &[(&str, &savefile::Block)]) {
        let mut block_indicies: HashMap<&str, NodeIndex<petgraph::graph::DefaultIx>> =
            HashMap::new();
        for &(id, block) in blocks {
            let node = self.graph.add_node(new_block(id, self.runtime, block));
            block_indicies.insert(id, node);
        }

        for &(id, block) in blocks {
            let curr_node = *block_indicies.get(id).unwrap();
            if let Some(parent_id) = &block.parent {
                if let Some(&parent_node) = block_indicies.get(parent_id.as_str()) {
                    self.graph.add_edge(parent_node, curr_node, 1);
                } else {
                    eprintln!("parent_id is invalid in block {}", id);
                }
            }
        }
    }
}

fn new_block<'runtime>(
    id: &str,
    runtime: &'runtime Runtime,
    block_info: &savefile::Block,
) -> Box<dyn Block + 'runtime> {
    let block = match block_info.opcode.as_str() {
        _ => MoveSteps::new(id, runtime),
        // _ => unreachable!(),
    };
    Box::new(block)
}
