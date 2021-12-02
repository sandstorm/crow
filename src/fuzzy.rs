use std::cmp::Reverse;

use fuzzy_matcher::FuzzyMatcher;

use crate::{
    command_scores::{CommandScore, CommandScores},
    crow_commands::{CrowCommand, Id},
};

/// The [FuzzResult] contains [CrowCommands] with scoring metadata
#[derive(Debug, Default, PartialEq)]
pub struct FuzzResult {
    scores: CommandScores,
    command_ids: Vec<Id>,
}

impl FuzzResult {
    pub fn new(scores: CommandScores, command_ids: Vec<Id>) -> Self {
        Self {
            scores,
            command_ids,
        }
    }

    /// Get a reference to the fuzz result's commands.
    pub fn scores(&self) -> &CommandScores {
        &self.scores
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
pub fn fuzzy_search_commands(commands: Vec<CrowCommand>, pattern: &str) -> Vec<CommandScore> {
    if pattern.is_empty() {
        return commands
            .into_iter()
            .map(|c| CommandScore::new(1, vec![], c.id))
            .collect();
    }

    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let mut scores: Vec<CommandScore> = commands
        .into_iter()
        .map(|c| match matcher.fuzzy_indices(&c.match_str(), pattern) {
            Some((score, indices)) => CommandScore::new(score, indices, c.id),
            None => CommandScore::new(0, vec![], c.id),
        })
        .filter(|c| c.score() > 50)
        .collect();

    scores.sort_by_key(|c| Reverse(c.score()));
    scores
}

#[cfg(test)]
mod tests {
    use crate::{command_scores::CommandScore, crow_commands::CrowCommand};

    use super::fuzzy_search_commands;

    #[test]
    fn dont_error_on_empty_command_list() {
        let result = fuzzy_search_commands(vec![], "test");
        let expected: Vec<CommandScore> = vec![];
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

        let score = CommandScore::new(1, vec![], command.id);
        let expected: Vec<CommandScore> = vec![score];
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

        let score_1 = CommandScore::new(91, vec![0, 1, 2, 3], command1.id);
        let score_2 = CommandScore::new(75, vec![0, 2, 9, 14], command2.id);

        let expected: Vec<CommandScore> = vec![score_1, score_2];
        assert_eq!(expected, result);
    }
}
