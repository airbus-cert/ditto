use crate::error::DittoResult;
use crate::tree::{Node, Visitor};

pub struct NodeCounter {
    count: usize,
    kind: String,
}

impl NodeCounter {
    pub fn new(kind: String) -> Self {
        Self { count: 0, kind }
    }

    pub fn get_count(&self) -> usize {
        self.count
    }
}

impl Visitor for NodeCounter {
    fn enter(&mut self, node: &Node) -> DittoResult<bool> {
        if node.kind() == self.kind {
            self.count += 1;
        }
        Ok(true)
    }
    fn leave(&mut self, _node: &Node) -> DittoResult<bool> {
        Ok(true)
    }
}
