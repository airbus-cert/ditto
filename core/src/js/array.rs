use self::ArrayTransformFunction::*;
use crate::Lang;
use crate::Lang::JavaScript;
use crate::Mutation;
use crate::js::utils::array::*;
use crate::js::utils::generate_junk_node;
use crate::pass::Pass;
use crate::tree::Node;
use log::trace;
use rand::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug)]
pub struct ArrayTransform {
    functions: Vec<ArrayTransformFunction>,
}

#[derive(Debug, EnumIter, PartialEq)]
pub enum ArrayTransformFunction {
    Reverse,
    ToReversed,
    Sort,
    ToSorted,
    Slice(usize), // max garbage quantity (times 2 because of the left and right side)
    Fill,
    Flat,
    ToSpliced,
}

impl ArrayTransform {
    pub fn new(functions: Vec<ArrayTransformFunction>) -> Self {
        Self { functions }
    }
}

impl Default for ArrayTransform {
    fn default() -> Self {
        let mut functions = vec![];
        for func in ArrayTransformFunction::iter() {
            functions.push(func);
        }
        Self::new(functions)
    }
}

impl Pass for ArrayTransform {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(&mut self, node: &Node) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() != "array" {
            return Ok(None);
        }

        let mut elements = Vec::new();
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            if child.kind() != "[" && child.kind() != "]" && child.kind() != "," {
                elements.push(child);
            }
        }

        let has_nested_array = elements.iter().any(|e| e.kind() == "array");

        let mut valid_fns = vec![];
        for func in &self.functions {
            match func {
                Reverse => valid_fns.push(Reverse),
                ToReversed => valid_fns.push(ToReversed),
                Sort if is_sorted(&elements) => valid_fns.push(Sort),
                ToSorted if is_sorted(&elements) => valid_fns.push(ToSorted),
                Slice(strength) => valid_fns.push(Slice(*strength)),
                Fill => valid_fns.push(Fill),
                Flat if !has_nested_array && elements.len() >= 2 => valid_fns.push(Flat),
                ToSpliced if !elements.is_empty() => valid_fns.push(ToSpliced),
                _ => {}
            }
        }

        if valid_fns.is_empty() {
            return Ok(None);
        }

        let mut rng = rand::rng();
        let func = valid_fns.choose(&mut rng).unwrap();

        trace!("Transforming array with function: {:?} ", func);

        match func {
            Reverse => {
                elements.reverse();
                let replacement = repack_array(&elements);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.reverse()", replacement),
                )?]))
            }
            ToReversed => {
                elements.reverse();
                let replacement = repack_array(&elements);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.toReversed()", replacement),
                )?]))
            }
            Sort => {
                elements.shuffle(&mut rng);
                let replacement = repack_array(&elements);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.sort()", replacement),
                )?]))
            }
            ToSorted => {
                elements.shuffle(&mut rng);
                let replacement = repack_array(&elements);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.toSorted()", replacement),
                )?]))
            }
            Slice(strength) => {
                let strength = if *strength <= 1 { 4 } else { *strength };
                let mut left_noise = Vec::new();
                for _ in 0..rng.random_range(1..=strength) {
                    left_noise.push(generate_junk_node(None, &mut rng));
                }
                let mut right_noise = Vec::new();
                for _ in 0..rng.random_range(1..=strength) {
                    right_noise.push(generate_junk_node(None, &mut rng));
                }
                let left_idx = left_noise.len();
                let right_idx = -(right_noise.len() as isize);
                let mut replacement = String::from("[");
                for element in left_noise {
                    replacement.push_str(element.as_str());
                    replacement.push_str(",");
                }
                elements.reverse();
                for element in elements.iter().rev() {
                    replacement.push_str(element.text()?);
                    replacement.push_str(",");
                }
                for element in right_noise {
                    replacement.push_str(element.as_str());
                    replacement.push_str(",");
                }
                replacement.pop();
                replacement.push_str("]");
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.slice({}, {})", replacement, left_idx, right_idx),
                )?]))
            }
            Fill => {
                if elements.is_empty() {
                    Ok(Some(vec![Mutation::substitute(
                        node,
                        format!(
                            "[].fill({},{},{})",
                            rng.random_range(0..u32::MAX),
                            rng.random_range(0..u32::MAX),
                            rng.random_range(0..u32::MAX)
                        ),
                    )?]))
                } else {
                    let index = rng.random_range(0..elements.len());
                    let replacement = format!(
                        "{}.fill({},{},{})",
                        node.text()?,
                        elements[index].text()?,
                        index,
                        index
                    );
                    Ok(Some(vec![Mutation::substitute(node, replacement)?]))
                }
            }
            Flat => {
                let packed = randomly_pack_array_elements(&elements, &mut rng);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.flat(Infinity)", packed),
                )?]))
            }
            ToSpliced => {
                if elements.is_empty() {
                    return Ok(None);
                }

                let start = rng.random_range(0..elements.len());
                let take = rng.random_range(1..=elements.len() - start);

                let mut base_parts = Vec::new();
                let mut inserted_parts = Vec::new();

                for (idx, element) in elements.iter().enumerate() {
                    let text = element.text()?.to_string();
                    if idx >= start && idx < start + take {
                        inserted_parts.push(text);
                    } else {
                        base_parts.push(text);
                    }
                }

                let base = format!("[{}]", base_parts.join(","));
                let inserted = inserted_parts.join(",");

                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.toSpliced({}, 0, {})", base, start, inserted),
                )?]))
            }
        }
    }

    fn leave(&mut self, _node: &Node) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::js::utils::generate_junk_node;

    #[test]
    fn test_junk_generation() {
        let mut rng = rand::rng();
        for _ in 0..100 {
            let junk = generate_junk_node(None, &mut rng);
            println!("{}", junk);
        }
    }
}
