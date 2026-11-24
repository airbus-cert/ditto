use crate::tree::Node;
use log::{trace, warn};
use rand::prelude::*;
use rand::{Rng, rng};
use std::cmp::min;
use std::collections::HashMap;

pub fn is_directive_string(node: &Node) -> bool {
    node.parent()
        .map(|parent| parent.kind() == "expression_statement")
        .unwrap_or(false)
}

pub fn unescape_js_string(s: &str) -> String {
    if s.len() < 2 {
        return s.to_string();
    }

    let mut result = String::new();
    let mut chars = s[1..s.len() - 1].chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    'u' => {
                        let mut hex = String::new();
                        if let Some('{') = chars.peek() {
                            chars.next(); // consume '{'
                            for h in chars.by_ref() {
                                if h == '}' {
                                    break;
                                }
                                hex.push(h);
                            }
                        } else {
                            for _ in 0..4 {
                                if let Some(h) = chars.next() {
                                    hex.push(h);
                                } else {
                                    warn!("ParseString: incomplete unicode escape sequence");
                                    break;
                                }
                            }
                        }

                        if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = std::char::from_u32(code_point) {
                                result.push(ch);
                            } else {
                                warn!("ParseString: invalid unicode code point: {}", hex);
                            }
                        } else {
                            warn!("ParseString: invalid unicode escape sequence: {}", hex);
                        }
                    }
                    'x' => {
                        let mut hex = String::new();
                        for _ in 0..2 {
                            if let Some(h) = chars.next() {
                                hex.push(h);
                            } else {
                                warn!("ParseString: incomplete hex escape sequence");
                                break;
                            }
                        }
                        if let Ok(code_point) = u8::from_str_radix(&hex, 16) {
                            result.push(code_point as char);
                        } else {
                            warn!("ParseString: invalid hex escape sequence: {}", hex);
                        }
                    }
                    _ => {
                        warn!("ParseString: unrecognized escape sequence: \\{}", next);
                        result.push(next);
                    }
                }
            } else {
                warn!("ParseString: trailing backslash in string literal");
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub fn escape_js_string(s: &str, quote: bool) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        match c {
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            '\\' => escaped.push_str("\\\\"),
            '\'' => escaped.push_str("\\'"),
            c if (c as u32) < 0x20 => escaped.push_str(&format!("\\x{:02x}", c as u32)),
            c => escaped.push(c),
        }
    }

    if quote {
        format!("'{}'", escaped)
    } else {
        escaped
    }
}

pub fn is_lowercase(str: &str) -> bool {
    str.chars().all(|c| {
        c.is_lowercase()
            || c.is_whitespace()
            || c.is_ascii_punctuation()
            || c == '_'
            || c.is_ascii_digit()
    })
}

pub fn is_uppercase(str: &str) -> bool {
    str.chars().all(|c| {
        c.is_uppercase()
            || c.is_whitespace()
            || c.is_ascii_punctuation()
            || c == '_'
            || c.is_ascii_digit()
    })
}

pub fn randomize_case(str: &str, mut rng: impl Rng) -> String {
    str.chars()
        .map(|c| {
            if rng.random_bool(0.5) {
                c.to_uppercase().next().unwrap()
            } else {
                c.to_lowercase().next().unwrap()
            }
        })
        .collect()
}

const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789)(*&^%$#@!~";
const SAFE_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const ALPHABET_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

pub const TRIM_CHARS: &[char] = &[' ', '\t', '\n', '\r'];

pub fn generate_noise(n: usize, rng: &mut dyn Rng) -> String {
    generate_noise_from(CHARSET, n, rng)
}

pub fn generate_safe_noise(n: usize, rng: &mut dyn Rng) -> String {
    generate_noise_from(SAFE_CHARSET, n, rng)
}

pub fn generate_trim_noise(n: usize, rng: &mut dyn Rng) -> String {
    generate_noise_from(TRIM_CHARS, n, rng)
}

pub fn generate_alphabetic_noise(n: usize, rng: &mut dyn Rng) -> String {
    generate_noise_from(ALPHABET_CHARSET, n, rng)
}

pub fn generate_noise_from<T: Copy + Into<char>>(
    charset: &[T],
    n: usize,
    rng: &mut dyn Rng,
) -> String {
    let noise = (0..n)
        .map(|_| charset[rng.random_range(0..charset.len())].into())
        .collect();
    trace!("Generated noise: {:?}", noise);
    noise
}

pub fn randomly_cut_string(
    min_size: usize,
    max_size: usize,
    str: &str,
    escape: bool,
) -> Vec<String> {
    let mut parts = vec![];
    let mut rng = rng();
    let chars: Vec<char> = str.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let remaining = chars.len() - i;
        let upper = min(max_size, remaining);
        let lower = min(min_size, upper);
        let take = if lower == upper {
            lower
        } else {
            rng.random_range(lower..=upper)
        };

        let piece: String = chars[i..i + take].iter().collect();
        if escape {
            let escaped = escape_js_string(&piece, true);
            parts.push(escaped);
        } else {
            parts.push(piece);
        }
        i += take;
    }
    parts
}

pub fn encode_uri_component(input: &str, encode_all: bool) -> String {
    if encode_all {
        encode_uri_set(input, |_| false)
    } else {
        encode_uri_set(input, |b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')'
                )
        })
    }
}

pub fn encode_uri(input: &str, encode_all: bool) -> String {
    if encode_all {
        encode_uri_set(input, |_| false)
    } else {
        encode_uri_set(input, |b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')' // unreserved
                | b';' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b',' | b'#' // reserved + #
                )
        })
    }
}

fn encode_uri_set(input: &str, keep: impl Fn(u8) -> bool) -> String {
    let mut out = String::with_capacity(input.len());
    for &byte in input.as_bytes() {
        if keep(byte) {
            out.push(byte as char);
        } else {
            out.push_str(format!("%{:02X}", byte).as_str())
        }
    }
    out
}

pub fn escape(input: &str, escape_all: bool) -> String {
    fn keep(c: u16) -> bool {
        matches!(c,
            0x30..=0x39   // 0-9
            | 0x41..=0x5A // A-Z
            | 0x61..=0x7A // a-z
            | 0x40 // @
            | 0x2A // *
            | 0x5F // _
            | 0x2B // +
            | 0x2D // -
            | 0x2E // .
            | 0x2F // /
        )
    }

    let mut out = String::new();
    for unit in input.encode_utf16() {
        if keep(unit) && !escape_all {
            out.push(unit as u8 as char);
        } else if unit < 0x100 {
            out.push_str(format!("%{:02X}", unit).as_str())
        } else {
            out.push_str(format!("%u{:04X}", unit).as_str())
        }
    }
    out
}

pub fn char_spell(s: &str, map: &HashMap<char, (u8, &'static str)>) -> String {
    s.chars()
        .map(|c| {
            let tpl = map
                .get(&c)
                .unwrap_or_else(|| panic!("no template for char {:?} while spelling {:?}", c, s))
                .1;
            format!("({})", tpl)
        })
        .collect::<Vec<_>>()
        .join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_uri_component() {
        assert_eq!(encode_uri_component("a b", false), "a%20b");
        assert_eq!(encode_uri_component("-_.!~*'()", false), "-_.!~*'()");
        assert_eq!(
            encode_uri_component("@&=+$,/?#", false),
            "%40%26%3D%2B%24%2C%2F%3F%23"
        );
        assert_eq!(encode_uri_component("é", false), "%C3%A9");
        assert_eq!(encode_uri_component("😀", false), "%F0%9F%98%80");
    }

    #[test]
    fn test_encode_uri() {
        assert_eq!(
            encode_uri("http://example.com/a b?x=1&y=2#frag", false),
            "http://example.com/a%20b?x=1&y=2#frag"
        );
        assert_eq!(encode_uri("@&=+$,/?#;:", false), "@&=+$,/?#;:");
        assert_eq!(encode_uri("é", false), "%C3%A9");
    }

    #[test]
    fn test_escape() {
        assert_eq!(escape("a b", false), "a%20b");
        assert_eq!(escape("@*_+-./", false), "@*_+-./");
        assert_eq!(escape("é", false), "%E9");
        assert_eq!(escape("™", false), "%u2122");
        assert_eq!(escape("😀", false), "%uD83D%uDE00");
    }
}
