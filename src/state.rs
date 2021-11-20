use crate::{
    crow_db::{CrowDB, Id},
    fuzzy::{fuzzy_search_commands, ScoredCommand},
};
use std::{collections::HashMap, fmt::Debug};

use crate::crow_db::CrowCommand;
use tui::widgets::ListState;

type Commands = HashMap<Id, CrowCommand>;
type ScoredCommands = HashMap<Id, ScoredCommand>;

#[derive(Debug, Default)]
struct CrowCommands {
    /// Lookup hashmap of commands
    commands: Commands,

    /// List of all command ids
    command_ids: Vec<Id>,
}

#[derive(Debug, Default)]
struct FuzzResult {
    commands: ScoredCommands,
    command_ids: Vec<Id>,
}

impl FuzzResult {
    /// Get a reference to the fuzz result's commands.
    fn commands(&self) -> &ScoredCommands {
        &self.commands
    }

    /// Get a reference to the fuzz result's command ids.
    fn command_ids(&self) -> &[String] {
        self.command_ids.as_ref()
    }
}

#[derive(Debug, Default)]
pub struct State {
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
    pub fn new() -> State {
        let mut state: State = Default::default();

        // TODO expose error handling and use [eject] where possible
        let commands = CrowDB::read().commands().clone();
        state.set_crow_command_ids(commands.iter().map(|c| c.id.clone()).collect());

        state.set_crow_commands(State::normalize_crow_commands(commands));
        state.select_command(0);

        state
    }

    /// Normalizes a vec of commands into a lookup HashMap<Id, CrowCommand>
    pub fn normalize_crow_commands(commands: Vec<CrowCommand>) -> Commands {
        commands.into_iter().fold(HashMap::new(), |mut acc, c| {
            acc.insert(c.id.clone(), c);
            acc
        })
    }

    /// Denormalizes a lookup HashMap<Id, CrowCommand> into a vec of CrowCommands
    pub fn denormalize_crow_commands(commands: &Commands) -> Vec<CrowCommand> {
        commands.values().cloned().collect()
    }

    /// Gets a denormalized vec of CrowCommands
    pub fn crow_commands(&self) -> Vec<CrowCommand> {
        State::denormalize_crow_commands(&self.crow_commands.commands)
    }

    /// Writes the current command state to the crow_db file
    pub fn write_commands_to_db(&self) {
        let mut db = CrowDB::read();
        db.set_commands(self.crow_commands());
        db.write();
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

    /// Sets the commands lookup hash map
    pub fn set_crow_commands(&mut self, commands: Commands) {
        self.crow_commands = CrowCommands {
            commands,
            command_ids: self.crow_commands.command_ids.clone(),
        };
    }

    /// Updates the description of a command
    pub fn update_crow_command_description(&mut self, command_id: Id, description: &str) {
        if let Some(c) = self.crow_commands.commands.get_mut(&command_id) {
            *c = CrowCommand {
                description: description.to_string(),
                ..c.clone()
            }
        }
    }

    /// Updates the command of a command
    pub fn update_crow_command(&mut self, command_id: Id, command: &str) {
        if let Some(c) = self.crow_commands.commands.get_mut(&command_id) {
            *c = CrowCommand {
                command: command.to_string(),
                ..c.clone()
            }
        }
    }

    pub fn normalize_scored_commands(commands: Vec<ScoredCommand>) -> ScoredCommands {
        commands.into_iter().fold(HashMap::new(), |mut acc, c| {
            acc.insert(c.command().id.clone(), c);
            acc
        })
    }

    pub fn denormalize_scored_commands(commands: &ScoredCommands) -> Vec<ScoredCommand> {
        commands.values().cloned().collect()
    }

    pub fn scored_commands(&self) -> Vec<ScoredCommand> {
        State::denormalize_scored_commands(&self.fuzz_result.commands())
    }

    /// Set the state's fuzz result.
    pub fn set_fuzz_result(&mut self, fuzz_result: Vec<ScoredCommand>) {
        self.fuzz_result = FuzzResult {
            commands: State::normalize_scored_commands(fuzz_result.clone()),
            command_ids: fuzz_result.iter().map(|c| c.command().id.clone()).collect(),
        };
    }

    /// Get a reference to the state's fuzz result.
    pub fn fuzz_result_or_all(&mut self) -> Vec<ScoredCommand> {
        if !self.scored_commands().is_empty() || !self.input.is_empty() {
            self.scored_commands()
        } else {
            let fuzz_result = fuzzy_search_commands(self.crow_commands(), "");
            self.set_fuzz_result(fuzz_result.clone());
            fuzz_result
        }
    }

    /// Set the state's selected command.
    pub fn set_selected_command(&mut self, id: Option<Id>) {
        self.selected_command = id;
    }

    /// Get a reference to the state's selected command.
    pub fn selected_command(&self) -> Option<&CrowCommand> {
        match &self.selected_command {
            Some(id) => self.crow_commands.commands.get(id),
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

    /// Set the state's command ids.
    pub fn set_crow_command_ids(&mut self, command_ids: Vec<Id>) {
        self.crow_commands.command_ids = command_ids;
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
        !self.crow_commands.commands.is_empty()
    }
}
