use clap::ArgMatches;
use crossterm::style::Stylize;
use dialoguer::{Confirm, Editor};
use nanoid::nanoid;

use crate::{
    crow_commands::CrowCommand,
    crow_db::{CrowDBConnection, FilePath},
};

use std::io::Error;

/// Uses the command given by the user as CLI argument and prompts to save it.
/// Upon save the user is asked to provided a description.
/// When the command is saved, it is written to the crow_db json file.
pub fn run(arg_matches: &ArgMatches) -> Result<(), Error> {
    let command = arg_matches.value_of("command").expect("Has command");

    let save_prompt = format!("Do you want to save command: {}?", command.cyan());
    let should_save = Confirm::new()
        .with_prompt(save_prompt)
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
        command: command.to_string(),
        description,
    };

    if let Some(p) = arg_matches.value_of("db_path") {
        println!("{}", p);
    }

    CrowDBConnection::new(FilePath::new(
        arg_matches.value_of("db_path"),
        arg_matches.value_of("db_name"),
    ))
    .add_command(new_command)
    .write();
    Ok(())
}
