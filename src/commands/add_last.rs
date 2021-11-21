use clap::ArgMatches;
use crossterm::style::Stylize;
use dialoguer::{Confirm, Editor};
use nanoid::nanoid;

use crate::{
    crow_commands::CrowCommand,
    crow_db::{CrowDBConnection, FilePath},
    history::read_last_history_command,
};

use std::io::Error;

/// Tries to read the last command from the history of the users configured default shell and asks
/// the user if it should be saved.
/// If the command should be saved, the user is prompted for a description.
/// Upon saving the command will be written to the crow_db json file.
pub fn run(arg_matches: &ArgMatches) -> Result<(), Error> {
    let last_history_command = read_last_history_command();
    println!(
        "\nThe last command was: {}",
        last_history_command.clone().cyan()
    );

    let should_save = Confirm::new()
        .with_prompt("Do you want to save that command?")
        .default(false)
        .interact()?;

    if !should_save {
        return Ok(());
    };

    let description = Confirm::new()
        .with_prompt("Do you want to add a description")
        .default(true)
        .interact()?;

    let description = if description {
        Editor::new().edit("")?.unwrap()
    } else {
        "".to_string()
    };

    let new_command = CrowCommand {
        id: nanoid!(),
        command: last_history_command,
        description,
    };

    CrowDBConnection::new(FilePath::new(
        arg_matches.value_of("db_path"),
        arg_matches.value_of("db_name"),
    ))
    .add_command(new_command)
    .write();
    Ok(())
}
