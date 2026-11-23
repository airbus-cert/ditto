use crate::Lang;
use crate::Lang::JavaScript;
use crate::Mutation;
use crate::js::utils::jsfuck::*;
use crate::js::utils::number::*;
use crate::js::utils::string::{is_directive_string, unescape_js_string};
use crate::pass::Pass;
use log::trace;

#[derive(Default, Debug)]
pub struct JsFuckNumbers;

impl Pass for JsFuckNumbers {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if let Some(number) = parse_number(node) {
            if is_integer(number) {
                // (!+[]+!+[]+[]) -> '2'
                // (!+[]+!+[]+!+[]+!+[]+[] -> '4'
                // (!+[]+!+[]+[]) + (!+[]+!+[]+!+[]+!+[]+[]) -> '24'
                // +((!+[]+!+[]+[]) + (!+[]+!+[]+!+[]+!+[]+[])) -> 24

                let number = number as i64;
                let is_neg = number < 0;
                let number = number.abs();

                // smaller result
                if number < 10 {
                    let digit = number.to_string().chars().next().unwrap();
                    let jsfuck_number = DIGITS_TABLE
                        .iter()
                        .find(|d| d.0 == digit)
                        .expect("digit should always be in DIGITS_TABLE")
                        .1;
                    let operator = if is_neg { "-" } else { "+" };
                    return Ok(Some(vec![Mutation::substitute(
                        node,
                        format!("{}({})", operator, jsfuck_number),
                    )?]));
                }

                let mut result = String::new();
                if is_neg {
                    result.push_str("-(");
                } else {
                    result.push_str("+(");
                }
                for digit in number.to_string().chars() {
                    let jsfuck_number = DIGITS_TABLE
                        .iter()
                        .find(|d| d.0 == digit)
                        .expect("digit should always be in DIGITS_TABLE")
                        .1;

                    result.push_str("(");
                    result.push_str(jsfuck_number);
                    result.push_str(")+");
                }
                result.pop();
                result.push(')');

                trace!("JsFucked {} into {}", number, result);
                return Ok(Some(vec![Mutation::substitute(node, result)?]));
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

/// JsFuckStrings pass is based on https://stackoverflow.com/a/63713987 <br>
/// The higher the level, the more resource-intensive the processing will be for the JavaScript engine.<br>
/// The output size is based on the level of the chars and is also exponetial.<br>
/// You can see the list of chars above based on the level.<br>
/// Level 9 means it can handle any character, but processing is very slow for the JavaScript engine.<br>
/// If it cannot recreate a character based on the current level, it will keep it as is.
#[derive(Debug)]
pub struct JsFuckStrings {
    pub max_level: u8,
}

impl JsFuckStrings {
    pub fn new(max_level: u8) -> Self {
        Self { max_level }
    }
}

impl Default for JsFuckStrings {
    fn default() -> Self {
        Self::new(9)
    }
}

impl Pass for JsFuckStrings {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
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

        let replacement = encode(value.as_str(), self.max_level);
        trace!("JsFucked '{}'", value);
        Ok(Some(vec![Mutation::substitute(node, replacement)?]))
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

#[derive(Default, Debug)]
pub struct JsFuckBooleans;

impl Pass for JsFuckBooleans {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "true" || node.kind() == "false" {
            return match node.kind() == "true" {
                true => {
                    trace!("JsFuck true into !![]");
                    Ok(Some(vec![Mutation::substitute(node, "!![]".to_string())?]))
                }
                false => {
                    trace!("JsFuck false into ![]");
                    Ok(Some(vec![Mutation::substitute(node, "![]".to_string())?]))
                }
            };
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

#[derive(Default, Debug)]
pub struct JsFuckSpecials;

impl Pass for JsFuckSpecials {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "identifier" => {
                if let Ok(name) = node.text() {
                    return match name {
                        "Infinity" => {
                            trace!("JsFucked Infinity into {INFINITY}");
                            Ok(Some(vec![Mutation::substitute(
                                node,
                                INFINITY.to_string(),
                            )?]))
                        }
                        "NaN" => {
                            trace!("JsFucked NaN into +[![]]");
                            Ok(Some(vec![Mutation::substitute(
                                node,
                                "+[![]]".to_string(),
                            )?]))
                        }
                        _ => Ok(None),
                    };
                }
            }
            "undefined" => {
                trace!("JsFucked undefined into [][[]]");
                return Ok(Some(vec![Mutation::substitute(
                    node,
                    "[][[]]".to_string(),
                )?]));
            }
            _ => {}
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

#[derive(Default, Debug)]
pub struct JsFuckDefaultObjects;

impl Pass for JsFuckDefaultObjects {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "identifier" {
            if let Ok(name) = node.text() {
                if name == "Boolean" || name == "Number" || name == "String" || name == "Function" {
                    trace!("JsFucked {name}")
                }
                return match name {
                    "Boolean" => Ok(Some(vec![Mutation::substitute(
                        node,
                        BOOLEAN_OBJECT.to_string(),
                    )?])),
                    "Number" => Ok(Some(vec![Mutation::substitute(
                        node,
                        NUMBER_OBJECT.to_string(),
                    )?])),
                    "String" => Ok(Some(vec![Mutation::substitute(
                        node,
                        STRING_OBJECT.to_string(),
                    )?])),
                    "Function" => Ok(Some(vec![Mutation::substitute(
                        node,
                        FUNCTION_OBJECT.to_string(),
                    )?])),
                    _ => Ok(None),
                };
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
