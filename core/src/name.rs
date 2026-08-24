use itertools::Itertools;
use rand::distr::Alphanumeric;
use rand::seq::SliceRandom;
use rand::{RngExt, rng};
use std::iter;

pub trait NameGenerator {
    fn generate(&mut self, is_var: bool) -> String;
}

#[derive(Default, Debug)]
pub struct RandomNameGenerator;

impl RandomNameGenerator {
    pub fn random_string(range: std::ops::Range<usize>) -> String {
        rng()
            .sample_iter(&Alphanumeric)
            .take(rng().random_range(range))
            .map(char::from)
            .collect()
    }
}

impl NameGenerator for RandomNameGenerator {
    fn generate(&mut self, is_var: bool) -> String {
        let name = RandomNameGenerator::random_string(3..7);

        if is_var {
            format!("${{{}}}", &name)
        } else {
            name
        }
    }
}

#[derive(Debug)]
pub struct CloseNameGenerator {
    cache: Vec<String>,
    seed: String,
    len: usize,
}

impl Default for CloseNameGenerator {
    fn default() -> Self {
        Self {
            cache: Vec::new(),
            seed: String::from("abcdefhij"),
            len: 3,
        }
    }
}

impl CloseNameGenerator {
    fn next_cache(&mut self) -> Vec<String> {
        self.len += 1;
        let mut cache: Vec<String> = iter::repeat_n(self.seed.chars().take(self.len), self.len)
            .multi_cartesian_product()
            .map(|v| v.iter().collect())
            .collect();
        cache.shuffle(&mut rng());
        cache
    }
}

impl NameGenerator for CloseNameGenerator {
    fn generate(&mut self, is_var: bool) -> String {
        if self.cache.is_empty() {
            self.cache = self.next_cache();
        }

        let name = self.cache.pop().unwrap();
        if is_var {
            format!("${{{}}}", &name)
        } else {
            name
        }
    }
}

static VERBS: &'static [&'static str] = &[
    "Add", "Clear", "Close", "Copy", "Enter", "Exit", "Find", "Format", "Get", "Hide", "Join",
    "Lock", "Move", "New", "Open", "Optimize", "Pop", "Push", "Redo", "Remove", "Rename", "Reset",
    "Resize", "Search", "Select", "Set", "Show", "Skip", "Split", "Step", "Switch", "Undo",
    "Unlock", "Watch",
];

#[derive(Default, Debug)]
pub struct VerbsNameGenerator;

impl NameGenerator for VerbsNameGenerator {
    fn generate(&mut self, is_var: bool) -> String {
        let name: String = rng()
            .sample_iter(&Alphanumeric)
            .take(rng().random_range(3..7))
            .map(char::from)
            .collect();

        let name = VERBS[rng().random_range(0..VERBS.len() - 1)].to_owned() + "-" + &name;

        if is_var {
            format!("${{{}}}", &name)
        } else {
            name
        }
    }
}
