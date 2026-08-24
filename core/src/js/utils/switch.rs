use crate::js::utils::string::generate_noise;
use rand::{Rng, RngExt};

pub fn generate_switch_conditions(rng: &mut impl Rng) -> (String, String) {
    match rng.random_range(0..3) {
        0 => {
            let right = rng.random_range(0..u32::MAX);
            let mut wrong = right;
            while wrong == right {
                wrong = rng.random_range(0..u32::MAX);
            }
            (right.to_string(), wrong.to_string())
        }
        1 => {
            if rng.random_bool(0.5) {
                (true.to_string(), false.to_string())
            } else {
                (false.to_string(), true.to_string())
            }
        }
        2 => {
            let right = generate_noise(rng.random_range(1..10), rng);
            let mut wrong = right.clone();
            while wrong == right {
                wrong = generate_noise(rng.random_range(1..10), rng);
            }
            (format!("'{}'", right), format!("'{}'", wrong))
        }
        _ => unreachable!(),
    }
}

pub fn generate_false(rng: &mut impl Rng) -> String {
    match rng.random_range(0..=5) {
        0 => false.to_string(),
        1 => "!!''".to_string(),
        2 => "![]".to_string(),
        3 => "!!0".to_string(),
        4 => format!("!{}", rng.random_range(1..u32::MAX)),
        5 => "!true".to_string(),
        _ => unreachable!(),
    }
}

pub fn generate_true(rng: &mut impl Rng) -> String {
    match rng.random_range(0..=5) {
        0 => true.to_string(),
        1 => "!''".to_string(),
        2 => "!![]".to_string(),
        3 => "!0".to_string(),
        4 => format!("!!{}", rng.random_range(1..u32::MAX)),
        5 => "!false".to_string(),
        _ => unreachable!(),
    }
}

pub fn make_switch_replacement(statement_text: &str, rng: &mut impl Rng) -> String {
    let conditions = generate_switch_conditions(rng);

    if rng.random_bool(0.5) {
        format!(
            "switch({}){{case {}: {};break;case {}: throw new Error()}}",
            conditions.0, conditions.0, statement_text, conditions.1
        )
    } else {
        format!(
            "switch({}){{case {}: throw new Error();break;case {}: {};break}}",
            conditions.0, conditions.1, conditions.0, statement_text
        )
    }
}

pub fn make_if_replacement(statement_text: &str, rng: &mut impl Rng) -> String {
    if rng.random_bool(0.5) {
        let true_statement = generate_true(rng);
        format!(
            "if({}){{{}}} else{{throw new Error();}}",
            true_statement, statement_text
        )
    } else {
        let false_statement = generate_false(rng);
        format!(
            "if({}){{throw new Error();}} else{{{}}}",
            false_statement, statement_text
        )
    }
}
