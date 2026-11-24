use crate::Lang;
use crate::Lang::JavaScript;
use crate::Mutation;
use crate::js::utils::switch::*;
use crate::pass::Pass;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, PartialEq)]
pub enum TransformTarget {
    ExpressionStatement,
    ReturnStatement,
    VariableDeclaration,
    ForInStatement,
    ForOfStatement,
    DebuggerStatement,
}

impl TransformTarget {
    pub fn node_kind(&self) -> &'static str {
        match self {
            Self::ExpressionStatement => "expression_statement",
            Self::ReturnStatement => "return_statement",
            Self::VariableDeclaration => "variable_declaration",
            Self::ForInStatement => "for_in_statement",
            Self::ForOfStatement => "for_of_statement",
            Self::DebuggerStatement => "debugger_statement",
        }
    }
}

#[derive(Debug)]
pub struct ExpressionToSwitchCase {
    pub targets: Vec<TransformTarget>,
}

impl Default for ExpressionToSwitchCase {
    fn default() -> Self {
        let mut targets = vec![];
        for target in TransformTarget::iter() {
            targets.push(target);
        }
        Self::new(targets)
    }
}

impl ExpressionToSwitchCase {
    pub fn new(targets: Vec<TransformTarget>) -> Self {
        Self { targets }
    }
}

impl Pass for ExpressionToSwitchCase {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        replace_expression(node, &self.targets, true)
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

#[derive(Debug)]
pub struct ExpressionToIfElse {
    pub targets: Vec<TransformTarget>,
}

impl Default for ExpressionToIfElse {
    fn default() -> Self {
        let mut targets = vec![];
        for target in TransformTarget::iter() {
            targets.push(target);
        }
        Self::new(targets)
    }
}

impl ExpressionToIfElse {
    pub fn new(targets: Vec<TransformTarget>) -> Self {
        Self { targets }
    }
}

impl Pass for ExpressionToIfElse {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        replace_expression(node, &self.targets, false)
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

fn replace_expression(
    node: &crate::tree::Node,
    targets: &Vec<TransformTarget>,
    is_switch_case: bool,
) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
    let kind = node.kind();
    let know_kind = targets.iter().any(|target| target.node_kind() == kind);
    if !know_kind {
        return Ok(None);
    }

    if kind == "variable_declaration" {
        if let Ok(text) = node.text() {
            let trimmed = text.trim_start();
            if !trimmed.starts_with("var") {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }
    }

    if kind == "expression_statement" {
        if let Some(child) = node.child(0) {
            if child.kind() == "string" {
                return Ok(None);
            }
        }
    }

    let mut rng = rand::rng();
    let statement_text = match node.text() {
        Ok(text) => text,
        Err(_) => return Ok(None),
    };

    let replacement = if is_switch_case {
        make_switch_replacement(statement_text, &mut rng)
    } else {
        make_if_replacement(statement_text, &mut rng)
    };
    Ok(Some(vec![Mutation::substitute(node, replacement)?]))
}
