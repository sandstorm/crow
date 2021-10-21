use crate::eject;
use dirs::home_dir;
use regex::Regex;
use std::{env, fs::File, io::BufRead, io::BufReader, path::PathBuf};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum History {
    Zsh,
    Bash,
}

impl History {
    /// Returns the typical history file location [PathBuf] for the history type.
    ///
    /// # Panics
    /// This function will terminate if the users home directory can't be determined.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use crow::history::History;
    /// let zsh_hist = History::Zsh;
    /// let hist_file_path = zsh_hist.history_path(); // => "~/.zsh_history"
    /// ```
    fn history_path(&self) -> PathBuf {
        use History::*;

        let kind = match self {
            Zsh => ".zsh_history",
            Bash => ".bash_history",
        };

        let mut dir = home_dir().unwrap_or_else(|| {
            eject("Unable to determine home path");
        });
        dir.push(kind);
        dir
    }
}

/// Tries to determine the users default shell by checking if the SHELL environment
/// variable contains an identifier (e.g. "zsh" or "bash").
///
/// # Example
///
/// ```ignore
/// use std::env;
/// use crow::history::*;
///
/// env::set_var("SHELL", "/bin/bash");
///
/// assert_eq!(detect_shell().unwrap(), History::Bash)
/// ```
fn detect_shell() -> Option<History> {
    const SHELL_MATCHES: &[(&str, History)] = &[("zsh", History::Zsh), ("bash", History::Bash)];

    let shell_path = env::var("SHELL").ok()?;

    for (text, sh) in SHELL_MATCHES {
        if shell_path.contains(text) {
            return Some(*sh);
        }
    }

    None
}

/// Reads the users history file from the determined default shell and returns
/// its content as lines.
fn read_history_file() -> Vec<String> {
    let shell = if let Some(shell) = detect_shell() {
        shell
    } else {
        eject("Did not find a proper shell!");
    };

    let path = shell.history_path();
    let file = File::open(&path).unwrap_or_else(|_| {
        eject(&format!("Unable to open detected history file: {:?}", path));
    });
    let file = BufReader::new(file);

    let lines: Vec<String> = file.lines().filter_map(|line| line.ok()).collect();
    lines
}

/// Reads out the last entered command from the history file of the users determined
/// default shell.
pub fn read_last_history_command() -> String {
    let lines = read_history_file();
    // Get the penultimate line because we would otherwise retrieve the current
    // command (crow add:last).
    let last_command = &lines[lines.len() - 2];

    // Because we might encounter a .zsh_history we need to make sure that we remove
    // timestamps in front of the actual command.
    let re = Regex::new(r": [0-9]*:[0-9];").unwrap_or_else(|e| {
        eject(&format!("Invalid regular expression! {}", e));
    });

    let last_command_cleaned_up = re.replace(last_command, "");
    last_command_cleaned_up.to_string()
}
