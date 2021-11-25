#![warn(missing_docs)]

//! This library provides the [run] and [eject] functions which are used by the crow binary crate

mod commands;
mod crow_commands;
mod crow_db;
mod events;
mod fuzzy;
mod history;
mod input;
mod rendering;
mod scored_commands;
mod state;

use crossterm::{event::DisableMouseCapture, execute, terminal::disable_raw_mode};
use std::io::Error;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg, SubCommand};

fn initialize_arg_parser() -> App<'static, 'static> {
    let db_path_arg = Arg::with_name("db_path")
        .help("File path to the json file where commands are saved.\nDefaults to '~/.config/crow/'")
        .short("p")
        .long("path")
        .takes_value(true);

    let db_file_arg = Arg::with_name("db_name")
        .help("Name of the json file where commands are saved.\nDefaults to 'crow_db.json'")
        .short("f")
        .long("file")
        .takes_value(true);

    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .subcommand(
            SubCommand::with_name("search")
                .about("Search through saved commands.\nThis subcommand can be omitted if only default arguments are used, because it is crow default behavior when run without a subcommand.")
                .version("0.1.0")
                .author(crate_authors!("\n"))
                .arg(&db_path_arg)
                .arg(&db_file_arg),
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
                )
                .arg(&db_path_arg)
                .arg(&db_file_arg),
        )
        .subcommand(
            SubCommand::with_name("add:last")
                .about("add last used CLI command to crow")
                .version("0.1.0")
                .author(crate_authors!("\n"))
                .arg(&db_path_arg)
                .arg(&db_file_arg),
        )
        .subcommand(
            SubCommand::with_name("add:pick")
                .about("NOTE: THIS COMMAND IS NOT YET IMPLEMENTED!\nAllows the user to add a command by picking from the last history commands")
                .version("0.1.0")
                .author(crate_authors!("\n")),
        )
}

/// Starts crow, parses command line arguments and runs the chosen command.
pub fn run() -> Result<(), Error> {
    let arg_parser = initialize_arg_parser();
    let matches = arg_parser.get_matches();

    match matches.subcommand() {
        ("add", Some(sub_matches)) => commands::add::run(sub_matches),
        ("add:last", Some(sub_matches)) => commands::add_last::run(sub_matches),
        ("add:pick", Some(_sub_matches)) => {
            // TODO
            println!("Sorry, this command is not yet implemented!");
            Ok(())
        }
        ("search", sub_matches) => commands::default::run(sub_matches),
        (_, sub_matches) => commands::default::run(sub_matches),
    }
}

/// Disables the terminals raw mode, prints a message to stderr and exits the currently running
/// program.
pub fn eject(reason: &str) -> ! {
    disable_raw_mode().unwrap();
    execute!(std::io::stdout(), DisableMouseCapture).unwrap();

    eprintln!("{}", reason);
    std::process::exit(-1);
}
