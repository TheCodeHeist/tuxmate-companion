use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Category {
  #[serde(rename = "AI Tools")]
  AITools,
  #[serde(rename = "CLI Tools")]
  CLITools,
  #[serde(rename = "Communication")]
  Communication,
  #[serde(rename = "Creative")]
  Creative,
  #[serde(rename = "Dev: Editors")]
  DevEditors,
  #[serde(rename = "Dev: Languages")]
  DevLanguages,
  #[serde(rename = "Dev: Tools")]
  DevTools,
  #[serde(rename = "File Sharing")]
  FileSharing,
  Gaming,
  Media,
  Office,
  Security,
  System,
  Terminal,
  #[serde(rename = "VPN & Network")]
  VPNNetwork,
  #[serde(rename = "Web Browsers")]
  WebBrowsers,
}

impl Category {
  pub fn from_str(s: &str) -> Self {
    match s {
      "AI Tools" => Category::AITools,
      "CLI Tools" => Category::CLITools,
      "Communication" => Category::Communication,
      "Creative" => Category::Creative,
      "Dev: Editors" => Category::DevEditors,
      "Dev: Languages" => Category::DevLanguages,
      "Dev: Tools" => Category::DevTools,
      "File Sharing" => Category::FileSharing,
      "Gaming" => Category::Gaming,
      "Media" => Category::Media,
      "Office" => Category::Office,
      "Security" => Category::Security,
      "System" => Category::System,
      "Terminal" => Category::Terminal,
      "VPN & Network" => Category::VPNNetwork,
      "Web Browsers" => Category::WebBrowsers,
      &_ => panic!("Unknown category: {}", s),
    }
  }

  pub fn to_str(&self) -> &str {
    match self {
      Category::AITools => "AI Tools",
      Category::CLITools => "CLI Tools",
      Category::Communication => "Communication",
      Category::Creative => "Creative",
      Category::DevEditors => "Dev: Editors",
      Category::DevLanguages => "Dev: Languages",
      Category::DevTools => "Dev: Tools",
      Category::FileSharing => "File Sharing",
      Category::Gaming => "Gaming",
      Category::Media => "Media",
      Category::Office => "Office",
      Category::Security => "Security",
      Category::System => "System",
      Category::Terminal => "Terminal",
      Category::VPNNetwork => "VPN & Network",
      Category::WebBrowsers => "Web Browsers",
    }
  }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SupportedTarget {
  #[serde(rename = "ubuntu")]
  Ubuntu,
  #[serde(rename = "debian")]
  Debian,
  #[serde(rename = "arch")]
  Arch,
  #[serde(rename = "fedora")]
  Fedora,
  #[serde(rename = "opensuse")]
  OpenSUSE,
  #[serde(rename = "nix")]
  Nix,
  #[serde(rename = "flatpak")]
  Flatpak,
  #[serde(rename = "snap")]
  Snap,
  #[serde(rename = "homebrew")]
  Homebrew,
  #[serde(rename = "npm")]
  Npm,
  #[serde(rename = "script")]
  Script,
}

impl SupportedTarget {
  pub fn from_id(s: &str) -> Self {
    match s {
      "ubuntu" => SupportedTarget::Ubuntu,
      "debian" => SupportedTarget::Debian,
      "arch" => SupportedTarget::Arch,
      "fedora" => SupportedTarget::Fedora,
      "opensuse" => SupportedTarget::OpenSUSE,
      "nix" => SupportedTarget::Nix,
      "flatpak" => SupportedTarget::Flatpak,
      "snap" => SupportedTarget::Snap,
      "homebrew" => SupportedTarget::Homebrew,
      "npm" => SupportedTarget::Npm,
      "script" => SupportedTarget::Script,
      &_ => panic!("Unknown target: {}", s),
    }
  }

  pub fn to_id(&self) -> &str {
    match self {
      SupportedTarget::Ubuntu => "ubuntu",
      SupportedTarget::Debian => "debian",
      SupportedTarget::Arch => "arch",
      SupportedTarget::Fedora => "fedora",
      SupportedTarget::OpenSUSE => "opensuse",
      SupportedTarget::Nix => "nix",
      SupportedTarget::Flatpak => "flatpak",
      SupportedTarget::Snap => "snap",
      SupportedTarget::Homebrew => "homebrew",
      SupportedTarget::Npm => "npm",
      SupportedTarget::Script => "script",
    }
  }

  pub fn to_name(&self) -> &str {
    match self {
      SupportedTarget::Ubuntu => "Ubuntu",
      SupportedTarget::Debian => "Debian",
      SupportedTarget::Arch => "Arch Linux",
      SupportedTarget::Fedora => "Fedora",
      SupportedTarget::OpenSUSE => "OpenSUSE",
      SupportedTarget::Nix => "Nix",
      SupportedTarget::Flatpak => "Flatpak",
      SupportedTarget::Snap => "Snap",
      SupportedTarget::Homebrew => "Homebrew",
      SupportedTarget::Npm => "NPM",
      SupportedTarget::Script => "External Script",
    }
  }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct IconDef {
  #[serde(rename = "type")]
  pub icon_type: String, // "iconify" or "url"
  #[serde(skip_serializing_if = "Option::is_none")]
  pub set: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub color: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub url: Option<String>, // Only for type "url"
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AppData {
  id: String,
  name: String, 
  description: String,
  category: String,

  icon: IconDef,
  targets: HashMap<SupportedTarget, String>, // target_id -> command
  #[serde(rename = "unavailableReason", skip_serializing_if = "Option::is_none")]
  unavailable_reason: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  note: Option<String>,
}
