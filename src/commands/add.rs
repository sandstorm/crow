use crossterm::style::Stylize;
use dialoguer::{Confirm, Editor};
use nanoid::nanoid;

use crate::crow_db::{CrowCommand, CrowDB};

use std::io::{self, Error};

/// Uses the command given by the user as CLI argument and prompts to save it.
/// Upon save the user is asked to provided a description.
/// When the command is saved, it is written to the crow_db json file.
pub fn run(command: &str) -> Result<(), Error> {
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

    match CrowDB::add_command(new_command) {
        Ok(()) => Ok(()),
        Err(error) => {
            let err = format!("Error: Could not save command! Reason: {}", error);
            Err(Error::new(io::ErrorKind::Other, err))
        }
    }
}
