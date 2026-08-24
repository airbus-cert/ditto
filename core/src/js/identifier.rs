use self::IdentifierType::*;
use crate::Lang;
use crate::Lang::JavaScript;
use crate::Mutation;
use crate::pass::Pass;
use itertools::Itertools;
use log::trace;
use rand::{Rng, RngExt};
use std::collections::HashMap;
use std::fmt::Debug;

/// Replace identifiers (variable names, function names, and globals) with their unicode equivalents
///
/// For example:
/// ```js
/// eval("thing")
/// ```
/// Will become this when you apply `transform(IdentifierNameEscape::new(100))`:
/// ```js
/// \u0065\u0076\u0061\u006C("thing")
/// ```
///
/// The arg represents the probability, expressed as a percentage, that a character will be omitted from the identifier name.
/// By default, this value is set to 75%, so it will become this for example:
/// ```js
/// \u0065v\u0061\u006C("thing")
/// ```
///
/// It is recommended to not set it to 100%, so the var name seems different for deobfuscators but is in fact the same.
#[derive(Debug)]
pub struct IdentifierNameEscape {
    /// percentage of chance to escape a char inside the identifier name
    pub prob: u8,
    pub types_to_transform: Vec<IdentifierType>,
}

impl IdentifierNameEscape {
    pub fn new(prob: u8, types_to_transform: Vec<IdentifierType>) -> Self {
        Self {
            prob,
            types_to_transform,
        }
    }
}

impl Default for IdentifierNameEscape {
    fn default() -> Self {
        Self {
            prob: 75,
            types_to_transform: vec![Identifier, MemberIdentifier, PropertyIdentifier],
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum IdentifierType {
    Identifier,
    MemberIdentifier,
    PropertyIdentifier,
}

impl Pass for IdentifierNameEscape {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if (node.kind() == "identifier" && self.types_to_transform.contains(&Identifier))
            || (node.kind() == "member_identifier"
                && self.types_to_transform.contains(&MemberIdentifier))
            || (node.kind() == "property_identifier"
                && self.types_to_transform.contains(&PropertyIdentifier))
        {
            let mut new_name = String::new();
            let mut rng = rand::rng();
            for c in node.text()?.chars() {
                let prob = rng.random_range(0..100);
                if prob < self.prob {
                    new_name.push_str(format!("\\u{:04X}", c as u32).as_str());
                } else {
                    new_name.push(c);
                }
            }

            trace!(
                "Escaped `{}` name '{}' into '{}'",
                node.kind(),
                node.text()?,
                new_name
            );
            return Ok(Some(vec![Mutation::substitute(node, new_name)?]));
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

pub trait IdentifierNameGenerator: Debug {
    fn next(&mut self, rng: &mut dyn Rng) -> String;
}

#[derive(Debug)]
pub struct IdentifierRenamer {
    generator: Box<dyn IdentifierNameGenerator>,
    cache: HashMap<String, String>,
}

impl IdentifierRenamer {
    pub fn new(generator: impl IdentifierNameGenerator + 'static) -> Self {
        Self {
            generator: Box::new(generator),
            cache: HashMap::new(),
        }
    }

    pub fn build_cache(
        &mut self,
        node: &crate::tree::Node,
        rng: &mut dyn Rng,
    ) -> crate::error::DittoResult<()> {
        if node.kind() == "identifier" {
            if let Some(parent) = node.parent() {
                let parent_kind = parent.kind();

                let should_cache = parent_kind.ends_with("_declaration")
                    || parent_kind.ends_with("_declarator")
                    || parent
                        .parent()
                        .map_or(false, |gp| gp.kind() == "function_declaration");

                if should_cache {
                    let name = node.text()?;

                    if !self.cache.contains_key(name) {
                        for _ in 0..128 {
                            let new_name = self.generator.next(rng);

                            if !self.cache.values().contains(&new_name) {
                                self.cache.insert(name.to_string(), new_name.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
        for child_id in 0..node.child_count() {
            let node = &node.child(child_id).unwrap();
            self.build_cache(node, rng)?;
        }

        Ok(())
    }
}

impl Pass for IdentifierRenamer {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if node.kind() == "program" {
            self.build_cache(node, &mut rand::rng())?;
            trace!("Cache content: {:?}", self.cache);
            return Ok(None);
        }
        if node.kind() != "identifier" {
            return Ok(None);
        }

        match self.cache.get(node.text()?) {
            Some(new_name) => {
                trace!("Renaming {} to {}", node.text()?, new_name);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    new_name.to_string(),
                )?]))
            }
            None => Ok(None),
        }
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}
