use crate::Lang::JavaScript;
use crate::js::utils::curse::*;
use crate::js::utils::number::{is_js_safe_integer, parse_number};
use crate::js::utils::string::{is_directive_string, unescape_js_string};
use crate::pass::Pass;
use crate::{Lang, Mutation};
use rand::rng;

#[derive(Debug, Default)]
pub struct CurseString;

impl Pass for CurseString {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if node.kind() != "string" {
            return Ok(None);
        }

        if let Some(parent) = node.parent() {
            if parent.kind() == "pair" {
                return Ok(None);
            }
        }

        if is_directive_string(node) {
            return Ok(None);
        }

        let value = unescape_js_string(node.text()?);
        let replacement = curse_string(value.as_str(), &mut rng());
        Ok(Some(vec![Mutation::substitute(node, replacement)?]))
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}

#[derive(Debug, Default)]
pub struct CurseNumber;

impl Pass for CurseNumber {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if node.kind() != "number" {
            return Ok(None);
        }

        if let Some(number) = parse_number(node) {
            let replacement = if is_js_safe_integer(number) {
                curse_integer_math(number as i64, &mut rng())
            } else {
                format!("+({})", curse_string(&number.to_string(), &mut rng()))
            };

            return Ok(Some(vec![Mutation::substitute(node, replacement)?]));
        }

        Ok(None)
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}
