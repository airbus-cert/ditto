use crate::js::utils::string::generate_noise;
use log::trace;
use rand::Rng;
use rand::prelude::*;

pub mod array;
pub mod curse;
pub mod global_this;
pub mod identifier;
pub mod jsfuck;
pub mod number;
pub mod regex;
pub mod string;
pub mod switch;

#[derive(Clone)]
pub enum JunkType {
    Integer,
    Float,
    String(usize), // max length
    Array(usize),  // max elements
    BuiltinObject,
    Special,
}
pub fn generate_junk_node(junk_types: Option<Vec<JunkType>>, rng: &mut impl Rng) -> String {
    let junk_types = junk_types.unwrap_or_else(|| {
        vec![
            JunkType::Integer,
            JunkType::Float,
            JunkType::String(10),
            JunkType::Array(4),
            JunkType::BuiltinObject,
            JunkType::Special,
        ]
    });

    let junk_type = junk_types.choose(rng).unwrap();

    let junk = match junk_type {
        JunkType::Integer => {
            let number = rng.random_range(0..u32::MAX);
            format!("{}", number)
        }
        JunkType::Float => {
            let number = rng.random_range(0.0..f64::MAX);
            format!("{:.6e}", number)
        }
        JunkType::String(max_size) => {
            let size = rng.random_range(0..=*max_size);
            let string = generate_noise(size, rng);
            format!("'{}'", string)
        }
        JunkType::Array(max_elements) => {
            let mut elements = Vec::new();
            let mut junk_types = junk_types.clone();
            junk_types.remove(
                junk_types
                    .iter()
                    .position(|jt| matches!(jt, JunkType::Array(_)))
                    .unwrap(),
            );

            for _ in 0..rng.random_range(0..=*max_elements) {
                elements.push(generate_junk_node(Some(junk_types.clone()), rng));
            }
            format!("[{}]", elements.join(", "))
        }
        JunkType::BuiltinObject => {
            const BUILTIN_OBJECTS: &[&str] = &["Object", "Array", "Math", "RegExp"];
            let object = BUILTIN_OBJECTS.choose(rng).unwrap();
            format!("{}", object)
        }
        JunkType::Special => {
            const SPECIALS: &[&str] = &["null", "undefined", "NaN", "Infinity"];
            let special = SPECIALS.choose(rng).unwrap();
            format!("{}", special)
        }
    };
    trace!("Generated junk: {}", junk);
    junk
}
