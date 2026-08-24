use crate::Lang;
use crate::Lang::JavaScript;
use crate::Mutation;
use crate::js::utils::number::*;
use crate::pass::Pass;
use log::{trace, warn};
use rand::Rng;
use rand::prelude::*;
use std::cmp::min;

const SAFE_MAX: f64 = (1u64 << 20) as f64;

#[derive(Debug)]
pub struct FloatToExpression {
    pub strength: usize,
    pub mode: FloatExpressionMode,
}

#[derive(Debug)]
pub enum FloatExpressionMode {
    NoMath,
    WithMath,
    MathOnly,
}

impl FloatToExpression {
    pub fn new(strength: usize, mode: FloatExpressionMode) -> Self {
        Self { strength, mode }
    }
}

impl Pass for FloatToExpression {
    type Mutation = Mutation;

    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        if let Some(number) = parse_number(node) {
            if !is_integer(number) {
                let precision =
                    decimal_places_from_literal(number.to_string().as_str()).unwrap_or(2) + 1;
                if precision > (10 - min(self.strength, 10)) {
                    return Ok(None);
                }
                let obf = obfuscate(number, self.strength, &self.mode, &mut rand::rng());
                let obf = format!("(+(({}).toFixed({})))", obf, precision - 1);
                return Ok(Some(vec![Mutation::substitute(node, obf)?]));
            }
        }
        Ok(None)
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Mutation>>> {
        Ok(None)
    }
}

fn fmt(n: f64) -> String {
    format!("{:?}", n)
}

/// Op discriminant bytes for plain arithmetic and Math.* functions.<br>
/// Plain: b'+' b'-' b'*' b'/'<br>
/// Math:
/// - 'N' = Math.min(x, y)
/// - 'X' = Math.max(x, y)
/// - 'P' = Math.pow(x, y)
/// - 'S' = Math.sqrt(x)          -> x >= 0
/// - 'R' = Math.cbrt(x)          -> x >= 0
/// - 'L' = Math.log(exp(x))      -> x > 0
/// - '2' = Math.log2(2^x)        -> x > 0
/// - '1' = Math.log10(10^x)      -> x > 0
/// - 'G' = Math.log1p(expm1(x))  -> x > 0
/// - 'E' = Math.exp(log(x))      -> x > 0
/// - 'e' = Math.expm1(log1p(x))  -> x > -1
/// - 'c' = Math.cos(x)           -> 0 <= x <= 1
/// - 's' = Math.sin(x)           -> 0 <= x <= 1
/// - 't' = Math.tan(x)           -> 0 <= x <= 1
/// - 'a' = Math.acosh(x)         -> 1 <= x < 20
/// - 'h' = Math.asinh(x)         -> 0 < x < 1
/// - 'q' = Math.atanh(x)         -> x >= 0
pub fn obfuscate(
    n: f64,
    strength: usize,
    mode: &FloatExpressionMode,
    rng: &mut impl Rng,
) -> String {
    if strength == 0 || !n.is_finite() {
        return fmt(n);
    }

    if n.fract().abs() < EPS && n.fract() != 0.0 {
        warn!("Skipping obfuscation of zero-fractional number: {}", n);
        return fmt(n);
    }

    let sub_strength = strength - 1;

    let mut ops: Vec<u8> = Vec::new();

    if matches!(mode, FloatExpressionMode::NoMath) {
        ops.extend_from_slice(b"+-");

        if strength <= 3 && n.abs() >= 1.0 && fract_leading_zeros(n) < 2 {
            ops.extend_from_slice(b"*/");
        }
    }

    if !matches!(mode, FloatExpressionMode::NoMath) {
        ops.extend_from_slice(b"NX"); // min, max

        if n.fract() == 0.0 && n >= 0.0 {
            ops.push(b'P'); // pow
        }

        if n >= 0.0 && sub_val_is_safe(n * n, sub_strength) {
            ops.push(b'S'); // sqrt(n*n): sub = n^2
        }

        if sub_val_is_safe(n * n * n, sub_strength) {
            ops.push(b'R'); // cbrt(n*n*n): sub = n^3
        }

        if n > 0.0 {
            let exp_n = n.exp();
            if exp_n.is_finite() && sub_val_is_safe(exp_n, sub_strength) {
                ops.push(b'L'); // log(exp(n)): sub = exp(n)
            }
            let pow2_n = 2f64.powf(n);
            if pow2_n.is_finite() && sub_val_is_safe(pow2_n, sub_strength) {
                ops.push(b'2'); // log2(2^n): sub = 2^n
            }
            let pow10_n = 10f64.powf(n);
            if pow10_n.is_finite() && sub_val_is_safe(pow10_n, sub_strength) {
                ops.push(b'1'); // log10(10^n): sub = 10^n
            }
            let expm1_n = n.exp_m1();
            if expm1_n.is_finite() && expm1_n > -1.0 && sub_val_is_safe(expm1_n, sub_strength) {
                ops.push(b'G'); // log1p(expm1(n)): sub = expm1(n)
            }
            if sub_val_is_safe(n.ln().abs(), sub_strength) {
                ops.push(b'E'); // exp(log(n)): sub = ln(n)
            }
        }

        if n > -1.0 {
            let ln1p_n = (n + 1.0).ln();
            if sub_val_is_safe(ln1p_n.abs(), sub_strength) {
                ops.push(b'e'); // expm1(log1p(n)): sub = ln(n+1)
            }
        }

        if n >= -1.0 && n <= 1.0 {
            // asin/acos/atan subs are bounded in [-PI/2, PI/2] always safe
            ops.push(b's'); // sin(asin(n))
            ops.push(b't'); // tan(atan(n))
            ops.push(b'c'); // cos(acos(n))
        }

        if n.abs() < 20.0 {
            let sinh_n = n.sinh();
            if sub_val_is_safe(sinh_n.abs(), sub_strength) {
                ops.push(b'h'); // asinh(sinh(n)): sub = sinh(n)
            }
        }

        if n > -1.0 && n < 1.0 {
            let tanh_n = n.tanh();
            if sub_val_is_safe(tanh_n.abs(), sub_strength) {
                ops.push(b'q'); // atanh(tanh(n)): sub = tanh(n)
            }
        }

        if n >= 0.0 && fract_leading_zeros(n) < 2 {
            let cosh_n = n.cosh();
            if cosh_n.is_finite() && sub_val_is_safe(cosh_n, sub_strength) {
                ops.push(b'a'); // acosh(cosh(n)): sub = cosh(n)
            }
        }
    }

    if ops.is_empty() {
        return fmt(n);
    }

    let op = *ops.choose(rng).unwrap();
    trace!("Transforming float {n} with function: {} ", op as char);

    let sub = |v: f64, r: &mut _| obfuscate(v, sub_strength, mode, r);

    match op {
        b'+' => {
            let a = rng.random_range(-SAFE_MAX..SAFE_MAX);
            format!("({} + {})", sub(a, rng), sub(n - a, rng))
        }
        b'-' => {
            let a = rng.random_range(-SAFE_MAX..SAFE_MAX);
            format!("({} - {})", sub(a, rng), sub(a - n, rng))
        }
        b'*' => {
            let factors = [2.0, 4.0, 8.0, 16.0];
            let a = *factors.choose(rng).unwrap();
            format!("({} * {})", sub(a, rng), sub(n / a, rng))
        }
        b'/' => {
            let factors = [2.0, 4.0, 8.0, 16.0];
            let a = *factors.choose(rng).unwrap();
            format!("({} / {})", sub(n * a, rng), sub(a, rng))
        }
        b'N' => {
            let delta = rng.random_range(0.1..1000.0);
            format!("Math.min({}, {})", sub(n + delta, rng), sub(n, rng))
        }
        b'X' => {
            let delta = rng.random_range(0.1..1000.0);
            format!("Math.max({}, {})", sub(n - delta, rng), sub(n, rng))
        }
        b'P' => {
            let exponents = [2.0, 3.0, 0.5, 1.0];
            let e = *exponents.choose(rng).unwrap();
            if is_close(e, 1.0) {
                format!("Math.pow({}, {})", sub(n, rng), sub(1.0, rng))
            } else if is_close(e, 2.0) && n >= 0.0 {
                let base = n.sqrt();
                format!("Math.pow({}, {})", sub(base, rng), sub(2.0, rng))
            } else if is_close(e, 3.0) {
                let base = n.cbrt();
                format!("Math.pow({}, {})", sub(base, rng), sub(3.0, rng))
            } else {
                format!("Math.pow({}, {})", sub(n, rng), sub(1.0, rng))
            }
        }

        b'S' => format!("Math.sqrt({})", sub(n * n, rng)),
        b'R' => format!("Math.cbrt({})", sub(n * n * n, rng)),
        b'L' => format!("Math.log({})", sub(n.exp(), rng)),
        b'2' => format!("Math.log2({})", sub(2f64.powf(n), rng)),
        b'1' => format!("Math.log10({})", sub(10f64.powf(n), rng)),
        b'G' => format!("Math.log1p({})", sub(n.exp_m1(), rng)),
        b'E' => format!("Math.exp({})", sub(n.ln(), rng)),
        b'e' => format!("Math.expm1({})", sub((n + 1.0).ln(), rng)),
        b's' => format!("Math.sin({})", sub(n.asin(), rng)),
        b'c' => format!("Math.cos({})", sub(n.acos(), rng)),
        b't' => format!("(Math.tan({}))", sub(n.atan() + 0.0000000000000001, rng)),
        b'h' => format!("Math.asinh({})", sub(n.sinh(), rng)),
        b'q' => format!("Math.atanh({})", sub(n.tanh(), rng)),
        b'a' => format!("Math.acosh(Math.max(1, {}))", sub(n.cosh(), rng)),
        _ => fmt(n),
    }
}
