use std::io::Stdout;

use tui::backend::CrosstermBackend;
use tui::text::Text;
use tui::widgets::{Clear, Widget, Wrap};
use tui::Frame;
use tui::{layout::Rect, text::Spans};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{BorderType, Paragraph},
};
use tui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};
use tui::{text::Span, widgets::Tabs};

use crate::crow_db::CrowCommand;
use crate::state::MenuItem;

/// Base layout of the program
pub fn layout(rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(rect)
}

/// A 40%/60% horizontal split layout
pub fn inner_split_layout(rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(rect)
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn popup(frame: &mut Frame<CrosstermBackend<Stdout>>, widget: impl Widget) {
    let popup_area = centered_rect(60, 40, frame.size());
    frame.render_widget(Clear, popup_area); //this clears out the background
    frame.render_widget(widget, popup_area);
}

/// Renders the deletion prompt for the currently selected command
pub fn delete_command(selected_command: &CrowCommand) -> Paragraph {
    Paragraph::new(Spans::from(vec![
        Span::styled("Do you really want to ", Style::default().fg(Color::White)),
        Span::styled("delete ", Style::default().fg(Color::Red)),
        Span::styled("command: ", Style::default().fg(Color::White)),
        Span::styled(&selected_command.command, Style::default().fg(Color::Cyan)),
        Span::styled("? (y/N)", Style::default().fg(Color::White)),
    ]))
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

/// Renders the edit prompt for the currently selected command
pub fn edit_command() -> Paragraph<'static> {
    Paragraph::new(Spans::from(vec![
        Span::styled(
            "C",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::styled("ommand", Style::default().fg(Color::White)),
        Span::styled(" / ", Style::default().fg(Color::White)),
        Span::styled(
            "D",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::styled("escription", Style::default().fg(Color::White)),
    ]))
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

/// Renders a list of keybindings to the top of the terminal output
pub fn keybindings(active_menu_item: &MenuItem) -> Tabs<'static> {
    // TODO find a way to better couple these with [MenutItem]
    // TODO add arrows for list navigation and <C-J>/<C-K> for scrolling
    let label_list = vec!["Find", "Edit", "Delete", "Quit"];
    let labels = label_list
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    Tabs::new(labels)
        .select(active_menu_item.clone().into())
        .block(
            Block::default()
                .title("Keys (press CTRL+<KEY> or ENTER to copy command and quit)")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::LightYellow))
        .highlight_style(Style::default().fg(Color::Green))
        .divider(Span::raw("|"))
}

/// Renders a list of commands with teh currently selected item being highlighted.
/// For selection to work this needs to be rendered inside a stateful_widget
/// NOTE: Selection input is handled inside [crate::input]
/// NOTE: The stateful_widget binding happens in [crate::commands::default::render]
pub fn command_list(crow_commands: Vec<CrowCommand>) -> List<'static> {
    let list_items: Vec<ListItem> = crow_commands.iter().map(ListItem::new).collect();

    List::new(list_items)
        .block(Block::default().title("Commands").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">> ")
}

/// Handles the display of the command details (command + description) for the currently
/// selected command.
pub fn command_detail<'a>(selected_command: &CrowCommand, scroll_position: u16) -> Paragraph<'a> {
    let mut detail = Text::styled(
        selected_command.command.clone(),
        Style::default().fg(Color::Cyan),
    );
    detail.extend(Text::styled(
        format!("\n\n{}", selected_command.description.clone()),
        Style::default().fg(Color::White),
    ));

    Paragraph::new(detail)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .scroll((scroll_position, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}

/// Renders the empty command list hint
pub fn empty_command_list() -> Paragraph<'static> {
    let mut text = Text::styled(
        "There are no saved commands!\nPlease quit and run one of the following crow commands first:\n\n",
        Style::default().fg(Color::White),
    );

    text.extend(Text::styled("crow add\n", Style::default().fg(Color::Cyan)));
    text.extend(Text::styled(
        "crow add:last\n",
        Style::default().fg(Color::Cyan),
    ));
    text.extend(Text::styled(
        "crow add:pick\n",
        Style::default().fg(Color::Cyan),
    ));
    text.extend(Text::styled(
        "\n\nSee <crow help> for more information.",
        Style::default().fg(Color::Yellow),
    ));

    Paragraph::new(text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::LightCyan))
                .border_type(BorderType::Plain),
        )
}

/// Renders the input prompt which is used for fuzzy searching.
/// The actual input handling is located in [crate::input].
pub fn input(input: &str) -> Paragraph {
    Paragraph::new(Spans::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::styled(input, Style::default().fg(Color::White)),
    ]))
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan))
            .border_type(BorderType::Plain),
    )
}
