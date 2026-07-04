// pub mod tui;
pub mod utils;

use clap::Parser;

use crate::utils::distro::DistroInfo;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
  // Shows current distro
  #[arg(short, long)]
  distro: bool,
}

fn main() {
  let args = Args::parse();

  if args.distro {
    let mut distro = DistroInfo::new();
    distro.fetch().unwrap();
    println!("Current Linux distribution: {:?}", distro);
  } else {
    println!("No arguments provided. Use --help for usage information.");
  }
}

// === NONE OF OUR BUSINESS FOR NOW, BUT KEEPING IT HERE FOR FUTURE REFERENCE ===

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
//   color_eyre::install()?;

//   let (ui_tx, ui_rx) = mpsc::channel();
//   let server_handle = tokio::spawn(start_socket_server(ui_tx));

//   println!("Welcome to TuxMate Companion Console!");
//   println!("Waiting for a connection from the TuxMate Companion client...");

//   // Loop until the server accepts a connection and the UI is ready to proceed
//   loop {
//     if let Ok(event) = ui_rx.try_recv() {
//       match event {
//         UiEvent::Connected => {
//           break; // Exit the loop when connected
//         }
//         _ => {}
//       }
//     }
//     tokio::time::sleep(std::time::Duration::from_millis(100)).await;
//   }

//   enable_raw_mode()?;
//   let mut stdout = io::stdout();
//   execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

//   let backend = CrosstermBackend::new(stdout);
//   let mut terminal = Terminal::new(backend)?;

//   let mut app = TuiApp::new();
//   app.table_state.select_first();
//   app.table_state.select_first_column();
//   let res = run_app(&mut terminal, &mut app, &ui_rx);

//   // Restore terminal
//   disable_raw_mode()?;
//   execute!(
//     terminal.backend_mut(),
//     LeaveAlternateScreen,
//     DisableMouseCapture
//   )?;
//   terminal.show_cursor()?;

//   println!("Goodbye! Thank you for using TuxMate Companion.");

//   server_handle.abort();
//   Ok(())
// }
