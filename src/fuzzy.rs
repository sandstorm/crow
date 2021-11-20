use fuzzy_matcher::FuzzyMatcher;
use tui::text::Text;

use crate::crow_db::CrowCommand;

#[derive(Debug)]
pub struct ScoredCommand {
    score: i64,
    indices: Vec<usize>,
    command: CrowCommand,
}

impl Clone for ScoredCommand {
    fn clone(&self) -> ScoredCommand {
        ScoredCommand {
            score: self.score,
            command: self.command.clone(),
            indices: self.indices.clone(),
        }
    }
}

impl From<&ScoredCommand> for Text<'_> {
    fn from(cmd: &ScoredCommand) -> Self {
        Text::from(cmd.command().command.clone())
    }
}

impl ScoredCommand {
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
}

/// Given a list of [CrowCommand] this filters all commands by a given pattern.
/// Commands stay inside the list as long as they reach a certain score.
/// NOTE: the score is still being fine tuned - this is just a first draft
/// Results are also sorted according to their score
pub fn fuzzy_search_commands(commands: Vec<CrowCommand>, pattern: &str) -> Vec<ScoredCommand> {
    if pattern.is_empty() {
        return commands
            .into_iter()
            .map(|c| ScoredCommand {
                score: 1,
                indices: vec![],
                command: c,
            })
            .collect();
    }

    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let mut commands: Vec<ScoredCommand> = commands
        .into_iter()
        .map(|c| match matcher.fuzzy_indices(&c.match_str(), pattern) {
            Some((score, indices)) => ScoredCommand {
                score,
                indices,
                command: c,
            },
            None => ScoredCommand {
                score: 0,
                indices: vec![],
                command: c,
            },
        })
        .collect();

    commands.sort_by(|a, b| b.score.cmp(&a.score));
    commands.into_iter().filter(|c| c.score > 0).collect()
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
