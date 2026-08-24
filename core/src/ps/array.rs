use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::error::DittoResult;
use crate::name::{NameGenerator, RandomNameGenerator, VerbsNameGenerator};
use crate::pass::Pass;
use crate::tree::Node;
use rand::seq::SliceRandom;

#[derive(Default, Debug)]
pub struct ArrayAllConst<T> {
    cache_value: Vec<String>,
    cache_name: Vec<String>,
    generator: T,
    array_name: String,
}

impl<T: NameGenerator + Default> Pass for ArrayAllConst<T> {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "program" => {
                if self.array_name == "" {
                    self.array_name = self.generator.generate(true);
                };
                Ok(None)
            }
            "string_literal" | "integer_literal" => {
                if let Some(assignement) = node.get_parent_of_types(vec!["assignment_expression"]) {
                    if let Some(right_assignement) = assignement.child(2) {
                        // Assert that the assignement is a string or integer constant
                        let value = node.text().unwrap();
                        if right_assignement.text().unwrap() == value {
                            if let Some(left_assignement) = assignement.child(0) {
                                self.cache_value.push(value.to_string());
                                self.cache_name
                                    .push(left_assignement.text().unwrap().to_string());

                                return Ok(Some(vec![Mutation::substitute(
                                    &assignement,
                                    String::from(""),
                                )?]));
                            }
                        }
                    }
                }
                Ok(None)
            }
            "variable" => {
                let var_name = node.text().unwrap();

                // Check if variable in cache
                let var_idx = self
                    .cache_name
                    .clone()
                    .into_iter()
                    .position(|c| c == var_name);
                if let Some(var_idx) = var_idx {
                    return Ok(Some(vec![Mutation::substitute(
                        node,
                        "(".to_owned()
                            + &self.array_name.clone()
                            + "["
                            + &var_idx.to_string()
                            + "])",
                    )?]));
                }
                Ok(None)
            }

            _ => Ok(None),
        }
    }

    fn leave(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "program" => {
                let muta = Mutation::global_statement(
                    node,
                    self.array_name.clone() + "=@(" + &self.cache_value.join(",") + ");",
                )?;
                Ok(Some(vec![muta]))
            }
            _ => Ok(None),
        }
    }
}
pub type RandomArrayAllConst = ArrayAllConst<RandomNameGenerator>;
pub type VerbsArrayAllConst = ArrayAllConst<VerbsNameGenerator>;

#[derive(Default, Debug)]
pub struct ShuffleArray {
    cache: Vec<String>,
    perms: Vec<Vec<usize>>,
}

impl Pass for ShuffleArray {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "array_expression" => {
                if let Some(parent) = node.get_parent_of_types(vec!["assignment_expression"]) {
                    if let Some(left_assignement) = parent.child(0) {
                        let array_name = left_assignement.text()?;
                        self.cache.push(String::from(array_name));

                        if let Some(literal) =
                            node.get_childs_of_type("array_literal_expression").next()
                        {
                            let unaries: Vec<String> = (0..literal.child_count())
                                .filter_map(|x| literal.child(x))
                                .filter(|x| x.kind() == "unary_expression")
                                .map(|n| n.text().unwrap().to_string())
                                .collect();

                            let mut permutation: Vec<usize> =
                                (0..unaries.len()).into_iter().collect();
                            permutation.shuffle(&mut rand::rng());

                            self.perms.push(permutation.clone());

                            let new_array: Vec<String> = permutation
                                .into_iter()
                                .map(|x| unaries[x].to_string())
                                .collect();

                            return Ok(Some(vec![Mutation::substitute(
                                &literal,
                                new_array.join(","),
                            )?]));
                        }
                    }
                }
                Ok(None)
            }
            "variable" => {
                let var_name = node.text().unwrap();
                if let Some(var_idx) = self.cache.clone().into_iter().position(|c| c == var_name) {
                    if let Some(parent) = node.parent() {
                        if parent.kind() == "element_access" {
                            if let Some(logical_exp) = parent.child(2) {
                                if logical_exp.kind() == "logical_expression" {
                                    let old = logical_exp.text().unwrap().to_string();
                                    if let Ok(old_idx) = old.parse::<usize>() {
                                        let new_idx = self.perms[var_idx]
                                            .clone()
                                            .into_iter()
                                            .position(|x| x == old_idx)
                                            .unwrap_or(old_idx);

                                        return Ok(Some(vec![Mutation::substitute(
                                            &logical_exp,
                                            new_idx.to_string(),
                                        )?]));
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}
