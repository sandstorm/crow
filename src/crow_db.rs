use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    fs::{create_dir_all, read_to_string, write},
    path::PathBuf,
};

use dirs::home_dir;

use crate::eject;

// TODO maybe change this so that it uses the newtype pattern
pub type Id = String;

#[derive(Serialize, Deserialize)]
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

impl Clone for CrowCommand {
    fn clone(&self) -> CrowCommand {
        CrowCommand {
            id: self.id.clone(),
            command: self.command.clone(),
            description: self.description.clone(),
        }
    }
}

impl fmt::Debug for CrowCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrowCommand")
            .field("id", &self.id)
            .field("command", &self.command)
            .field("description", &self.description)
            .finish()
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

impl PartialEq for CrowCommand {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct CrowDB {
    commands: Vec<CrowCommand>,
}

impl CrowDB {
    /// Initializes the default crow database json file (typically at `$HOME/.config/crow/crow_db.json` on UNIX systems).
    ///
    /// # Panics
    /// This function may panic for various reasons:
    /// * if paths could not be resolved
    /// * if the file could not be written
    /// * if the default content of the file could not be parsed to JSON
    fn initialize_file() {
        let file_path_buffer = get_db_path();
        if !file_path_buffer.as_path().exists() {
            match file_path_buffer.to_str() {
                Some(file) => {
                    println!("Creating config file: {}", file);
                }
                None => eject("Could not initialize crow config file"),
            }

            let crow_db = CrowDB { commands: vec![] };
            crow_db.write();
        }
    }

    /// Returns a list reference to the commands in the database
    pub fn commands(&self) -> &Vec<CrowCommand> {
        &self.commands
    }

    /// Writes all commands which are currently inside the memory database into
    /// the crow_db file.
    pub fn write(&self) {
        let file_path_buffer = get_db_path();

        let crow_db_json = match serde_json::to_string(&self) {
            Ok(json) => json,
            Err(error) => eject(&format!("Could not parse to JSON. {}", error)),
        };

        if let Err(error) = write(file_path_buffer.as_path(), crow_db_json) {
            eject(&format!("Cloud not write database file. {}", error));
        };
    }

    /// Adds a command to the in memory database and saves the in memory database
    /// to the database json file.
    /// TODO we should probably handle the "in memory" part in [crate::state] instead and just
    /// trigger a write from the state.
    pub fn add_command(command: CrowCommand) -> Result<(), &'static str> {
        let mut db = CrowDB::read();
        db.commands.push(command);
        db.write();

        Ok(())
    }

    /// Removes a command from the in memory database and saves the in memory database
    /// to the database json file.
    /// TODO we should probably handle the "in memory" part in [crate::state] instead and just
    /// trigger a write from the state.
    pub fn remove_command(command: &CrowCommand) -> Result<Vec<CrowCommand>, &'static str> {
        let mut db = CrowDB::read();
        let filtered: Vec<CrowCommand> = db
            .commands()
            .iter()
            .filter(|c| *c != command)
            .cloned()
            .collect();

        db.set_commands(filtered.clone());
        db.write();

        Ok(filtered)
    }

    /// Reads the database json file, parses the json and returns an in-memory [CrowDB]
    pub fn read() -> CrowDB {
        CrowDB::initialize_file();

        let db_path = get_db_path();

        let db_file = read_to_string(db_path.as_path())
            .expect("Error: crow_db.json file has not been initialized!");

        let crow_db: CrowDB =
            serde_json::from_str(&db_file).expect("Error: unable to parse crow_db.json file!");

        crow_db
    }

    /// Set the crow db's commands.
    pub fn set_commands(&mut self, commands: Vec<CrowCommand>) {
        self.commands = commands;
    }
}

/// Creates a path buffer for a local config path inside the users home directory
/// Typically this path is `$HOME/.config/crow/` on UNIX systems
///
/// # Panics
///
/// If this function is somehow unable to either find the home directory or
/// create the full path, it will panic.
fn get_db_path() -> PathBuf {
    let mut path_buffer = PathBuf::new();
    let home_dir = match home_dir() {
        Some(dir) => dir,
        None => eject("Could not retrieve home directory. {}"),
    };
    let home_dir = match home_dir.to_str() {
        Some(str) => str,
        None => eject("Could not parse home directory into string"),
    };

    path_buffer.push(format!("{}/.config/crow/", home_dir));

    if !path_buffer.as_path().exists() {
        match path_buffer.to_str() {
            Some(str) => {
                println!("Creating config path: {}", str);
            }
            None => eject("Could not parse config path to string"),
        }

        if let Err(error) = create_dir_all(path_buffer.as_path()) {
            eject(&format!(
                "Could not create directories up to config path. {}",
                error
            ));
        };
    }

    path_buffer.push("crow_db.json");
    path_buffer
}
