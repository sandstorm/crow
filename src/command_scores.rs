//! Models which result from [fuzzy::fuzzy_search_commands] over [CrowCommands].
//! [ScoredCommands] contain an actual [CrowCommand], a fuzzy-score as well as the matching indices
//! of the search result.

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use indexmap::IndexMap;

use crate::crow_commands::Id;

/// A [ScoredCommand] contains a [CrowCommand] alongside scoring metadata and
/// a list of matching indices.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct CommandScore {
    score: i64,
    indices: Vec<usize>,
    command_id: Id,
}

impl CommandScore {
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
pub struct CommandScores(IndexMap<Id, CommandScore>);

impl Deref for CommandScores {
    type Target = IndexMap<Id, CommandScore>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CommandScores {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for CommandScores {
    fn default() -> Self {
        Self(IndexMap::default())
    }
}

impl CommandScores {
    pub fn normalize(scores: &[CommandScore]) -> Self {
        Self(
            scores
                .iter()
                .map(|score| (score.command_id.clone(), score.clone()))
                .collect(),
        )
    }

    pub fn denormalize(&self) -> impl Iterator<Item = &CommandScore> {
        self.values()
    }
}

#[cfg(test)]
mod tests {
    use crate::command_scores::CommandScores;

    use super::CommandScore;

    #[test]
    fn correctly_normalizes_and_denormalizes() {
        let score = CommandScore::new(1, vec![1, 2], "sc_1".to_string());

        let scores = CommandScores::normalize(&[score.clone()]);

        assert_eq!(scores.get("sc_1").unwrap(), &score);

        let denormalized: Vec<CommandScore> = scores.denormalize().cloned().collect();
        assert_eq!(denormalized, vec![score]);
    }
}
