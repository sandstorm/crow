#![warn(missing_docs)]

//! This library provides the [run] and [eject] functions which are used by the crow binary crate

mod commands;
mod crow_db;
mod events;
mod fuzzy;
mod history;
mod input;
mod rendering;
mod state;

use crossterm::terminal::disable_raw_mode;
use std::io::Error;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};

fn initialize_arg_parser() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("search")
                .about("Search through saved commands. This subcommand can be omitted, because it is crow default behavior when run without a subcommand.")
                .version("0.1.0")
                .author(crate_authors!("\n")),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("add a new command to crow")
                .version("0.1.0")
                .author(crate_authors!("\n"))
                .arg(
                    Arg::with_name("command")
                        .help("command to add")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("add:last")
                .about("add last used CLI command to crow")
                .version("0.1.0")
                .author(crate_authors!("\n")),
        )
        .subcommand(
            SubCommand::with_name("add:pick")
                .about("allows the user to add a command by picking from the last history commands")
                .version("0.1.0")
                .author(crate_authors!("\n")),
        )
}

/// Starts crow, parses command line arguments and runs the chosen command.
pub fn run() -> Result<(), Error> {
    let arg_parser = initialize_arg_parser();
    let matches = arg_parser.get_matches();

    match matches.subcommand() {
        ("add", Some(sub_matches)) => commands::add::run(sub_matches.value_of("command").unwrap()),
        ("add:last", _) => commands::add_last::run(),
        ("add:pick", _) => {
            println!("Sorry, this command is not yet implemented!");
            Ok(())
        } // TODO,
        _ => commands::default::run(),
    }
}

/// Disables the terminals raw mode, prints a message to stderr and exits the currently running
/// program.
pub fn eject(reason: &str) -> ! {
    disable_raw_mode().unwrap();

    eprintln!("{}", reason);
    std::process::exit(-1);
}
