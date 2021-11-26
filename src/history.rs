use crate::eject;

use regex::Regex;
use std::{fs::File, io::BufRead, io::BufReader, path::PathBuf};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Shell {
    Zsh,
    Bash,
}

impl Shell {
    /// Tries to determine the users default shell by checking if the SHELL environment
    /// variable contains an identifier (e.g. "zsh" or "bash").
    pub fn from_path(shell_path: String) -> Option<Self> {
        const SHELL_MATCHES: &[(&str, Shell)] = &[("zsh", Shell::Zsh), ("bash", Shell::Bash)];

        for (text, sh) in SHELL_MATCHES {
            if shell_path.contains(text) {
                return Some(*sh);
            }
        }

        None
    }

    /// Returns the typical history file location [PathBuf] for the history type.
    ///
    /// # Panics
    /// This function will terminate if the users home directory can't be determined.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use crow::history::Shell;
    /// let zsh= Shell::Zsh;
    /// let hist_file_path = zsh.history_path(); // => "~/.zsh_history"
    /// ```
    fn history_file_name(&self) -> &str {
        match self {
            Self::Zsh => ".zsh_history",
            Self::Bash => ".bash_history",
        }
    }

    /// Reads the users history file from the determined default shell and returns
    /// its content as lines.
    fn read_history_file(&self, mut base_dir: PathBuf) -> Vec<String> {
        let file_name = self.history_file_name();

        base_dir.push(file_name);

        let file = File::open(&base_dir).unwrap_or_else(|_| {
            eject(&format!(
                "Unable to open detected history file: {:?}",
                base_dir
            ));
        });

        let file = BufReader::new(file);

        let lines: Vec<String> = file.lines().filter_map(|line| line.ok()).collect();
        lines
    }

    /// Reads out the last entered command from the history file of the users determined
    /// default shell.
    pub fn read_last_history_command(&self, base_dir: PathBuf) -> String {
        let lines = self.read_history_file(base_dir);

        // Get the penultimate line because we would otherwise retrieve the current
        // command (crow add:last).
        let last_command = &lines[lines.len() - 2];

        // Because we might encounter a .zsh_history we need to make sure that we remove
        // timestamps in front of the actual command.
        let re = Regex::new(r": [0-9]*:[0-9];").unwrap();
        re.replace(last_command, "").to_string()
    }
}

#[cfg(test)]
mod tests {
    mod from_path {
        use crate::history::Shell;

        #[test]
        fn detects_bash() {
            assert_eq!(Shell::from_path("/bin/bash".to_string()), Some(Shell::Bash));
        }

        #[test]
        fn detects_zsh() {
            assert_eq!(Shell::from_path("/bin/zsh".to_string()), Some(Shell::Zsh));
        }

        #[test]
        fn does_not_detect_others() {
            assert_eq!(Shell::from_path("/bin/fish".to_string()), None);
        }
    }

    mod read_last_history_command {
        use std::path::PathBuf;

        use crate::history::Shell;

        #[test]
        fn returns_correct_command_from_history() {
            let shell = Shell::from_path("/bin/bash".to_string()).unwrap();

            // Note: the path is relative to the root dir of the repository, because
            // this is where the cargo test command is invoked from!
            let path = PathBuf::from("./testdata/");

            let result = shell.read_last_history_command(path);

            assert_eq!(result, "echo \"Hi from test history\"");
        }

        #[test]
        fn correctly_cleans_up_zsh_commands() {
            let shell = Shell::from_path("/bin/zsh".to_string()).unwrap();

            // Note: the path is relative to the root dir of the repository, because
            // this is where the cargo test command is invoked from!
            let path = PathBuf::from("./testdata/");

            let result = shell.read_last_history_command(path);

            assert_eq!(result, "echo 'Hi from test zsh_history'");
        }
    }
}
