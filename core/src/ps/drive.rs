use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::error::DittoResult;
use crate::impostor::apply_engine;
use crate::pass::Pass;
use crate::tree::Node;
use regex::Regex;

/// The `Ditto` driver: apply an engine only where you ask for it.
///
/// It scans the script for directive comments of the form `# ditto(<engine>)`
/// and applies the named engine to the statement that *immediately follows* the
/// comment, leaving the rest of the script untouched. The directive comment
/// itself is removed from the output.
///
/// # Example
/// ```
/// use ditto::pass::Source;
/// use ditto::ps::drive::DriveByComment;
///
/// let output = Source::new(
///     "# ditto(pikachu)\n[Ref].Assembly.GetType('System.Management.Automation.AmsiUtils')",
/// )
/// .transform(DriveByComment::default())
/// .unwrap()
/// .output()
/// .unwrap();
///
/// // The directive comment is consumed...
/// assert!(!output.contains("# ditto"));
/// // ...and the statement that followed it has been obfuscated (no longer verbatim).
/// assert!(!output.contains("[Ref].Assembly.GetType('System.Management.Automation.AmsiUtils')"));
/// ```
#[derive(Debug)]
pub struct DriveByComment {
    parser: Regex,
}

impl Default for DriveByComment {
    fn default() -> DriveByComment {
        DriveByComment {
            parser: Regex::new(r"# ditto\(([a-z]+)\)").unwrap(),
        }
    }
}

impl Pass for DriveByComment {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "comment" {
            // parse comment to check if it's a ditto directive
            if let Some(m) = self.parser.captures(node.text()?) {
                let engine = m.get(1).map_or("", |m| m.as_str());

                if let Some(parent) = node.parent() {
                    let mut i = 0;
                    for child in parent.iter() {
                        i += 1;
                        if child == *node {
                            break;
                        }
                    }
                    if let Some(target_node) = parent.child(i) {
                        let str = String::from(target_node.text()?);
                        return Ok(Some(vec![
                            // Remove comment
                            Mutation::substitute(&node, "".to_string())?,
                            Mutation::substitute(&target_node, apply_engine(engine, &str)?)?,
                        ]));
                    }
                }
            }
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}
