extern crate ditto;
mod cli;
mod utils;

use clap::{CommandFactory, Parser};
use clap_help::Printer;
use cli::*;
use ditto::impostor::apply_engine;
use log::{LevelFilter, debug, error, info};
use std::{fs, process};
use termimad::ansi;
use utils::*;

fn main() {
    let mut cli = Cli::parse();
    if cli.help {
        let mut printer = Printer::new(Cli::command())
            .with("introduction", INTRO)
            .with("options", clap_help::TEMPLATE_OPTIONS_MERGED_VALUE);
        printer.template_keys_mut().push("languages");
        printer.set_template("languages", LANGUAGES_LIST_TEMPLATE);
        printer.template_keys_mut().push("examples");
        printer.set_template("examples", EXAMPLES_TEMPLATE);
        let skin = printer.skin_mut();
        skin.headers[0].compound_style.set_fg(ansi(177));
        skin.bold.set_fg(ansi(177));
        skin.italic.set_fg(ansi(177));
        for (i, example) in EXAMPLES.iter().enumerate() {
            printer
                .expander_mut()
                .sub("examples")
                .set("example-number", i + 1)
                .set("example-title", example.title)
                .set("example-cmd", example.cmd);
        }
        printer
            .expander_mut()
            .set("impostors_tree", render_impostors_tree());
        printer.print_help();

        return;
    }

    if cli.quiet {
        cli.debug_level = DebugLevel::Off;
        cli.list = false;
        cli.time = false;
    }

    pretty_env_logger::formatted_builder()
        .filter(None, LevelFilter::Off)
        .filter_module("ditto", LevelFilter::from(cli.debug_level))
        .filter_module(APPLICATION_NAME, LevelFilter::Info)
        .init();

    debug!("{:?}", cli);

    if cli.list {
        println!("{}", render_impostors_tree());
        process::exit(0);
    }

    let impostor = match cli.impostor {
        Some(i) => i,
        None => {
            error!("No specified specified. Use --impostor/-i to specify the impostor.");
            println!("{}", render_impostors_tree());
            process::exit(1);
        }
    };

    if !is_valid_impostor(&impostor) {
        error!("Invalid impostor specified: {impostor}");
        println!("{}", render_impostors_tree());
        process::exit(1);
    }

    let path = match cli.path {
        Some(p) => p,
        None => {
            error!("No file path provided. Use --path/-p to specify the script file.");
            process::exit(1);
        }
    };

    let source = fs::read_to_string(&path).unwrap_or_else(|e| {
        error!("Failed to read file {}: {}", path, e);
        process::exit(1);
    });

    let now = std::time::Instant::now();
    let result = apply_engine(impostor.as_str(), &source);
    let elapsed = now.elapsed();

    match result {
        Ok(r) => println!("{r}"),
        Err(err) => {
            error!("Failed to apply engine: {}", err);
            process::exit(1);
        }
    }
    if cli.time {
        println!("\n");
        info!("Obfuscation time: {:.2?}", elapsed);
    }
}
