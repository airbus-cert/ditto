use rand::Rng;
use rand::prelude::IndexedRandom;

fn random_hieroglyph_base(rng: &mut dyn Rng) -> u32 {
    *vec![0x13000u32, 0x13100u32, 0x13200u32, 0x13300u32]
        .choose(rng)
        .unwrap()
}

pub fn curse_string(input: &str, rng: &mut dyn Rng) -> String {
    if input.is_ascii() {
        curse_string_8bit(input, rng)
    } else {
        curse_string_16bit(input, rng)
    }
}

fn curse_string_8bit(input: &str, rng: &mut dyn Rng) -> String {
    let mut hieroglyphs = String::new();

    for byte in input.bytes() {
        let char_u32 = random_hieroglyph_base(rng) + (byte as u32);

        if let Some(c) = char::from_u32(char_u32) {
            hieroglyphs.push(c);
        } else {
            panic!("Invalid unicode character generated.");
        }
    }

    format!(
        "unescape(escape('{}').replace(/%uD80C%uD[C-F]/g, '%'))",
        hieroglyphs
    )
}

fn curse_string_16bit(input: &str, rng: &mut dyn Rng) -> String {
    let mut hieroglyphs = String::new();

    for code_unit in input.encode_utf16() {
        let high_byte = (code_unit >> 8) & 0xFF;
        let low_byte = code_unit & 0xFF;

        let high_char = char::from_u32(random_hieroglyph_base(rng) + (high_byte as u32)).unwrap();
        let low_char = char::from_u32(random_hieroglyph_base(rng) + (low_byte as u32)).unwrap();

        hieroglyphs.push(high_char);
        hieroglyphs.push(low_char);
    }

    format!(
        "unescape(escape('{}').replace(/%uD80C%uD[C-F](..)%uD80C%uD[C-F](..)/g, '%u$1$2'))",
        hieroglyphs
    )
}

pub fn curse_integer_math(val: i64, rng: &mut dyn Rng) -> String {
    if val == 0 {
        return "('𓀀'.charCodeAt(1)&0xff)".to_string();
    }

    let is_negative = val < 0;
    let mut n = val.abs() as u64;
    let mut chunks = Vec::new();
    let mut shift = 0;

    while n > 0 {
        let byte = (n & 0xff) as u32;
        let hieroglyph =
            char::from_u32(random_hieroglyph_base(rng) + byte).expect("Invalid Hieroglyph");

        let chunk_expr = if shift == 0 {
            format!("('{}'.charCodeAt(1)&0xff)", hieroglyph)
        } else if shift < 24 {
            format!("((('{}'.charCodeAt(1)&0xff))<<{})", hieroglyph, shift)
        } else {
            format!(
                "((('{}'.charCodeAt(1)&0xff))*0x{:x})",
                hieroglyph,
                1u64 << shift
            )
        };

        chunks.push(chunk_expr);

        n >>= 8;
        shift += 8;
    }

    chunks.reverse();

    let mut expr = format!("({})", chunks.join("+"));

    if is_negative {
        expr = format!("-({})", expr);
    }

    expr
}
