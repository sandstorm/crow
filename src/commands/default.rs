use crate::events::{CliEvent, InputEvent};
use crate::state::{MenuItem, State};
use crate::{eject, input};
use crossterm::event::EnableMouseCapture;
use crossterm::execute;

use std::sync::mpsc::TryRecvError;
use std::{
    io::{self, Error, Stdout},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use unicode_width::UnicodeWidthStr;

use crossterm::{
    event::{self, Event as CEvent},
    terminal::enable_raw_mode,
};
use tui::{backend::CrosstermBackend, Terminal};

use crate::rendering::{self, empty_command_list};

pub enum InputWorkerEvent {
    Suspend,
    Resume,
}

/// Polls for input [crossterm::event:Event]s and sends them down the main channel.
/// The polling can be suspended/resumed.
fn poll_input_thread(
    input_worker_tx: Sender<CliEvent<CEvent>>,
    main_rx: Receiver<InputWorkerEvent>,
) {
    let tick_rate = Duration::from_millis(200);
    let mut suspended = false;

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            match main_rx.try_recv() {
                Ok(InputWorkerEvent::Suspend) => suspended = true,
                Ok(InputWorkerEvent::Resume) => suspended = false,
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => eject("Input channel disconnected"),
            }

            if suspended {
                continue;
            }

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                let crossterm_event = event::read().expect("can read events");
                input_worker_tx
                    .send(CliEvent::Input(crossterm_event))
                    .expect("can send events");
            }

            if last_tick.elapsed() >= tick_rate && input_worker_tx.send(CliEvent::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });
}

/// Renders the application to the terminal
fn render(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut State,
) -> Result<(), Error> {
    terminal.draw(|frame| {
        let rect = frame.size();
        let layout = rendering::layout(rect);

        frame.render_widget(rendering::keybindings(state.active_menu_item()), layout[0]);

        let commands = state.fuzz_result_or_all();

        let inner_split_layout = rendering::inner_split_layout(layout[1]);

        if state.has_crow_commands() {
            frame.render_stateful_widget(
                rendering::command_list(commands),
                inner_split_layout[0],
                state.mut_command_list(),
            );
        } else {
            frame.render_widget(empty_command_list(), inner_split_layout[0]);
        }

        if let Some(c) = state.selected_command() {
            frame.render_widget(
                rendering::command_detail(c, state.detail_scroll_position()),
                inner_split_layout[1],
            );
        };

        frame.render_widget(rendering::input(state.input()), layout[2]);

        frame.set_cursor(
            layout[2].x + UnicodeWidthStr::width(state.input().as_str()) as u16 + 3,
            layout[2].y + 1,
        );

        match state.active_menu_item() {
            MenuItem::Edit => {
                if state.selected_command().is_some() {
                    rendering::popup(frame, rendering::edit_command());
                };
            }

            MenuItem::Delete => {
                if let Some(c) = state.selected_command() {
                    rendering::popup(frame, rendering::delete_command(c));
                };
            }

            _ => {}
        }
    })?;

    Ok(())
}

/// Main thread.
/// Renders the application to the terminal and reacts to input events received by
/// the input polling worker thread.
fn main_loop(
    main_tx: Sender<InputWorkerEvent>,
    input_worker_rx: Receiver<CliEvent<CEvent>>,
) -> Result<(), Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut state = State::new();

    loop {
        render(&mut terminal, &mut state).expect("Can render");

        if let Ok(InputEvent::Quit) =
            input::handle_input(&main_tx, &input_worker_rx, &mut terminal, &mut state)
        {
            break;
        };
    }

    Ok(())
}

/// Default command when running 'crow' without arguments
pub fn run() -> Result<(), Error> {
    enable_raw_mode().expect("Can run in raw mode");
    execute!(io::stdout(), EnableMouseCapture)?;

    let (input_worker_tx, input_worker_rx) = mpsc::channel();
    let (main_tx, main_rx) = mpsc::channel();

    poll_input_thread(input_worker_tx, main_rx);
    main_loop(main_tx, input_worker_rx).expect("Main loop runs");

    Ok(())
}
