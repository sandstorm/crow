use std::cmp::Reverse;

use fuzzy_matcher::FuzzyMatcher;

use crate::{
    crow_commands::{CrowCommand, Id},
    scored_commands::{ScoredCommand, ScoredCommands},
};

/// The [FuzzResult] contains [CrowCommands] with scoring metadata
#[derive(Debug, Default, PartialEq)]
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

    /// Get a reference to the fuzz result's command ids.
    pub fn _command_ids(&self) -> &[String] {
        self.command_ids.as_ref()
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
            .map(|c| ScoredCommand::new(1, vec![], c.id))
            .collect();
    }

    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let mut commands: Vec<ScoredCommand> = commands
        .into_iter()
        .map(|c| match matcher.fuzzy_indices(&c.match_str(), pattern) {
            Some((score, indices)) => ScoredCommand::new(score, indices, c.id),
            None => ScoredCommand::new(0, vec![], c.id),
        })
        .filter(|c| c.score() > 50)
        .collect();

    commands.sort_by_key(|c| Reverse(c.score()));
    commands
}

#[cfg(test)]
mod tests {
    use crate::{crow_commands::CrowCommand, scored_commands::ScoredCommand};

    use super::fuzzy_search_commands;

    #[test]
    fn dont_error_on_empty_command_list() {
        let result = fuzzy_search_commands(vec![], "test");
        let expected: Vec<ScoredCommand> = vec![];
        assert_eq!(expected, result);
    }

    #[test]
    fn return_full_list_for_empty_pattern() {
        let command = CrowCommand {
            id: "test1".to_string(),
            command: "echo 'hi'".to_string(),
            description: "test command".to_string(),
        };

        let result = fuzzy_search_commands(vec![command.clone()], "");

        let scored_command = ScoredCommand::new(1, vec![], command.id);
        let expected: Vec<ScoredCommand> = vec![scored_command];
        assert_eq!(expected, result);
    }

    #[test]
    fn return_matches_by_score() {
        let command1 = CrowCommand {
            id: "test1".to_string(),
            command: "echo 'hi'".to_string(),
            description: "test command".to_string(),
        };

        let command2 = CrowCommand {
            id: "test2".to_string(),
            command: "e c something o".to_string(),
            description: "test command".to_string(),
        };

        let command3 = CrowCommand {
            id: "test3".to_string(),
            command: "find".to_string(),
            description: "test command".to_string(),
        };

        let result =
            fuzzy_search_commands(vec![command1.clone(), command2.clone(), command3], "echo");

        let scored_command1 = ScoredCommand::new(91, vec![0, 1, 2, 3], command1.id);
        let scored_command2 = ScoredCommand::new(75, vec![0, 2, 9, 14], command2.id);

        let expected: Vec<ScoredCommand> = vec![scored_command1, scored_command2];
        assert_eq!(expected, result);
    }
}
