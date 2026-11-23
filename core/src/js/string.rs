use crate::Lang;
use crate::Lang::JavaScript;
use crate::Mutation;
use crate::error::Error;
use crate::js::string::StringEncodeFunction::*;
use crate::js::string::StringTransformFunction::*;
use crate::js::utils::regex::create_replace_regex;
use crate::js::utils::string::*;
use crate::pass::Pass;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use log::trace;
use rand::prelude::*;
use rand::rng;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug)]
pub struct StringSplit {
    pub min_size: Option<usize>,
    pub max_size: Option<usize>,
}

impl StringSplit {
    pub fn new(min_size: Option<usize>, max_size: Option<usize>) -> Self {
        Self { min_size, max_size }
    }
}

impl Default for StringSplit {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl Pass for StringSplit {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if node.kind() != "string" {
            return Ok(None);
        }

        if let Some(parent) = node.parent() {
            if parent.kind() == "pair" {
                return Ok(None);
            }
        }

        if is_directive_string(node) {
            return Ok(None);
        }

        let value = unescape_js_string(node.text()?);
        if value.len() < 2 {
            return Ok(None);
        }

        let min_size = self.min_size.unwrap_or(1).max(1);
        let max_size = self.max_size.unwrap_or(value.len() - 1).max(min_size);

        let parts = randomly_cut_string(min_size, max_size, value.as_str(), true);

        if parts.len() <= 1 {
            return Ok(None);
        }

        let replacement = format!("({})", parts.join(" + "));
        Ok(Some(vec![Mutation::substitute(node, replacement)?]))
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}

#[derive(Debug)]
pub struct StringTransform {
    pub functions: Vec<StringTransformFunction>,
}

#[derive(Debug, EnumIter, PartialEq)]
pub enum StringTransformFunction {
    ToUpperCase,
    ToLowerCase,
    Concat,
    Slice(usize),  // max garbage length
    Substr(usize), // max garbage length
    ArrayJoin { min_size: usize, max_size: usize },
    Trim(usize),      // max garbage length
    TrimStart(usize), // max garbage length
    TrimEnd(usize),   // max garbage length
    Replace { min_size: usize, max_size: usize },
    ReplaceAll { min_size: usize, max_size: usize },
    ToString,
    Tags,
    SpecialTags,
    FromCharCode,
    StringWrap,
    NewString,
    StringRaw,
}

impl StringTransform {
    pub fn new(functions: Vec<StringTransformFunction>) -> Self {
        Self { functions }
    }
}

impl Default for StringTransform {
    //noinspection RsUnresolvedPath
    fn default() -> Self {
        let mut functions = vec![];
        for func in StringTransformFunction::iter() {
            functions.push(func);
        }
        Self::new(functions)
    }
}

impl Pass for StringTransform {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if node.kind() != "string" {
            return Ok(None);
        }

        if let Some(parent) = node.parent() {
            if parent.kind() == "pair" {
                return Ok(None);
            }
        }

        if is_directive_string(node) {
            return Ok(None);
        }

        let value = unescape_js_string(node.text()?);

        let mut valid_fns = vec![];
        for func in &self.functions {
            match func {
                ToLowerCase if is_lowercase(value.as_str()) => {
                    valid_fns.push(ToLowerCase);
                }
                ToUpperCase if is_uppercase(value.as_str()) => {
                    valid_fns.push(ToUpperCase);
                }
                Concat if value.len() > 1 => {
                    valid_fns.push(Concat);
                }
                Slice(strength) => {
                    valid_fns.push(Slice(*strength));
                }
                Substr(strength) => {
                    valid_fns.push(Substr(*strength));
                }
                ArrayJoin { min_size, max_size } => {
                    valid_fns.push(ArrayJoin {
                        min_size: *min_size,
                        max_size: *max_size,
                    });
                }
                Trim(max_size) => {
                    if TRIM_CHARS
                        .iter()
                        .any(|c| value.starts_with(*c) || value.ends_with(*c))
                    {
                        continue;
                    }
                    valid_fns.push(Trim(*max_size));
                }
                TrimStart(max_size) => {
                    if TRIM_CHARS.iter().any(|c| value.starts_with(*c)) {
                        continue;
                    }
                    valid_fns.push(TrimStart(*max_size));
                }
                TrimEnd(max_size) => {
                    if TRIM_CHARS.iter().any(|c| value.ends_with(*c)) {
                        continue;
                    }
                    valid_fns.push(TrimEnd(*max_size));
                }
                Tags => {
                    valid_fns.push(Tags);
                }
                SpecialTags => {
                    valid_fns.push(SpecialTags);
                }
                ToString => {
                    valid_fns.push(ToString);
                }
                Replace { min_size, max_size } => {
                    valid_fns.push(Replace {
                        min_size: *min_size,
                        max_size: *max_size,
                    });
                }
                ReplaceAll { min_size, max_size } => {
                    valid_fns.push(ReplaceAll {
                        min_size: *min_size,
                        max_size: *max_size,
                    });
                }
                FromCharCode => {
                    valid_fns.push(FromCharCode);
                }
                StringWrap => {
                    valid_fns.push(StringWrap);
                }
                NewString => {
                    valid_fns.push(NewString);
                }
                StringRaw => {
                    valid_fns.push(StringRaw);
                }
                _ => {}
            }
        }

        if valid_fns.is_empty() {
            return Ok(None);
        }

        let mut rng = rng();
        let func = valid_fns.choose(&mut rng).unwrap();

        match func {
            ToUpperCase => {
                let replacement =
                    escape_js_string(randomize_case(value.as_str(), rng).as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.toUpperCase()", replacement),
                )?]))
            }
            ToLowerCase => {
                let replacement =
                    escape_js_string(randomize_case(value.as_str(), rng).as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.toLowerCase()", replacement),
                )?]))
            }
            Concat => {
                let half = value.len() / 2;
                let left = escape_js_string(&value[..half], true);
                let right = escape_js_string(&value[half..], true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.concat('', {})", left, right),
                )?]))
            }
            Slice(strength) => {
                let strength = if *strength <= 1 { 2 } else { *strength };
                let left_noise = generate_noise(rng.random_range(1..strength), &mut rng);
                let right_noise = generate_noise(rng.random_range(1..strength), &mut rng);
                let left_idx = left_noise.len();
                let right_idx = -(right_noise.len() as isize);
                let noisy_str = escape_js_string(
                    format!("{}{}{}", left_noise, value, right_noise).as_str(),
                    true,
                );
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.slice({}, {})", noisy_str, left_idx, right_idx),
                )?]))
            }
            Substr(strength) => {
                let strength = if *strength <= 1 { 2 } else { *strength };
                let left_noise = generate_noise(rng.random_range(1..strength), &mut rng);
                let right_noise = generate_noise(rng.random_range(1..strength), &mut rng);
                let left_idx = left_noise.len();
                let right_idx = value.len();
                let noisy_str = escape_js_string(
                    format!("{}{}{}", left_noise, value, right_noise).as_str(),
                    true,
                );
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.substr({}, {})", noisy_str, left_idx, right_idx),
                )?]))
            }
            ArrayJoin { min_size, max_size } => {
                let min_size = if *min_size <= 0 { 1 } else { *min_size };
                let max_size = if *max_size <= 0 { 5 } else { *max_size };

                let parts = randomly_cut_string(min_size, max_size, value.as_str(), true);

                if parts.len() <= 1 {
                    return Ok(None);
                }

                let replacement = format!("[{}].join('')", parts.join(", "));
                Ok(Some(vec![Mutation::substitute(node, replacement)?]))
            }
            Trim(max_size) => {
                let max_size = if *max_size <= 0 { 5 } else { *max_size };
                let left_noise = generate_trim_noise(rng.random_range(0..max_size), &mut rng);
                let right_noise = generate_trim_noise(rng.random_range(0..max_size), &mut rng);
                let replacement = escape_js_string(
                    format!("{}{}{}", left_noise, value, right_noise).as_str(),
                    true,
                );
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.trim()", replacement),
                )?]))
            }
            TrimStart(max_size) => {
                let max_size = if *max_size <= 0 { 5 } else { *max_size };
                let noise = generate_trim_noise(rng.random_range(0..max_size), &mut rng);
                let replacement = escape_js_string(format!("{}{}", noise, value).as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.trimStart()", replacement),
                )?]))
            }
            TrimEnd(max_size) => {
                let max_size = if *max_size <= 0 { 5 } else { *max_size };
                let noise = generate_trim_noise(rng.random_range(0..max_size), &mut rng);
                let replacement = escape_js_string(format!("{}{}", value, noise).as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.trimEnd()", replacement),
                )?]))
            }
            Tags => {
                let tags = vec![
                    ("big", "big"),
                    ("blink", "blink"),
                    ("bold", "b"),
                    ("fixed", "tt"),
                    ("italics", "i"),
                    ("small", "small"),
                    ("strike", "strike"),
                    ("sub", "sub"),
                    ("sup", "sup"),
                ];

                let tag = tags.choose(&mut rng).unwrap();
                let left_idx = tag.1.len() + 2;
                let right_idx = -(tag.1.len() as isize + 3);
                let replacement = format!(
                    "{}.{}().slice({}, {})",
                    escape_js_string(value.as_str(), true),
                    tag.0,
                    left_idx,
                    right_idx
                );
                Ok(Some(vec![Mutation::substitute(node, replacement)?]))
            }
            SpecialTags => {
                let tag = vec!["anchor", "link"].choose(&mut rng).unwrap().to_owned();
                let replacement = format!(
                    "''.{}({}).slice(9, -6)",
                    tag,
                    escape_js_string(value.as_str(), true)
                );
                Ok(Some(vec![Mutation::substitute(node, replacement)?]))
            }
            ToString => {
                let replacement = escape_js_string(value.as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.toString()", replacement),
                )?]))
            }
            Replace { min_size, max_size } => {
                let (pattern, replacement) =
                    match create_replace_regex(value.as_str(), *min_size, *max_size, &mut rng) {
                        None => return Ok(None),
                        Some((pattern, replacement)) => (pattern, replacement),
                    };

                let replacement = escape_js_string(replacement.as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    // adding g flag makes it "global" so it act as a replaceAll
                    format!("{}.replace(/{}/g, '')", replacement, pattern),
                )?]))
            }
            ReplaceAll { min_size, max_size } => {
                let (pattern, replacement) =
                    match create_replace_regex(value.as_str(), *min_size, *max_size, &mut rng) {
                        None => return Ok(None),
                        Some((pattern, replacement)) => (pattern, replacement),
                    };

                let replacement = escape_js_string(replacement.as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("{}.replaceAll(/{}/g, '')", replacement, pattern),
                )?]))
            }
            FromCharCode => {
                let char_codes: Vec<String> =
                    value.chars().map(|c| (c as u16).to_string()).collect();
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("String.fromCharCode({})", char_codes.join(", ")),
                )?]))
            }
            StringWrap => Ok(Some(vec![Mutation::substitute(
                node,
                format!("String({})", escape_js_string(value.as_str(), true)),
            )?])),
            NewString => Ok(Some(vec![Mutation::substitute(
                node,
                format!(
                    "new String({}).valueOf()",
                    escape_js_string(value.as_str(), true)
                ),
            )?])),
            StringRaw => {
                let replacement = escape_js_string(value.as_str(), false).replace("`", "\\`");
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("String.raw`{}`", replacement),
                )?]))
            }
        }
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}

#[derive(Debug)]
pub struct StringEscape {
    pub types_probabilities: Vec<EscapeType>,
}

#[derive(Debug, PartialEq)]
pub enum EscapeType {
    Hex(u8),
    Unicode(u8),
    LongUnicode(u8),
}

impl StringEscape {
    pub fn new(types_probabilities: Vec<EscapeType>) -> Self {
        Self {
            types_probabilities,
        }
    }
}

impl Default for StringEscape {
    fn default() -> Self {
        let types_probabilities = vec![
            EscapeType::Hex(25),
            EscapeType::Unicode(25),
            EscapeType::LongUnicode(25),
        ];
        Self::new(types_probabilities)
    }
}

impl Pass for StringEscape {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if node.kind() != "string" {
            return Ok(None);
        }

        if let Some(parent) = node.parent() {
            if parent.kind() == "pair" {
                return Ok(None);
            }
        }

        if is_directive_string(node) {
            return Ok(None);
        }

        let value = unescape_js_string(node.text()?);
        if value.is_empty() {
            return Ok(None);
        }

        let mut sum_percent: u32 = 0;
        for escape_type in &self.types_probabilities {
            match escape_type {
                EscapeType::Hex(p) => sum_percent += *p as u32,
                EscapeType::Unicode(p) => sum_percent += *p as u32,
                EscapeType::LongUnicode(p) => sum_percent += *p as u32,
            }
        }

        if sum_percent > 100 {
            let percent: f32 = sum_percent as f32 / 100.0;
            return Err(Error::InvalidProbability(percent));
        }

        let mut rng = rng();

        let result: String = value
            .chars()
            .map(|c| {
                let code = c as u32;
                let percentage = rng.random_range(1u32..=100u32);

                let mut threshold: u32 = 0;

                for escape_type in &self.types_probabilities {
                    match escape_type {
                        EscapeType::Hex(p) => {
                            threshold += *p as u32;
                            if percentage <= threshold && code <= 0xFF {
                                return format!("\\x{:02X}", code);
                            }
                        }
                        EscapeType::Unicode(p) => {
                            threshold += *p as u32;
                            if percentage <= threshold && code <= 0xFFFF {
                                return format!("\\u{:04X}", code);
                            }
                        }
                        EscapeType::LongUnicode(p) => {
                            threshold += *p as u32;
                            if percentage <= threshold {
                                let random_useless_zeroes = rng.random_range(1..5);
                                let mut zeroes = String::new();
                                for _ in 0..random_useless_zeroes {
                                    zeroes.push('0');
                                }
                                return format!("\\u{{{}{:04X}}}", zeroes, code);
                            }
                        }
                    }
                }

                escape_js_string(c.to_string().as_str(), false)
            })
            .collect();

        let replacement = format!("'{}'", result);
        trace!("Escaped string '{}' into {}", value, replacement);
        Ok(Some(vec![Mutation::substitute(node, replacement)?]))
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}

#[derive(Debug)]
pub struct StringEncode {
    pub functions: Vec<StringEncodeFunction>,
}

#[derive(Debug, EnumIter, PartialEq)]
pub enum StringEncodeFunction {
    Escape,
    EncodeURI,
    EncodeURIComponent,
    Base64, // btoa
}

impl StringEncode {
    pub fn new(functions: Vec<StringEncodeFunction>) -> Self {
        Self { functions }
    }
}

impl Default for StringEncode {
    //noinspection RsUnresolvedPath
    fn default() -> Self {
        let mut functions = vec![];
        for func in StringEncodeFunction::iter() {
            functions.push(func);
        }
        Self::new(functions)
    }
}

impl Pass for StringEncode {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if node.kind() != "string" {
            return Ok(None);
        }

        if let Some(parent) = node.parent() {
            if parent.kind() == "pair" {
                return Ok(None);
            }
        }

        if is_directive_string(node) {
            return Ok(None);
        }

        let value = unescape_js_string(node.text()?);

        let mut valid_fns = vec![];
        for func in &self.functions {
            match func {
                Escape => valid_fns.push(Escape),
                EncodeURI => valid_fns.push(EncodeURI),
                EncodeURIComponent => valid_fns.push(EncodeURIComponent),
                Base64 if value.is_ascii() => valid_fns.push(Base64),
                _ => {}
            }
        }

        if valid_fns.is_empty() {
            return Ok(None);
        }

        let func = valid_fns.choose(&mut rng()).unwrap();
        trace!("Using encode function {:?}", func);

        match func {
            Escape => {
                let replacement = escape_js_string(escape(value.as_str(), true).as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("unescape({})", replacement),
                )?]))
            }
            EncodeURI => {
                let replacement = escape_js_string(encode_uri(value.as_str(), true).as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("decodeURI({})", replacement),
                )?]))
            }
            EncodeURIComponent => {
                let replacement =
                    escape_js_string(encode_uri_component(value.as_str(), true).as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("decodeURIComponent({})", replacement),
                )?]))
            }
            Base64 => {
                let replacement = escape_js_string(BASE64_STANDARD.encode(value).as_str(), true);
                Ok(Some(vec![Mutation::substitute(
                    node,
                    format!("atob({})", replacement),
                )?]))
            }
        }
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}
