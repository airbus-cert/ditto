use crate::Lang::JavaScript;
use crate::error::Error::ConditionsNotMeet;
use crate::js::utils::global_this::*;
use crate::pass::Pass;
use crate::{Lang, Mutation};
use log::trace;

#[derive(Default, Debug)]
pub struct GlobalThis;

impl Pass for GlobalThis {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "identifier" {
            if let Some(parent) = node.parent() {
                if parent.kind() == "formal_parameters" {
                    return Ok(None);
                }
                if let Some(grandparent) = parent.parent() {
                    if grandparent.kind() == "member_expression" {
                        return Ok(None);
                    }
                }
            }
            if let Ok(name) = node.text() {
                if is_global_this_supported(name) {
                    if let Some(parent) = node.parent() {
                        if parent.kind() == "variable_declarator"
                            || parent.kind() == "assignment_expression"
                        {
                            return Err(ConditionsNotMeet(format!(
                                "You can't use GlobalThis while overwriting the name `{}` in your base code ({})",
                                name,
                                parent.text()?
                            )));
                        }
                    }
                    trace!("Replaced {} with globalThis['{}']", name, name);
                    let replacement = format!("globalThis['{}']", name);
                    return Ok(Some(vec![Mutation::substitute(node, replacement)?]));
                }
            }
        }

        Ok(None)
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}
