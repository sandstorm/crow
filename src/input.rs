use crate::commands::default::InputWorkerEvent;
use crate::crow_db::CrowDB;
use crate::eject;
use crate::events::{CliEvent, InputEvent};
use crate::fuzzy::fuzzy_search_commands;
use crate::state::{MenuItem, State};
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{
    Event as CEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
};
use crossterm::style::Stylize;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use dialoguer::Editor;

use std::sync::mpsc::Sender;
use std::{
    io::{Error, Stdout},
    sync::mpsc::Receiver,
};

use tui::{backend::CrosstermBackend, Terminal};

/// Handles user input and returns either Ok(InputEvent::Quit) if the program should be
/// terminated after the current input or Ok(InputEvent::Continue) if the handling loop should
/// continue.
pub fn handle_input(
    main_tx: &Sender<InputWorkerEvent>,
    input_worker_rx: &Receiver<CliEvent<CEvent>>,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut State,
) -> Result<InputEvent, Error> {
    match input_worker_rx.recv().expect("Open input channel") {
        CliEvent::Input(event) => {
            // TODO feels like I am doing the work twice
            if let InputEvent::Quit = handle_general(event, terminal, state)? {
                return Ok(InputEvent::Quit);
            };

            match state.active_menu_item() {
                MenuItem::Find => {
                    if let InputEvent::Quit = handle_find(event, terminal, state)? {
                        return Ok(InputEvent::Quit);
                    };
                }
                &MenuItem::Edit => {
                    return handle_edit(main_tx, event, state);
                }
                MenuItem::Delete => {
                    handle_delete(event, state)?;
                }
            }
        }
        CliEvent::Tick => {}
    }

    Ok(InputEvent::Continue)
}

/// Handles input which is specific to [MenuItem::Delete]
fn handle_delete(event: CEvent, state: &mut State) -> Result<(), Error> {
    if let CEvent::Key(key_event) = event {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('y'),
                modifiers: KeyModifiers::NONE,
            } => {
                if let Some(c) = state.selected_command() {
                    // TODO revert control here:
                    // We should just make a call to our state which in turn should handle
                    // the database update.
                    match CrowDB::remove_command(c) {
                        Ok(commands) => {
                            state.set_command_ids(commands.iter().map(|c| c.id.clone()).collect());
                            state.set_commands(State::normalize_commands(commands));
                            state.set_fuzz_result(vec![]);
                            state.set_input("".to_string());
                            state.set_active_menu_item(MenuItem::Find);
                        }
                        Err(e) => eject(&format!("Could not remove command: {}", e)),
                    }
                }
            }

            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            }
            | KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::NONE,
            } => {
                state.set_active_menu_item(MenuItem::Find);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Handles input which is specific to [MenuItem::Edit]
fn handle_edit(
    main_tx: &Sender<InputWorkerEvent>,
    event: CEvent,
    state: &mut State,
) -> Result<InputEvent, Error> {
    if let Some(c) = state.selected_command() {
        if let CEvent::Key(key_event) = event {
            match key_event {
                KeyEvent {
                    code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    suspend_input_thread(main_tx);

                    let command = c.clone();
                    let edited_description = Editor::new()
                        .edit(&command.description)
                        .unwrap_or_else(|e| eject(&format!("Could not edit description. {}", e)));
                    state.update_command_description(
                        command.id,
                        &edited_description.unwrap_or_else(|| "".to_string()),
                    );
                    state.write_commands_to_db();

                    resume_input_thread(main_tx);
                }
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    suspend_input_thread(main_tx);

                    let command = c.clone();
                    let edited_command = Editor::new()
                        .edit(&command.command)
                        .unwrap_or_else(|e| eject(&format!("Could not edit command. {}", e)));
                    state.update_command_command(
                        command.id,
                        &edited_command.unwrap_or_else(|| "".to_string()),
                    );
                    state.write_commands_to_db();

                    resume_input_thread(main_tx);
                }
                _ => {}
            }
        }
    }

    Ok(InputEvent::Continue)
}

/// Handles input which is specific to [MenuItem::Find]
fn handle_find(
    event: CEvent,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut State,
) -> Result<InputEvent, Error> {
    let fuzz_result_count = state.fuzz_result_or_all().len();

    match event {
        CEvent::Key(key_event) => {
            match key_event {
                ///////////////////
                // List handling //
                ///////////////////
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => {
                    if let Some(selected) = state.command_list().selected() {
                        let selected_index = if selected >= fuzz_result_count - 1 {
                            0
                        } else {
                            selected + 1
                        };

                        state.select_command(selected_index);
                    }
                }

                KeyEvent {
                    code: KeyCode::Up, ..
                } => {
                    if let Some(selected) = state.command_list().selected() {
                        let selected_index = if selected > 0 {
                            selected - 1
                        } else {
                            fuzz_result_count - 1
                        };

                        state.select_command(selected_index);
                    }
                }

                ///////////////////////////
                // Input prompt handling //
                ///////////////////////////
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if let Some(c) = state.selected_command() {
                        let mut ctx = ClipboardContext::new().unwrap_or_else(|e| {
                            eject(&format!("Could not create clipboard context. {}", e))
                        });
                        ctx.set_contents(c.command.clone()).unwrap_or_else(|e| {
                            eject(&format!("Could not add command to clipboard. {}", e))
                        });

                        return quit(
                            terminal,
                            Some(&format!(
                                "\nCommand:\n  {}\ncopied to clipboard!\n",
                                c.command.clone().cyan()
                            )),
                        );
                    }
                }

                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                } => {
                    state.mut_input().push(c);
                    state.set_fuzz_result(fuzzy_search_commands(state.commands(), state.input()));

                    // We always want to select the first list element, when a new fuzzy search is being
                    // triggered
                    state.select_command(0);
                }

                KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                } => {
                    state.mut_input().pop();

                    state.set_fuzz_result(fuzzy_search_commands(state.commands(), state.input()));

                    // We always want to select the first list element, when a new fuzzy search is being
                    // triggered
                    state.select_command(0);
                }

                _ => {}
            }
        }

        CEvent::Mouse(mouse_event) => match mouse_event {
            MouseEvent {
                kind: MouseEventKind::ScrollUp,
                ..
            } => {
                let new_scroll_value = if state.detail_scroll_position() == 0 {
                    0
                } else {
                    state.detail_scroll_position() - 1
                };
                state.set_detail_scroll_position(new_scroll_value);
            }
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                ..
            } => {
                // TODO define upper boundary (probably by measuring text size)
                let new_scroll_value = state.detail_scroll_position() + 1;
                state.set_detail_scroll_position(new_scroll_value);
            }
            _ => {}
        },
        _ => {}
    }

    Ok(InputEvent::Continue)
}

/// Quit crow by gracefully terminating
fn quit(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    msg: Option<&str>,
) -> Result<InputEvent, Error> {
    disable_raw_mode()?;
    terminal.clear()?;
    terminal.show_cursor()?;

    println!("{}", msg.unwrap_or(""));

    Ok(InputEvent::Quit)
}

/// Handle input which should be available for all [MenuItem]
fn handle_general(
    event: CEvent,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut State,
) -> Result<InputEvent, Error> {
    if let CEvent::Key(key_event) = event {
        match key_event {
            ///////////////////
            // Menu handling //
            ///////////////////
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                return quit(terminal, None);
            }

            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                state.set_active_menu_item(MenuItem::Find);
            }

            KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                state.set_active_menu_item(MenuItem::Edit);
            }

            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
            } => {
                state.set_active_menu_item(MenuItem::Delete);
            }

            _ => {}
        }
    }

    Ok(InputEvent::Continue)
}

/// Suspend input thread so that events are not consumed by the crossterm backend and
/// can be consumed by other applications
fn suspend_input_thread(main_tx: &Sender<InputWorkerEvent>) {
    disable_raw_mode().unwrap_or_else(|e| eject(&format!("Could not disable raw mode! {}", e)));

    main_tx
        .send(InputWorkerEvent::Suspend)
        .unwrap_or_else(|e| eject(&format!("Could not send suspend signal. {}", e)));
}

/// Resume input thread so that input events are consumed by the crossterm backend and are no
/// longer available for other applications
fn resume_input_thread(main_tx: &Sender<InputWorkerEvent>) {
    enable_raw_mode().unwrap_or_else(|e| eject(&format!("Could not enable raw mode. {}", e)));
    main_tx
        .send(InputWorkerEvent::Resume)
        .unwrap_or_else(|e| eject(&format!("Could not send resume signal. {}", e)));
}
