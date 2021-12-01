//! Models which result from [fuzzy::fuzzy_search_commands] over [CrowCommands].
//! [ScoredCommands] contain an actual [CrowCommand], a fuzzy-score as well as the matching indices
//! of the search result.

use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::crow_commands::Id;

/// A [ScoredCommand] contains a [CrowCommand] alongside scoring metadata and
/// a list of matching indices.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ScoredCommand {
    score: i64,
    indices: Vec<usize>,
    command_id: Id,
}

impl ScoredCommand {
    pub fn new(score: i64, indices: Vec<usize>, command_id: Id) -> Self {
        Self {
            score,
            indices,
            command_id,
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

    /// Get a reference to the scored command's command id.
    pub fn command_id(&self) -> &Id {
        &self.command_id
    }

    /// Set the scored command's score.
    pub fn _set_score(&mut self, score: i64) {
        self.score = score;
    }
}

#[derive(Debug, Clone, PartialEq)]
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
                .map(|cmd| (cmd.command_id.clone(), cmd.clone()))
                .collect(),
        )
    }

    pub fn denormalize(&self) -> impl Iterator<Item = &ScoredCommand> {
        self.values()
    }
}

#[cfg(test)]
mod tests {
    use crate::scored_commands::ScoredCommands;

    use super::ScoredCommand;

    #[test]
    fn correctly_normalizes_and_denormalizes() {
        let command = ScoredCommand::new(1, vec![1, 2], "sc_1".to_string());

        let scored_commands = ScoredCommands::normalize(&[command.clone()]);

        assert_eq!(scored_commands.get("sc_1").unwrap(), &command);

        let denormalized: Vec<ScoredCommand> = scored_commands.denormalize().cloned().collect();
        assert_eq!(denormalized, vec![command]);
    }
}
