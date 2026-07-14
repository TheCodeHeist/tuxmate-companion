use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::process::Command;

use strum::EnumIter;

use crate::utils::package::SupportedTarget;

#[derive(Debug, Clone)]
pub struct DistroInfo {
  pub name: String,
  pub id: String,
  pub version: String,
  pub pretty_name: String,

  // Base Distros
  pub variant: Option<DistroVariants>,
  // pub is_ubuntu: bool,
  // pub is_debian: bool,
  // pub is_arch: bool,
  // pub is_fedora: bool,
  // pub is_opensuse: bool,
  // pub is_nixos: bool,

  // Package Managers & Tools
  pub has_nix: bool,
  pub has_flatpak: bool,
  pub has_snap: bool,
  pub has_homebrew: bool,
  pub has_npm: bool,
  pub has_curl: bool,

  // AUR specific
  pub has_aur: bool,
  pub aur_helper: Option<String>, // "yay" or "paru", for now
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter)]
pub enum DistroVariants {
  Ubuntu,
  Debian,
  Arch,
  Fedora,
  OpenSUSE,
  NixOS,
}

impl DistroVariants {
  pub fn to_name(&self) -> &'static str {
    match self {
      DistroVariants::Ubuntu => "Ubuntu",
      DistroVariants::Debian => "Debian",
      DistroVariants::Arch => "Arch Linux",
      DistroVariants::Fedora => "Fedora",
      DistroVariants::OpenSUSE => "openSUSE",
      DistroVariants::NixOS => "NixOS",
    }
  }

  pub fn from_name(name: &str) -> Option<Self> {
    match name.to_lowercase().as_str() {
      "ubuntu" => Some(DistroVariants::Ubuntu),
      "debian" => Some(DistroVariants::Debian),
      "arch" | "arch linux" => Some(DistroVariants::Arch),
      "fedora" => Some(DistroVariants::Fedora),
      "opensuse" => Some(DistroVariants::OpenSUSE),
      "nixos" => Some(DistroVariants::NixOS),
      _ => None,
    }
  }

  pub fn to_target(&self) -> SupportedTarget {
    match self {
      DistroVariants::Ubuntu => SupportedTarget::Ubuntu,
      DistroVariants::Debian => SupportedTarget::Debian,
      DistroVariants::Arch => SupportedTarget::Arch,
      DistroVariants::Fedora => SupportedTarget::Fedora,
      DistroVariants::OpenSUSE => SupportedTarget::OpenSUSE,
      DistroVariants::NixOS => SupportedTarget::Nix,
    }
  }

  pub fn from_target(target: &SupportedTarget) -> Option<Self> {
    match target {
      SupportedTarget::Ubuntu => Some(DistroVariants::Ubuntu),
      SupportedTarget::Debian => Some(DistroVariants::Debian),
      SupportedTarget::Arch => Some(DistroVariants::Arch),
      SupportedTarget::Fedora => Some(DistroVariants::Fedora),
      SupportedTarget::OpenSUSE => Some(DistroVariants::OpenSUSE),
      SupportedTarget::Nix => Some(DistroVariants::NixOS),
      _ => None,
    }
  }

  pub fn to_rgb(&self) -> (u8, u8, u8) {
    match self {
      DistroVariants::Ubuntu => (255, 136, 0),
      DistroVariants::Debian => (166, 0, 51),
      DistroVariants::Arch => (23, 147, 209),
      DistroVariants::Fedora => (83, 159, 221),
      DistroVariants::OpenSUSE => (114, 185, 42),
      DistroVariants::NixOS => (86, 109, 185),
    }
  }
}

impl DistroInfo {
  pub fn new() -> Self {
    DistroInfo {
      name: String::new(),
      id: String::new(),
      version: String::new(),
      pretty_name: String::new(),

      variant: None,

      // is_ubuntu: false,
      // is_debian: false,
      // is_arch: false,
      // is_fedora: false,
      // is_opensuse: false,
      // is_nixos: false,
      has_nix: false,
      has_flatpak: false,
      has_snap: false,
      has_homebrew: false,
      has_npm: false,
      has_curl: false,
      has_aur: false,
      aur_helper: None,
    }
  }

  pub fn fetch(&mut self) -> Result<(), Error> {
    let os_release = self.parse_os_release();

    let name = os_release
      .get("PRETTY_NAME")
      .cloned()
      .unwrap_or_else(|| "Unknown".to_string());
    let id = os_release
      .get("ID")
      .cloned()
      .unwrap_or_else(|| "unknown".to_string());
    let version = os_release
      .get("VERSION_ID")
      .cloned()
      .unwrap_or_else(|| "Unknown".to_string());

    let aur_info = self.detect_aur_helper();

    let is_nixos = id.contains("nixos")
      || name.to_lowercase().contains("nixos")
      || fs::metadata("/etc/nixos").is_ok()
      || fs::metadata("/run/current-system").is_ok();

    let variant = if id.contains("ubuntu") || id.contains("linuxmint") || id.contains("pop") {
      Some(DistroVariants::Ubuntu)
    } else if id.contains("debian")
      || id.contains("raspbian")
      || id.contains("devuan")
      || id.contains("kali")
    {
      Some(DistroVariants::Debian)
    } else if id.contains("arch") || id.contains("manjaro") || id.contains("endeavouros") {
      Some(DistroVariants::Arch)
    } else if id.contains("fedora") {
      Some(DistroVariants::Fedora)
    } else if id.contains("opensuse") {
      Some(DistroVariants::OpenSUSE)
    } else if is_nixos {
      Some(DistroVariants::NixOS)
    } else {
      None
    };

    *self = DistroInfo {
      pretty_name: name.clone(),
      id: id.clone(),
      name,
      version,

      variant,

      // is_ubuntu: id.contains("ubuntu") || id.contains("linuxmint") || id.contains("pop"),
      // is_debian: id.contains("debian")
      //   || id.contains("raspbian")
      //   || id.contains("devuan")
      //   || id.contains("kali"),
      // is_arch: id.contains("arch") || id.contains("manjaro") || id.contains("endeavouros"),
      // is_fedora: id.contains("fedora"),
      // is_opensuse: id.contains("opensuse"),
      // is_nixos,
      has_nix: self.check_nix(),
      has_flatpak: self.check_flatpak(),
      has_snap: self.check_snap(),
      has_homebrew: self.check_homebrew(),
      has_npm: self.check_npm(),
      has_curl: self.check_curl(),
      has_aur: aur_info.is_some(),
      aur_helper: aur_info,
    };

    Ok(())
  }

  pub fn is_target_supported(&self, target_id: &Option<SupportedTarget>) -> bool {
    match target_id {
      Some(target) => match target {
        SupportedTarget::Ubuntu => self.variant == Some(DistroVariants::Ubuntu),
        SupportedTarget::Debian => self.variant == Some(DistroVariants::Debian),
        SupportedTarget::Arch => self.variant == Some(DistroVariants::Arch),
        SupportedTarget::Fedora => self.variant == Some(DistroVariants::Fedora),
        SupportedTarget::OpenSUSE => self.variant == Some(DistroVariants::OpenSUSE),
        _ => true, // For targets like Flatpak, Snap, Homebrew, NPM, we assume they are supported on all distros
      },
      None => true, // If no target is specified, we assume it's supported on the current distro
    }
  }

  fn parse_os_release(&self) -> HashMap<String, String> {
    fs::read_to_string("/etc/os-release")
      .or_else(|_| fs::read_to_string("/etc/lsb-release"))
      .unwrap_or_default()
      .lines()
      .filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
          return None;
        }
        line.split_once('=').map(|(k, v)| {
          (
            k.trim().to_string(),
            v.trim_matches(|c| c == '"' || c == '\'').trim().to_string(),
          )
        })
      })
      .collect()
  }

  fn check_command(&self, cmd: &str) -> bool {
    Command::new(cmd)
      .arg("--version")
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false)
  }

  fn detect_aur_helper(&self) -> Option<String> {
    let helpers = ["yay", "paru"];

    for helper in &helpers {
      if self.check_command(helper) {
        return Some(helper.to_string());
      }
    }

    // Check if base AUR is usable (pacman + makepkg)
    if self.check_command("pacman") && self.check_command("makepkg") {
      return Some("base-aur".to_string());
    }

    None
  }

  fn check_nix(&self) -> bool {
    fs::metadata("/nix/store").is_ok() || self.check_command("nix")
  }

  fn check_flatpak(&self) -> bool {
    self.check_command("flatpak") || fs::metadata("/var/lib/flatpak").is_ok()
  }

  fn check_snap(&self) -> bool {
    self.check_command("snap") || fs::metadata("/snap").is_ok()
  }

  fn check_homebrew(&self) -> bool {
    fs::metadata("/home/linuxbrew/.linuxbrew").is_ok()
      || fs::metadata("/opt/homebrew").is_ok()
      || std::env::var("HOMEBREW_PREFIX").is_ok()
  }

  fn check_npm(&self) -> bool {
    self.check_command("npm")
  }

  fn check_curl(&self) -> bool {
    self.check_command("curl")
  }
}
