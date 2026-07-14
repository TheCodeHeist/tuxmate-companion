// pub mod tui;
pub mod utils;

use clap::{Parser, Subcommand};
use colored::Colorize;

use crate::utils::{
  distro::DistroInfo,
  generate::CommandGenerator,
  package::SupportedTarget,
  registry::{load_app_registry_by_id, refresh_app_registry},
};

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

  /// Get information about a specific package
  Info {
    /// The package ID to get information about
    #[arg(required = true)]
    package_id: String,
  },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageDef {
  pub package_id: String,
  pub target_id: Option<SupportedTarget>,
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

  match args.command {
    Some(Commands::Distro) => {
      let mut distro = DistroInfo::new();
      distro.fetch().unwrap();
      println!(
        "{}\n",
        "Current Linux distribution".green().bold().underline()
      );
      println!("{} {}", "Name:".blue(), distro.name);
      println!("{} {}", "Version:".blue(), distro.version);
      println!("{} {}", "ID:".blue(), distro.id);
    }
    Some(Commands::Refresh) => {
      println!("Refreshing cached app registry data...");

      match refresh_app_registry() {
        Ok(_) => println!("{}", "App registry refreshed successfully.".green()),
        Err(e) => eprintln!(
          "{} {}",
          "Failed to refresh app registry:".red(),
          e.to_string().red()
        ),
      }
    }
    Some(Commands::Install { packages }) => {
      let parsed_packages: Vec<PackageDef> = packages
        .iter()
        .map(|p| parse_package_arg(p.clone()))
        .collect();

      let mut generator = CommandGenerator::init(parsed_packages.clone());

      match generator.generate_installation_command() {
        Ok(_) => {}
        Err(e) => eprintln!(
          "{} {}",
          "Failed to generate installation command:".red().underline(),
          e.to_string().red()
        ),
      }

      match generator.install() {
        Ok(_) => {}
        Err(e) => eprintln!(
          "{} {}",
          "Failed to install packages:".red().underline(),
          e.to_string().red()
        ),
      }
    }
    Some(Commands::Info { package_id }) => match load_app_registry_by_id(package_id) {
      Ok(app) => {
        println!(
          "{}{}{}\n",
          "App information for '".green().bold().underline(),
          app.name.green().bold().underline(),
          "'".green().bold().underline()
        );

        println!("{} {}", "App ID:".blue(), app.id);
        println!("{} {}", "Name:".blue(), app.name);
        println!("{} {}", "Description:".blue(), app.description);
        println!("{} {}", "Category:".blue(), app.category.to_name());

        println!("\n{}", "Supported Targets:".blue().bold().underline());
        for (target, handle) in app.targets.iter() {
          println!("  - {}: \"{}\"", target.to_name().bright_yellow(), handle);
        }

        if let Some(reason) = app.unavailable_reason {
          println!("\n{} {} {}", "**".yellow(), reason.yellow(), "**".yellow());
        }
        if let Some(note) = app.note {
          println!("\n{} {}", "Note:".blue().bold(), note.blue());
        }
      }
      Err(e) => {
        eprintln!(
          "{} {}",
          "Error occurred while fetching app info:".red(),
          e.to_string().red()
        );

        println!(
          "\n{} {}",
          "Tip:".blue().bold(),
          "Try running 'tuxmate refresh' to update the app registry.".blue()
        );
      }
    },
    None => {
      println!("No command provided. Use --help for usage information.");
    }
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
