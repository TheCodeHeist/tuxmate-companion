use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DistroInfo {
  pub name: String,
  pub id: String,
  pub version: String,
  pub pretty_name: String,

  // Base Distros
  pub is_ubuntu: bool,
  pub is_debian: bool,
  pub is_arch: bool,
  pub is_fedora: bool,
  pub is_opensuse: bool,

  // Package Managers & Tools
  pub has_nix: bool,
  pub has_flatpak: bool,
  pub has_snap: bool,
  pub has_homebrew: bool,

  // AUR specific
  pub has_aur: bool,
  pub aur_helper: Option<String>, // "yay" or "paru", for now
}

impl DistroInfo {
  pub fn new() -> Self {
    DistroInfo {
      name: String::new(),
      id: String::new(),
      version: String::new(),
      pretty_name: String::new(),

      is_ubuntu: false,
      is_debian: false,
      is_arch: false,
      is_fedora: false,
      is_opensuse: false,

      has_nix: false,
      has_flatpak: false,
      has_snap: false,
      has_homebrew: false,

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

    *self = DistroInfo {
      pretty_name: name.clone(),
      id: id.clone(),
      name,
      version,

      is_ubuntu: id.contains("ubuntu") || id.contains("linuxmint") || id.contains("pop"),
      is_debian: id.contains("debian"),
      is_arch: id.contains("arch") || id.contains("manjaro") || id.contains("endeavouros"),
      is_fedora: id.contains("fedora"),
      is_opensuse: id.contains("opensuse"),

      has_nix: self.check_nix(),
      has_flatpak: self.check_flatpak(),
      has_snap: self.check_snap(),
      has_homebrew: self.check_homebrew(),

      has_aur: aur_info.is_some(),
      aur_helper: aur_info,
    };

    Ok(())
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
}
