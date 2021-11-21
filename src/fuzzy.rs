use std::cmp::Reverse;

use fuzzy_matcher::FuzzyMatcher;

use crate::{
    crow_commands::{CrowCommand, Id},
    scored_commands::{ScoredCommand, ScoredCommands},
};

/// The [FuzzResult] contains [CrowCommands] with scoring metadata
#[derive(Debug, Default)]
pub struct FuzzResult {
    commands: ScoredCommands,
    command_ids: Vec<Id>,
}

impl FuzzResult {
    pub fn new(commands: ScoredCommands, command_ids: Vec<Id>) -> Self {
        Self {
            commands,
            command_ids,
        }
    }

    /// Get a reference to the fuzz result's commands.
    pub fn commands(&self) -> &ScoredCommands {
        &self.commands
    }
}

/// Given a list of [CrowCommand] this filters all commands by a given pattern.
/// Commands stay inside the list as long as they reach a certain score.
/// NOTE: the score is still being fine tuned - this is just a first draft
/// Results are also sorted according to their score
pub fn fuzzy_search_commands(commands: Vec<CrowCommand>, pattern: &str) -> Vec<ScoredCommand> {
    if pattern.is_empty() {
        return commands
            .into_iter()
            .map(|c| ScoredCommand::new(1, vec![], c))
            .collect();
    }

    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let mut commands: Vec<ScoredCommand> = commands
        .into_iter()
        .map(|c| match matcher.fuzzy_indices(&c.match_str(), pattern) {
            Some((score, indices)) => ScoredCommand::new(score, indices, c),
            None => ScoredCommand::new(0, vec![], c),
        })
        .filter(|c| c.score() > 50)
        .collect();

    commands.sort_by_key(|c| Reverse(c.score()));
    commands
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "not yet implemented"]
    fn dont_error_on_empty_command_list() {}

    #[test]
    #[ignore = "not yet implemented"]
    fn return_full_list_for_empty_pattern() {}

    #[test]
    #[ignore = "not yet implemented"]
    fn return_matches_by_score() {}
}
