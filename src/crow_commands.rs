//! [CrowCommand] models which represent a command saved by the user inside [CrowDB] containing
//! a unique [Id], the actual command and a description.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    ops::{Deref, DerefMut},
};

// TODO maybe change this so that it uses the newtype pattern
pub type Id = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd)]
pub struct CrowCommand {
    pub id: Id,
    pub command: String,
    pub description: String,
}

impl CrowCommand {
    /// Creates a single string from the command and the description which can
    /// be used to be matched agains (e.g. for fuzzy searching).
    pub fn match_str(&self) -> String {
        format!("{}: {}", &self.command, &self.description)
    }
}

impl Display for CrowCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Id: {}, Command: {}, Description: {}",
            self.id, self.command, self.description
        )
    }
}

#[derive(PartialEq, Clone)]
pub struct Commands(HashMap<Id, CrowCommand>);

impl Commands {
    pub fn normalize(commands: &[CrowCommand]) -> Self {
        Self(
            commands
                .iter()
                .map(|cmd| (cmd.id.clone(), cmd.clone()))
                .collect(),
        )
    }

    // TODO returning an arbitrary order is a bit weird from a users perspective,
    // we should probably make this somehow sorted.
    pub fn denormalize(&self) -> impl Iterator<Item = &CrowCommand> {
        self.values()
    }

    pub fn update_command(&mut self, command_id: Id, command: &str) {
        if let Some(c) = self.get_mut(&command_id) {
            *c = CrowCommand {
                command: command.to_string(),
                ..c.clone()
            }
        }
    }

    pub fn update_description(&mut self, command_id: Id, description: &str) {
        if let Some(c) = self.get_mut(&command_id) {
            *c = CrowCommand {
                description: description.to_string(),
                ..c.clone()
            }
        }
    }
}

impl Deref for Commands {
    type Target = HashMap<Id, CrowCommand>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Commands {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl Debug for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Commands").field(&self.0).finish()
    }
}

/// Crow commands are a normalized view of the commands that are stored inside
/// the crow_db.json file.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct CrowCommands {
    commands: Commands,

    /// List of all command ids
    command_ids: Vec<Id>,
}

impl CrowCommands {
    pub fn _new(commands: Commands, command_ids: Vec<Id>) -> Self {
        Self {
            commands,
            command_ids,
        }
    }

    /// Get a reference to the crow commands's commands.
    pub fn commands(&self) -> &Commands {
        &self.commands
    }

    /// Set the crow commands's command ids.
    pub fn set_command_ids(&mut self, command_ids: Vec<Id>) {
        self.command_ids = command_ids;
    }

    /// Get a mutable reference to the crow commands's commands.
    pub fn commands_mut(&mut self) -> &mut Commands {
        &mut self.commands
    }

    /// Set the crow commands's commands.
    pub fn set_commands(&mut self, commands: Commands) {
        self.commands = commands;
    }
}
