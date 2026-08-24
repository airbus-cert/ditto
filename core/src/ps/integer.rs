use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::pass::Pass;

#[derive(Default, Debug)]
pub struct ComputeAllInt;

impl Pass for ComputeAllInt {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "integer_literal" {
            if let Ok(value) = node.text()?.parse::<u8>() {
                let r = rand::random::<u8>();
                let substitution = if rand::random::<bool>() {
                    format!("({}+{})", r % value, value - r % value)
                } else {
                    let x = value + r % (u8::MAX - value);
                    format!("({}-{})", x, x - value)
                };

                return Ok(Some(vec![Mutation::substitute(node, substitution)?]));
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
