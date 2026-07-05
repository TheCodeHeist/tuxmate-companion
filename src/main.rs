// pub mod tui;
pub mod utils;

use clap::{Parser, Subcommand};

use crate::utils::{distro::DistroInfo, package::SupportedTarget, registry::refresh_app_registry};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
  /// Display the current Linux distribution
  Distro,

  /// Refresh cached app registry data
  Refresh,

  /// Install a list of packages
  Install {
    /// List of packages to install
    #[arg(required = true, num_args = 1..)]
    packages: Vec<String>,
  },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PackageDef {
  package_id: String,
  target_id: Option<SupportedTarget>,
}

fn parse_package_arg(arg: String) -> PackageDef {
  // Format: <package_id> OR <target_id>/<package_id>
  if let Some((target_id, package_id)) = arg.split_once('/') {
    let target_id = match SupportedTarget::from_id(target_id) {
      Ok(target) => Some(target),
      Err(_) => None, // Invalid target_id, treat as None
    };
    PackageDef {
      package_id: package_id.to_string(),
      target_id,
    }
  } else {
    PackageDef {
      package_id: arg,
      target_id: None,
    }
  }
}

fn main() {
  let args = Args::parse();

  if let Some(Commands::Distro) = args.command {
    let mut distro = DistroInfo::new();
    distro.fetch().unwrap();
    println!("Current Linux distribution: {:?}", distro);
  } else if let Some(Commands::Refresh) = args.command {
    println!("Refreshing cached app registry data...");

    match refresh_app_registry() {
      Ok(_) => println!("App registry refreshed successfully."),
      Err(e) => eprintln!("Failed to refresh app registry: {}", e),
    }
  } else if let Some(Commands::Install { packages }) = args.command {
    let parsed_packages: Vec<PackageDef> = packages
      .iter()
      .map(|p| parse_package_arg(p.clone()))
      .collect();
    println!("Installing packages: {:?}", parsed_packages);
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
