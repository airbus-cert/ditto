use crate::Lang;
use crate::error::DittoResult;
use crate::error::Error::InvalidProbability;
use crate::tree::{Node, Tree, Visitor};
use log::debug;
use std::fmt::Debug;
use tree_sitter_javascript::LANGUAGE as javascript_language;
use tree_sitter_powershell::LANGUAGE as powershell_language;

pub trait Replace {
    fn start_byte(&self) -> usize;
    fn end_byte(&self) -> usize;
    fn new_source(&self) -> String;
}

pub trait Pass {
    type Mutation: Replace;
    fn lang(&self) -> Lang;
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>>;
    fn leave(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>>;
}

pub struct SourceVisitor<T: Pass> {
    mutations: Vec<T::Mutation>,
    pass: T,
    pub percentage: f32,
}

impl<T: Pass> SourceVisitor<T> {
    pub fn new(pass: T, percentage: f32) -> Self {
        Self {
            mutations: Vec::new(),
            pass,
            percentage,
        }
    }
}

impl<T: Pass> Visitor for SourceVisitor<T> {
    fn enter(&mut self, node: &Node) -> DittoResult<bool> {
        let prob = rand::random::<f32>();
        if prob < self.percentage {
            let result = self.pass.enter(node);
            if let Ok(Some(commands)) = result {
                self.mutations.extend(commands);
                return Ok(false);
            } else if let Err(e) = result {
                return Err(e);
            }
            Ok(true)
        } else {
            Ok(true)
        }
    }

    fn leave(&mut self, node: &Node) -> DittoResult<bool> {
        let result = self.pass.leave(node);
        if let Ok(Some(commands)) = result {
            self.mutations.extend(commands);
            return Ok(false);
        } else if let Err(e) = result {
            return Err(e);
        }
        Ok(true)
    }
}

pub struct Source {
    source: Vec<u8>,
}

impl Source {
    pub fn new(src: &str) -> Self {
        // Powershell is case insensitive
        // And the grammar is specified in lowercase
        Self {
            source: src.as_bytes().to_vec(),
        }
    }

    pub fn transform_with_probability<P: Pass + Debug>(
        mut self,
        pass: P,
        percentage: f32,
    ) -> DittoResult<Self> {
        if percentage < 0.0 || percentage > 1.0 {
            return Err(InvalidProbability(percentage));
        }

        debug!(
            "Applying pass: {:?} with probability: {:.0}%",
            pass,
            percentage * 100.0
        );

        let mut parser = tree_sitter::Parser::new();

        match pass.lang() {
            Lang::PowerShell => parser.set_language(&powershell_language.into()).unwrap(),
            Lang::JavaScript => parser.set_language(&javascript_language.into()).unwrap(),
        }

        let text = String::from_utf8(self.source.clone())?;
        let tree = parser.parse(text, None).unwrap();
        let tree = Tree::new(&self.source, tree);

        let mut visitor = SourceVisitor::new(pass, percentage);
        tree.visit(&mut visitor)?;

        let mut mutations = visitor.mutations;
        // Sort by start, then by end so that zero-width insertions (e.g.
        // `global_statement`, whose start == end) are emitted before any wider
        // substitution sharing the same start byte. Otherwise a substitution
        // starting at byte N advances `last` past a later insertion at byte N,
        // producing a reversed slice range and a panic below.
        mutations.sort_by(|a, b| {
            a.start_byte()
                .cmp(&b.start_byte())
                .then(a.end_byte().cmp(&b.end_byte()))
        });

        let mut result = Vec::new();
        let mut last = 0;

        for sub in mutations {
            result.extend_from_slice(&self.source[last..sub.start_byte()]);
            result.extend_from_slice(sub.new_source().as_bytes());
            last = sub.end_byte();
        }

        result.extend_from_slice(&self.source[last..]);
        self.source = result;
        Ok(self)
    }

    pub fn transform<P: Pass + Debug>(self, pass: P) -> DittoResult<Self> {
        self.transform_with_probability(pass, 1.0)
    }

    pub fn output(&self) -> DittoResult<String> {
        Ok(String::from_utf8(self.source.clone())?)
    }
}
