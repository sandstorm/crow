use crate::{
    command_scores::{CommandScore, CommandScores},
    crow_commands::{Commands, CrowCommand, CrowCommands, Id},
    crow_db::{CrowDBConnection, FilePath},
    fuzzy::{fuzzy_search_commands, FuzzResult},
};
use std::fmt::Debug;

use tui::widgets::ListState;

#[derive(Debug, Default)]
pub struct State {
    db_file_path: FilePath,

    /// User input which is used for fuzzy searching
    input: String,

    /// List state which is used for [crate::rendering::command_list]
    command_list_state: ListState,

    crow_commands: CrowCommands,

    /// List of filtered commands
    fuzz_result: FuzzResult,

    /// The currently selected command
    selected_command_id: Option<Id>,

    /// The currently selected menu item which determines in what mode
    /// crow is in
    active_menu_item: MenuItem,

    /// The vertical scroll position of the detail view for commands
    detail_scroll_position: u16,
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Find,
    Edit,
    Delete,
    // NOTE: Quit is only a shortcut not an actual menu item
}

/// TODO we need to find a better way to couple these with [crate::rendering::keybindings]
impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Find => 0,
            MenuItem::Edit => 1,
            MenuItem::Delete => 2,
        }
    }
}

impl Default for MenuItem {
    fn default() -> MenuItem {
        MenuItem::Find
    }
}

impl State {
    /// Initializes the default state by filling most of the state with default
    /// values, but also reading and normalizing all commands from the crow_db file.
    pub fn new(db_file_path: Option<FilePath>) -> Self {
        let mut state: State = Self::default();

        if let Some(path) = db_file_path {
            state.set_db_file_path(path);
        }

        // TODO expose error handling and use [eject] where possible
        let commands = CrowDBConnection::new(state.db_file_path.clone())
            .commands()
            .to_vec();

        state
            .crow_commands
            .set_command_ids(commands.iter().map(|c| c.id.clone()).collect());

        state
            .crow_commands_mut()
            .set_commands(Commands::normalize(&commands));

        state.select_command(0);

        state
    }

    /// Writes the current command state to the crow_db file
    pub fn write_commands_to_db(&self) {
        CrowDBConnection::new(self.db_file_path.clone())
            .set_commands(
                self.crow_commands()
                    .commands()
                    .denormalize()
                    .cloned()
                    .collect(),
            )
            .write();
    }

    /// Gets the current fuzzy_search user input value
    pub fn input(&self) -> &String {
        &self.input
    }

    /// Gets the currently active [MenuItem]
    pub fn active_menu_item(&self) -> &MenuItem {
        &self.active_menu_item
    }

    /// Returns mutable reference to the user fuzzy_search input field
    pub fn mut_input(&mut self) -> &mut String {
        &mut self.input
    }

    /// Returns the command list state used for [crate::rendering::command_list]
    pub fn command_list_state(&self) -> &ListState {
        &self.command_list_state
    }

    /// Returns the mutable command list state used for [crate::rendering::command_list]
    pub fn mut_command_list(&mut self) -> &mut ListState {
        &mut self.command_list_state
    }

    /// Sets the active menu item to the specified [MenuItem]
    pub fn set_active_menu_item(&mut self, item: MenuItem) {
        self.active_menu_item = item;
    }

    /// Set the state's fuzz result.
    pub fn set_fuzz_result(&mut self, command_scores: Vec<CommandScore>) {
        self.fuzz_result = FuzzResult::new(
            CommandScores::normalize(&command_scores),
            command_scores
                .iter()
                .map(|c| c.command_id().clone())
                .collect(),
        );
    }

    /// Get a reference to the state's fuzz result.
    pub fn fuzz_result_or_all(&mut self) -> Vec<CommandScore> {
        if !self.fuzz_result().scores().is_empty() || !self.input.is_empty() {
            self.fuzz_result().scores().denormalize().cloned().collect()
        } else {
            let fuzz_result = fuzzy_search_commands(
                self.crow_commands()
                    .commands()
                    .denormalize()
                    .cloned()
                    .collect(),
                "",
            );
            self.set_fuzz_result(fuzz_result.clone());
            fuzz_result
        }
    }

    /// Set the state's selected command.
    pub fn set_selected_command_id(&mut self, id: Option<Id>) {
        self.selected_command_id = id;
    }

    /// Get a reference to the state's selected crow command.
    pub fn selected_crow_command(&self) -> Option<&CrowCommand> {
        match &self.selected_command_id {
            Some(id) => self.crow_commands.commands().get(id),
            None => None,
        }
    }

    /// Selects the command at a certain index inside the command_list_state and
    /// also retrieves the commands id from the fuzzy search result.
    pub fn select_command(&mut self, index: usize) {
        self.command_list_state.select(Some(index));

        // WHY:
        // When we fuzzy search the rendered list might shrink.
        // Therefore we retrieve our command by the index of the comman_list_state not from the
        // full list, but from the fuzzyed one. This works, because the command_list_state is rendered inside a stateful_widget which
        // also receives the same list of commands.
        let selected_command_id = self
            .fuzz_result_or_all()
            .get(index)
            .map(|c| c.command_id().clone());

        self.set_selected_command_id(selected_command_id);
    }

    /// Set the state's input.
    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    /// Set the state's detail scroll position.
    pub fn set_detail_scroll_position(&mut self, detail_scroll_position: u16) {
        self.detail_scroll_position = detail_scroll_position;
    }

    /// Get a reference to the state's detail scroll position.
    pub fn detail_scroll_position(&self) -> u16 {
        self.detail_scroll_position
    }

    /// Checks if there are any commands at all inside the state
    pub fn has_crow_commands(&self) -> bool {
        !self.crow_commands.commands().is_empty()
    }

    /// Get a reference to the state's fuzz result.
    pub fn fuzz_result(&self) -> &FuzzResult {
        &self.fuzz_result
    }

    /// Get a reference to the state's crow commands.
    pub fn crow_commands(&self) -> &CrowCommands {
        &self.crow_commands
    }

    /// Get a mutable reference to the state's crow commands.
    pub fn crow_commands_mut(&mut self) -> &mut CrowCommands {
        &mut self.crow_commands
    }

    /// Get a reference to the state's db file path.
    pub fn db_file_path(&self) -> &FilePath {
        &self.db_file_path
    }

    /// Set the state's db file path.
    pub fn set_db_file_path(&mut self, db_file_path: FilePath) {
        self.db_file_path = db_file_path;
    }

    /// Get a reference to the state's selected command id.
    pub fn _selected_command_id(&self) -> Option<&String> {
        self.selected_command_id.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use nanoid::nanoid;

    use crate::{
        command_scores::{CommandScore, CommandScores},
        crow_commands::{Commands, CrowCommand, CrowCommands, Id},
        crow_db::FilePath,
    };

    use super::State;

    #[test]
    fn initializes_with_correct_data() {
        let file_path = FilePath::new(Some("./testdata"), Some("crow.json"));

        let state = State::new(Some(file_path));

        assert_eq!(state.input(), "");
        assert_eq!(&**state.db_file_path(), "./testdata/crow.json");

        assert_eq!(state.detail_scroll_position(), 0);
        assert_eq!(state.has_crow_commands(), true);
        assert_eq!(state.command_list_state().selected().unwrap(), 0);
    }

    #[test]
    fn writes_updated_state_to_db() {
        let file_path = FilePath::new(Some("./testdata"), Some("crow_tmp.json"));

        let mut state = State::new(Some(file_path.clone()));

        let crow_command = CrowCommand {
            id: "test_command_1".to_string(),
            command: "echo 'hi from db'".to_string(),
            description: "This is a test command".to_string(),
        };
        let commands = [crow_command];
        let command_ids: Vec<Id> = vec!["test_command_1".to_string()];
        let crow_commands = CrowCommands::_new(Commands::normalize(&commands), command_ids);
        *state.crow_commands_mut() = crow_commands.clone();

        // Assert that current state holds correct commands
        assert_eq!(state.crow_commands(), &crow_commands);

        state.write_commands_to_db();

        // Assert that new state which also accesses the file holds the correct
        // commands
        let new_state = State::new(Some(file_path));

        assert_eq!(new_state.crow_commands(), &crow_commands);

        std::fs::remove_file("./testdata/crow_tmp.json").unwrap();
    }

    #[test]
    #[ignore = "FIXME arbitrary order of commands because of denormalization"]
    fn correctly_selects_command() {
        let file_path = FilePath::new(Some("./testdata"), Some("crow.json"));

        let mut state = State::new(Some(file_path));

        assert_eq!(state.command_list_state().selected(), Some(0));
        assert_eq!(
            state._selected_command_id(),
            Some(&"test_command_1".to_string())
        );

        state.select_command(1);

        assert_eq!(state.command_list_state().selected(), Some(1));
        assert_eq!(
            state._selected_command_id(),
            Some(&"test_command_2".to_string())
        );
    }

    #[test]
    fn correctly_sets_crow_commands() {
        let file_path = FilePath::new(Some("./testdata"), Some("crow.json"));

        let state = State::new(Some(file_path));

        let crow_command_1 = CrowCommand {
            id: "test_command_1".to_string(),
            command: "echo 'hi from db'".to_string(),
            description: "This is a test command".to_string(),
        };
        let crow_command_2 = CrowCommand {
            id: "test_command_2".to_string(),
            command: "".to_string(),
            description: "".to_string(),
        };
        let crow_commands = [crow_command_1, crow_command_2];
        let crow_command_ids: Vec<Id> =
            vec!["test_command_1".to_string(), "test_command_2".to_string()];
        let expected = CrowCommands::_new(Commands::normalize(&crow_commands), crow_command_ids);

        assert_eq!(state.crow_commands(), &expected);
    }

    #[test]
    fn returns_denormalized_fuzz_result_if_exists() {
        let file_path = FilePath::new(Some("./testdata"), Some("crow.json"));

        let state = State::new(Some(file_path));

        let crow_command_1 = CrowCommand {
            id: "test_command_1".to_string(),
            command: "echo 'hi from db'".to_string(),
            description: "This is a test command".to_string(),
        };
        let crow_command_2 = CrowCommand {
            id: "test_command_2".to_string(),
            command: "".to_string(),
            description: "".to_string(),
        };

        let command_scores = CommandScores::normalize(&[
            CommandScore::new(1, vec![], crow_command_1.id),
            CommandScore::new(1, vec![], crow_command_2.id),
        ]);

        assert_eq!(state.fuzz_result().scores(), &command_scores);
        assert!(state
            .fuzz_result()
            ._command_ids()
            .contains(&"test_command_1".to_string()));
        assert!(state
            .fuzz_result()
            ._command_ids()
            .contains(&"test_command_2".to_string()));
    }

    #[test]
    fn updates_fuzz_result_and_returns_it_if_not_exists() {
        let fn_path = &format!("./testdata/tmp/{}", nanoid!());
        let file_path = FilePath::new(Some(fn_path), Some("crow.json"));

        let state = State::new(Some(file_path));

        let command_scores = CommandScores::normalize(&[]);

        assert_eq!(state.fuzz_result().scores(), &command_scores);

        std::fs::remove_dir_all(Path::new(fn_path)).unwrap();
    }
}
