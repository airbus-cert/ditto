use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::name::RandomNameGenerator;
use crate::pass::Pass;

#[derive(Debug)]
struct InvokationBuilder {
    var_name: String,
    left: String,
    right: String,
    params: String,
}

/// Convert one-line recursive calls in multiple lines of calls
///
/// For example:
/// ```powershell
/// [Ref].Assembly.GetType('System.Management.Automation.AmsiUtils').GetField('amsiInitFailed','NonPublic,Static').SetValue($null,$true)
/// ```
/// Becomes:
/// ```powershell
/// $qGt = $[Ref].Assembly.GetType('System.Management.Automation.AmsiUtils')
/// $UlE = $qGt.GetField('amsiInitFailed','NonPublic,Static')
/// $UlE.SetValue($null,$true)
/// ```
#[derive(Default, Debug)]
pub struct BreakInvokationExpressions {
    cache: Vec<InvokationBuilder>,
    buffer: String,
    last_var: String,
}

impl Pass for BreakInvokationExpressions {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "invokation_expression" => {
                if !self.cache.is_empty() || node.is_statement() {
                    let mut childs = node.iter();
                    if let (Some(child1), Some(dot), Some(child2), Some(params)) =
                        (childs.next(), childs.next(), childs.next(), childs.next())
                    {
                        if dot.text() == Ok(".") {
                            self.cache.push(InvokationBuilder {
                                var_name: RandomNameGenerator::random_string(3..7), // TODO: Add var manager
                                left: child1.text()?.to_string(),
                                right: child2.text()?.to_string(),
                                params: params.text()?.to_string(),
                            });
                        }
                    }
                }

                Ok(None)
            }
            "member_access" => Ok(None), // TODO:
            _ => Ok(None),
        }
    }

    fn leave(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "invokation_expression" {
            if let Some(InvokationBuilder {
                var_name,
                left,
                right,
                params,
            }) = self.cache.pop()
            {
                if !self.cache.is_empty() {
                    if self.buffer.is_empty() {
                        self.buffer = format!("${} = {}.{}{}", var_name, left, right, params);
                    } else {
                        self.buffer +=
                            &format!("\n${} = ${}.{}{}", var_name, self.last_var, right, params);
                    }
                    self.last_var = var_name.clone();
                } else {
                    let declaration_buffer = self.buffer.clone();
                    let last_var = self.last_var.clone();
                    self.buffer = String::new();
                    self.last_var = String::new();

                    let mut childs = node.iter();
                    if let (Some(child1), Some(dot), Some(child2)) =
                        (childs.next(), childs.next(), childs.next())
                    {
                        return Ok(Some(vec![
                            Mutation::substitute(&child1, declaration_buffer)?,
                            Mutation::substitute(&dot, format!("\n${}.", last_var))?,
                            Mutation::substitute(&child2, right)?,
                        ]));
                    }
                }
            }
        }

        Ok(None)
    }
}
