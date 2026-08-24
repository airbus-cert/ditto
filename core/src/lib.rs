extern crate base64;
extern crate rand;
extern crate tree_sitter;
extern crate tree_sitter_powershell;

use crate::error::{DittoResult, Error};
use crate::pass::Replace;
use crate::tree::Node;
use std::fmt::Display;

#[macro_use]
pub mod tree;
pub mod error;
pub mod impostor;
pub mod js;
pub mod name;
pub mod pass;
pub mod ps;
pub mod utils;

#[derive(Clone, Copy)]
pub enum Lang {
    JavaScript,
    PowerShell,
}

impl Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lang::JavaScript => write!(f, "JavaScript"),
            Lang::PowerShell => write!(f, "PowerShell"),
        }
    }
}

pub struct Mutation {
    pub start_byte: usize,
    pub end_byte: usize,
    pub new: String,
}

impl Replace for Mutation {
    fn start_byte(&self) -> usize {
        self.start_byte
    }

    fn end_byte(&self) -> usize {
        self.end_byte
    }

    fn new_source(&self) -> String {
        self.new.clone()
    }
}

impl Mutation {
    pub fn substitute(node: &Node, new: String) -> DittoResult<Self> {
        Ok(Self {
            start_byte: node.start(),
            end_byte: node.end(),
            new,
        })
    }

    pub fn global_statement(node: &Node, new: String) -> DittoResult<Self> {
        let parent_program = node.get_parent_of_types(vec!["program"]);
        let program = if node.kind() == "program" {
            Some(node)
        } else if parent_program.is_some() {
            parent_program.as_ref()
        } else {
            None
        };

        if let Some(program) = program {
            if let Some(first_child) = program.child(0) {
                return Ok(Self {
                    start_byte: first_child.start(),
                    end_byte: first_child.start(),
                    new,
                });
            }
        }
        Err(Error::NodeNotFound(String::from("program")))
    }
}
