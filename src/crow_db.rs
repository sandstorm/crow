//! Abstraction of read and write processes to the crow configuration file.

use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, read_to_string, write},
    path::{Path, PathBuf},
};

use dirs::home_dir;

use crate::{crow_commands::CrowCommand, eject};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Commands {
    commands: Vec<CrowCommand>,
}

impl Default for Commands {
    fn default() -> Self {
        Self { commands: vec![] }
    }
}

impl Commands {
    /// Get a reference to the commands's commands.
    fn commands(&self) -> &[CrowCommand] {
        self.commands.as_ref()
    }

    /// Set the commands's commands.
    fn set_commands(&mut self, commands: Vec<CrowCommand>) {
        self.commands = commands;
    }

    /// Get a mutable reference to the commands's commands.
    fn commands_mut(&mut self) -> &mut Vec<CrowCommand> {
        &mut self.commands
    }
}

#[derive(Debug, Clone)]
pub struct FilePath(PathBuf);

impl Default for FilePath {
    fn default() -> Self {
        Self(Self::create_path_and_intermediate_dirs(None, None))
    }
}

impl FilePath {
    const DEFAULT_CONFIG_FILE: &'static str = "crow_db.json";

    pub fn new(path: Option<&str>, file_name: Option<&str>) -> Self {
        let path_buffer = match path {
            Some(p) => {
                let mut path_buffer = PathBuf::new();
                path_buffer.push(shellexpand::tilde(p).as_ref());
                Some(path_buffer)
            }
            None => None,
        };

        Self(Self::create_path_and_intermediate_dirs(
            path_buffer,
            file_name,
        ))
    }

    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    pub fn to_str(&self) -> Option<&str> {
        self.0.to_str()
    }

    /// Creates a path buffer for a local config path inside the users home directory
    /// Typically this path is `$HOME/.config/crow/` on UNIX systems
    ///
    /// # Panics
    ///
    /// If this function is somehow unable to either find the home directory or
    /// create the full path, it will panic.
    fn create_path_and_intermediate_dirs(
        path_buffer: Option<PathBuf>,
        file: Option<&str>,
    ) -> PathBuf {
        let mut path_buffer = path_buffer.unwrap_or_else(Self::default_path);

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

        path_buffer.push(file.unwrap_or(Self::DEFAULT_CONFIG_FILE));
        path_buffer
    }

    fn default_path() -> PathBuf {
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
        path_buffer
    }
}

#[derive(Clone, Debug)]
pub struct CrowDBConnection {
    commands: Commands,
    path: FilePath,
}

impl Default for CrowDBConnection {
    fn default() -> Self {
        Self {
            commands: Commands::default(),
            path: FilePath::default(),
        }
    }
}

impl CrowDBConnection {
    pub fn new(file_path: FilePath) -> Self {
        Self::connect_and_initialize_file_if_not_exists(file_path)
    }

    /// Initializes the crow database json file if it does not exist (typically at `$HOME/.config/crow/crow_db.json` on UNIX systems).
    ///
    /// # Panics
    /// This function may panic for various reasons:
    /// * if paths could not be resolved
    /// * if the file could not be written
    /// * if the default content of the file could not be parsed to JSON
    fn connect_and_initialize_file_if_not_exists(file_path: FilePath) -> Self {
        if !file_path.as_path().exists() {
            match file_path.to_str() {
                Some(file_path) => {
                    println!("Creating config file: {}", file_path);
                }
                None => eject("Could not parse path to string"),
            }

            let connection = Self {
                commands: Commands::default(),
                path: file_path,
            };
            connection.write();

            return connection;
        }

        Self {
            commands: Commands::default(),
            path: file_path,
        }
    }

    /// Returns a list reference to the commands in the database
    pub fn commands(&self) -> &[CrowCommand] {
        self.commands.commands()
    }

    /// Writes all commands which are currently inside the memory database into
    /// the crow_db file.
    pub fn write(&self) -> Self {
        let crow_db_json = match serde_json::to_string(&self.commands) {
            Ok(json) => json,
            Err(error) => eject(&format!("Could not parse to JSON. {}", error)),
        };

        if let Err(error) = write(self.path().as_path(), crow_db_json) {
            eject(&format!("Could not write database file. {}", error));
        };

        self.clone()
    }

    /// Adds a command to the in memory database.
    /// The in memory database is being read from file before the command is added.
    /// [self.write()] needs to be called in order to save to the json file.
    pub fn add_command(&mut self, command: CrowCommand) -> Self {
        let mut connection = self.read();
        connection.commands.commands_mut().push(command);

        connection
    }

    /// Removes a command from the in memory database.
    /// The in memory database is being read from file before the command is removed.
    /// [self.write()] needs to be called in order to save to the json file.
    pub fn remove_command(&mut self, command: &CrowCommand) -> Self {
        let mut connection = self.read();

        connection
            .commands
            .commands_mut()
            .retain(|c| c.id != command.id);

        connection
    }

    /// Reads the database json file, parses the json and returns an in-memory [CrowDBConnection]
    pub fn read(&self) -> Self {
        let db_file = read_to_string(self.path().as_path())
            .expect("Error: crow_db.json file has not been initialized!");

        let commands: Commands =
            serde_json::from_str(&db_file).expect("Error: unable to parse crow_db.json file!");

        CrowDBConnection {
            commands,
            path: self.path.clone(),
        }
    }

    /// Set the crow db's commands.
    pub fn set_commands(&mut self, commands: Vec<CrowCommand>) -> Self {
        self.commands.set_commands(commands);
        self.clone()
    }

    /// Get a reference to the crow dbconnection's path.
    pub fn path(&self) -> &FilePath {
        &self.path
    }
}
