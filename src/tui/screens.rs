use ratatui::{
  Frame,
  backend::Backend,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Style, Stylize},
  text::{Line, Span, Text},
  widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
};

use crate::{
  tui::tuiapp::{CurrentScreen, TuiApp, UiState},
  utils::generate::generate_installation_command,
};

pub fn ui(frame: &mut ratatui::Frame, app: &mut TuiApp) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
      [
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(10),
        Constraint::Length(3),
      ]
      .as_ref(),
    )
    .split(frame.area());

  // === Header Chunk ===
  let title_block = Block::default()
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded)
    .border_style(Style::default().fg(Color::Cyan))
    .style(Style::default().bg(Color::Cyan).bold());

  let title = Paragraph::new(Text::styled(
    "TuxMate Companion Console",
    Style::default().fg(Color::Black).bold(),
  ))
  .alignment(Alignment::Center)
  .block(title_block);

  frame.render_widget(title, chunks[0]);
  // === Header Chunk End ===

  // === Main Content Chunk ===
  match app.current_screen {
    CurrentScreen::AppConsole => match app.ui_state {
      UiState::WaitingForApps => {
        apps_awaiting(frame, chunks[1]);
      }
      UiState::AppList => {
        app_list_area(frame, chunks[1], app);
      }
    },
    CurrentScreen::Exiting => {
      exiting_block(frame, chunks[1]);
    }
    _ => {}
  }
  // === Main Content Chunk End ===

  // === Output Chunk ===
  let output_block = Block::default()
    .borders(Borders::ALL)
    .border_type(BorderType::Rounded)
    .border_style(Style::default().fg(Color::Green))
    .style(Style::default());

  let output = Paragraph::new(Text::from_iter([
    Span::styled(
      "Generated Installation Command:",
      Style::default().fg(Color::Green).bold(),
    ),
    Span::raw("\n"),
    Span::styled(
      app
        .generated_command
        .as_deref()
        .unwrap_or("No apps available to generate command."),
      Style::default().fg(Color::White),
    ),
  ]))
  .wrap(Wrap { trim: true })
  .alignment(Alignment::Left)
  .block(output_block);

  frame.render_widget(output, chunks[2]);
  // === Output Chunk End ===

  // === Footer Chunk ===
  let footer_block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::White))
    .style(Style::default());

  let footer = Paragraph::new(Text::styled(
    "Press 'q' to quit, 'r' to refresh, 'h' for help.",
    Style::default().fg(Color::White),
  ))
  .alignment(Alignment::Center)
  .block(footer_block);

  frame.render_widget(footer, chunks[3]);
  // === Footer Chunk End ===
}

fn app_list_area(frame: &mut Frame, area: Rect, app: &mut TuiApp) {
  // Create a new Rect for the table area, leaving some padding around the edges
  let table_area = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(100)].as_ref())
    .margin(1)
    .split(area)[0];

  let block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Blue));

  frame.render_widget(block, area);
  render_app_list_table(frame, table_area, app);
}

fn render_app_list_table(frame: &mut Frame, area: Rect, app: &mut TuiApp) {
  let header = Row::new(vec![
    Cell::from("App ID"),
    Cell::from("Name"),
    Cell::from("Category"),
    Cell::from("Target"),
    Cell::from("Handle"),
  ])
  .style(Style::new().bold())
  .bottom_margin(1);

  let rows = app.app_list.iter().map(|app| {
    Row::new(vec![
      Cell::from(app.app_id.clone()),
      Cell::from(app.app_name.clone()).style(Color::LightYellow),
      Cell::from(app.category.to_str().to_string()),
      Cell::from(app.target.to_name().to_string()).style(Color::LightBlue),
      Cell::from(app.target_handle.clone()),
    ])
  });

  let footer = Row::new(vec![
    Cell::from(Line::from_iter([
      Span::from("Total Apps: ").bold().fg(Color::Cyan),
      Span::from(app.app_list.len().to_string()).fg(Color::Cyan),
    ])),
    Cell::from(""),
    Cell::from(""),
    Cell::from(""),
    Cell::from(""),
  ]);

  let widths = [
    Constraint::Percentage(10),
    Constraint::Percentage(30),
    Constraint::Percentage(20),
    Constraint::Percentage(10),
    Constraint::Percentage(30),
  ];

  let table = Table::new(rows, widths)
    .header(header)
    .footer(footer.italic())
    .column_spacing(1)
    .style(Color::White)
    .row_highlight_style(Style::new().on_black().bold())
    .column_highlight_style(Color::Gray)
    .cell_highlight_style(Style::new().reversed().yellow())
    .highlight_symbol(">> ");

  frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn apps_awaiting(frame: &mut ratatui::Frame, area: Rect) {
  let block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Yellow));

  let text = Paragraph::new(Text::styled(
    "Waiting for apps from the client...",
    Style::default().fg(Color::Yellow),
  ))
  .alignment(Alignment::Center)
  .block(block);

  frame.render_widget(text, area);
}

fn exiting_block(frame: &mut ratatui::Frame, area: Rect) {
  let popup_area = popup_rect(50, 50, area);

  let main_block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::Red));

  let text_content_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
    .split(popup_area);

  let text = Paragraph::new(Text::styled(
    "Quit TuxMate Companion Console",
    Style::default().fg(Color::Red).bold(),
  ));

  let main_text = Paragraph::new(Text::styled(
    "Are you sure you want to quit? (y/n)",
    Style::default().fg(Color::Red),
  ));

  frame.render_widget(main_block, popup_area);
  frame.render_widget(text, text_content_layout[0]);
  frame.render_widget(main_text, text_content_layout[1]);
}

fn popup_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ])
    .margin(2)
    .split(r);

  let vertical_chunk = popup_layout[1];

  let horizontal_layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(
      [
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
      ]
      .as_ref(),
    )
    .split(vertical_chunk);

  horizontal_layout[1]
}
