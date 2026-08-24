use crate::Lang;
use crate::Lang::PowerShell;
use crate::error::DittoResult;
use crate::error::Error::EngineNotFound;
use crate::js::impostor::*;
use crate::pass::Source;
use crate::ps::drive::*;
use crate::ps::impostor::*;
use log::debug;

pub trait Impostor {
    fn lang(&self) -> Lang;
    fn transform(&self, source: &str) -> DittoResult<String>;
}

#[derive(Default)]
pub struct Ditto;

impl Impostor for Ditto {
    fn lang(&self) -> Lang {
        PowerShell
    }

    fn transform(&self, source: &str) -> DittoResult<String> {
        Ok(Source::new(&source)
            .transform(DriveByComment::default())?
            .output()?)
    }
}

macro_rules! register_pokemon {
    ( $( $name:literal => $type:ty ),* $(,)? ) => {
        pub fn impostors() -> Vec<(&'static str, Lang)> {
            vec![
                $( ($name, <$type>::default().lang()), )*
            ]
        }

        pub fn apply_engine(engine: &str, source: &str) -> DittoResult<String> {
            match engine.to_lowercase().as_str() {
                $( $name => {
                    debug!("Applying {} engine", $name);
                    <$type>::default().transform(source)
                }, )*
                _ => Err(EngineNotFound(engine.to_string())),
            }
        }
    };
}

// keep them in lowercase please
register_pokemon! {
    "pikachu"   => Pikachu,
    "bulbasaur" => Bulbasaur,
    "charizard" => Charizard,
    "ivysaur"   => Ivysaur,
    "onix"      => Onix,
    "eevee"     => Eevee,
    "ditto"     => Ditto,
    "alakazam"  => Alakazam,
    "tinkatuff" => Tinkatuff,
    "unown"     => Unown,
    "metagross" => Metagross,
    "wailord"   => Wailord,
    "clefable"  => Clefable,
    "cofagrigus"  => Cofagrigus,
}
