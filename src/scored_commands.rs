//! Models which result from [fuzzy::fuzzy_search_commands] over [CrowCommands].
//! [ScoredCommands] contain an actual [CrowCommand], a fuzzy-score as well as the matching indices
//! of the search result.

use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use tui::text::Text;

use crate::crow_commands::{CrowCommand, Id};

/// A [ScoredCommand] contains a [CrowCommand] alongside scoring metadata and
/// a list of matching indices.
#[derive(Debug, Clone)]
pub struct ScoredCommand {
    score: i64,
    indices: Vec<usize>,
    command: CrowCommand,
}

impl From<&ScoredCommand> for Text<'_> {
    fn from(cmd: &ScoredCommand) -> Self {
        Text::from(cmd.command().command.clone())
    }
}

impl ScoredCommand {
    pub fn new(score: i64, indices: Vec<usize>, command: CrowCommand) -> Self {
        Self {
            score,
            indices,
            command,
        }
    }

    /// Get a reference to the scored command's score.
    pub fn score(&self) -> i64 {
        self.score
    }

    /// Get a reference to the scored command's indices.
    pub fn indices(&self) -> &[usize] {
        self.indices.as_ref()
    }

    /// Get a reference to the scored command's command.
    pub fn command(&self) -> &CrowCommand {
        &self.command
    }

    /// Set the scored command's score.
    pub fn set_score(&mut self, score: i64) {
        self.score = score;
    }
}

#[derive(Debug, Clone)]
pub struct ScoredCommands(HashMap<Id, ScoredCommand>);

impl Deref for ScoredCommands {
    type Target = HashMap<Id, ScoredCommand>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScoredCommands {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for ScoredCommands {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl ScoredCommands {
    pub fn normalize(commands: &[ScoredCommand]) -> Self {
        Self(
            commands
                .iter()
                .map(|cmd| (cmd.command.id.clone(), cmd.clone()))
                .collect(),
        )
    }

    pub fn denormalize(&self) -> impl Iterator<Item = &ScoredCommand> {
        self.values()
    }
}
