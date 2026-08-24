use crate::Lang;
use crate::Lang::JavaScript;
use crate::error::DittoResult;
use crate::impostor::Impostor;
use crate::js::array::*;
use crate::js::curse::*;
use crate::js::float::*;
use crate::js::global_this::GlobalThis;
use crate::js::identifier::*;
use crate::js::integer::*;
use crate::js::jsfuck::*;
use crate::js::member::*;
use crate::js::string::*;
use crate::js::switch::*;
use crate::js::utils::identifier::*;
use crate::pass::Source;

#[derive(Default)]
pub struct Tinkatuff;

impl Impostor for Tinkatuff {
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        Ok(Source::new(&source)
            .transform(GlobalThis::default())?
            .transform(IdentifierRenamer::new(AlphabetNameGenerator::new(5, 7)))?
            .transform_with_probability(ExpressionToSwitchCase::default(), 1.0 / 7.0)?
            .transform_with_probability(ExpressionToIfElse::default(), 1.0 / 7.0)?
            .transform(IntegerToExpression::new(6, IntegerExpressionMode::WithMath))?
            .transform(FloatToExpression::new(8, FloatExpressionMode::WithMath))?
            .transform(MemberToSubscript::default())?
            .transform(ArrayTransform::default())?
            .transform(StringEncode::default())?
            .transform(StringSplit::new(Some(1), Some(3)))?
            .transform(StringTransform::default())?
            .transform(StringEscape::default())?
            .transform(IdentifierNameEscape::default())?
            .output()?)
    }
}

#[derive(Default)]
pub struct Unown;

impl Impostor for Unown {
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        Ok(Source::new(&source)
            .transform(MemberToSubscript::default())?
            .transform(JsFuckNumbers::default())?
            .transform(JsFuckStrings::default())?
            .transform(JsFuckBooleans::default())?
            .transform(JsFuckSpecials::default())?
            .transform(JsFuckDefaultObjects::default())?
            .transform(IdentifierRenamer::new(
                DollarUnderscoreNameGenerator::default(),
            ))?
            .output()?)
    }
}

/// Wait, it's all numbers?
/// Always has been...
#[derive(Default)]
pub struct Metagross;

impl Impostor for Metagross {
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        Ok(Source::new(&source)
            .transform(MemberToSubscript::default())?
            .transform(StringTransform::new(vec![
                StringTransformFunction::FromCharCode,
            ]))?
            .transform(IntegerToExpression::new(2, IntegerExpressionMode::NoMath))?
            .transform(IntegerToExpression::new(2, IntegerExpressionMode::MathOnly))?
            .transform(FloatToExpression::new(3, FloatExpressionMode::NoMath))?
            .transform(FloatToExpression::new(2, FloatExpressionMode::MathOnly))?
            .transform(IdentifierRenamer::new(NumberNameGenerator::default()))?
            .output()?)
    }
}

/// Produce a **MASSIVE** output. Only use it for small scripts, the ratio is on average x4643.7
#[derive(Default)]
pub struct Wailord;

impl Impostor for Wailord {
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        Ok(Source::new(&source)
            .transform(IdentifierRenamer::new(BubbleNameGenerator::new(6, 9)))?
            .transform(MemberToSubscript::default())?
            .transform(StringSplit::new(Some(1), Some(3)))?
            .transform(StringTransform::default())?
            .transform(StringEscape::default())?
            .transform(IntegerToExpression::new(5, IntegerExpressionMode::WithMath))?
            .transform(FloatToExpression::new(5, FloatExpressionMode::WithMath))?
            .transform(JsFuckNumbers::default())?
            .transform(ArrayTransform::default())?
            .transform(MemberToSubscript::default())?
            .transform(StringSplit::new(Some(1), Some(3)))?
            .output()?)
    }
}

/// Basically ```use rand::prelude::*;```<br>
/// It will pick randomly pick in all the rules so you'll have a "small" output while having all the protections.
/// Someone trying to reverse your script will need to handle all the possible `Pass`.<br>
/// You'll probably need to test it multiple times because some rules can infer with others
#[derive(Default)]
pub struct Clefable;

impl Impostor for Clefable {
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        Ok(Source::new(&source)
            .transform_with_probability(MemberToSubscript::default(), 0.2)?
            .transform(IdentifierRenamer::new(AlphabetNameGenerator::new(5, 7)))?
            .transform(StringSplit::new(Some(1), Some(2)))?
            .transform_with_probability(StringTransform::default(), 0.2)?
            .transform_with_probability(ArrayTransform::default(), 0.2)?
            .transform_with_probability(StringEncode::default(), 0.2)?
            .transform_with_probability(CurseString::default(), 0.2)?
            .transform_with_probability(
                FloatToExpression::new(2, FloatExpressionMode::WithMath),
                0.2,
            )?
            .transform_with_probability(
                IntegerToExpression::new(2, IntegerExpressionMode::WithMath),
                0.2,
            )?
            .transform_with_probability(CurseNumber::default(), 0.2)?
            .transform(StringEscape::default())?
            .transform(IdentifierNameEscape::new(
                15,
                vec![
                    IdentifierType::Identifier,
                    IdentifierType::MemberIdentifier,
                    IdentifierType::PropertyIdentifier,
                ],
            ))?
            .transform_with_probability(JsFuckNumbers::default(), 0.2)?
            .transform_with_probability(JsFuckStrings::new(3), 0.2)?
            .transform_with_probability(JsFuckBooleans::default(), 0.2)?
            .transform_with_probability(JsFuckSpecials::default(), 0.2)?
            .transform_with_probability(JsFuckDefaultObjects::default(), 0.2)?
            // mess up with jsfuck deobfuscators
            .transform_with_probability(ArrayTransform::default(), 0.01)?
            .output()?)
    }
}

/// 𓉔𓇋𓅂𓂋𓅱𓎼𓃭𓇌𓊪𓉔𓋴 𓅂𓆯𓅂𓂋𓇌𓅃𓉔𓅂𓂋𓅂
#[derive(Default)]
pub struct Cofagrigus;

impl Impostor for Cofagrigus {
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        Ok(Source::new(&source)
            .transform(MemberToSubscript::default())?
            .transform(StringEncode::default())?
            .transform(IdentifierRenamer::new(CurseOfTheNileNameGenerator::new(
                24, 32,
            )))?
            .transform(IntegerToExpression::new(2, IntegerExpressionMode::NoMath))?
            .transform(FloatToExpression::new(2, FloatExpressionMode::NoMath))?
            .transform(StringTransform::new(vec![
                StringTransformFunction::Slice(128),
                StringTransformFunction::Substr(128),
            ]))?
            .transform(CurseNumber::default())?
            .transform(CurseString::default())?
            .output()?)
    }
}
