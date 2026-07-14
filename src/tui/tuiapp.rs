use std::{
  io,
  sync::mpsc::{self, Receiver},
  time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
  DefaultTerminal, Frame, Terminal,
  backend::Backend,
  buffer::Buffer,
  layout::{Constraint, Rect},
  style::{Color, Modifier, Style, Stylize},
  symbols::border,
  text::{Line, Text},
  widgets::{Block, Cell, Paragraph, Row, Table, TableState, Widget, Wrap},
};

use crate::{
  tui::screens::ui,
  utils::{apps::AppDetail, generate::generate_installation_command},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentScreen {
  AppConsole,
  Installation,
  Exiting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiState {
  WaitingForApps,
  AppList,
}

#[derive(Debug, Clone)]
pub enum UiEvent {
  Connected,
  AppsReceived(Vec<AppDetail>),
}

#[derive(Debug)]
pub struct TuiApp {
  pub current_screen: CurrentScreen,
  pub ui_state: UiState,
  pub app_list: Vec<AppDetail>,
  pub table_state: TableState,
  pub generated_command: Option<String>,
}

impl TuiApp {
  pub fn new() -> Self {
    Self {
      current_screen: CurrentScreen::AppConsole,
      ui_state: UiState::WaitingForApps,
      app_list: Vec::new(),
      table_state: TableState::default(),
      generated_command: None,
    }
  }

  fn add_app(&mut self, app: AppDetail) {
    self.app_list.push(app);
  }

  fn remove_app(&mut self, app_id: &str) {
    self.app_list.retain(|app| app.app_id != app_id);
  }

  fn clear_apps(&mut self) {
    self.app_list.clear();
  }

  fn install_all_apps(&mut self) {
    // Placeholder for installation logic
    self.clear_apps();
  }
}

pub fn run_app<B: Backend>(
  terminal: &mut Terminal<B>,
  app: &mut TuiApp,
  ui_rx: &mpsc::Receiver<UiEvent>,
) -> io::Result<bool>
where
  io::Error: From<B::Error>,
{
  loop {
    terminal.draw(|f| ui(f, app))?;

    if let Ok(event) = ui_rx.try_recv() {
      match event {
        UiEvent::AppsReceived(apps) => {
          app.app_list = apps;
          generate_installation_command(app).ok();
          app.ui_state = UiState::AppList;
        }
        _ => {}
      }
    }

    if let Event::Key(key) = event::read()? {
      if key.kind == event::KeyEventKind::Release {
        // Skip key release events to avoid double handling
        continue;
      }

      match app.current_screen {
        CurrentScreen::AppConsole => match key.code {
          KeyCode::Char('q') => {
            app.current_screen = CurrentScreen::Exiting;
          }
          KeyCode::Left | KeyCode::Char('h') => {
            app.table_state.select_previous_column();
          }
          KeyCode::Right | KeyCode::Char('l') => {
            app.table_state.select_next_column();
          }
          KeyCode::Up | KeyCode::Char('k') => {
            app.table_state.select_previous();
          }
          KeyCode::Down | KeyCode::Char('j') => {
            app.table_state.select_next();
          }
          _ => {}
        },
        CurrentScreen::Exiting => {
          match key.code {
            KeyCode::Char('q') | KeyCode::Char('y') => {
              return Ok(false); // Exit the application
            }
            _ => {}
          }
        }
        _ => todo!("Implement other screens"),
      }
    }
  }
}

// impl Widget for &TuiApp {
//   fn render(self, area: Rect, buf: &mut Buffer) {
//     let title = Line::from(" TuxMate Companion ".bold().cyan());
//     let footer = Line::from(vec![
//       " Press ".into(),
//       "q".blue().bold(),
//       " to quit ".into(),
//     ]);
//     let block = Block::bordered()
//       .title(title)
//       .title_bottom(footer.centered())
//       .border_set(border::THICK);

//     let content = match self.status {
//       AppUiState::WaitingForConnection => Text::from(vec![
//         Line::from(
//           "  ________             ______  ___      _____     ______________________".cyan(),
//         ),
//         Line::from(
//           "  ___  __/___  _____  ____   |/  /_____ __  /_______  ____/__  /____  _/".cyan(),
//         ),
//         Line::from(
//           "  __  /  _  / / /_  |/_/_  /|_/ /_  __ `/  __/  _ \\  /    __  /  __  /  ".cyan(),
//         ),
//         Line::from(
//           "  _  /   / /_/ /__>  < _  /  / / / /_/ // /_ /  __/ /___  _  /____/ /   ".cyan(),
//         ),
//         Line::from(
//           "  /_/    \\__,_/ /_/|_| /_/  /_/  \\__,_/ \\__/ \\___/\\____/  /_____/___/   ".cyan(),
//         ),
//         Line::from(""),
//         Line::from("  TuxMate Companion".bold().yellow()),
//         Line::from(""),
//         Line::from("  Waiting for TuxMate socket.io connection...".green()),
//       ]),
//       AppUiState::WaitingForApps => Text::from(vec![
//         Line::from("  Socket.IO connected".green().bold()),
//         Line::from(""),
//         Line::from("  Waiting for the TuxMate website to send app details...".yellow()),
//       ]),
//       AppUiState::AppList => {
//         if self.app_list.is_empty() {
//           let content = Text::from(vec![Line::from("  No apps received yet.".yellow())]);
//           return Paragraph::new(content)
//             .block(block)
//             .wrap(Wrap { trim: true })
//             .render(area, buf);
//         }

//         let header = Row::new(vec!["App", "Category", "Target"]).style(
//           Style::default()
//             .fg(Color::Cyan)
//             .add_modifier(Modifier::BOLD),
//         );
//         let rows = self.app_list.iter().map(|app| {
//           Row::new(vec![
//             Cell::from(app.app_name.clone()),
//             Cell::from(app.category.to_str().to_string()),
//             Cell::from(app.target.to_name().to_string()),
//           ])
//         });

//         let table = Table::new(
//           rows,
//           [
//             Constraint::Length(24),
//             Constraint::Length(18),
//             Constraint::Length(16),
//           ],
//         )
//         .header(header)
//         .block(Block::default().title("Apps ready to install"))
//         .widths(&[
//           Constraint::Length(24),
//           Constraint::Length(18),
//           Constraint::Length(16),
//         ])
//         .column_spacing(1);

//         table.render(area, buf);
//         return;
//       }
//     };

//     Paragraph::new(content)
//       .block(block)
//       .wrap(Wrap { trim: true })
//       .render(area, buf);
//   }
// }

// #[cfg(test)]
// mod tests {
//   use super::*;

//   #[test]
//   fn updates_state_when_socket_connects_and_apps_arrive() {
//     let mut app = TuiApp::default();

//     app.handle_ui_event(UiEvent::Connected);
//     assert!(matches!(app.status, AppUiState::WaitingForApps));

//     app.handle_ui_event(UiEvent::AppsReceived(vec![AppDetail {
//       app_id: "demo".into(),
//       app_name: "Demo App".into(),
//       category: crate::utils::apps::Category::System,
//       target: crate::utils::apps::SupportedTarget::Ubuntu,
//       target_handle: "demo".into(),
//     }]));

//     assert!(matches!(app.status, AppUiState::AppList));
//     assert_eq!(app.app_list.len(), 1);
//   }
// }
