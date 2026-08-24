use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::error::{DittoResult, Error};
use crate::pass::Pass;
use crate::tree::Node;

/// Convert a command name into invoke expression of the associated string
/// Write-Output -> . "Write-Output"
///
/// Static string will be the input of many transformer afterward
///
/// # Example
/// ```
/// use ditto::pass::Source;
/// use ditto::ps::stringify::CmdletAsString;
/// assert_eq!(
/// Source::new("Write-Output")
///     .transform(CmdletAsString::default())
///     .unwrap().output().unwrap(),
/// ". 'Write-Output'"
/// )
/// ```
#[derive(Default, Debug)]
pub struct CmdletAsString;

impl Pass for CmdletAsString {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "command_name" {
            if let Some(parent) = node.parent() {
                if parent.kind() == "command" {
                    return Ok(Some(vec![Mutation::substitute(
                        node,
                        ". '".to_string() + node.text()? + "'",
                    )?]));
                }
            } else {
                return Err(Error::NodeNotFound(String::from("command")));
            }
        }
        return Ok(None);
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

/// Convert expandable string without any expandable sub node into
/// verbatim string literal
///
/// Static string will be the input of many transformer afterward
///
/// # Example
/// ```
/// use ditto::pass::Source;
/// use ditto::ps::stringify::AsStaticString;
///
/// assert_eq!(
/// Source::new("\"toto\"")
///     .transform(AsStaticString::default())
///     .unwrap().output().unwrap(),
/// "'toto'"
/// )
/// ```
#[derive(Default, Debug)]
pub struct AsStaticString;

impl Pass for AsStaticString {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "expandable_string_literal" {
            let str = String::from(node.text()?);
            let str_without_quote = String::from(&str[1..str.len() - 1]);

            // no child means no expandable
            if node.child_count() == 0 {
                return Ok(Some(vec![Mutation::substitute(
                    node,
                    "'".to_string() + &str_without_quote + "'",
                )?]));
            }
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

/// Convert member call (aka $toto.length) into a static string
/// $toto.length -> $toto.'toto'
///
/// Static string will be the input of many transformer afterward
///
/// # Example
/// ```
/// use ditto::pass::Source;
/// use ditto::ps::stringify::MemberAsStaticString;
///
/// assert_eq!(
/// Source::new("$toto.length")
///     .transform(MemberAsStaticString::default())
///     .unwrap().output().unwrap(),
/// "$toto.'length'"
/// )
/// ```
#[derive(Default, Debug)]
pub struct MemberAsStaticString;

impl Pass for MemberAsStaticString {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "member_name" {
            let str = String::from(node.text()?);

            // no child means no expandable
            if let Some(child) = node.child(0) {
                if child.kind() == "simple_name" {
                    return Ok(Some(vec![Mutation::substitute(
                        node,
                        "'".to_string() + &str + "'",
                    )?]));
                }
            }
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

/// Transform any type as a static string by using cast conversion
/// [toto]$a -> 'toto' -as [type]
/// Static string will be the input of many transformer afterward
///
/// # Example
/// ```
/// use ditto::pass::Source;
/// use ditto::ps::stringify::TypeAsString;
///
/// assert_eq!(
/// Source::new("[toto]$a")
///     .transform(TypeAsString::default())
///     .unwrap().output().unwrap(),
/// "('toto' -as [type])$a"
/// )
/// ```
#[derive(Default, Debug)]
pub struct TypeAsString;

impl Pass for TypeAsString {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "type_literal" {
            if let Some(type_spec) = node.child(1) {
                if type_spec.kind() == "type_spec" {
                    return Ok(Some(vec![Mutation::substitute(
                        node,
                        format!("('{}' -as [type])", type_spec.text()?),
                    )?]));
                }
            }
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}
