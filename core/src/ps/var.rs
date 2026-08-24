use crate::Lang;
use crate::Lang::PowerShell;
use crate::Mutation;
use crate::error::DittoResult;
use crate::name::{CloseNameGenerator, NameGenerator, RandomNameGenerator, VerbsNameGenerator};
use crate::pass::Pass;
use crate::tree::Node;
use log::warn;
use rand::prelude::*;
use rand::rng;
use rand::seq::SliceRandom;
use regex::RegexBuilder;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VarManager {
    cache: Vec<String>,
}

impl VarManager {
    const MAX_VAR: usize = 4096;

    // Clean the variable name and add it to the cache
    pub fn push(&mut self, new_var: String) {
        self.cache.push(VarManager::extract_name(&new_var));
    }

    pub fn extract_name(var: &str) -> String {
        let mut chars = var.chars();
        if var.starts_with("$") {
            chars.next();
            let new_var = chars.as_str().to_string();
            if new_var.starts_with("{") && new_var.ends_with("}") {
                chars.next();
                chars.next_back();
            }
        }

        chars.collect()
    }
}

impl Default for VarManager {
    fn default() -> Self {
        Self {
            cache: vec![
                "args",
                "ConfirmPreference",
                "ConsoleFileName",
                "DebugPreference",
                "Error",
                "ErrorActionPreference",
                "ErrorView",
                "ExecutionContext",
                "false",
                "FormatEnumerationLimit",
                "HOME",
                "Host",
                "InformationPreference",
                "input",
                "MaximumAliasCount",
                "MaximumDriveCount",
                "MaximumErrorCount",
                "MaximumFunctionCount",
                "MaximumHistoryCount",
                "MaximumVariableCount",
                "MyInvocation",
                "NestedPromptLevel",
                "null",
                "OutputEncoding",
                "PID",
                "PROFILE",
                "ProgressPreference",
                "PSBoundParameters",
                "PSCommandPath",
                "PSCulture",
                "PSDefaultParameterValues",
                "PSEdition",
                "PSEmailServer",
                "PSHOME",
                "PSScriptRoot",
                "PSSessionApplicationName",
                "PSSessionConfigurationName",
                "PSSessionOption",
                "PSUICulture",
                "PSVersionTable",
                "PWD",
                "ShellId",
                "StackTrace",
                "true",
                "VerbosePreference",
                "WarningPreference",
                "WhatIfPreference",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }
}

#[derive(Debug)]
pub struct RenameAllVar<'a, T: Default> {
    generator: T,
    cache: HashMap<String, String>,
    var_manager: &'a mut VarManager,
}

impl<'a, T: Default> RenameAllVar<'a, T> {
    pub fn new(var_manager: &'a mut VarManager) -> Self {
        Self {
            generator: T::default(),
            cache: HashMap::new(),
            var_manager,
        }
    }
}

impl<'a, T: NameGenerator + Default> Pass for RenameAllVar<'a, T> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "variable" => {
                // FIXME:
                // Make difference between [Bytes]$test = ... and $test[$ok] = ...
                let name = node.text()?;

                if let Some(left) = node.get_parent_of_types(vec!["left_assignment_expression"]) {
                    let mut replace = false;

                    if let Some(cast) = node.get_parent_of_types(vec!["cast_expression"]) {
                        if let Some(casted) = cast.child(1) {
                            if casted.text() == Ok(name) {
                                replace = true;
                            }
                        }
                    }

                    if left.text() == Ok(name) {
                        replace = true;
                    }

                    if replace {
                        let new = self.generator.generate(true);
                        self.cache.insert(name.to_string(), new.clone());
                        self.var_manager.push(new)
                    }
                }

                if let Some(new_name) = self.cache.get(name) {
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

pub type VerbsRenameAllVar<'a> = RenameAllVar<'a, VerbsNameGenerator>;
pub type RandomRenameAllVar<'a> = RenameAllVar<'a, RandomNameGenerator>;
pub type CloseRenameAllVar<'a> = RenameAllVar<'a, CloseNameGenerator>;

pub trait VariableMask {
    fn declare_variable(&self, name: String, value: String) -> String;
    fn use_variable(&self, name: String, var_cache: Vec<String>) -> String;
}

#[derive(Default, Debug)]
pub struct NoMask {}
impl VariableMask for NoMask {
    fn declare_variable(&self, name: String, value: String) -> String {
        if name.chars().all(|c| c.is_alphanumeric()) {
            format!("${}={};", name, value)
        } else {
            format!("${{{}}}={};", name, value)
        }
    }
    fn use_variable(&self, name: String, _: Vec<String>) -> String {
        if name.chars().all(|c| c.is_alphanumeric()) {
            format!("${}", name)
        } else {
            format!("${{{}}}", name)
        }
    }
}

#[derive(Default, Debug)]
pub struct WildMask {}

impl<'a> WildMask {
    fn get_mask(arr: Vec<char>, regex_mask: bool) -> String {
        let wildcard = if regex_mask { ".*" } else { "*" };
        let mask = arr
            .iter()
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
            .to_string();

        if regex_mask {
            format!("^{}$", mask)
        } else {
            mask
        }
    }
}

impl VariableMask for WildMask {
    fn declare_variable(&self, name: String, value: String) -> String {
        format!("sv {} {};", name, value)
    }
    fn use_variable(&self, name: String, var_cache: Vec<String>) -> String {
        let min = 4;
        let size = name.len();
        if size < min {
            return name;
        };

        let name: Vec<char> = name.chars().collect();
        let mut masked = vec!['*'; size - 1]; // Fill with '*'
        masked.push(name[size - 1]);

        let mut permutation: Vec<usize> = (0..size - 1).into_iter().collect();
        permutation.shuffle(&mut rng());

        let mut c = 1;
        while let Some(idx) = permutation.pop() {
            masked[idx] = name[idx];
            c += 1;

            let r = RegexBuilder::new(&Self::get_mask(masked.clone(), true))
                .case_insensitive(true)
                .build()
                .unwrap();

            if c >= min && var_cache.iter().filter(|n| r.is_match(n)).count() == 1 {
                break;
            }
        }

        format!(
            "(({}{}).Value)",
            match rng().random_range(0..5) {
                0 => "LS VARIABLE:",
                1 => "LS VARIABLE:/",
                2 => "GCI VARIABLE:",
                3 => "GCI VARIABLE:/",
                _ => "VARIABLE ",
            },
            &Self::get_mask(masked, false)
        )
    }
}

pub enum LiteralType {
    INTEGER,
    STRING,
}

impl ToString for LiteralType {
    fn to_string(&self) -> String {
        match self {
            LiteralType::STRING => String::from("string_literal"),
            LiteralType::INTEGER => String::from("integer_literal"),
        }
    }
}

#[derive(Debug)]
pub struct VarAll<'a, T, U> {
    cache: HashMap<String, String>,
    param_block: bool,
    counter: usize,
    literal_type: Vec<String>,
    generator: T,
    mask: U,
    vars: Vec<String>,
    var_manager: &'a mut VarManager,
    vars_to_declare: Vec<String>,
}

impl<'a, T: Default, U: Default> VarAll<'a, T, U> {
    pub fn new(var_manager: &'a mut VarManager, types: Vec<LiteralType>) -> Self {
        Self {
            cache: HashMap::new(),
            param_block: false,
            counter: var_manager.cache.len(),
            mask: U::default(),
            literal_type: types.iter().map(|t| t.to_string()).collect(),
            var_manager,
            generator: T::default(),
            vars: vec![],
            vars_to_declare: vec![],
        }
    }
}

impl<'a, T: NameGenerator, U: VariableMask> Pass for VarAll<'a, T, U> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "program" => {
                let mut total_literals = self
                    .literal_type
                    .iter()
                    .map(|t| node.count_childs_of_type(t))
                    .sum();

                // TODO: We don't count really well the number of vars
                // Keep a margin of 3/4
                // TODO: Take into account SV/SetVariable command (used in ditto)
                if total_literals > VarManager::MAX_VAR * 3 / 4 {
                    warn!(
                        "The number of variables may exceed Powershell max: {}\n |  Some variable mutations might be skipped.",
                        VarManager::MAX_VAR
                    );
                    total_literals = VarManager::MAX_VAR * 3 / 4;
                }

                self.counter += node.count_childs_of_type("assignment_expression");
                self.vars = (0..total_literals)
                    .map(|_| {
                        let new = self.generator.generate(false);
                        self.var_manager.push(new.clone());
                        new
                    })
                    .collect();

                Ok(None)
            }
            "param_block" => {
                self.param_block = true;
                Ok(None)
            }
            x if self.literal_type.contains(&x.to_string())
                && self.counter < VarManager::MAX_VAR * 3 / 4 =>
            {
                if self.param_block {
                    return Ok(None);
                }

                let value = node.text()?;
                // TODO what if quoted and double quoted string?
                // Must replace double with single quote here?
                if let Some(var_name) = self.cache.get(value) {
                    return Ok(Some(vec![Mutation::substitute(
                        node,
                        self.mask
                            .use_variable(String::from(var_name), self.var_manager.cache.clone()),
                    )?]));
                } else {
                    self.counter += 1;
                    let var_name = self.vars.pop().unwrap();
                    self.cache.insert(value.to_string(), var_name.clone());

                    self.vars_to_declare.push(
                        self.mask
                            .declare_variable(var_name.clone(), String::from(value)),
                    );

                    return Ok(Some(vec![Mutation::substitute(
                        node,
                        self.mask
                            .use_variable(String::from(var_name), self.var_manager.cache.clone()),
                    )?]));
                }
            }
            _ => Ok(None),
        }
    }

    fn leave(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        match node.kind() {
            "param_block" => {
                self.param_block = false;
                Ok(None)
            }
            "program" => {
                self.vars_to_declare.shuffle(&mut rng());
                Ok(Some(vec![Mutation::global_statement(
                    node,
                    self.vars_to_declare.concat(),
                )?]))
            }
            _ => Ok(None),
        }
    }
}

pub type RandomVarAll<'a> = VarAll<'a, RandomNameGenerator, NoMask>;
pub type VerbsVarAll<'a> = VarAll<'a, VerbsNameGenerator, NoMask>;
pub type CloseVarAll<'a> = VarAll<'a, CloseNameGenerator, NoMask>;
pub type VerbsVarAllWithWildMask<'a> = VarAll<'a, VerbsNameGenerator, WildMask>;

#[derive(Default, Debug)]
pub struct VarCommand<T: Default> {
    generator: T,
}

impl<T: NameGenerator + Default> Pass for VarCommand<T> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "command_name_expr" {
            let var_name = self.generator.generate(true);

            return Ok(Some(vec![
                Mutation::global_statement(
                    node,
                    var_name.clone() + "=" + node.text().unwrap() + ";",
                )?,
                Mutation::substitute(node, var_name)?,
            ]));
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

pub type RandomVarCommand = VarCommand<RandomNameGenerator>;

#[derive(Default)]
pub struct VarParenthesisExpression<T: Default> {
    generator: T,
}

impl<T: NameGenerator + Default> Pass for VarParenthesisExpression<T> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "parenthesized_expression" && node.count_childs_of_type("variable") == 0 {
            let var_name = self.generator.generate(true);

            return Ok(Some(vec![
                Mutation::global_statement(
                    node,
                    var_name.clone() + "=" + node.text().unwrap() + ";",
                )?,
                Mutation::substitute(node, var_name)?,
            ]));
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

pub type RandomVarParenthesisExpression = VarParenthesisExpression<RandomNameGenerator>;

#[derive(Debug)]
pub struct VarType<'a, T: Default> {
    var_manager: &'a mut VarManager,
    cache: HashMap<String, String>,
    generator: T,
}

impl<'a, T: Default> VarType<'a, T> {
    pub fn new(var_manager: &'a mut VarManager) -> Self {
        Self {
            var_manager,
            cache: HashMap::new(),
            generator: T::default(),
        }
    }
}

impl<'a, T: NameGenerator + Default> Pass for VarType<'a, T> {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "type_literal" {
            if let Some(parent) = node.parent() {
                if (parent.kind() == "member_access" || parent.kind() == "invokation_expression")
                    && parent.child(0).unwrap() == *node
                    && node.count_childs_of_type("variable") == 0
                {
                    let type_name = String::from(node.text().unwrap());

                    if let Some(var_name) = self.cache.get(&type_name) {
                        return Ok(Some(vec![Mutation::substitute(node, var_name.clone())?]));
                    } else {
                        let var_name = self.generator.generate(true);

                        self.cache.insert(type_name, var_name.clone());
                        self.var_manager.push(var_name.clone());

                        return Ok(Some(vec![
                            Mutation::global_statement(
                                node,
                                var_name.clone() + "=" + node.text()? + ";",
                            )?,
                            Mutation::substitute(node, var_name)?,
                        ]));
                    }
                }
            }
        }
        Ok(None)
    }

    fn leave(&mut self, _node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

pub type RandomVarType<'a> = VarType<'a, RandomNameGenerator>;
pub type VerbVarType<'a> = VarType<'a, VerbsNameGenerator>;

pub struct CountVar {
    count: usize,
}

impl Pass for CountVar {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        PowerShell
    }
    fn enter(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "assignement_expression" {
            self.count += 1;
        }
        Ok(None)
    }
    fn leave(&mut self, node: &Node) -> DittoResult<Option<Vec<Self::Mutation>>> {
        if node.kind() == "program" {
            if self.count > VarManager::MAX_VAR {
                // raise error
                ()
            }
        }
        Ok(None)
    }
}
