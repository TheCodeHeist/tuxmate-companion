use colored::Colorize;
use std::collections::HashMap;
use strum::IntoEnumIterator;

use inquire::{Confirm, Select, error::InquireError};

use crate::{
  PackageDef,
  utils::{
    app_probe::{is_aur_package, is_flathub_verified, is_snap_verified, is_unfree_nix_package},
    distro::{DistroInfo, DistroVariants},
    package::SupportedTarget,
    registry::resolve_package,
  },
};

pub struct CommandGenerator {
  host: DistroInfo,
  packages: HashMap<SupportedTarget, Vec<String>>,
  aur_packages: Vec<String>,
  install_aur_helper: bool,
  preferred_aur_helper: Option<String>,
  nix_unfree_packages: Vec<String>,
  flathub_unverified_packages: Vec<String>,
  snap_unverified_packages: Vec<String>,
  generated_command: String,
  generated_config: String,
  get_nix_config: bool,
}

impl CommandGenerator {
  pub fn init(packages: Vec<PackageDef>) -> Self {
    // Show detected distro information to the user
    let mut distro_info = DistroInfo::new();
    distro_info.fetch().unwrap_or_else(|_| {
      eprintln!(
        "{}",
        "Warning: Could not detect Linux distribution information.".yellow()
      );
    });

    // DEBUGGING
    // distro_info.variant = Some(DistroVariants::Ubuntu);
    // distro_info.aur_helper = Some("yay".into());

    println!(
      "{} {}",
      "Detected Linux Distribution Variant:"
        .bold()
        .bright_yellow(),
      format!(
        "{}",
        distro_info
          .variant
          .as_ref()
          .map_or("Unknown".to_string().red(), |v| v
            .to_name()
            .to_string()
            .truecolor(v.to_rgb().0, v.to_rgb().1, v.to_rgb().2))
      )
    );

    // if variant is None, ask the user to select a supported distro variant
    if distro_info.variant.is_none() {
      let variants = DistroVariants::iter()
        .map(|v| v.to_name().to_string())
        .collect::<Vec<String>>();

      let selected_variant = Select::new("Select your Linux distribution:", variants)
        .prompt()
        .unwrap_or_else(|e| match e {
          InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            eprintln!("{}", "Operation canceled by the user.".red());
            std::process::exit(1);
          }
          _ => {
            eprintln!(
              "{}",
              "An error occurred while selecting the Linux distribution variant.".red()
            );
            std::process::exit(1);
          }
        });

      // Set the selected variant in distro_info
      distro_info.variant = DistroVariants::from_name(&selected_variant);
    }

    // Sort the packages into their respective categories
    let mut sorted_packages: HashMap<SupportedTarget, Vec<String>> = HashMap::new();
    let mut unsupported_packages: Vec<PackageDef> = Vec::new();

    let mut nix_unfree_packages: Vec<String> = Vec::new();
    let mut aur_packages: Vec<String> = Vec::new();
    let mut flathub_unverified_packages: Vec<String> = Vec::new();
    let mut snap_unverified_packages: Vec<String> = Vec::new();

    let mut install_aur_helper = false;
    let mut preferred_aur_helper: Option<String> = None;

    let mut get_nix_config = false;

    for package in &packages {
      let mut this_package: PackageDef = package.clone(); // Create a mutable copy of the package to modify its target_id if needed

      // Skip packages that are not supported for the detected distro
      if !distro_info.is_target_supported(&package.target_id) {
        unsupported_packages.push(package.clone());
        this_package.target_id = None; // Reset the target_id to None for unsupported packages
        continue;
      }

      let resolved_package_info = resolve_package(&this_package, &distro_info);

      match resolved_package_info {
        Ok(resolved_package) => {
          // If the package is resolved successfully, add it to the sorted_packages
          sorted_packages
            .entry(
              this_package
                .target_id
                .clone()
                .unwrap_or_else(|| distro_info.variant.as_ref().unwrap().to_target()),
            )
            .or_insert_with(Vec::new)
            .push(resolved_package);
        }
        Err(fallback) => match fallback {
          Some(other_options) => {
            let ans = Select::new(
              format!(
                "The TuxMate package '{}' is not supported for the detected Linux distribution variant: {}.\nPlease select an alternative target from the following options:",
                this_package.package_id.bold(),
                distro_info
                  .variant
                  .as_ref()
                  .map_or("Unknown".to_string().red(), |v| v.to_name().to_string().truecolor(v.to_rgb().0, v.to_rgb().1, v.to_rgb().2))
              ).as_str(),
              other_options.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
            ).prompt();

            match ans {
              Ok(selected_target) => {
                // Re-resolve the package with the selected target
                match resolve_package(
                  &PackageDef {
                    target_id: Some(SupportedTarget::from_id(selected_target).unwrap()),
                    ..this_package.clone()
                  },
                  &distro_info,
                ) {
                  Ok(resolved_package) => {
                    sorted_packages
                      .entry(SupportedTarget::from_id(selected_target).unwrap())
                      .or_insert_with(Vec::new)
                      .push(resolved_package);
                  }
                  Err(_) => {
                    eprintln!(
                      "{}",
                      format!(
                        "The TuxMate package '{}' could not be resolved for the selected target: {}.",
                        this_package.package_id.bold(),
                        selected_target.truecolor(255, 0, 0)
                      )
                      .red()
                    );
                    std::process::exit(1);
                  }
                }
              }
              Err(e) => match e {
                InquireError::OperationCanceled | InquireError::OperationInterrupted => {
                  eprintln!("{}", "Operation canceled by the user.".red());
                  std::process::exit(1);
                }
                _ => {
                  eprintln!(
                    "{}",
                    "An error occurred while selecting an alternative target.".red()
                  );
                  std::process::exit(1);
                }
              },
            }
          }
          None => {
            // This means some error occurred while resolving the package, either app not found or target could not be inferred.
            // In this case, we will cancel the operation and inform the user about the issue.

            eprintln!(
              "{} {}\n\n{} {}\n\n",
              "Error:".red().bold(),
              format!(
                "Could not resolve the TuxMate package '{}' for the detected Linux distribution variant: {}.\nPlease check if the package is available in the TuxMate registry.",
                this_package.package_id.bold(),
                distro_info
                  .variant
                  .as_ref()
                  .map_or("Unknown".to_string().red(), |v| v
                    .to_name()
                    .to_string()
                    .truecolor(v.to_rgb().0, v.to_rgb().1, v.to_rgb().2))
              ),
              "Help:".on_cyan().bold().truecolor(0, 0, 0),
              "You should ensure that your local TuxMate registry is up to date. Try running `tuxmate refresh`.".green()
            );
          }
        },
      }
    }

    // Check if there are any unsupported packages
    if !unsupported_packages.is_empty() {
      println!(
        "{} {}",
        "Warning:".yellow().bold(),
        format!(
          "We found {} package(s) that are from a different Linux distribution variant than the detected one: {}\nWe have defaulted to the detected variant for these packages:",
          unsupported_packages.len(),
          distro_info
            .variant
            .as_ref()
            .map_or("Unknown".to_string().red(), |v| v
              .to_name()
              .to_string()
              .truecolor(v.to_rgb().0, v.to_rgb().1, v.to_rgb().2))
        )
      );

      for package in unsupported_packages {
        println!(
          "  - {} (selected target: {})",
          package.package_id.bold(),
          DistroVariants::from_target(package.target_id.as_ref().unwrap()).map_or(
            "Unknown".to_string().red(),
            |v| v
              .to_name()
              .to_string()
              .truecolor(v.to_rgb().0, v.to_rgb().1, v.to_rgb().2)
          )
        );
      }

      println!("\n\n");
    }

    // Check if there are any AUR packages, Nix unfree packages, Flathub unverified packages, or Snap unverified packages
    for (target, packages) in &sorted_packages {
      match target {
        SupportedTarget::Arch => {
          for package in packages {
            if is_aur_package(package) {
              aur_packages.push(package.clone());
            }
          }
        }
        SupportedTarget::Nix => {
          for package in packages {
            if is_unfree_nix_package(package) {
              nix_unfree_packages.push(package.clone());
            }
          }
        }
        SupportedTarget::Flatpak => {
          for package in packages {
            if !is_flathub_verified(package) {
              flathub_unverified_packages.push(package.clone());
            }
          }
        }
        SupportedTarget::Snap => {
          for package in packages {
            if !is_snap_verified(package) {
              snap_unverified_packages.push(package.clone());
            }
          }
        }
        _ => {}
      }
    }

    // If there are any unverified or unfree packages, inform the user and ask for confirmation to proceed
    if !nix_unfree_packages.is_empty()
      || !aur_packages.is_empty()
      || !flathub_unverified_packages.is_empty()
      || !snap_unverified_packages.is_empty()
    {
      println!(
        "{} {}",
        "Warning:".yellow().bold(),
        "We have detected some packages that may require additional attention before installation:"
      );

      if !nix_unfree_packages.is_empty() {
        println!(
          "  - {} ({} package(s))",
          "Unfree Nix packages".bold(),
          nix_unfree_packages.len()
        );

        for package in &nix_unfree_packages {
          println!("    - {}", package);
        }

        println!(
          "    {}",
          "These packages may have licensing restrictions or may not be open-source. You may have to add `allowUnfree` to your Nix configuration if not already added.".yellow()
        );
      }
      if !aur_packages.is_empty() {
        println!(
          "  - {} ({} package(s))",
          "AUR packages".bold(),
          aur_packages.len()
        );

        for package in &aur_packages {
          println!("    - {}", package);
        }

        println!(
          "    {}",
          "These packages are from the Arch User Repository (AUR) and may require an AUR helper to be installed. You may have to install an AUR helper (yay or paru) if not already installed.".yellow()
        );
      }
      if !flathub_unverified_packages.is_empty() {
        println!(
          "  - {} ({} package(s))",
          "Unverified Flathub packages".bold(),
          flathub_unverified_packages.len()
        );

        for package in &flathub_unverified_packages {
          println!("    - {}", package);
        }

        println!(
          "    {}",
          "These packages are from Flathub but are not verified. You may have to verify these packages before installation.".yellow()
        );
      }
      if !snap_unverified_packages.is_empty() {
        println!(
          "  - {} ({} package(s))",
          "Unverified Snap packages".bold(),
          snap_unverified_packages.len()
        );

        for package in &snap_unverified_packages {
          println!("    - {}", package);
        }

        println!(
          "    {}",
          "These packages are from Snap but are not verified. You may have to verify these packages before installation.".yellow()
        );
      }

      println!("\n");
    }

    // Check if the user is an Arch user, and is trying to install AUR packages.
    // If so, check if they have an AUR helper installed, and if not, ask them if they want to install one.
    if !aur_packages.is_empty() && distro_info.variant == Some(DistroVariants::Arch) {
      println!(
        "{} {}",
        "Note:".bright_cyan().bold(),
        "We have detected that you are trying to install AUR packages on an Arch-based system.\nAUR packages require an AUR helper to be installed."
      );

      match &distro_info.aur_helper {
        Some(helper) => {
          println!(
            "  - {} {}",
            "Detected AUR helper:".bold(),
            helper.truecolor(255, 165, 0)
          );

          preferred_aur_helper = Some(helper.clone());
        }
        None => {
          println!(
            "  - {}",
            "No AUR helper detected. You will need to install one to proceed with AUR package installation.".yellow()
          );

          let install_helper = Select::new(
            "Do you want to install an AUR helper now?",
            vec!["Yes", "No"],
          )
          .prompt()
          .unwrap_or_else(|e| match e {
            InquireError::OperationCanceled | InquireError::OperationInterrupted => {
              eprintln!("{}", "Operation canceled by the user.".red());
              std::process::exit(1);
            }
            _ => {
              eprintln!(
                "{}",
                "An error occurred while selecting an option for AUR helper installation.".red()
              );
              std::process::exit(1);
            }
          });

          if install_helper == "Yes" {
            let aur_helpers = vec!["yay", "paru"];
            preferred_aur_helper = Some(
              Select::new("Select an AUR helper to install:", aur_helpers.clone())
                .prompt()
                .unwrap_or_else(|e| match e {
                  InquireError::OperationCanceled | InquireError::OperationInterrupted => {
                    eprintln!("{}", "Operation canceled by the user.".red());
                    std::process::exit(1);
                  }
                  _ => {
                    eprintln!(
                      "{}",
                      "An error occurred while selecting an AUR helper.".red()
                    );
                    std::process::exit(1);
                  }
                })
                .to_string(),
            );
            install_aur_helper = true;

            println!(
              "{} {}",
              "Selected AUR helper:".bold(),
              preferred_aur_helper
                .as_ref()
                .unwrap_or(&String::new())
                .truecolor(255, 165, 0)
            );
          } else {
            // If the user does not want to install an AUR helper, we will abort the operation and inform them that they cannot proceed with AUR package installation.
            eprintln!(
              "{} {}",
              "Error:".red().bold(),
              "You have chosen not to install an AUR helper. AUR packages cannot be installed without an AUR helper. Please install an AUR helper (yay or paru) and try again.".red()
            );
            std::process::exit(1);
          }
        }
      }
    }

    // Check if the user is a Nix user, then ask if they want TuxMate to run installation commands, or if they want TuxMate to show them the configuration lines.
    // Automatic installation is only possible if the user is not installing any unfree packages.
    // If the user is installing unfree packages, they will have to manually add the configuration lines to their Nix configuration file.
    if (distro_info.variant == Some(DistroVariants::NixOS) || distro_info.has_nix)
      && nix_unfree_packages.is_empty()
    {
      let get_config = Select::new(
        "Do you want TuxMate to run the installation commands for Nix packages, or do you want TuxMate to show you the configuration lines to add to your Nix configuration file?",
        vec!["Run installation commands", "Show configuration lines"],
      )
      .prompt()
      .unwrap_or_else(|e| match e {
        InquireError::OperationCanceled | InquireError::OperationInterrupted => {
          eprintln!("{}", "Operation canceled by the user.".red());
          std::process::exit(1);
        }
        _ => {
          eprintln!(
            "{}",
            "An error occurred while selecting an option for Nix package installation.".red()
          );
          std::process::exit(1);
        }
      });

      if get_config == "Show configuration lines" {
        println!(
          "{} {}",
          "Note:".bright_cyan().bold(),
          "You have chosen to show the configuration lines for Nix packages instead of running the installation commands.\nYou will need to manually add these lines to your Nix configuration file and run `nixos-rebuild switch` to apply the changes."
        );
        get_nix_config = true;
      } else {
        println!(
          "{} {}",
          "Note:".bright_cyan().bold(),
          "You have chosen to run the installation commands for Nix packages.\nTuxMate will attempt to run the necessary commands to install the packages."
        );
      }
    } else {
      get_nix_config = true;
    }

    Self {
      host: distro_info,
      packages: sorted_packages,
      aur_packages,
      install_aur_helper,
      preferred_aur_helper,
      nix_unfree_packages,
      flathub_unverified_packages,
      snap_unverified_packages,
      generated_command: String::new(),
      generated_config: String::new(),
      get_nix_config,
    }
  }

  /// Install the generated command
  pub fn install(&self) -> Result<(), Box<dyn std::error::Error>> {
    // Show the generated command to the user and ask for confirmation before executing it

    println!(
      "{}\n",
      "Generated installation command".green().bold().underline()
    );
    println!("{}\n\n", self.generated_command);

    let confirm_install =
      Confirm::new("Do you want to execute this command to install the packages?")
        .with_default(true)
        .prompt()
        .unwrap_or_else(|e| match e {
          InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            eprintln!("{}", "Installation canceled by the user.".red());
            std::process::exit(1);
          }
          _ => {
            eprintln!(
              "{}",
              "An error occurred while confirming the installation command.".red()
            );
            std::process::exit(1);
          }
        });

    if confirm_install {
      println!(
        "{}",
        "Executing the installation command...\n\n".green().bold()
      );
      self.run_command_dynamic(&self.generated_command)?;

      println!(
        "{}",
        "\n\nInstallation completed successfully.".green().bold()
      );
    } else {
      println!("{}", "Installation canceled by the user.".red());
    }

    if self.get_nix_config && !self.generated_config.is_empty() {
      println!(
        "{}\n",
        "Generated Nix configuration lines"
          .green()
          .bold()
          .underline()
      );
      println!("{}\n\n", self.generated_config);
      println!(
        "{}",
        "Please add these lines to your Nix configuration file and run `nixos-rebuild switch` to apply the changes."
          .yellow()
          .bold()
      );
    }

    Ok(())
  }

  /// Generate commands for package installation
  pub fn generate_installation_command(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    if self.packages.is_empty() {
      return Ok(());
    }

    // Check for dependencies and install with confirmation if necessary
    self.check_and_install_dependencies()?;

    let mut command = String::from("");
    let mut nixconfig = String::from("");

    for (target, packages) in &self.packages {
      match target {
        SupportedTarget::Debian | SupportedTarget::Ubuntu => {
          command.push_str(&self.generate_apt_command(packages.to_vec()));
          command.push_str(" && ");
        }
        SupportedTarget::Arch => {
          if self.aur_packages.is_empty() {
            command.push_str(&self.generate_pacman_command(packages.to_vec()));
            command.push_str(" && ");
          } else {
            if let Some(helper) = &self.preferred_aur_helper {
              command
                .push_str(&self.generate_aur_command(self.aur_packages.clone(), helper.clone()));
              command.push_str(" && ");
            } else {
              eprintln!(
                "{} {}\n\n",
                "Error:".red().bold(),
                "No preferred AUR helper was selected. Cannot proceed with AUR package installation.".red()
              );
              std::process::exit(1);
            }
          }
        }
        SupportedTarget::Fedora => {
          command.push_str(&self.generate_dnf_command(packages.to_vec()));
          command.push_str(" && ");
        }
        SupportedTarget::OpenSUSE => {
          command.push_str(&self.generate_zypper_command(packages.to_vec()));
          command.push_str(" && ");
        }
        SupportedTarget::Nix => {
          if self.get_nix_config {
            nixconfig.push_str(&self.generate_nix_config(packages.to_vec()));
          } else {
            command.push_str(&self.generate_nix_command(packages.to_vec()));
            command.push_str(" && ");
          }
        }
        SupportedTarget::Flatpak => {
          command.push_str(&self.generate_flatpak_command(packages.to_vec()));
          command.push_str(" && ");
        }
        SupportedTarget::Snap => {
          command.push_str(&self.generate_snap_command(packages.to_vec()));
          command.push_str(" && ");
        }
        SupportedTarget::Homebrew => {
          command.push_str(&self.generate_homebrew_command(packages.to_vec()));
          command.push_str(" && ");
        }
        SupportedTarget::Npm => {
          command.push_str(&self.generate_npm_command(packages.to_vec()));
          command.push_str(" && ");
        }
        SupportedTarget::Script => {
          command.push_str(&self.generate_script_command(packages.to_vec()));
          command.push_str(" && ");
        }
      }
    }

    // Remove the trailing " && " from the command
    if !command.is_empty() {
      command.truncate(command.len().saturating_sub(4));
    }

    self.generated_command = command;
    self.generated_config = nixconfig;
    Ok(())
  }

  /// Check for dependencies and install with confirmation if necessary
  fn check_and_install_dependencies(&self) -> Result<(), Box<dyn std::error::Error>> {
    let mut missing_dependencies_commands: Vec<String> = Vec::new();

    for (target, _) in &self.packages {
      match target {
        SupportedTarget::Arch => {
          if !self.aur_packages.is_empty() && !self.host.has_aur {
            if self.install_aur_helper {
              if let Some(helper) = &self.preferred_aur_helper {
                missing_dependencies_commands
                  .push(self.generate_aur_helper_install_command(helper.clone()));
              } else {
                eprintln!(
                  "{} {}\n\n",
                  "Error:".red().bold(),
                  "No preferred AUR helper was selected. Cannot proceed with AUR helper installation.".red(),
                );
                std::process::exit(1);
              }
            } else {
              eprintln!(
                "{} {}\n\n",
                "Error:".red().bold(),
                "You have chosen not to install an AUR helper. AUR packages cannot be installed without an AUR helper. Please install an AUR helper (yay or paru) and try again.".red()
              );
              std::process::exit(1);
            }
          }
        }
        SupportedTarget::Nix => {
          if !self.host.has_nix {
            eprintln!(
              "{} {}\n\n{} {}\n\n",
              "Error:".red().bold(),
              "Nix package manager is not available on this system. Please install Nix and try again.".red(),
              "Help:".on_cyan().bold().truecolor(0, 0, 0),
              "You can install Nix by following the instructions at https://nixos.wiki/wiki/Nix_Installation_Guide".green()
            );
            std::process::exit(1);

            // let install_nix = Confirm::new("Do you want to install Nix now?")
            //   .with_default(false)
            //   .prompt()
            //   .unwrap_or_else(|e| match e {
            //     InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            //       eprintln!("{}", "Operation canceled by the user.".red());
            //       std::process::exit(1);
            //     }
            //     _ => {
            //       eprintln!(
            //         "{}",
            //         "An error occurred while selecting an option for Nix installation.".red()
            //       );
            //       std::process::exit(1);
            //     }
            //   });

            // if install_nix {
            //   missing_dependencies_commands
            //     .push("sudo install -d -m755 -o $(id -u) -g $(id -g) /nix".to_string());
            //   missing_dependencies_commands
            //     .push("curl -L https://nixos.org/nix/install | sh".to_string());
            //   missing_dependencies_commands.push(
            //     "echo 'source $HOME/.nix-profile/etc/profile.d/nix.sh' >> ~/.bashrc".to_string(),
            //   );
            // } else {
            //   std::process::exit(1);
            // }
          }
        }
        SupportedTarget::Flatpak => {
          if !self.host.has_flatpak {
            eprintln!(
              "{} {}\n\n{} {}\n\n",
              "Error:".red().bold(),
              "Flatpak package manager is not available on this system. Please install Flatpak and try again.".red(),
              "Help:".on_cyan().bold().truecolor(0, 0, 0),
              "You can install Flatpak by following the instructions at https://flathub.org/en-GB/setup/".green()
            );
            std::process::exit(1);

            // let install_flatpak = Confirm::new("Do you want to install Flatpak now?")
            //   .with_default(false)
            //   .prompt()
            //   .unwrap_or_else(|e| match e {
            //     InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            //       eprintln!("{}", "Operation canceled by the user.".red());
            //       std::process::exit(1);
            //     }
            //     _ => {
            //       eprintln!(
            //         "{}",
            //         "An error occurred while selecting an option for Flatpak installation.".red()
            //       );
            //       std::process::exit(1);
            //     }
            //   });

            // if install_flatpak {
            //   match self.host.variant {
            //     Some(DistroVariants::Ubuntu) | Some(DistroVariants::Debian) => {
            //       missing_dependencies_commands
            //         .push("sudo apt update && sudo apt install -y flatpak".to_string());
            //     }
            //     Some(DistroVariants::Arch) => {
            //       missing_dependencies_commands
            //         .push("sudo pacman -S --needed --noconfirm flatpak".to_string());
            //     }
            //     Some(DistroVariants::Fedora) => {
            //       missing_dependencies_commands.push("sudo dnf install -y flatpak".to_string());
            //     }
            //     Some(DistroVariants::OpenSUSE) => {
            //       missing_dependencies_commands.push("sudo zypper install -y flatpak".to_string());
            //     }
            //     Some(DistroVariants::NixOS) => {
            //       missing_dependencies_commands
            //         .push("sudo nix-env -iA nixpkgs.flatpak".to_string());
            //     }
            //     _ => {
            //       eprintln!(
            //         "{} {}",
            //         "Error:".red().bold(),
            //         "Unsupported distribution for Flatpak installation.".red()
            //       );
            //       std::process::exit(1);
            //     }
            //   }
            // } else {
            //   std::process::exit(1);
            // }
          }
        }
        SupportedTarget::Snap => {
          if !self.host.has_snap {
            eprintln!(
              "{} {}\n\n{} {}\n\n",
              "Error:".red().bold(),
              "Snap package manager is not available on this system. Please install Snap and try again.".red(),
              "Help:".on_cyan().bold().truecolor(0, 0, 0),
              "You can install Snap by following the instructions at https://snapcraft.io/docs/tutorials/install-the-daemon/".green()
            );
            std::process::exit(1);

            // let install_snap = Confirm::new("Do you want to install Snap now?")
            //   .with_default(false)
            //   .prompt()
            //   .unwrap_or_else(|e| match e {
            //     InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            //       eprintln!("{}", "Operation canceled by the user.".red());
            //       std::process::exit(1);
            //     }
            //     _ => {
            //       eprintln!(
            //         "{}",
            //         "An error occurred while selecting an option for Snap installation.".red()
            //       );
            //       std::process::exit(1);
            //     }
            //   });

            // if install_snap {
            //   match self.host.variant {
            //     Some(DistroVariants::Ubuntu) | Some(DistroVariants::Debian) => {
            //       missing_dependencies_commands
            //         .push("sudo apt update && sudo apt install -y snapd".to_string());
            //       missing_dependencies_commands
            //         .push("systemctl enable --now snapd apparmor".to_string());
            //     }
            //     Some(DistroVariants::Arch) => {
            //       missing_dependencies_commands.push(format!(
            //         "sudo {} -S --needed --noconfirm snapd",
            //         self
            //           .preferred_aur_helper
            //           .as_ref()
            //           .unwrap_or(&"yay".to_string())
            //       ));
            //       missing_dependencies_commands
            //         .push("sudo systemctl enable --now snapd.socket".to_string());
            //     }
            //     Some(DistroVariants::Fedora) => {
            //       missing_dependencies_commands.push("sudo dnf install -y snapd".to_string());
            //       missing_dependencies_commands
            //         .push("sudo ln -s /var/lib/snapd/snap /snap".to_string());
            //     }
            //     Some(DistroVariants::OpenSUSE) => {
            //       missing_dependencies_commands.push("sudo zypper install -y snapd".to_string());
            //     }
            //     Some(DistroVariants::NixOS) => {
            //       missing_dependencies_commands.push("sudo nix-env -iA nixpkgs.snapd".to_string());
            //     }
            //     _ => {
            //       eprintln!(
            //         "{} {}",
            //         "Error:".red().bold(),
            //         "Unsupported distribution for Snap installation.".red()
            //       );
            //       std::process::exit(1);
            //     }
            //   }
            // } else {
            //   std::process::exit(1);
            // }
          }
        }
        SupportedTarget::Homebrew => {
          if !self.host.has_homebrew {
            eprintln!(
              "{} {}\n\n{} {}\n\n",
              "Error:".red().bold(),
              "Homebrew package manager is not available on this system. Please install Homebrew and try again.".red(),
              "Help:".on_cyan().bold().truecolor(0, 0, 0),
              "You can install Homebrew by following the instructions at https://brew.sh/".green()
            );
            std::process::exit(1);
          }
        }
        SupportedTarget::Npm => {
          if !self.host.has_npm {
            eprintln!(
              "{} {}\n\n{} {}\n\n",
              "Error:".red().bold(),
              "NPM package manager is not available on this system. Please install NPM and try again.".red(),
              "Help:".on_cyan().bold().truecolor(0, 0, 0),
              "You can install NPM by following the instructions at https://nodejs.org/en/download/".green()
            );
            std::process::exit(1);
          }
        }
        SupportedTarget::Script => {
          if !self.host.has_curl {
            eprintln!(
              "{} {}\n\n{} {}\n\n",
              "Error:".red().bold(),
              "Curl is not available on this system. Please install Curl and try again.".red(),
              "Help:".on_cyan().bold().truecolor(0, 0, 0),
              "You can install Curl by following the instructions at https://curl.se/download.html"
                .green()
            );
            std::process::exit(1);
          }
        }
        _ => {}
      }
    }

    for command in missing_dependencies_commands {
      println!(
        "{}",
        format!("[Running] {}", command).truecolor(0, 255, 255)
      );
      self.run_command_dynamic(&command)?;
    }

    Ok(())
  }

  /// Run a command dynamically in the shell
  fn run_command_dynamic(&self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::{Command, Stdio};

    let mut child = Command::new("sh")
      .arg("-c")
      .arg(command)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .spawn()?;

    let status = child.wait()?;

    if !status.success() {
      return Err(format!("Command failed with status: {}", status).into());
    }

    Ok(())
  }

  /// Edit the Nix configuration file to add packages with admin privileges
  /// [SAFETY WARNING: This function is commented out because it requires admin privileges and can potentially modify system files. This will be implemented in a future version of TuxMate with proper safety checks and user confirmation.]
  // fn edit_nix_config(&self, nix_packages: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
  //   use std::fs::{self, OpenOptions};
  //   use std::io::{self, Write};

  //   let nix_config_path = "/etc/nixos/configuration.nix";

  //   // Read the existing configuration.nix file
  //   let mut config_content = match fs::read_to_string(nix_config_path) {
  //     Ok(content) => content,
  //     Err(_) => {
  //       eprintln!("Error: Could not read {}", nix_config_path);
  //       std::process::exit(1);
  //     }
  //   };

  //   // Parse the existing configuration.nix content and add the new packages to the environment.systemPackages list, with the existing packages preserved
  //   let new_packages = nix_packages
  //     .iter()
  //     .map(|handle| format!("nixpkgs.{}", handle))
  //     .collect::<Vec<String>>()
  //     .join("\n    ");

  //   // Add allowUnfree = true; to the configuration.nix file if any of the packages are unfree
  //   if self.nix_unfree_packages.len() > 0 {
  //     if !config_content.contains("nixpkgs.config.allowUnfree = true;") {
  //       config_content.push_str("\nnixpkgs.config.allowUnfree = true;\n");
  //     }
  //   }

  //   // Check if the environment.systemPackages line exists in the configuration.nix file
  //   if config_content.contains("environment.systemPackages") {
  //     // If it exists, append the new packages to the existing list
  //     config_content = config_content.replace(
  //       "environment.systemPackages = with pkgs; [",
  //       &format!(
  //         "environment.systemPackages = with pkgs; [\n    {}\n    ",
  //         new_packages
  //       ),
  //     );
  //   } else {
  //     // If it does not exist, add the environment.systemPackages line with the new packages
  //     config_content.push_str(&format!(
  //       "\n\nenvironment.systemPackages = with pkgs; [\n    {}\n];",
  //       new_packages
  //     ));
  //   }

  //   // Write the updated configuration.nix file with admin privileges
  //   let mut file = OpenOptions::new()
  //     .write(true)
  //     .truncate(true)
  //     .open(nix_config_path)
  //     .map_err(|e| {
  //       eprintln!(
  //         "Error: Could not open {} for writing: {}",
  //         nix_config_path, e
  //       );
  //       std::process::exit(1);
  //     })?;

  //   file.write_all(config_content.as_bytes()).map_err(|e| {
  //     eprintln!("Error: Could not write to {}: {}", nix_config_path, e);
  //     std::process::exit(1);
  //   })?;

  //   // Run nixos-rebuild switch to apply the changes
  //   let rebuild_status = std::process::Command::new("sudo")
  //     .arg("nixos-rebuild")
  //     .arg("switch")
  //     .status()
  //     .map_err(|e| {
  //       eprintln!("Error: Could not run nixos-rebuild switch: {}", e);
  //       std::process::exit(1);
  //     })?;

  //   if !rebuild_status.success() {
  //     eprintln!(
  //       "Error: nixos-rebuild switch failed with status: {}",
  //       rebuild_status
  //     );
  //     std::process::exit(1);
  //   }

  //   Ok(())
  // }

  /// Generate a command to install packages using apt package manager (Debian/Ubuntu)
  fn generate_apt_command(&self, apt_packages: Vec<String>) -> String {
    format!(
      "sudo apt update && sudo apt install -y {}",
      apt_packages.join(" ")
    )
  }

  /// Generate a command to install packages using pacman package manager (Arch)
  fn generate_pacman_command(&self, pacman_packages: Vec<String>) -> String {
    format!(
      "sudo pacman -S --needed --noconfirm {}",
      pacman_packages.join(" ")
    )
  }

  /// Generate a command to install packages from AUR (Arch)
  fn generate_aur_command(&self, aur_packages: Vec<String>, aur_helper: String) -> String {
    format!(
      "{} -S --needed --noconfirm {}",
      aur_helper,
      aur_packages.join(" ")
    )
  }

  /// Generate commands to install a chosen AUR helper tool
  fn generate_aur_helper_install_command(&self, aur_helper: String) -> String {
    // Create a temporary directory for building the AUR helper
    let temp_dir = "/tmp/tuxmate-aur-build";

    format!(
      "mkdir -p {temp_dir} && cd {temp_dir} && git clone https://aur.archlinux.org/{aur_helper}.git && cd {aur_helper} && makepkg -si --noconfirm && cd .. && rm -rf {temp_dir}",
      temp_dir = temp_dir,
      aur_helper = aur_helper
    )
  }

  /// Generate a command to install packages using dnf package manager (Fedora)
  fn generate_dnf_command(&self, dnf_packages: Vec<String>) -> String {
    format!("sudo dnf install -y {}", dnf_packages.join(" "))
  }

  /// Generate a command to install packages using zypper package manager (OpenSUSE)
  fn generate_zypper_command(&self, zypper_packages: Vec<String>) -> String {
    format!("sudo zypper install -y {}", zypper_packages.join(" "))
  }

  /// Generate a command to install packages using nix package manager (NixOS)
  fn generate_nix_command(&self, nix_packages: Vec<String>) -> String {
    let nixpkgs = nix_packages
      .iter()
      .map(|handle| format!("nixpkgs.{}", handle))
      .collect::<Vec<String>>();

    format!("sudo nix-env -iA {}", nixpkgs.join(" "))
  }

  /// Generate configuration file content for nix package manager (NixOS)
  fn generate_nix_config(&self, nix_packages: Vec<String>) -> String {
    if self.nix_unfree_packages.len() > 0 {
      format!(
        "# In /etc/nixos/configuration.nix\n\nnixpkgs.config.allowUnfree = true; # Allow installation of unfree packages, add this line to your NixOS configuration, if not already present.\n\nenvironment.systemPackages = with pkgs; [\n    {}\n];",
        nix_packages.join("\n    ")
      )
    } else {
      format!(
        "# In /etc/nixos/configuration.nix\n\nenvironment.systemPackages = with pkgs; [\n    {}\n];",
        nix_packages.join("\n    ")
      )
    }
  }

  /// Generate a command to install packages using flatpak package manager
  fn generate_flatpak_command(&self, flatpak_packages: Vec<String>) -> String {
    format!(
      "flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo && flatpak install flathub -y {}",
      flatpak_packages.join(" ")
    )
  }

  /// Generate a command to install packages using snap package manager
  fn generate_snap_command(&self, snap_packages: Vec<String>) -> String {
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
  fn generate_homebrew_command(&self, homebrew_packages: Vec<String>) -> String {
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
  fn generate_npm_command(&self, npm_packages: Vec<String>) -> String {
    format!("npm install -g {}", npm_packages.join(" "))
  }

  /// Generate a command to run scripts
  fn generate_script_command(&self, script_packages: Vec<String>) -> String {
    script_packages.join(" && ")
  }
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
