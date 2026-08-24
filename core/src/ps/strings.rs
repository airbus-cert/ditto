use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::error::DittoResult;
use crate::pass::Pass;
use crate::tree::Node;

pub trait Trigger {
    fn trigger(&mut self) -> bool;
}

#[derive(Debug)]
pub struct ModuloPass {
    modulo: u16,
    count: u16,
}

impl Default for ModuloPass {
    fn default() -> Self {
        ModuloPass {
            modulo: 1,
            count: 0,
        }
    }
}

impl ModuloPass {
    pub fn new(modulo: u16) -> Self {
        ModuloPass { modulo, count: 0 }
    }
}

impl Trigger for ModuloPass {
    fn trigger(&mut self) -> bool {
        self.count += 1;
        return (self.count % self.modulo) == 0;
    }
}

pub trait ApplyHexlify {
    fn apply(&mut self, str: String) -> String;
}

#[derive(Default, Debug)]
pub struct InlineHexlify;

impl ApplyHexlify for InlineHexlify {
    fn apply(&mut self, str: String) -> String {
        let mut map = Vec::new();
        for c in str.chars() {
            map.push(format!("[char]{}", c as u8));
        }
        format!("({})", map.join("+"))
    }
}

#[derive(Default, Debug)]
pub struct ForEachHexlify;

impl ApplyHexlify for ForEachHexlify {
    fn apply(&mut self, str: String) -> String {
        // TODO: Do not hardcode $total
        let foreach_template = r#"Foreach {$total=""} {$total += [char] $_} {$total}"#;
        let chars: Vec<String> = str.chars().map(|x| (x as u8).to_string()).collect();

        format!(r#"(@({}) | {})"#, chars.join(","), foreach_template)
    }
}

#[derive(Debug)]
pub struct Hexlify<T, U> {
    trigger: T,
    hexlify_function: U,
}

impl<T: Default, U: Default> Default for Hexlify<T, U> {
    fn default() -> Self {
        Hexlify {
            trigger: T::default(),
            hexlify_function: U::default(),
        }
    }
}

impl<T: Trigger, U: ApplyHexlify> Pass for Hexlify<T, U> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "verbatim_string_characters" {
            if !self.trigger.trigger() {
                return Ok(None);
            }

            let str = String::from(node.text().unwrap());
            let str_without_quote = String::from(&str[1..str.len() - 1]);

            return Ok(Some(vec![Mutation::substitute(
                node,
                self.hexlify_function.apply(str_without_quote),
            )?]));
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

impl<U> Hexlify<ModuloPass, U> {
    pub fn modulo(mut self, modulo: u16) -> Self {
        self.trigger.modulo = modulo;
        self
    }
}

pub type ModuloInlineHexlify = Hexlify<ModuloPass, InlineHexlify>;
pub type ModuloForEachHexlify = Hexlify<ModuloPass, ForEachHexlify>;

#[derive(Debug)]
pub struct Reverse<T> {
    trigger: T,
}

pub type ModuloReverse = Reverse<ModuloPass>;

impl<T: Default> Default for Reverse<T> {
    fn default() -> Self {
        Reverse {
            trigger: T::default(),
        }
    }
}

impl<T: Trigger> Pass for Reverse<T> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "verbatim_string_characters" {
            if !self.trigger.trigger() {
                return Ok(None);
            }

            let str = String::from(node.text().unwrap());
            let str_without_quote = String::from(&str[1..str.len() - 1]);
            let rev_str = str_without_quote.chars().rev().collect::<String>();
            return Ok(Some(vec![Mutation::substitute(
                node,
                "-join'".to_string() + &rev_str + "'[-1..-" + &rev_str.len().to_string() + "]",
            )?]));
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

impl Reverse<ModuloPass> {
    pub fn modulo(mut self, modulo: u16) -> Self {
        self.trigger.modulo = modulo;
        self
    }
}
