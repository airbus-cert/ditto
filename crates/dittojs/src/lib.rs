extern crate ditto;
use ditto::impostor::apply_engine;

const IMPOSTORS: [&str; 8] = [
    "pikachu",
    "bulbasaur",
    "charizard",
    "ivysaur",
    "onix",
    "eevee",
    "alakazam",
    "ditto",
];

wit_bindgen::generate!({
    world: "ditto",
});

struct Ditto;
impl Guest for Ditto {
    fn get_impostors() -> Vec<String> {
        IMPOSTORS.iter().map(|s| s.to_string()).collect()
    }

    fn obfuscate(source: String, impostor: String) -> String {
        apply_engine(&impostor, &source).unwrap()
    }
}

// export! defines that the `MyHost` struct defined below is going to define
// the exports of the `world`, namely the `run` function.
export!(Ditto);
