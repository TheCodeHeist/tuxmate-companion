use std::collections::HashMap;

use crate::{tui::tuiapp::TuiApp, utils::apps::SupportedTarget};

pub fn generate_installation_command(app: &mut TuiApp) -> Result<(), Box<dyn std::error::Error>> {
  let app_details = app.app_list.clone();

  if app_details.is_empty() {
    app.generated_command = None;
    Ok(())
  } else {
    let mut sorted_list: HashMap<SupportedTarget, Vec<String>> = HashMap::new();

    for app_detail in app_details {
      sorted_list
        .entry(app_detail.target)
        .or_insert_with(Vec::new)
        .push(app_detail.target_handle);
    }

    let mut command = String::from("");

    for (target, handles) in sorted_list {
      match target {
        SupportedTarget::Debian | SupportedTarget::Ubuntu => {
          command.push_str(&format!("sudo apt install -y {} && ", handles.join(" ")));
        }
        SupportedTarget::Arch => {
          command.push_str(&format!(
            "sudo pacman -S --needed --noconfirm {} && ",
            handles.join(" ")
          ));
        }
        SupportedTarget::Fedora => {
          command.push_str(&format!("sudo dnf install -y {} && ", handles.join(" ")));
        }
        SupportedTarget::OpenSUSE => {
          command.push_str(&format!("sudo zypper install -y {} && ", handles.join(" ")));
        }
        SupportedTarget::Nix => {
          let nixpkgs = handles
            .iter()
            .map(|handle| format!("nixpkgs.{}", handle))
            .collect::<Vec<String>>();
          command.push_str(&format!("nix-env -iA {} && ", nixpkgs.join(" ")));
        }
        SupportedTarget::Flatpak => {
          command.push_str(&format!(
            "flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && flatpak install flathub -y {} && ",
            handles.join(" ")
          ));
        }
        SupportedTarget::Snap => {
          // Keep separate commands to enable classic confinement for snaps that require it
          for handle in handles {
            command.push_str(&format!("sudo snap install {} && ", handle));
          }
        }
        SupportedTarget::Homebrew => {
          // Support for Formulae and Casks (casks have --cask in their handle)
          let (formulae, casks): (Vec<String>, Vec<String>) = handles
            .into_iter()
            .partition(|handle| !handle.starts_with("--cask"));

          let formulae_commands = Vec::from_iter(
            formulae
              .iter()
              .map(|handle| format!("brew install {}", handle)),
          );

          let cask_commands = Vec::from_iter(
            casks
              .iter()
              .map(|handle| format!("brew install --cask {}", handle.replace("--cask ", ""))),
          );

          command.push_str(&formulae_commands.join(" && "));
          command.push_str(&cask_commands.join(" && "));

          // command.push_str(&format!("brew install {} && ", handles.join(" ")));
        }
        SupportedTarget::Npm => {
          command.push_str(&format!("npm install -g {} && ", handles.join(" ")));
        }
        SupportedTarget::Script => {
          command.push_str(&format!("{} && ", handles.join(" && ")));
        }
      }
    }

    // Remove the trailing " && " from the command
    command.truncate(command.len().saturating_sub(4));

    app.generated_command = Some(command);
    Ok(())
  }
}
