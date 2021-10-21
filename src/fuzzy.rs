use fuzzy_matcher::FuzzyMatcher;

use crate::crow_db::CrowCommand;

struct ScoredCommand {
    score: i64,
    command: CrowCommand,
}

/// Given a list of [CrowCommand] this filters all commands by a given pattern.
/// Commands stay inside the list as long as they reach a certain score.
/// NOTE: the score is still being fine tuned - this is just a first draft
/// Results are also sorted according to their score
pub fn fuzzy_search_commands(commands: Vec<CrowCommand>, pattern: &str) -> Vec<CrowCommand> {
    if pattern.is_empty() {
        return commands.into_iter().collect();
    }

    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
    let mut commands: Vec<ScoredCommand> = commands
        .into_iter()
        .map(|c| match matcher.fuzzy_match(&c.match_str(), pattern) {
            Some(score) => ScoredCommand { score, command: c },
            None => ScoredCommand {
                score: 0,
                command: c,
            },
        })
        .collect();

    commands.sort_by(|a, b| b.score.cmp(&a.score));

    commands
        .into_iter()
        .filter(|c| c.score > 0)
        .map(|c| c.command)
        .collect()
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
