//! Abstraction of read and write processes to the crow configuration file.

use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs::{create_dir_all, read_to_string, write},
    ops::Deref,
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

#[derive(Debug, Clone, PartialEq)]
pub struct FilePath(PathBuf);

impl Deref for FilePath {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.to_str().unwrap_or("")
    }
}

impl Default for FilePath {
    fn default() -> Self {
        Self(Self::create_path_and_intermediate_dirs(None, None))
    }
}

impl Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self)
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
    /// This does only create intermediate directories not the crow db file itself!
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
        .read()
    }

    /// Returns a list reference to the commands in the database
    pub fn commands(&self) -> &[CrowCommand] {
        self.commands.commands()
    }

    /// Writes all commands which are currently inside the memory database into
    /// the crow_db file.
    pub fn write(&self) -> &Self {
        let crow_db_json = match serde_json::to_string(&self.commands) {
            Ok(json) => json,
            Err(error) => eject(&format!("Could not parse to JSON. {}", error)),
        };

        if let Err(error) = write(self.path().as_path(), crow_db_json) {
            eject(&format!("Could not write database file. {}", error));
        };

        self
    }

    /// Adds a command to the in memory database.
    /// [self.write()] needs to be called in order to save to the json file.
    pub fn add_command(&mut self, command: CrowCommand) -> &mut Self {
        self.commands.commands_mut().push(command);
        self
    }

    /// Removes a command from the in memory database.
    /// [self.write()] needs to be called in order to save to the json file.
    pub fn remove_command(&mut self, command: &CrowCommand) -> &mut Self {
        self.commands.commands_mut().retain(|c| c.id != command.id);
        self
    }

    /// Reads the database json file into an existing connection, parses the json and returns an in-memory [CrowDBConnection]
    pub fn read(mut self) -> Self {
        let db_file = read_to_string(self.path().as_path())
            .expect("Error: crow_db.json file has not been initialized!");

        let commands: Commands =
            serde_json::from_str(&db_file).expect("Error: unable to parse crow_db.json file!");

        self.commands = commands;
        self
    }

    /// Set the crow db's commands.
    pub fn set_commands(mut self, commands: Vec<CrowCommand>) -> Self {
        self.commands.set_commands(commands);
        self
    }

    /// Get a reference to the crow dbconnection's path.
    pub fn path(&self) -> &FilePath {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    // NOTE: we always use a separate directory unique to the respective test function, because our
    // tests run concurrently most of the time and we want to avoid collisions between tests!

    mod file_path {
        use nanoid::nanoid;
        use std::path::Path;

        use crate::crow_db::FilePath;

        #[test]
        fn correctly_creates_path_and_intermediate_dirs() {
            let fn_path = &format!("./testdata/tmp/{}", nanoid!());
            let file_path = FilePath::new(Some(&fn_path), Some("crow_db.json"));

            assert_eq!(
                file_path.to_str().unwrap(),
                format!("{}/crow_db.json", fn_path)
            );

            let expected_path = Path::new(fn_path);

            assert!(
                expected_path.exists(),
                "Path {} does not exist",
                expected_path.to_str().unwrap()
            );

            std::fs::remove_dir_all(expected_path).unwrap();
        }
    }

    mod shell {
        use nanoid::nanoid;
        use std::path::Path;

        use crate::{
            crow_commands::CrowCommand,
            crow_db::{CrowDBConnection, FilePath},
        };

        #[test]
        fn initializes_db_file_if_not_exists() {
            let fn_path = &format!("./testdata/tmp/{}", nanoid!());
            let file_path = FilePath::new(Some(&fn_path), Some("crow_db.json"));

            let connection = CrowDBConnection::new(file_path.clone());

            connection.write();

            assert!(
                file_path.as_path().exists(),
                "Path {} does not exist",
                file_path.to_str().unwrap()
            );

            std::fs::remove_dir_all(Path::new(fn_path)).unwrap();
        }

        #[test]
        fn reads_existing_file_instead_of_overwrite() {
            // NOTE: We use our actual fixture file here instead of a temporary one!
            let fn_path = "./testdata";
            let file_path = FilePath::new(Some(&fn_path), Some("crow.json"));

            let connection = CrowDBConnection::new(file_path);

            let expected_command_1 = CrowCommand {
                id: "test_command_1".to_string(),
                command: "echo 'hi from db'".to_string(),
                description: "This is a test command".to_string(),
            };
            let expected_command_2 = CrowCommand {
                id: "test_command_2".to_string(),
                command: "".to_string(),
                description: "".to_string(),
            };

            assert_eq!(
                connection.commands(),
                &[expected_command_1, expected_command_2]
            );
        }

        #[test]
        fn correctly_adds_command() {
            let fn_path = &format!("./testdata/tmp/{}", nanoid!());
            let file_path = FilePath::new(Some(&fn_path), Some("crow.json"));

            let command_1 = CrowCommand {
                id: "1".to_string(),
                command: "".to_string(),
                description: "".to_string(),
            };

            let command_2 = CrowCommand {
                id: "2".to_string(),
                command: "".to_string(),
                description: "".to_string(),
            };

            let mut connection = CrowDBConnection::new(file_path);
            connection
                .add_command(command_1.clone())
                .add_command(command_2.clone());

            assert_eq!(connection.commands(), &[command_1, command_2]);
            std::fs::remove_dir_all(Path::new(fn_path)).unwrap();
        }

        #[test]
        fn correctly_removes_command() {
            let fn_path = &format!("./testdata/tmp/{}", nanoid!());
            let file_path = FilePath::new(Some(&fn_path), Some("crow.json"));

            let command_1 = CrowCommand {
                id: "1".to_string(),
                command: "".to_string(),
                description: "".to_string(),
            };

            let command_2 = CrowCommand {
                id: "2".to_string(),
                command: "".to_string(),
                description: "".to_string(),
            };

            let mut connection = CrowDBConnection::new(file_path.clone());
            connection
                .add_command(command_1.clone())
                .add_command(command_2.clone())
                .write();

            // Make sure that our current connection contains the correct values before removing a
            // command.
            assert_eq!(
                connection.commands(),
                &[command_1.clone(), command_2.clone()]
            );

            connection.remove_command(&command_1).write();

            // Make sure that our in memory representation has the correct commands after
            // removing a command.
            assert_eq!(connection.commands(), &[command_2.clone()]);

            let connection_2 = CrowDBConnection::new(file_path);

            // Assert that commands have been written to the database file correctly be opening
            // another connection.
            assert_eq!(connection_2.commands(), &[command_2]);
            std::fs::remove_dir_all(Path::new(fn_path)).unwrap();
        }
    }
}
