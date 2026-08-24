use crate::js::utils::string::{char_spell, escape_js_string};
use log::trace;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

// I had to do this because re-encoding the same chars over and over is extremely slow on level 7+.
static M: OnceLock<HashMap<char, (u8, &'static str)>> = OnceLock::new();
static CACHE: OnceLock<Mutex<HashMap<(char, u8), String>>> = OnceLock::new();
// I had to do this because recompiling over and over the same regex is slow.
static RE_Q: OnceLock<Regex> = OnceLock::new();
static RE_BD: OnceLock<Regex> = OnceLock::new();
static RE_D: OnceLock<Regex> = OnceLock::new();

fn primitive_mum(n: u32) -> String {
    if n == 0 {
        "+[]".to_string()
    } else {
        "+!+[]".repeat(n as usize)
    }
}

fn char_map() -> &'static HashMap<char, (u8, &'static str)> {
    M.get_or_init(|| CHARS.iter().map(|(c, l, t)| (*c, (*l, *t))).collect())
}

fn get_cache() -> &'static Mutex<HashMap<(char, u8), String>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn encode(input: &str, max_level: u8) -> String {
    if input.is_empty() {
        return "[]+[]".to_string(); // ""
    }

    let map = char_map();
    let mut parts: Vec<String> = Vec::new();
    let mut lit = String::new();

    for c in input.chars() {
        let buildable = map.get(&c).filter(|(lvl, _)| *lvl <= max_level);
        if let Some((_, tpl)) = buildable {
            if !lit.is_empty() {
                parts.push(escape_js_string(&lit, true));
                lit.clear();
            }

            let encoded = {
                let mut cache = get_cache().lock().unwrap();
                cache
                    .entry((c, max_level))
                    .or_insert_with(|| format!("({})", resolve(tpl, map, c)))
                    .clone()
            };
            parts.push(encoded);
        } else if max_level >= 9 {
            if !lit.is_empty() {
                parts.push(escape_js_string(&lit, true));
                lit.clear();
            }

            let encoded = {
                let mut cache = get_cache().lock().unwrap();
                cache
                    .entry((c, max_level))
                    .or_insert_with(|| format!("({})", level9(c, map)))
                    .clone()
            };
            parts.push(encoded);
        } else {
            lit.push(c);
        }
    }
    if !lit.is_empty() {
        parts.push(escape_js_string(&lit, true));
    }
    parts.join("+")
}

pub fn resolve(template: &str, map: &HashMap<char, (u8, &'static str)>, c: char) -> String {
    let start = Instant::now();
    let re_q = RE_Q.get_or_init(|| Regex::new(r#""([^"]*)""#).unwrap());
    let re_bd = RE_BD.get_or_init(|| Regex::new(r"\[(\d)]").unwrap());
    let re_d = RE_D.get_or_init(|| Regex::new(r"\d").unwrap());

    let mut s = template.to_string();

    let mut guard = 1_000_000i64;
    loop {
        if s.contains('"') {
            s = re_q
                .replace_all(&s, |c: &Captures| format!("({})", char_spell(&c[1], map)))
                .into_owned();
        } else if s.contains("Infinity") || s.contains("RegExp") {
            s = s.replace("Infinity", r#"(+("1e1000"))"#);
            s = s.replace(
                "RegExp",
                r#"([]["flat"]["constructor"]("return RegExp")())"#,
            );
        } else {
            break;
        }
        guard -= 1;
        if guard < 0 {
            panic!("resolve did not converge for template");
        }
    }

    s = s.replace("undefined", "[][[]]");
    s = s.replace("false", "![]");
    s = s.replace("true", "!![]");
    s = s.replace("NaN", "+[![]]");

    // number shorthand: `[d]` then bare `d`
    s = re_bd
        .replace_all(&s, |c: &Captures| {
            format!("[{}]", primitive_mum(c[1].parse().unwrap()))
        })
        .into_owned();
    s = re_d
        .replace_all(&s, |c: &Captures| primitive_mum(c[0].parse().unwrap()))
        .into_owned();
    trace!("resolved {} in {:?}", c, start.elapsed());
    s
}

/// Level 9, universal builder: `Function("return '\uXXXX'")()` as pure JSFuck.
fn level9(c: char, map: &HashMap<char, (u8, &'static str)>) -> String {
    let start = Instant::now();
    let cc = c as u32;
    let esc = if cc <= 0xFFFF {
        format!("\\u{:04x}", cc)
    } else {
        format!("\\u{{{:x}}}", cc)
    };
    let template = format!(r#"[]["flat"]["constructor"]("return '{}'")()"#, esc);
    let result = resolve(&template, map, c);
    trace!("Resolved level9 {} in {:?}", c, start.elapsed());
    result
}

pub const CHARS: &[(char, u8, &str)] = &[
    ('0', 1, r#"0+[]"#),
    ('1', 1, r#"1+[]"#),
    ('2', 1, r#"2+[]"#),
    ('3', 1, r#"3+[]"#),
    ('4', 1, r#"4+[]"#),
    ('5', 1, r#"5+[]"#),
    ('6', 1, r#"6+[]"#),
    ('7', 1, r#"7+[]"#),
    ('8', 1, r#"8+[]"#),
    ('9', 1, r#"9+[]"#),
    ('a', 1, r#"(false+[])[1]"#),
    ('d', 1, r#"(undefined+[])[2]"#),
    ('e', 1, r#"(true+[])[3]"#),
    ('f', 1, r#"(false+[])[0]"#),
    ('i', 1, r#"([false]+undefined)[1+[0]]"#),
    ('l', 1, r#"(false+[])[2]"#),
    ('n', 1, r#"(undefined+[])[1]"#),
    ('r', 1, r#"(true+[])[1]"#),
    ('s', 1, r#"(false+[])[3]"#),
    ('t', 1, r#"(true+[])[0]"#),
    ('u', 1, r#"(undefined+[])[0]"#),
    ('N', 1, r#"(NaN+[])[0]"#),
    (' ', 2, r#"([false]+[]["flat"])[2+[0]]"#),
    ('(', 2, r#"([]+[]["flat"])[1+[3]]"#),
    (')', 2, r#"([]+[]["flat"])[1+[4]]"#),
    ('+', 2, r#"(+("11e100")+[])[4]"#),
    ('.', 2, r#"(+("11e100")+[])[1]"#),
    ('[', 2, r#"([]+[]["entries"]())[0]"#),
    (']', 2, r#"([]+[]["entries"]())[2+[2]]"#),
    ('{', 2, r#"([true]+[]["flat"])[2+[0]]"#),
    ('c', 2, r#"([]["flat"]+[])[3]"#),
    ('j', 2, r#"([]+[]["entries"]())[3]"#),
    ('o', 2, r#"([true]+[]["flat"])[1+[0]]"#),
    ('y', 2, r#"(true+[Infinity])[1+[1]]"#),
    ('A', 2, r#"([NaN]+([]+[]["entries"]()))[1+[1]]"#),
    ('I', 2, r#"(Infinity+[])[0]"#),
    ('-', 3, r#"(+(".0000001")+[])[2]"#),
    ('b', 3, r#"([]+(+[])["constructor"])[1+[2]]"#),
    ('g', 3, r#"(false+[0]+([]+[])["constructor"])[2+[0]]"#),
    ('m', 3, r#"([]+(+[])["constructor"])[1+[1]]"#),
    ('B', 3, r#"([NaN]+(![])["constructor"])[1+[2]]"#),
    ('F', 3, r#"([NaN]+[]["flat"]["constructor"])[1+[2]]"#),
    ('S', 3, r#"([NaN]+([]+[])["constructor"])[1+[2]]"#),
    ('h', 4, r#"(+(1+[0]+[1]))["toString"](2+[1])[1]"#),
    ('k', 4, r#"(+(2+[0]))["toString"](2+[1])"#),
    ('p', 4, r#"(+(2+[1]+[1]))["toString"](3+[1])[1]"#),
    ('q', 4, r#"(+(2+[1]+[2]))["toString"](3+[1])[1]"#),
    ('v', 4, r#"(+(3+[1]))["toString"](3+[2])"#),
    ('w', 4, r#"(+(3+[2]))["toString"](3+[3])"#),
    ('x', 4, r#"(+(1+[0]+[1]))["toString"](3+[4])[1]"#),
    ('z', 4, r#"(+(3+[5]))["toString"](3+[6])"#),
    ('}', 4, r#"([true]+[]["flat"])["slice"]("-1")"#),
    (
        'E',
        5,
        r#"([false]+[]["flat"]["constructor"]("try{String().normalize(false)}catch(f){return f}")())[1+[0]]"#,
    ),
    (
        'R',
        5,
        r#"([]+[]["flat"]["constructor"]("try{String().normalize(false)}catch(f){return f}")())[0]"#,
    ),
    (
        '/',
        6,
        r#"([]+[]["flat"]["constructor"]("return RegExp")()())[0]"#,
    ),
    (
        ':',
        6,
        r#"([]+[]["flat"]["constructor"]("return RegExp")()())[3]"#,
    ),
    (
        '?',
        6,
        r#"([]+[]["flat"]["constructor"]("return RegExp")()())[2]"#,
    ),
    ('\\', 7, r#"([]+RegExp("/"))[1]"#),
    (
        '\'',
        8,
        r#"RegExp(([]+[]["entries"]())[0]+([]+RegExp("/"))[1]+(undefined+[])[0]+[+[]]+[+[]]+[+!+[]+!+[]]+[+!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+!+[]]+([]+[]["entries"]())[2+[2]])[(true+[])[3]+(+(1+[0]+[1]))["toString"](3+[4])[1]+(true+[])[3]+([]["flat"]+[])[3]]([]["flat"]["constructor"]((true+[])[0]+(true+[])[1]+(true+[Infinity])[1+[1]]+([true]+[]["flat"])[2+[0]]+[]["flat"]["constructor"]["name"]+([]+[]["flat"])[1+[3]]+([]+[]["entries"]())[0]+([]+[]["entries"]())[2+[2]]+(+("11e100")+[])[4]+([]+[]["entries"]())[0]+([]+[]["entries"]())[0]+([]+[]["entries"]())[2+[2]]+([]+[]["entries"]())[2+[2]]+(+("11e100")+[])[1]+([]["flat"]+[])[3]+([true]+[]["flat"])[1+[0]]+(undefined+[])[1]+([]["flat"]+[])[3]+(false+[])[1]+(true+[])[0]+([]+[]["flat"])[1+[3]]+([]+[]["entries"]())[0]+([]+[]["entries"]())[0]+([]+[]["entries"]())[2+[2]]+([]+[]["entries"]())[2+[2]]+([]+[]["flat"])[1+[4]]+([]+[]["flat"])[1+[4]]+([]+[]["flat"])[1+[3]]+([]+[]["flat"])[1+[4]]+([true]+[]["flat"])["slice"]("-1")+([]["flat"]+[])[3]+(false+[])[1]+(true+[])[0]+([]["flat"]+[])[3]+(+(1+[0]+[1]))["toString"](2+[1])[1]+([]+[]["flat"])[1+[3]]+(false+[])[0]+([]+[]["flat"])[1+[4]]+([true]+[]["flat"])[2+[0]]+(true+[])[1]+(true+[])[3]+(true+[])[0]+(undefined+[])[0]+(true+[])[1]+(undefined+[])[1]+([false]+[]["flat"])[2+[0]]+(false+[])[0]+([true]+[]["flat"])["slice"]("-1"))())[0]"#,
    ),
];

pub const DIGITS_TABLE: [(char, &str); 10] = [
    ('0', "+[]+[]"),
    ('1', "+!+[]+[]"),
    ('2', "!+[]+!+[]+[]"),
    ('3', "!+[]+!+[]+!+[]+[]"),
    ('4', "!+[]+!+[]+!+[]+!+[]+[]"),
    ('5', "!+[]+!+[]+!+[]+!+[]+!+[]+[]"),
    ('6', "!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+[]"),
    ('7', "!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+[]"),
    ('8', "!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+[]"),
    ('9', "!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+!+[]+[]"),
];
pub const INFINITY: &str =
    "+((+!+[]+[])+((!![]+[])[+!+[]+!+[]+!+[]])+(+!+[]+[])+(+[]+[])+(+[]+[])+(+[]+[]))";
pub const BOOLEAN_OBJECT: &str = "((![])[(([][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))]+[])[+!+[]+!+[]+!+[]])+(([!![]]+[][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))])[+!+[]+[+[]]])+(([][[]]+[])[+!+[]])+((![]+[])[+!+[]+!+[]+!+[]])+((!![]+[])[+[]])+((!![]+[])[+!+[]])+(([][[]]+[])[+[]])+(([][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))]+[])[+!+[]+!+[]+!+[]])+((!![]+[])[+[]])+(([!![]]+[][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))])[+!+[]+[+[]]])+((!![]+[])[+!+[]])])";
pub const NUMBER_OBJECT: &str = "((+[])[(([][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))]+[])[+!+[]+!+[]+!+[]])+(([!![]]+[][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))])[+!+[]+[+[]]])+(([][[]]+[])[+!+[]])+((![]+[])[+!+[]+!+[]+!+[]])+((!![]+[])[+[]])+((!![]+[])[+!+[]])+(([][[]]+[])[+[]])+(([][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))]+[])[+!+[]+!+[]+!+[]])+((!![]+[])[+[]])+(([!![]]+[][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))])[+!+[]+[+[]]])+((!![]+[])[+!+[]])])";
pub const STRING_OBJECT: &str = "(([]+[])[(([][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))]+[])[+!+[]+!+[]+!+[]])+(([!![]]+[][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))])[+!+[]+[+[]]])+(([][[]]+[])[+!+[]])+((![]+[])[+!+[]+!+[]+!+[]])+((!![]+[])[+[]])+((!![]+[])[+!+[]])+(([][[]]+[])[+[]])+(([][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))]+[])[+!+[]+!+[]+!+[]])+((!![]+[])[+[]])+(([!![]]+[][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))])[+!+[]+[+[]]])+((!![]+[])[+!+[]])])";
pub const FUNCTION_OBJECT: &str = "([][((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]])][(([][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))]+[])[+!+[]+!+[]+!+[]])+(([!![]]+[][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))])[+!+[]+[+[]]])+(([][[]]+[])[+!+[]])+((![]+[])[+!+[]+!+[]+!+[]])+((!![]+[])[+[]])+((!![]+[])[+!+[]])+(([][[]]+[])[+[]])+(([][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))]+[])[+!+[]+!+[]+!+[]])+((!![]+[])[+[]])+(([!![]]+[][(((![]+[])[+[]])+((![]+[])[+!+[]+!+[]])+((![]+[])[+!+[]])+((!![]+[])[+[]]))])[+!+[]+[+[]]])+((!![]+[])[+!+[]])])";

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_full_alphabet() {
        let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;:\",./<>?€";
        let mut output = String::new();
        for c in chars.chars() {
            let e = encode(&c.to_string(), 9);
            output.push_str(format!("console.log('{c}', {e});\n").as_str());
        }
        let mut file = File::create("toto_test.js").unwrap();
        file.write(output.as_bytes()).unwrap();

        let output = std::process::Command::new("node")
            .arg("toto_test.js")
            .output()
            .unwrap();

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let mut parts = line.split(" ");
            let left = parts.next().unwrap();
            let right = parts.next().unwrap();
            assert_eq!(left, right);
        }
        assert!(output.status.success());
        std::fs::remove_file("toto_test.js").unwrap();
    }

    #[test]
    fn cache_test() {
        let chars =
            "€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€€";
        for c in chars.chars() {
            encode(&c.to_string(), 9);
        }
    }
}
