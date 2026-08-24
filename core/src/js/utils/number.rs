use crate::tree::Node;
use log::trace;
use std::time::Instant;

pub const EPS: f64 = 2.220446049250313e-16; // from Number.EPSILON

pub fn parse_number(view: &Node) -> Option<f64> {
    if view.kind() != "number" && view.kind() != "identifier" {
        return None;
    }

    let token = view.text().unwrap();

    match view.kind() {
        "number" => parse_number_from_string(token),
        "identifier" => {
            if view.text().unwrap() == "Infinity" {
                Some(f64::INFINITY)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_number_from_string(input: &str) -> Option<f64> {
    let negate = input.starts_with("-");
    // skip bigints
    if input.ends_with("n") {
        return None;
    }
    let input = if negate { &input[1..] } else { input };

    let input = if !input.starts_with("_") && !input.ends_with("_") {
        input.replace("_", "")
    } else {
        input.to_string()
    };

    if input.len() > 2 && (input.starts_with("0x") || input.starts_with("0X")) {
        if let Ok(n) = u64::from_str_radix(&input[2..], 16) {
            return if negate { None } else { Some(n as f64) };
        }
    } else if input.len() > 2 && (input.starts_with("0o") || input.starts_with("0O")) {
        if let Ok(n) = u64::from_str_radix(&input[2..], 8) {
            return if negate { None } else { Some(n as f64) };
        }
    } else if input.len() > 2 && (input.starts_with("0b") || input.starts_with("0B")) {
        if let Ok(n) = u64::from_str_radix(&input[2..], 2) {
            return if negate { None } else { Some(n as f64) };
        }
    } else {
        if input.starts_with("0")
            && let Ok(n) = u64::from_str_radix(&input[1..], 8)
        {
            return if negate {
                Some(-(n as f64))
            } else {
                Some(n as f64)
            };
        }

        // JS fallback to decimal parsing on fail
        if let Ok(n) = input.parse::<f64>() {
            return Some(if negate { -n } else { n });
        }
    }

    None
}

pub fn is_integer(input: f64) -> bool {
    input.fract() == 0.0
}

pub fn is_close(a: f64, b: f64) -> bool {
    let rel: f64 = 1e-10;
    let abs: f64 = 1e-12;
    (a - b).abs() <= abs.max(rel * a.abs().max(b.abs()))
}

pub fn decimal_places_from_literal(s: &str) -> Option<usize> {
    let s = s.replace('_', "");
    let mantissa = s.split(['e', 'E']).next()?;
    let frac = mantissa.split('.').nth(1)?;
    Some(frac.len())
}

pub fn fits_i32(n: f64) -> bool {
    n.is_finite() && n.fract() == 0.0 && n >= i32::MIN as f64 && n <= i32::MAX as f64
}
pub fn fits_u32(n: f64) -> bool {
    n.is_finite() && n.fract() == 0.0 && n >= 0.0 && n <= u32::MAX as f64
}

pub fn mod_inv_i32(a: i32) -> i32 {
    let mut x = a as i64;
    for i in 0..5 {
        x = x.wrapping_mul(2i64.wrapping_sub((a as i64).wrapping_mul(x)));
        trace!("mod_inv_i32({}) iteration {}: x = {:#010x}", a, i, x as u32);
    }
    let result = x as i32;
    trace!("mod_inv_i32({}) = {}", a, result);
    result
}

/// Find (a, b) such that a^2 + b^2 = n^2
// https://exercism.org/tracks/rust/exercises/pythagorean-triplet/solutions
pub fn find_pythagorean_triple(n: i64) -> Option<(i64, i64)> {
    let start = Instant::now();
    let mut iterations = 0;
    for m in 2..=32i64 {
        for k in 1..m {
            iterations += 1;
            if (m - k) % 2 == 0 {
                continue;
            }
            if gcd(m, k) != 1 {
                continue;
            }
            let c = m * m + k * k;
            if n % c == 0 {
                let t = n / c;
                trace!(
                    "Found pythagorean triplet {}^2 + {}^2 = {}^2 in {:?} after {} iterations",
                    (m * m - k * k) * t,
                    2 * m * k * t,
                    c * t,
                    start.elapsed(),
                    iterations
                );
                return Some(((m * m - k * k) * t, 2 * m * k * t));
            }
        }
    }
    None
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

const JS_MAX_SAFE_INTEGER: f64 = 9007199254740991.0; // 2^53 - 1

pub fn is_neg_zero(n: f64) -> bool {
    n == 0.0 && n.is_sign_negative()
}

pub fn same_js_number(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

pub fn is_js_safe_integer(n: f64) -> bool {
    n.is_finite() && n.fract() == 0.0 && n.abs() <= JS_MAX_SAFE_INTEGER && !is_neg_zero(n)
}

pub fn exact_add(a: f64, b: f64, expected: f64) -> bool {
    let r = a + b;
    r.is_finite() && same_js_number(r, expected)
}

pub fn exact_sub(a: f64, b: f64, expected: f64) -> bool {
    let r = a - b;
    r.is_finite() && same_js_number(r, expected)
}

pub fn exact_mul_add(a: f64, b: f64, c: f64, expected: f64) -> bool {
    let r = a * b + c;
    r.is_finite() && same_js_number(r, expected)
}

pub fn exact_square_roundtrip(n: f64) -> bool {
    let x = n * n;
    x.is_finite() && same_js_number(x.sqrt(), n)
}

pub fn exact_cube_roundtrip(n: f64) -> bool {
    let x = n * n * n;
    x.is_finite() && same_js_number(x.cbrt(), n)
}

fn min_safe_sub_val(remaining_strength: usize) -> f64 {
    if remaining_strength == 0 {
        return 0.0;
    }
    let exponent = 1.0_f64 / (3_usize.pow(remaining_strength as u32) as f64);
    (1e-20_f64).powf(exponent)
}

pub fn sub_val_is_safe(sub_val: f64, remaining_strength: usize) -> bool {
    sub_val.is_finite() && sub_val.abs() >= min_safe_sub_val(remaining_strength)
}

pub fn fract_leading_zeros(n: f64) -> usize {
    let fract = n.fract().abs();
    if fract == 0.0 {
        return 0;
    }
    let log = fract.log10();
    if log >= -1.0 {
        return 0;
    }
    (-log).floor() as usize - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fract_leading_zeros() {
        println!("{:?}", fract_leading_zeros(0.0)); // 0
        println!("{:?}", fract_leading_zeros(0.01)); // 1
        println!("{:?}", fract_leading_zeros(0.1)); // 0
        println!("{:?}", fract_leading_zeros(0.5)); // 0
        println!("{:?}", fract_leading_zeros(0.9)); // 0
        println!("{:?}", fract_leading_zeros(1.0)); // 0
        println!("{:?}", fract_leading_zeros(1.000001)); // 5
    }
}
