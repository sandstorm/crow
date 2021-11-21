use crate::{
    crow_commands::{Commands, CrowCommand, CrowCommands, Id},
    crow_db::{CrowDBConnection, FilePath},
    fuzzy::{fuzzy_search_commands, FuzzResult},
    scored_commands::{ScoredCommand, ScoredCommands},
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
    selected_command: Option<Id>,

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
    pub fn new(db_file_path: Option<FilePath>) -> State {
        let mut state: State = Default::default();

        if let Some(path) = db_file_path {
            state.set_db_file_path(path);
        }

        // TODO expose error handling and use [eject] where possible
        let commands = CrowDBConnection::new(state.db_file_path.clone())
            .read()
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
    pub fn set_fuzz_result(&mut self, fuzz_result: Vec<ScoredCommand>) {
        self.fuzz_result = FuzzResult::new(
            ScoredCommands::normalize(&fuzz_result),
            fuzz_result.iter().map(|c| c.command().id.clone()).collect(),
        );
    }

    /// Get a reference to the state's fuzz result.
    pub fn fuzz_result_or_all(&mut self) -> Vec<ScoredCommand> {
        if !self.fuzz_result().commands().is_empty() || !self.input.is_empty() {
            self.fuzz_result()
                .commands()
                .denormalize()
                .cloned()
                .collect()
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
    pub fn set_selected_command(&mut self, id: Option<Id>) {
        self.selected_command = id;
    }

    /// Get a reference to the state's selected crow command.
    pub fn selected_crow_command(&self) -> Option<&CrowCommand> {
        match &self.selected_command {
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
        let selected_command = self
            .fuzz_result_or_all()
            .get(index)
            .map(|c| c.command().id.clone());

        self.set_selected_command(selected_command);
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
}
