use crate::js::utils::string::unescape_js_string;
use crate::tree::Node;
use rand::Rng;
use rand::prelude::*;

pub fn repack_array(elements: &Vec<Node>) -> String {
    let mut result = String::from("[");
    for (i, element) in elements.iter().enumerate() {
        result.push_str(element.text().unwrap());
        if i != elements.len() - 1 {
            result.push_str(",");
        }
    }
    result.push_str("]");
    result
}

pub fn is_sorted(elements: &Vec<Node>) -> bool {
    elements.windows(2).all(|pair| {
        if pair[0].kind() == "identifier" || pair[1].kind() == "identifier" {
            return false;
        }

        match (pair[0].text(), pair[1].text()) {
            (Ok(a), Ok(b)) => {
                let a = if pair[0].kind() == "string" {
                    unescape_js_string(a)
                } else {
                    a.to_owned()
                };

                let b = if pair[1].kind() == "string" {
                    unescape_js_string(b)
                } else {
                    b.to_owned()
                };

                a <= b
            }
            _ => false,
        }
    })
}

pub fn randomly_pack_array_elements(elements: &[Node], rng: &mut impl Rng) -> String {
    fn pack(items: &[String], rng: &mut impl Rng) -> String {
        if items.len() == 1 {
            return items[0].clone();
        }

        let mut parts = Vec::new();
        let mut i = 0;

        while i < items.len() {
            let remaining = items.len() - i;

            if remaining == 1 {
                parts.push(items[i].clone());
                i += 1;
                continue;
            }

            let take = if rng.random_bool(0.5) {
                1
            } else {
                rng.random_range(1..=remaining.min(3))
            };
            let chunk = &items[i..i + take];

            if chunk.len() == 1 {
                parts.push(chunk[0].clone());
            } else {
                parts.push(format!("[{}]", pack(chunk, rng)));
            }

            i += take;
        }

        parts.join(",")
    }

    let texts: Vec<String> = elements
        .iter()
        .map(|e| e.text().unwrap().to_string())
        .collect();

    format!("[{}]", pack(&texts, rng))
}
