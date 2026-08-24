use crate::{error::DittoResult, utils::NodeCounter};
use std::str::Utf8Error;
use tree_sitter::{Node as TreeNode, Tree as TreeSitter};
use tree_sitter_traversal2::{Order, traverse};

pub struct ChildNodeIterator<'a> {
    queue: Vec<Node<'a>>,
    kind: String,
}

impl<'a> Iterator for ChildNodeIterator<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.queue.pop() {
            self.queue.extend(current.iter());
            if current.kind() == self.kind {
                return Some(current);
            }
        }

        None
    }
}

/// A node view use to explore the tree
/// without mutability
pub struct Node<'a> {
    /// The inner tree-sitter node
    node: TreeNode<'a>,
    /// Source reference
    source: &'a [u8],
}

pub trait Visitor {
    fn enter(&mut self, node: &Node) -> DittoResult<bool>;
    fn leave(&mut self, node: &Node) -> DittoResult<bool>;
}

/// Two nodes are equals if they have the same node id
impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.node.id() == other.node.id()
    }
}

impl<'a> Node<'a> {
    pub fn new(node: TreeNode<'a>, source: &'a [u8]) -> Self {
        Self { node, source }
    }

    pub fn child(&self, index: usize) -> Option<Node<'a>> {
        let mut current = 0;
        for i in 0..self.node.child_count() {
            let child = self.node.child(i);
            if child == None {
                break;
            }

            // ignore extra node when requesting child at particular index
            /*if child.unwrap().is_extra() {
                continue;
            }*/

            if current == index {
                return Some(Node::new(child.unwrap(), self.source));
            }

            current += 1;
        }

        None
    }

    pub fn named_child(&self, index: &str) -> Option<Node<'a>> {
        self.node
            .child_by_field_name(index)
            .map(|node| Node::new(node, self.source))
    }

    pub fn iter(&self) -> NodeIterator<'a> {
        NodeIterator::new(Self::new(self.node, self.source), 0, None, 1)
    }

    pub fn range(
        &self,
        start: Option<usize>,
        end: Option<usize>,
        gap: Option<usize>,
    ) -> NodeIterator<'a> {
        NodeIterator::new(
            Self::new(self.node, self.source),
            start.unwrap_or(0),
            end,
            gap.unwrap_or(1),
        )
    }

    pub fn is_statement(&self) -> bool {
        self.kind() == "pipeline"
            || self
                .get_parent_of_types(vec!["pipeline"])
                .map(|p| p.text() == self.text() && p.parent().unwrap().kind() == "statement_list")
                .unwrap_or(false)
    }

    pub fn kind(&self) -> &'static str {
        self.node.kind()
    }

    pub fn start(&self) -> usize {
        self.node.start_byte()
    }

    pub fn end(&self) -> usize {
        self.node.end_byte()
    }

    pub fn is_extra(&self) -> bool {
        self.node.is_extra()
    }

    pub fn child_count(&self) -> usize {
        // we have to count only usable node
        let mut result = 0;
        for _ in self.iter() {
            result += 1;
        }
        result
    }

    pub fn text(&self) -> Result<&str, Utf8Error> {
        self.node.utf8_text(self.source)
    }

    pub fn parent(&self) -> Option<Node<'a>> {
        self.node.parent().map(|node| Self::new(node, self.source))
    }

    pub fn get_parent_of_types(&self, kinds: Vec<&str>) -> Option<Node<'a>> {
        let mut current = self.parent();
        loop {
            if let Some(current_node) = current {
                if kinds.contains(&current_node.kind()) {
                    return Some(current_node);
                }
                current = current_node.parent();
            } else {
                return None;
            }
        }
    }

    pub fn count_childs_of_type(&self, kind: &str) -> usize {
        let mut node_counter = NodeCounter::new(String::from(kind));
        match self.visit(&mut node_counter) {
            Ok(_) => node_counter.get_count(),
            _ => 0,
        }
    }

    pub fn get_childs_of_type(&self, kind: &str) -> ChildNodeIterator<'_> {
        ChildNodeIterator {
            queue: self.iter().collect(),
            kind: String::from(kind),
        }
    }

    fn visit(&self, visitor: &mut impl Visitor) -> DittoResult<()> {
        let mut is_visiting = true;
        // Stack use to call 'leave' method when all children are handled
        let mut stack: Vec<(TreeNode, usize, bool)> = vec![];

        for node in traverse(self.node.walk(), Order::Pre) {
            stack.push((node, node.child_count(), is_visiting));
            if is_visiting {
                is_visiting = is_visiting && visitor.enter(&Node::new(node, self.source))?;
            }

            // clean stack
            loop {
                let head = stack.last();
                if head.is_none() {
                    break;
                }

                let head_element = head.unwrap();

                // Do i have handle all children
                // if not continue to work on children
                if head_element.1 != 0 {
                    break;
                }

                // Reset the is_visiting status before leaving
                // If we mutate when entering a node, we also leave the node
                is_visiting = head_element.2;
                if is_visiting {
                    visitor.leave(&Node::new(head_element.0, self.source))?;
                }

                stack.pop();

                // decrement number of children handled
                if let Some(l) = stack.last_mut() {
                    l.1 = l.1 - 1;
                }
            }
        }
        Ok(())
    }
}

pub struct NodeIterator<'a> {
    inner: Node<'a>,
    index: usize,
    end: Option<usize>,
    gap: usize,
}

impl<'a> NodeIterator<'a> {
    fn new(node: Node<'a>, start: usize, end: Option<usize>, gap: usize) -> Self {
        Self {
            inner: node,
            index: start,
            end,
            gap,
        }
    }
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(end) = self.end {
            if self.index >= end {
                return None;
            }
        }

        match self.inner.child(self.index) {
            Some(node) => {
                self.index += self.gap;
                Some(node)
            }
            None => None,
        }
    }
}

pub struct Tree<'a> {
    tree_sitter: TreeSitter,
    pub source: &'a [u8],
}

impl<'a> Tree<'a> {
    pub fn new(source: &'a [u8], tree_sitter: TreeSitter) -> Self {
        Self {
            tree_sitter,
            source,
        }
    }

    pub fn visit(&self, visitor: &mut (impl Visitor + Sized)) -> DittoResult<()> {
        let node = Node::new(self.tree_sitter.root_node(), self.source);
        node.visit(visitor)
    }
}
