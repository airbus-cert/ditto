use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::name::RandomNameGenerator;
use crate::pass::Pass;
use crate::ps::var::VarManager;

/// Replace boolean $true and $false with -not computation of undefined variable
/// Also replace the $null special variable
///
/// # Example
/// ```
/// use ditto::pass::Source;
/// use ditto::ps::bool::ComputeAllBool;
///
/// let true_output = Source::new("$true")
///     .transform(ComputeAllBool::default())
///     .unwrap().output().unwrap();
/// assert!(true_output.starts_with("(-not $") && true_output.ends_with(")"));
///
/// let false_output = Source::new("$false")
///     .transform(ComputeAllBool::default())
///     .unwrap().output().unwrap();
/// assert!(false_output.starts_with("(-not -not $") && false_output.ends_with(")"));
/// ```
#[derive(Default, Debug)]
pub struct ComputeAllBool;

impl Pass for ComputeAllBool {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "variable" {
            let rand = RandomNameGenerator::random_string(7..10);
            Ok(match VarManager::extract_name(node.text()?).as_str() {
                "null" => Some(vec![Mutation::substitute(node, format!("${}", rand))?]),
                "true" => Some(vec![Mutation::substitute(
                    node,
                    format!("(-not ${})", rand),
                )?]),
                "false" => Some(vec![Mutation::substitute(
                    node,
                    format!("(-not -not ${})", rand),
                )?]),
                _ => None,
            })
        } else {
            Ok(None)
        }
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}
