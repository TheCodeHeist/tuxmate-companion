/// Generate a command to install packages using apt package manager (Debian/Ubuntu)
fn generate_apt_command(apt_packages: Vec<String>) -> String {
  format!(
    "sudo apt update && sudo apt install -y {}",
    apt_packages.join(" ")
  )
}

/// Generate a command to install packages using pacman package manager (Arch)
fn generate_pacman_command(pacman_packages: Vec<String>) -> String {
  format!(
    "sudo pacman -S --needed --noconfirm {}",
    pacman_packages.join(" ")
  )
}

/// Generate a command to install packages from AUR (Arch)
fn generate_aur_command(aur_packages: Vec<String>, aur_helper: String) -> String {
  format!(
    "{} -S --needed --noconfirm {}",
    aur_helper,
    aur_packages.join(" ")
  )
}

/// Generate commands to install a chosen AUR helper tool
fn generate_aur_helper_install_command(aur_helper: String) -> String {
  // Create a temporary directory for building the AUR helper
  let temp_dir = "/tmp/tuxmate-aur-build";

  format!(
    "mkdir -p {temp_dir} && cd {temp_dir} && git clone https://aur.archlinux.org/{aur_helper}.git && cd {aur_helper} && makepkg -si --noconfirm && cd .. && rm -rf {temp_dir}",
    temp_dir = temp_dir,
    aur_helper = aur_helper
  )
}

/// Generate a command to install packages using dnf package manager (Fedora)
fn generate_dnf_command(dnf_packages: Vec<String>) -> String {
  format!("sudo dnf install -y {}", dnf_packages.join(" "))
}

/// Generate a command to install packages using zypper package manager (OpenSUSE)
fn generate_zypper_command(zypper_packages: Vec<String>) -> String {
  format!("sudo zypper install -y {}", zypper_packages.join(" "))
}

/// Generate a command to install packages using nix package manager (NixOS)
fn generate_nix_command(nix_packages: Vec<String>) -> String {
  let nixpkgs = nix_packages
    .iter()
    .map(|handle| format!("nixpkgs.{}", handle))
    .collect::<Vec<String>>();
  format!("sudo nix-env -iA {}", nixpkgs.join(" "))
}

/// Generate configuration file content for nix package manager (NixOS)
fn generate_nix_config(nix_packages: Vec<String>) -> String {
  format!(
    "environment.systemPackages = with pkgs; [\n    {}\n];",
    nix_packages.join("\n    ")
  )
}

/// Generate a command to install packages using flatpak package manager
fn generate_flatpak_command(flatpak_packages: Vec<String>) -> String {
  format!(
    "flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && flatpak install flathub -y {}",
    flatpak_packages.join(" ")
  )
}

/// Generate a command to install packages using snap package manager
fn generate_snap_command(snap_packages: Vec<String>) -> String {
  // Keep separate commands to enable classic confinement for snaps that require it
  let mut command = String::new();
  for handle in snap_packages {
    command.push_str(&format!("sudo snap install {} && ", handle));
  }
  // Remove the trailing " && " from the command
  command.truncate(command.len().saturating_sub(4));
  command
}

/// Generate a command to install packages using Homebrew package manager
fn generate_homebrew_command(homebrew_packages: Vec<String>) -> String {
  // Support for Formulae and Casks (casks have --cask in their handle)
  let (formulae, casks): (Vec<String>, Vec<String>) = homebrew_packages
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

  let mut command = String::new();
  command.push_str(&formulae_commands.join(" && "));
  command.push_str(&cask_commands.join(" && "));

  command
}

/// Generate a command to install packages using npm package manager
fn generate_npm_command(npm_packages: Vec<String>) -> String {
  format!("npm install -g {}", npm_packages.join(" "))
}

/// Generate a command to run scripts
fn generate_script_command(script_packages: Vec<String>) -> String {
  script_packages.join(" && ")
}

// === NONE OF OUR BUSINESS FOR NOW, BUT KEEPING IT HERE FOR FUTURE REFERENCE ===

// pub fn generate_installation_command(app: &mut TuiApp) -> Result<(), Box<dyn std::error::Error>> {
//   let app_details = app.app_list.clone();

//   if app_details.is_empty() {
//     app.generated_command = None;
//     Ok(())
//   } else {
//     let mut sorted_list: HashMap<SupportedTarget, Vec<String>> = HashMap::new();

//     for app_detail in app_details {
//       sorted_list
//         .entry(app_detail.target)
//         .or_insert_with(Vec::new)
//         .push(app_detail.target_handle);
//     }

//     let mut command = String::from("");

//     for (target, handles) in sorted_list {
//       match target {
//         SupportedTarget::Debian | SupportedTarget::Ubuntu => {
//           command.push_str(&format!("sudo apt install -y {} && ", handles.join(" ")));
//         }
//         SupportedTarget::Arch => {
//           command.push_str(&format!(
//             "sudo pacman -S --needed --noconfirm {} && ",
//             handles.join(" ")
//           ));
//         }
//         SupportedTarget::Fedora => {
//           command.push_str(&format!("sudo dnf install -y {} && ", handles.join(" ")));
//         }
//         SupportedTarget::OpenSUSE => {
//           command.push_str(&format!("sudo zypper install -y {} && ", handles.join(" ")));
//         }
//         SupportedTarget::Nix => {
//           let nixpkgs = handles
//             .iter()
//             .map(|handle| format!("nixpkgs.{}", handle))
//             .collect::<Vec<String>>();
//           command.push_str(&format!("nix-env -iA {} && ", nixpkgs.join(" ")));
//         }
//         SupportedTarget::Flatpak => {
//           command.push_str(&format!(
//             "flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && flatpak install flathub -y {} && ",
//             handles.join(" ")
//           ));
//         }
//         SupportedTarget::Snap => {
//           // Keep separate commands to enable classic confinement for snaps that require it
//           for handle in handles {
//             command.push_str(&format!("sudo snap install {} && ", handle));
//           }
//         }
//         SupportedTarget::Homebrew => {
//           // Support for Formulae and Casks (casks have --cask in their handle)
//           let (formulae, casks): (Vec<String>, Vec<String>) = handles
//             .into_iter()
//             .partition(|handle| !handle.starts_with("--cask"));

//           let formulae_commands = Vec::from_iter(
//             formulae
//               .iter()
//               .map(|handle| format!("brew install {}", handle)),
//           );

//           let cask_commands = Vec::from_iter(
//             casks
//               .iter()
//               .map(|handle| format!("brew install --cask {}", handle.replace("--cask ", ""))),
//           );

//           command.push_str(&formulae_commands.join(" && "));
//           command.push_str(&cask_commands.join(" && "));

//           // command.push_str(&format!("brew install {} && ", handles.join(" ")));
//         }
//         SupportedTarget::Npm => {
//           command.push_str(&format!("npm install -g {} && ", handles.join(" ")));
//         }
//         SupportedTarget::Script => {
//           command.push_str(&format!("{} && ", handles.join(" && ")));
//         }
//       }
//     }

//     // Remove the trailing " && " from the command
//     command.truncate(command.len().saturating_sub(4));

//     app.generated_command = Some(command);
//     Ok(())
//   }
// }
