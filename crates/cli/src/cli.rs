use clap::{Parser, ValueEnum};

pub const APPLICATION_NAME: &str = env!("CARGO_PKG_NAME");

pub static INTRO: &str = "
ditto is an obfuscation tool that uses tree-sitter to parse and transform code.
It supports multiple languages and allows users to apply custom rulesets for deobfuscation.
";

#[derive(Parser, Debug, Clone)]
#[clap(
    name = APPLICATION_NAME,
    author,
    version,
    about,
    disable_help_flag = true,
    disable_version_flag = true,
)]
pub struct Cli {
    /// Help information
    #[arg(long, short)]
    pub help: bool,

    /// Version information
    #[arg(short = 'v', short_alias = 'V', long, action = clap::builder::ArgAction::Version)]
    version: (),

    /// Path to the script file
    #[arg(long, short)]
    pub path: Option<String>,

    /// Impostor used for obfuscation (not case sensitive)
    #[arg(long, short, value_enum)]
    pub impostor: Option<String>,

    /// List impostors
    #[arg(long, short = 'L', alias = "ls")]
    pub list: bool,

    /// Show computation time for the obfuscation process
    #[arg(long, short)]
    pub time: bool,

    /// Debug level for the deobfuscation process
    #[arg(long, short, value_enum, default_value_t = DebugLevel::Info, alias = "log")]
    pub debug_level: DebugLevel,

    /// Only output the obfuscated script
    #[arg(long, short)]
    pub quiet: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum DebugLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<DebugLevel> for log::LevelFilter {
    fn from(level: DebugLevel) -> Self {
        match level {
            DebugLevel::Off => log::LevelFilter::Off,
            DebugLevel::Error => log::LevelFilter::Error,
            DebugLevel::Warn => log::LevelFilter::Warn,
            DebugLevel::Info => log::LevelFilter::Info,
            DebugLevel::Debug => log::LevelFilter::Debug,
            DebugLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

pub struct Example {
    pub title: &'static str,
    pub cmd: &'static str,
}

pub static EXAMPLES_TEMPLATE: &str = "
**Examples:**

${examples
*${example-number})* ${example-title}: `${example-cmd}`
}
";

pub static EXAMPLES: &[Example] = &[
    Example {
        title: "List available impostors",
        cmd: "ditto -L",
    },
    Example {
        title: "Deobfuscate with an impostor",
        cmd: "ditto --i Tinkatuff --p test.js",
    },
];

pub static LANGUAGES_LIST_TEMPLATE: &str = r#"
**Available impostors:**
${impostors_tree}
"#;
