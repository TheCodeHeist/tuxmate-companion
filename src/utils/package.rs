use std::{
  collections::HashMap,
  io::{Error, ErrorKind},
};

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
  pub fn from_name(s: &str) -> Self {
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

  pub fn to_name(&self) -> &str {
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

  pub fn to_id(&self) -> &str {
    match self {
      Category::AITools => "ai-tools",
      Category::CLITools => "cli-tools",
      Category::Communication => "communication",
      Category::Creative => "creative",
      Category::DevEditors => "dev-editors",
      Category::DevLanguages => "dev-languages",
      Category::DevTools => "dev-tools",
      Category::FileSharing => "file-sharing",
      Category::Gaming => "gaming",
      Category::Media => "media",
      Category::Office => "office",
      Category::Security => "security",
      Category::System => "system",
      Category::Terminal => "terminal",
      Category::VPNNetwork => "vpn-network",
      Category::WebBrowsers => "web-browsers",
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
  pub fn from_id(s: &str) -> Result<Self, Error> {
    match s {
      "ubuntu" => Ok(SupportedTarget::Ubuntu),
      "debian" => Ok(SupportedTarget::Debian),
      "arch" => Ok(SupportedTarget::Arch),
      "fedora" => Ok(SupportedTarget::Fedora),
      "opensuse" => Ok(SupportedTarget::OpenSUSE),
      "nix" => Ok(SupportedTarget::Nix),
      "flatpak" => Ok(SupportedTarget::Flatpak),
      "snap" => Ok(SupportedTarget::Snap),
      "homebrew" => Ok(SupportedTarget::Homebrew),
      "npm" => Ok(SupportedTarget::Npm),
      "script" => Ok(SupportedTarget::Script),
      &_ => Err(Error::new(
        ErrorKind::InvalidInput,
        format!("Unknown target: {}", s),
      )),
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
  pub id: String,
  pub name: String,
  pub description: String,
  pub category: Category,
  pub icon: IconDef,
  pub targets: HashMap<SupportedTarget, String>, // target_id -> command
  #[serde(rename = "unavailableReason", skip_serializing_if = "Option::is_none")]
  pub unavailable_reason: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub note: Option<String>,
}
