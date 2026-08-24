use crate::Lang;
use crate::Lang::PowerShell;
use crate::error::DittoResult;
use crate::impostor::Impostor;
use crate::pass::Source;
use crate::ps::array::*;
use crate::ps::bool::*;
use crate::ps::encode::*;
use crate::ps::expression::*;
use crate::ps::function::*;
use crate::ps::integer::*;
use crate::ps::stringify::*;
use crate::ps::strings::*;
use crate::ps::var::*;
use rand::prelude::*;

#[derive(Default)]
pub struct Alakazam;

impl Impostor for Alakazam {
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        let mut var_manager = VarManager::default();
        Ok(Source::new(&source)
            .transform(AsStaticString::default())?
            .transform(ComputeAllBool::default())?
            .transform(BreakInvokationExpressions::default())?
            .transform(CmdletAsString::default())?
            .transform(MemberAsStaticString::default())?
            .transform(TypeAsString::default())?
            .transform(ModuloInlineHexlify::default())?
            .transform(ComputeAllInt::default())?
            .transform(CloseVarAll::new(
                &mut var_manager,
                vec![LiteralType::INTEGER],
            ))?
            .output()?)
    }
}

#[derive(Default)]
pub struct Eevee;

impl Impostor for Eevee {
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        let mut var_manager = VarManager::default();
        Ok(Source::new(&source)
            .transform(RandomRenameAllFunctionParam::new(&mut var_manager))?
            .transform(RandomRenameAllVar::new(&mut var_manager))?
            .transform(RandomRenameAllFunction::new(&mut var_manager))?
            .transform(AsStaticString::default())?
            .transform(CmdletAsString::default())?
            .transform(MemberAsStaticString::default())?
            .transform(RandomVarAll::new(
                &mut var_manager,
                vec![LiteralType::STRING],
            ))?
            .transform(UUIDStr::default())?
            .transform(CmdletAsString::default())?
            .transform(MemberAsStaticString::default())?
            .transform(VerbVarType::new(&mut var_manager))?
            .transform(VerbsVarAllWithWildMask::new(
                &mut var_manager,
                vec![LiteralType::STRING, LiteralType::INTEGER],
            ))?
            .output()?)
    }
}

#[derive(Default)]
pub struct Onix;

impl Impostor for Onix {
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        let mut var_manager = VarManager::default();
        Ok(Source::new(&source)
            .transform(AsStaticString::default())?
            .transform(CmdletAsString::default())?
            .transform(MemberAsStaticString::default())?
            .transform(RandomVarAll::new(
                &mut var_manager,
                vec![LiteralType::STRING],
            ))?
            .transform(RandomVarAll::new(
                &mut var_manager,
                vec![LiteralType::INTEGER],
            ))?
            .transform(VerbsArrayAllConst::default())?
            .transform(ModuloForEachHexlify::default())?
            .transform(ShuffleArray::default())?
            .transform(VerbsApplyEvalXorAddToAllInt::default())?
            .output()?)
    }
}

#[derive(Default)]
pub struct Pikachu;

impl Impostor for Pikachu {
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        let mut var_manager = VarManager::default();
        Ok(Source::new(&source)
            .transform(AsStaticString::default())?
        .transform(CmdletAsString::default())?
        .transform(RandomVarCommand::default())?
        .transform(Base64Str::default())?
        .transform(RandomVarType::new(&mut var_manager))?
        .transform(MemberAsStaticString::default())?
        .transform(RandomVarAll::new(
            &mut var_manager,
            vec![LiteralType::STRING],
        ))?
        .transform(ModuloInlineHexlify::default())?
        .transform(RandomVarAll::new(
            &mut var_manager,
            vec![LiteralType::INTEGER],
        ))?
        .output()?)
}
}

#[derive(Default)]
pub struct Bulbasaur;

impl Impostor for Bulbasaur {
fn lang(&self) -> Lang {
    PowerShell
}

fn transform(&self, source: &str) -> DittoResult<String> {
    let mut var_manager = VarManager::default();
    Ok(Source::new(&source)
        .transform(AsStaticString::default())?
        .transform(CmdletAsString::default())?
        .transform(RandomVarCommand::default())?
        .transform(Base64Str::default())?
        .transform(MemberAsStaticString::default())?
        .transform(RandomVarType::new(&mut var_manager))?
        .transform(RandomVarAll::new(
            &mut var_manager,
            vec![LiteralType::STRING],
        ))?
        .transform(ModuloReverse::default())?
        .output()?)
}
}

#[derive(Default)]
pub struct Charizard;

impl Impostor for Charizard {
fn lang(&self) -> Lang {
    PowerShell
}

fn transform(&self, source: &str) -> DittoResult<String> {
    let mut var_manager = VarManager::default();
    Ok(Source::new(&source)
        .transform(AsStaticString::default())?
        .transform(CmdletAsString::default())?
        .transform(Base64Str::default())?
        .transform(ModuloInlineHexlify::default().modulo(rand::rng().random_range(1..3)))?
        .transform(MemberAsStaticString::default())?
        .transform(RandomVarAll::new(
            &mut var_manager,
            vec![LiteralType::STRING],
        ))?
        .transform(ModuloReverse::default())?
        .transform(RandomVarType::new(&mut var_manager))?
        .transform(RandomVarAll::new(
            &mut var_manager,
            vec![LiteralType::INTEGER],
        ))?
        .output()?)
}
}

#[derive(Default)]
pub struct Ivysaur;

impl Impostor for Ivysaur {
fn lang(&self) -> Lang {
    PowerShell
}

fn transform(&self, source: &str) -> DittoResult<String> {
    let mut var_manager = VarManager::default();
    Ok(Source::new(&source)
        .transform(AsStaticString::default())?
        .transform(CmdletAsString::default())?
        .transform(Base64Str::default())?
        .transform(ModuloInlineHexlify::default().modulo(rand::rng().random_range(1..2)))?
        .transform(MemberAsStaticString::default())?
        .transform(VerbsVarAll::new(
            &mut var_manager,
            vec![LiteralType::STRING],
        ))?
        .transform(ModuloReverse::default())?
        .transform(RandomVarType::new(&mut var_manager))?
        .transform(VerbsVarAll::new(
            &mut var_manager,
            vec![LiteralType::INTEGER],
        ))?
        .output()?)
}
}
