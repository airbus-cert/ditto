use crate::Lang;
use crate::Lang::JavaScript;
use crate::Mutation;
use crate::pass::Pass;
use log::trace;
use std::fmt::Debug;

#[derive(Default, Debug)]
pub struct MemberToSubscript;

impl Pass for MemberToSubscript {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if node.kind() != "member_expression" || node.child_count() != 3 {
            return Ok(None);
        }

        let Some(object) = node.child(0) else {
            return Ok(None);
        };
        let Some(dot) = node.child(1) else {
            return Ok(None);
        };
        let Some(property) = node.child(2) else {
            return Ok(None);
        };

        if dot.kind() != "." || property.kind() != "property_identifier" {
            return Ok(None);
        }

        trace!(
            "Transformed {}.{} to {}['{}']",
            object.text()?,
            property.text()?,
            object.text()?,
            property.text()?
        );
        let replacement = format!("{}['{}']", object.text()?, property.text()?);
        Ok(Some(vec![Mutation::substitute(node, replacement)?]))
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}
