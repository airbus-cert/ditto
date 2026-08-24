use crate::Lang;
use crate::Lang::JavaScript;
use crate::Mutation;
use crate::js::utils::number::*;
use crate::pass::Pass;
use log::trace;
use rand::Rng;
use rand::prelude::*;

const SAFE_MAX: f64 = (1u64 << 48) as f64;

#[derive(Debug)]
pub struct IntegerToExpression {
    pub strength: usize,
    pub mode: IntegerExpressionMode,
}

#[derive(Debug)]
pub enum IntegerExpressionMode {
    NoMath,
    WithMath,
    MathOnly,
}

impl IntegerToExpression {
    pub fn new(strength: usize, mode: IntegerExpressionMode) -> Self {
        Self { strength, mode }
    }
}

impl Pass for IntegerToExpression {
    type Mutation = Mutation;
    fn lang(&self) -> Lang {
        JavaScript
    }

    fn enter(
        &mut self,
        node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        if let Some(number) = parse_number(node) {
            if is_integer(number) {
                let obf = obfuscate(number, self.strength, &self.mode, &mut rand::rng());
                return Ok(Some(vec![Mutation::substitute(node, obf)?]));
            }
        }

        Ok(None)
    }

    fn leave(
        &mut self,
        _node: &crate::tree::Node,
    ) -> crate::error::DittoResult<Option<Vec<Self::Mutation>>> {
        Ok(None)
    }
}

fn fmt(n: f64) -> String {
    if n == 0.0 && n.is_sign_negative() {
        return "-0".to_string();
    }

    if n.fract() == 0.0 && n.abs() < 1e16 {
        format!("{}", n as i64)
    } else {
        format!("{:?}", n)
    }
}

const MAX_RETRIES: usize = 16;

/// Op discriminant bytes for plain arithmetic and Math.* functions.<br>
/// Plain: b'+' b'-' b'*' b'^' b'&' b'|' b'~' b'>' b'u'<br>
/// Math:
/// - 'C' = Math.clz32
/// - 'I' = Math.imul
/// - 'N' = Math.min
/// - 'X' = Math.max
/// - 'P' = Math.pow
/// - 'S' = Math.sqrt
/// - 'R' = Math.cbrt
/// - 'H' = Math.hypot
/// - 'E' = Math.exp(0)         -> 1
/// - 'e' = Math.expm1(0)       -> 0
/// - 'L' = Math.log2(2^n)      -> n   (n in [0,52])
/// - 'T' = Math.log10(10^n)    -> n   (n in [0,15])
/// - 'G' = Math.log(2^n)/LN2   -> n   (alternate log encoding)
/// - 'g' = Math.log(10^n)/LN10 -> n   (alternate log encoding)
/// - 'l' = Math.log1p(0)       -> 0
/// - 'c' = Math.cos(0)         -> 1
/// - 'o' = Math.cos(PI)        -> -1
/// - 's' = Math.sin(0)         -> 0
/// - 't' = Math.tan(0)         -> 0
/// - 'a' = Math.acos(1)        -> 0
/// - 'A' = Math.asin(0)        -> 0
/// - 'B' = Math.atan(0)        -> 0
/// - 'D' = Math.atan2(0,1)     -> 0
/// - 'k' = Math.acosh(1)       -> 0
/// - 'h' = Math.asinh(0)       -> 0
/// - 'q' = Math.atanh(0)       -> 0
/// - 'K' = Math.cosh(0)        -> 1
pub fn obfuscate(
    n: f64,
    strength: usize,
    mode: &IntegerExpressionMode,
    rng: &mut impl Rng,
) -> String {
    if is_neg_zero(n) {
        return "-0".to_string();
    }

    if strength == 0 || !n.is_finite() {
        return fmt(n);
    }

    let js_safe_int = is_js_safe_integer(n);
    let i32_int = fits_i32(n);
    let u32_int = fits_u32(n);

    let mut ops = Vec::new();
    if !matches!(mode, IntegerExpressionMode::MathOnly) {
        ops.extend_from_slice(b"+-*");
        if i32_int {
            ops.extend_from_slice(b"^&|~>");
        }
        if u32_int {
            ops.push(b'u');
        }
    }

    if !matches!(mode, IntegerExpressionMode::NoMath) {
        // Math.clz32 returns integer [0, 32]
        if n >= 0.0 && n <= 32.0 && n.fract() == 0.0 {
            ops.push(b'C');
        }

        // Math.imul(i32, i32)
        if i32_int {
            ops.push(b'I');
        }

        // Math.min(x) / Math.max(x)
        if n.is_finite() {
            ops.push(b'N');
            ops.push(b'X');
        }

        // Math.pow(base, exp)
        if n >= 0.0 && js_safe_int {
            ops.push(b'P');
        }

        // Math.sqrt(x^2)
        if n >= 0.0 && exact_square_roundtrip(n) {
            ops.push(b'S');
        }

        // Math.cbrt(x^3)
        if exact_cube_roundtrip(n) {
            ops.push(b'R');
        }

        // Math.hypot(a, b)
        if n >= 0.0 && i32_int {
            ops.push(b'H');
        }

        // Math.exp(0) = 1
        if n == 1.0 {
            ops.push(b'E');
        }

        // Math.expm1(0) = 0
        if n == 0.0 && !is_neg_zero(n) {
            ops.push(b'e');
        }

        // Math.log2(2^n) = n exactly for n in [0, 52]
        if n >= 0.0 && n <= 52.0 && n.fract() == 0.0 {
            ops.push(b'L');
            ops.push(b'G');
        }

        if n >= 0.0 && n <= 15.0 && n.fract() == 0.0 {
            ops.push(b'T');
            ops.push(b'g');
        }

        if n == 0.0 && !is_neg_zero(n) {
            ops.push(b'l');
        }

        if n == 1.0 {
            ops.push(b'c');
            ops.push(b'K');
        }

        if n == -1.0 {
            ops.push(b'o');
        }

        if n == 0.0 && !is_neg_zero(n) {
            ops.extend_from_slice(b"staBDkhq");
            ops.push(b'a');
        }
    }

    if ops.is_empty() {
        return fmt(n);
    }

    let op = *ops.choose(rng).unwrap();
    trace!("Transforming int {n} with function: {} ", op as char);

    let sub = |v: f64, r: &mut _| {
        if strength <= 1 {
            fmt(v)
        } else {
            obfuscate(v, strength - 1, mode, r)
        }
    };

    match op {
        // "Classic" operations
        b'+' => {
            for _ in 0..MAX_RETRIES {
                let a = rng.random_range(-SAFE_MAX as i64 / 2..SAFE_MAX as i64 / 2) as f64;
                let b = n - a;

                if b.is_finite() && exact_add(a, b, n) {
                    return format!("({} + {})", sub(a, rng), sub(b, rng));
                }
            }
            fmt(n)
        }
        b'-' => {
            for _ in 0..MAX_RETRIES {
                let a = rng.random_range(-SAFE_MAX as i64 / 2..SAFE_MAX as i64 / 2) as f64;
                let b = a - n;

                if b.is_finite() && exact_sub(a, b, n) {
                    return format!("({} - {})", sub(a, rng), sub(b, rng));
                }
            }
            fmt(n)
        }
        b'*' => {
            for _ in 0..MAX_RETRIES {
                let a = rng.random_range(-1024..1024) as f64;
                let b = rng.random_range(-1024..1024) as f64;
                let c = n - a * b;

                if c.is_finite() && exact_mul_add(a, b, c, n) {
                    return format!("(({} * {}) + {})", sub(a, rng), sub(b, rng), sub(c, rng));
                }
            }
            fmt(n)
        }
        b'^' => {
            let a = rand::random::<i32>();
            let b = a ^ (n as i32);
            format!("({} ^ {})", sub(a as f64, rng), sub(b as f64, rng))
        }
        b'&' => {
            let m = rand::random::<i32>();
            let ni = n as i32;
            format!(
                "({} & {})",
                sub((ni | m) as f64, rng),
                sub((ni | !m) as f64, rng)
            )
        }
        b'|' => {
            let m = rand::random::<i32>();
            let ni = n as i32;
            format!(
                "({} | {})",
                sub((ni & m) as f64, rng),
                sub((ni & !m) as f64, rng)
            )
        }
        b'~' => format!("(~{})", sub((!(n as i32)) as f64, rng)),
        b'>' => {
            for _ in 0..MAX_RETRIES {
                let shift = rng.random_range(1..=8);
                let a = (n as i32).wrapping_shl(shift);

                if same_js_number((a >> shift) as f64, n) {
                    return format!("({} >> {})", sub(a as f64, rng), shift);
                }
            }
            fmt(n)
        }
        b'u' => {
            for _ in 0..MAX_RETRIES {
                let shift = rng.random_range(1..=8);
                let a = (n as u32).wrapping_shl(shift);

                if same_js_number((a >> shift) as f64, n) {
                    return format!("({} >>> {})", sub(a as f64, rng), shift);
                }
            }

            fmt(n)
        }

        // Math operations
        //clz32
        b'C' => {
            let ni = n as u32;
            let x: u32 = if ni == 32 { 0 } else { 1u32 << (31 - ni) };
            format!("Math.clz32({})", sub(x as f64, rng))
        }

        // imul
        b'I' => {
            let a = rng.random_range(1i32..=512) * 2 - 1; // random odd positive
            let a_inv = mod_inv_i32(a);
            let b = (n as i32).wrapping_mul(a_inv);
            format!("Math.imul({}, {})", sub(a as f64, rng), sub(b as f64, rng))
        }

        //min
        b'N' => {
            for _ in 0..MAX_RETRIES {
                let delta = rng.random_range(1i64..=4096) as f64;
                let other = n + delta;

                if other.is_finite() && other > n {
                    return if rng.random_bool(0.5) {
                        format!("Math.min({}, {})", sub(other, rng), sub(n, rng))
                    } else {
                        format!("Math.min({}, {})", sub(n, rng), sub(other, rng))
                    };
                }
            }
            fmt(n)
        }
        // max
        b'X' => {
            for _ in 0..MAX_RETRIES {
                let delta = rng.random_range(1i64..=4096) as f64;
                let other = n - delta;

                if other.is_finite() && other < n {
                    return if rng.random_bool(0.5) {
                        format!("Math.max({}, {})", sub(other, rng), sub(n, rng))
                    } else {
                        format!("Math.max({}, {})", sub(n, rng), sub(other, rng))
                    };
                }
            }
            fmt(n)
        }

        // pow
        b'P' => {
            let exps: &[i32] = &[2, 3, 4];
            let mut found: Option<(f64, f64)> = None;

            for &e in exps {
                let base = n.powf(1.0 / e as f64).round();

                if base >= 2.0 {
                    let pow = base.powi(e);
                    if same_js_number(pow, n) {
                        found = Some((base, e as f64));
                        break;
                    }
                }
            }

            match found {
                Some((base, e)) => {
                    format!("Math.pow({}, {})", sub(base, rng), sub(e, rng))
                }
                None => {
                    format!("Math.pow({}, {})", sub(n, rng), sub(1.0, rng))
                }
            }
        }

        // sqrt
        b'S' => {
            for _ in 0..MAX_RETRIES {
                let x = n * n;
                if x.is_finite() && same_js_number(x.sqrt(), n) && is_js_safe_integer(x) {
                    return format!("Math.sqrt({})", sub(x, rng));
                }
            }
            fmt(n)
        }
        // cbrt
        b'R' => {
            for _ in 0..MAX_RETRIES {
                let x = n * n * n;
                if x.is_finite() && same_js_number(x.cbrt(), n) && is_js_safe_integer(x) {
                    return format!("Math.cbrt({})", sub(x, rng));
                }
            }
            fmt(n)
        }

        // hypot
        b'H' => match find_pythagorean_triple(n as i64) {
            Some((a, b)) => {
                format!(
                    "Math.round(Math.hypot({}, {}))",
                    sub(a as f64, rng),
                    sub(b as f64, rng)
                )
            }
            None => fmt(n),
        },

        // exp expm1
        b'E' => format!("Math.exp({})", sub(0.0, rng)),
        b'e' => format!("Math.expm1({})", sub(0.0, rng)),

        // log2 log10 log log1p
        b'L' => {
            let pow2 = (1u64 << n as u32) as f64;
            format!("Math.round(Math.log2({}))", sub(pow2, rng))
        }
        b'T' => {
            let pow10 = 10f64.powi(n as i32);
            format!("Math.round(Math.log10({}))", sub(pow10, rng))
        }
        b'G' => {
            let pow2 = (1u64 << n as u32) as f64;
            format!("Math.round(Math.log({}) / Math.LN2)", sub(pow2, rng))
        }
        b'g' => {
            let pow10 = 10f64.powi(n as i32);
            format!("Math.round(Math.log({}) / Math.LN10)", sub(pow10, rng))
        }

        b'l' => format!("Math.log1p({})", sub(0.0, rng)),

        // trigonometric functions
        b'c' => format!("Math.cos({})", sub(0.0, rng)),
        b'o' => format!("(-Math.cos({}))", sub(0.0, rng)),
        b's' => format!("Math.sin({})", sub(0.0, rng)),
        b't' => format!("Math.tan({})", sub(0.0, rng)),
        b'a' => format!("Math.acos({})", sub(1.0, rng)),
        b'A' => format!("Math.asin({})", sub(0.0, rng)),
        b'B' => format!("Math.atan({})", sub(0.0, rng)),
        b'D' => format!("Math.atan2({}, {})", sub(0.0, rng), sub(1.0, rng)),
        b'k' => format!("Math.acosh({})", sub(1.0, rng)),
        b'h' => format!("Math.asinh({})", sub(0.0, rng)),
        b'q' => format!("Math.atanh({})", sub(0.0, rng)),
        b'K' => format!("Math.cosh({})", sub(0.0, rng)),

        _ => unreachable!("Invalid op"),
        //_ => fmt(n),
    }
}
