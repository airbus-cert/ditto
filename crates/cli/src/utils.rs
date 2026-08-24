use ditto::impostor::impostors;

use std::collections::BTreeMap;

pub fn get_impostors_tree() -> BTreeMap<String, Vec<String>> {
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (name, lang) in impostors() {
        let name = capitalize(name);
        grouped.entry(lang.to_string()).or_default().push(name);
    }

    grouped
}

pub fn render_impostors_tree() -> String {
    let mut out = String::new();
    for (lang, mut names) in get_impostors_tree() {
        names.sort_unstable();
        out.push_str(&format!("- {}:\n", lang));
        for name in names {
            out.push_str(&format!("  • {}\n", name));
        }
    }
    out
}

pub fn is_valid_impostor(impostor: &str) -> bool {
    for (i, _) in impostors() {
        if i.to_lowercase() == impostor.to_lowercase() {
            return true;
        }
    }
    false
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
