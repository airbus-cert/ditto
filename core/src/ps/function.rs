use std::collections::HashMap;

use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::error::DittoResult;
use crate::name::{NameGenerator, RandomNameGenerator, VerbsNameGenerator};
use crate::pass::Pass;
use crate::ps::var::VarManager;
use crate::tree::Node;
use indoc::indoc;
use rand::seq::SliceRandom;

pub trait PwshFunction {
    fn init(&mut self);
    fn apply(&self, x: u8) -> Vec<u8>;
    fn get_ps1_function_format(&self) -> String;
}

#[derive(Default, Debug)]
pub struct EvalXorAdd {
    cmdlet: [u8; 4],
    shiftkey: Vec<u8>,
}

impl PwshFunction for EvalXorAdd {
    fn init(&mut self) {
        // will try to execute `epal 1`
        self.cmdlet = b"epal".to_owned();

        // generate shiftkey
        let mut permutation: Vec<u8> = (0..=0xff).into_iter().collect();
        permutation.shuffle(&mut rand::rng());
        self.shiftkey = permutation[..4].to_vec();
    }

    fn apply(&self, x: u8) -> Vec<u8> {
        (0..4)
            .map(|i| x ^ (self.cmdlet[i].overflowing_sub(self.shiftkey[i]).0))
            .collect()
    }

    fn get_ps1_function_format(&self) -> String {
        format!(
            indoc! { r#"
            function <FUNCTIONNAME>($xorkey) {{
            [byte[]] $b = <BYTEARRAY>;$shiftkey=@({})
            while ($true) {{
            try {{
            $x = 0..3 | ForEach-Object {{ (($shiftkey[$_] + ($xorkey -bxor $b[$_])) % 0x100) }}
            $cmd = [System.Text.Encoding]::UTF8.GetString($x) + " 1"
            if ($cmd -cmatch "^[a-z]+ 1$") {{Invoke-Expression $cmd; break}}
            fail
            }} catch {{$b = $b[4..$b.length]}}
            }}
            return (0x100*4 - $b.length) / 4
            }}
            "#,
            },
            self.shiftkey
                .clone()
                .into_iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(","),
        )
        .to_string()
    }
}

#[derive(Default, Debug)]
pub struct ApplyFunctionToAllInt<T, U> {
    cache: Vec<u8>,
    function_name: String,
    bytearray: Vec<u8>,
    function: T,
    generator: U,
}

impl<T: PwshFunction + Default, U: NameGenerator> Pass for ApplyFunctionToAllInt<T, U> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "program" => {
                self.function_name = self.generator.generate(false);

                self.function.init();

                // generate permutation
                let mut permutation: Vec<u8> = (0..=0xff).into_iter().collect();
                permutation.shuffle(&mut rand::rng());
                self.cache = permutation.clone();

                // generate bytearray
                self.bytearray = permutation
                    .into_iter()
                    .flat_map(|x| self.function.apply(x))
                    .collect();

                Ok(None)
            }
            "integer_literal" => {
                if let Ok(text) = node.text() {
                    if let Ok(value) = text.parse::<usize>() {
                        assert!(value < self.cache.len());
                        return Ok(Some(vec![Mutation::substitute(
                            node,
                            format!("({}({}))", self.function_name.clone(), self.cache[value]),
                        )?]));
                    }
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn leave(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "program" => {
                let template_function = self.function.get_ps1_function_format();
                assert!(template_function.contains("<FUNCTIONNAME>"));
                assert!(template_function.contains("<BYTEARRAY>"));
                let function = template_function
                    .replace("<FUNCTIONNAME>", self.function_name.as_str())
                    .replace(
                        "<BYTEARRAY>",
                        self.bytearray
                            .clone()
                            .into_iter()
                            .map(|i| i.to_string())
                            .collect::<Vec<String>>()
                            .join(",")
                            .as_str(),
                    );

                Ok(Some(vec![Mutation::global_statement(node, function)?]))
            }
            _ => Ok(None),
        }
    }
}

pub type VerbsApplyEvalXorAddToAllInt = ApplyFunctionToAllInt<EvalXorAdd, VerbsNameGenerator>;

#[derive(Debug)]
pub struct RenameAllFunction<'a, T> {
    generator: T,
    cache: HashMap<String, String>,
    var_manager: &'a mut VarManager,
}

impl<'a, T: Default> RenameAllFunction<'a, T> {
    pub fn new(var_manager: &'a mut VarManager) -> Self {
        Self {
            var_manager,
            cache: HashMap::new(),
            generator: T::default(),
        }
    }
}

impl<'a, T: NameGenerator + Default> Pass for RenameAllFunction<'a, T> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "function_name" => {
                let new_name = self.generator.generate(false);
                self.cache
                    .insert(node.text()?.to_string(), new_name.clone());
                self.var_manager.push(new_name.clone());

                Ok(Some(vec![Mutation::substitute(node, new_name)?]))
            }
            "command_name" => {
                if let Some(new_name) = self.cache.get(node.text()?) {
                    Ok(Some(vec![Mutation::substitute(node, new_name.clone())?]))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }
    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

pub type VerbsRenameAllFunction<'a> = RenameAllFunction<'a, VerbsNameGenerator>;
pub type RandomRenameAllFunction<'a> = RenameAllFunction<'a, RandomNameGenerator>;

#[derive(Debug)]
pub struct RenameAllFunctionParam<'a, T> {
    generator: T,
    cache: HashMap<(String, String), String>,
    var_manager: &'a mut VarManager,
}

impl<'a, T: Default> RenameAllFunctionParam<'a, T> {
    pub fn new(var_manager: &'a mut VarManager) -> Self {
        Self {
            var_manager,
            cache: HashMap::new(),
            generator: T::default(),
        }
    }
}

impl<'a, T: NameGenerator + Default> Pass for RenameAllFunctionParam<'a, T> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "script_parameter" => {
                if let Some(function_node) = node.get_parent_of_types(vec!["function_statement"]) {
                    if let Some(function_name) = function_node
                        .iter()
                        .filter(|n| n.kind() == "function_name")
                        .next()
                    {
                        if let Some(var_node) =
                            node.iter().filter(|n| n.kind() == "variable").next()
                        {
                            let new_name = self.generator.generate(true);
                            self.cache.insert(
                                (
                                    var_node.text()?.to_string(),
                                    function_name.text()?.to_string(),
                                ),
                                new_name.clone(),
                            );
                            self.var_manager.push(new_name.clone());

                            return Ok(Some(vec![Mutation::substitute(&var_node, new_name)?]));
                        }
                    }
                }

                Ok(None)
            }
            "variable" => {
                if let Some(function_node) = node.get_parent_of_types(vec!["function_statement"]) {
                    if let Some(function_name) = function_node
                        .iter()
                        .filter(|n| n.kind() == "function_name")
                        .next()
                    {
                        if let Some(new_name) = self
                            .cache
                            .get(&(node.text()?.to_string(), function_name.text()?.to_string()))
                        {
                            return Ok(Some(vec![Mutation::substitute(node, new_name.clone())?]));
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

pub type VerbsRenameAllFunctionParam<'a> = RenameAllFunctionParam<'a, VerbsNameGenerator>;
pub type RandomRenameAllFunctionParam<'a> = RenameAllFunctionParam<'a, RandomNameGenerator>;
