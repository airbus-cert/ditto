use crate::js::utils::string::*;
use log::trace;
use rand::Rng;
use rand::prelude::*;

pub fn create_replace_regex(
    str: &str,
    min_size: usize,
    max_size: usize,
    rng: &mut impl Rng,
) -> Option<(String, String)> {
    let min_size = if min_size <= 0 { 1 } else { min_size };
    let max_size = if max_size <= 0 { 5 } else { max_size };

    let parts = randomly_cut_string(min_size, max_size, str, false);

    if parts.len() <= 1 {
        return None;
    }
    let mut pattern = String::new();
    pattern.push_str(generate_safe_noise(5, rng).as_str());
    pattern.push_str("\\d{3}");
    pattern.push_str(generate_safe_noise(4, rng).as_str());
    let to_replace = pattern.clone().replace(
        "\\d{3}",
        rand::rng().random_range(100..1000).to_string().as_str(),
    );
    let replacement = parts.join(to_replace.as_str());
    trace!(
        "Generated regex: /{}/g, replacement: {:?}",
        pattern, replacement
    );
    Some((pattern, replacement))
}
