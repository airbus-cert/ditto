use std::cmp;

use crate::error::DittoResult;
use crate::name::{NameGenerator, VerbsNameGenerator};
use crate::pass::Pass;
use crate::PsMutation;
use crate::tree::Node;
use rand::seq::SliceRandom;
use rand::Rng;
use regex::RegexBuilder;

pub trait NameMask {
    fn apply(&mut self, name: String, names: Vec<String>) -> String;
}

#[derive(Default)]
pub struct NoMask;
impl NameMask for NoMask {
    fn apply(&mut self, name: String, _: Vec<String>) -> String {
        name
    }
}

fn get_mask(arr: Vec<char>, regex_mask: bool) -> String {
    let wildcard = if regex_mask { ".*" } else { "*" };
    arr.iter()
        .fold(String::new(), |mut acc, &x| {
            match acc.chars().last() {
                Some('*') => {
                    if x != '*' {
                        acc.push(x);
                    }
                }
                _ if x == '*' => acc += wildcard,
                _ => acc.push(x),
            }
            acc
        })
        .to_string()
}

fn variable_getter(name: String) -> String {
    format!(
        "(({}{}).Value)",
        match rand::rng().random_range(0..5) {
            0 => "LS VARIABLE:",
            1 => "LS VARIABLE:/",
            2 => "GCI VARIABLE:",
            3 => "GCI VARIABLE:/",
            _ => "VARIABLE ",
        },
        name
    )
}

#[derive(Default)]
pub struct WildcardMask;
impl NameMask for WildcardMask {
    fn apply(&mut self, name: String, psvariable_keys: Vec<String>) -> String {
        let size = name.len();
        if size < 2 {
            return name;
        };

        let mut keys = psvariable_keys.clone();
        keys.extend(DEFAULT_PSVARIABLE.iter().map(|s| s.to_string()));

        let name: Vec<char> = name.chars().collect();
        let mut masked = vec!['*'; size - 1]; // Fill with '*'
        masked.push(name[size - 1]);

        let mut permutation: Vec<usize> = (0..size - 1).into_iter().collect();
        permutation.shuffle(&mut rand::rng());

        while let Some(idx) = permutation.pop() {
            masked[idx] = name[idx];

            let r = RegexBuilder::new(&get_mask(masked.clone(), true))
                .case_insensitive(true)
                .build()
                .unwrap();
            if keys.iter().filter(|n| r.is_match(n)).count() == 1 {
                break;
            }
        }

        get_mask(masked, false)
    }
}

#[derive(Default)]
pub struct PSVariableAllLiteral<T: Default, U> {
    cache: Vec<String>,
    vars: Vec<String>,
    generator: T,
    mask: U,
    param_block: bool,
    counter: usize,
}

impl<T: NameGenerator + Default, U: NameMask + Default> Pass for PSVariableAllLiteral<T, U> {
    type Mutation = PsMutation;

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "program" => {
                self.counter = DEFAULT_PSVARIABLE.len();
                self.param_block = false;

                let total_literals = cmp::min(
                    node.count_childs_of_type("string_literal")
                        + node.count_childs_of_type("integer_literal"),
                    MAX_PSVAR - self.counter,
                );

                self.vars = (0..total_literals)
                    .map(|_| self.generator.generate(false))
                    .collect();
                self.cache = self.vars.clone();
                Ok(None)
            }
            "param_block" => {
                self.param_block = true;
                Ok(None)
            }
            "string_literal" | "integer_literal" if self.counter < MAX_PSVAR => {
                if self.param_block {
                    return Ok(None);
                }

                self.counter += 1;
                let var = self.cache.pop().unwrap();
                Ok(Some(vec![
                    PsMutation::global_statement(
                        node,
                        format!("sv {} {};", var, node.text().unwrap()),
                    )?,
                    PsMutation::substitute(
                        node,
                        variable_getter(self.mask.apply(var, self.vars.clone())),
                    )?,
                ]))
            }

            _ => {
                if node.kind() == "param_block" {
                    self.param_block = false;
                }
                Ok(None)
            }
        }
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

pub type VerbPSVariableAllLiteralWithWildcardMask =
    PSVariableAllLiteral<VerbsNameGenerator, WildcardMask>;
